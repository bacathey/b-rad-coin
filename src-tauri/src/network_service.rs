//! BradCoin Network Service
//! Handles peer discovery, block propagation, and network communication

use crate::blockchain_database::{AsyncBlockchainDatabase, Block, Transaction, TransactionInput, TransactionOutput};
use crate::errors::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, timeout};

/// Default ports for BradCoin network
pub const DEFAULT_P2P_PORT: u16 = 8333;
pub const DEFAULT_RPC_PORT: u16 = 8334;

/// Network message types (Bitcoin protocol style)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    /// Ping message to check if peer is alive
    Ping {
        timestamp: u64,
        nonce: u64,
    },
    /// Pong response to ping
    Pong {
        timestamp: u64,
        nonce: u64,
    },
    /// Request for peer addresses
    GetAddr,
    /// Response with known peer addresses
    Addr {
        addresses: Vec<PeerAddress>,
    },
    /// Request for blockchain height
    GetHeight,
    /// Response with blockchain height
    Height {
        height: u64,
    },
    /// Request for block by height or hash
    GetBlock {
        height: Option<u64>,
        hash: Option<String>,
    },
    /// Response with block data
    Block {
        block: Block,
    },
    /// Announcement of new block
    NewBlock {
        block: Block,
    },
    /// Request for transaction
    GetTransaction {
        txid: String,
    },
    /// Response with transaction data
    Transaction {
        transaction: Transaction,
    },
    /// Announcement of new transaction
    NewTransaction {
        transaction: Transaction,
    },
    /// Request for multiple blocks (Bitcoin-style getblocks)
    GetBlocks {
        version: u32,
        block_locator_hashes: Vec<String>, // Most recent block hashes we have
        hash_stop: Option<String>, // Stop at this hash (empty for latest)
    },
    /// Response with block inventory (Bitcoin-style inv)
    Inv {
        inventory: Vec<InventoryItem>,
    },
    /// Request specific data (Bitcoin-style getdata)
    GetData {
        inventory: Vec<InventoryItem>,
    },
    /// Response with multiple blocks
    Blocks {
        blocks: Vec<Block>,
    },
    /// Version handshake message
    Version {
        version: u32,
        services: u64,
        timestamp: u64,
        addr_recv: PeerAddress,
        addr_from: PeerAddress,
        nonce: u64,
        user_agent: String,
        start_height: u64,
    },
    /// Version acknowledgment
    Verack,
    /// Request block headers (Bitcoin-style getheaders)
    GetHeaders {
        version: u32,
        block_locator_hashes: Vec<String>,
        hash_stop: Option<String>,
    },
    /// Response with block headers
    Headers {
        headers: Vec<BlockHeader>,
    },
}

/// Inventory item types (Bitcoin protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_type: InventoryType,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryType {
    Error = 0,
    Transaction = 1,
    Block = 2,
    CompactBlock = 4,
}

/// Block header (for headers-first sync)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub hash: String,
    pub previous_hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub merkle_root: String,
    pub nonce: u64,
    pub difficulty: f64,
}

/// Peer address information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerAddress {
    pub ip: IpAddr,
    pub port: u16,
    pub last_seen: u64,
    pub services: u64, // Bitfield for supported services
}

/// Peer connection information
#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub address: PeerAddress,
    pub connected_at: u64,
    pub last_ping: u64,
    pub version: Option<String>,
    pub height: Option<u64>,
    pub is_outbound: bool,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: u32,
    pub total_known_peers: u32,
    pub blocks_received: u64,
    pub transactions_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub network_height: u64,
    pub local_height: u64,
}

/// BradCoin Network Service
pub struct NetworkService {
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    listen_addr: SocketAddr,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
    known_addresses: Arc<RwLock<HashSet<PeerAddress>>>,
    message_sender: Option<mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>>,
    stats: Arc<RwLock<NetworkStats>>,
    app_handle: Option<AppHandle>,
    is_running: Arc<RwLock<bool>>,
}

impl NetworkService {
    /// Create a new network service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>, port: Option<u16>) -> Self {
        let port = port.unwrap_or(DEFAULT_P2P_PORT);
        let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

        Self {
            blockchain_db,
            listen_addr,
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_addresses: Arc::new(RwLock::new(HashSet::new())),
            message_sender: None,
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            app_handle: None,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the network service
    pub async fn initialize(&mut self, app_handle: AppHandle) -> AppResult<()> {
        info!("Initializing BradCoin network service on {}", self.listen_addr);
        self.app_handle = Some(app_handle);

        // Add some bootstrap nodes (in a real implementation, these would be well-known nodes)
        self.add_bootstrap_nodes().await;

        Ok(())
    }

    /// Start the network service
    pub async fn start(&mut self) -> AppResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("Network service is already running");
            return Ok(());
        }

        info!("Starting BradCoin network service...");
        *is_running = true;
        drop(is_running);

        // Create message channel
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.message_sender = Some(tx.clone());

        // Start TCP listener
        let listener = TcpListener::bind(&self.listen_addr).await
            .map_err(|e| AppError::Network(format!("Failed to bind to {}: {}", self.listen_addr, e)))?;

        info!("Network service listening on {}", self.listen_addr);

        // Clone references for async tasks
        let peers = Arc::clone(&self.peers);
        let known_addresses = Arc::clone(&self.known_addresses);
        let blockchain_db = Arc::clone(&self.blockchain_db);
        let stats = Arc::clone(&self.stats);
        let app_handle = self.app_handle.clone();
        let is_running_clone = Arc::clone(&self.is_running);

        // Start connection acceptor
        let acceptor_peers = Arc::clone(&peers);
        let acceptor_tx = tx.clone();
        tokio::spawn(async move {
            Self::accept_connections(listener, acceptor_peers, acceptor_tx).await;
        });

        // Start message handler
        let handler_peers = Arc::clone(&peers);
        let handler_blockchain = Arc::clone(&blockchain_db);
        let handler_stats = Arc::clone(&stats);
        tokio::spawn(async move {
            Self::handle_messages(rx, handler_peers, handler_blockchain, handler_stats, app_handle).await;
        });

        // Start peer discovery
        let discovery_known = Arc::clone(&known_addresses);
        let discovery_peers = Arc::clone(&peers);
        let discovery_tx = tx.clone();
        tokio::spawn(async move {
            Self::peer_discovery_loop(discovery_known, discovery_peers, discovery_tx, is_running_clone).await;
        });

        // Start periodic tasks
        let periodic_peers = Arc::clone(&peers);
        let periodic_stats = Arc::clone(&stats);
        let periodic_blockchain = Arc::clone(&blockchain_db);
        tokio::spawn(async move {
            Self::periodic_tasks(periodic_peers, periodic_stats, periodic_blockchain).await;
        });

        info!("BradCoin network service started successfully");
        Ok(())
    }

    /// Stop the network service
    pub async fn stop(&mut self) -> AppResult<()> {
        info!("Stopping BradCoin network service...");
        
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        // Close all peer connections
        let mut peers = self.peers.write().await;
        peers.clear();
        
        info!("BradCoin network service stopped");
        Ok(())
    }

    /// Add bootstrap nodes for initial peer discovery
    async fn add_bootstrap_nodes(&self) {
        let bootstrap_nodes = vec![
            // In a real implementation, these would be reliable bootstrap nodes
            PeerAddress {
                ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                port: DEFAULT_P2P_PORT + 1,
                last_seen: Self::current_timestamp(),
                services: 1, // Full node
            },
            PeerAddress {
                ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                port: DEFAULT_P2P_PORT + 2,
                last_seen: Self::current_timestamp(),
                services: 1,
            },
        ];

        let mut known_addresses = self.known_addresses.write().await;
        for addr in bootstrap_nodes {
            known_addresses.insert(addr);
        }
        
        info!("Added {} bootstrap nodes", known_addresses.len());
    }

    /// Accept incoming connections
    async fn accept_connections(
        listener: TcpListener,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
        message_sender: mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>,
    ) {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Accepted connection from {}", addr);
                    
                    let peer_connection = PeerConnection {
                        address: PeerAddress {
                            ip: addr.ip(),
                            port: addr.port(),
                            last_seen: Self::current_timestamp(),
                            services: 0,
                        },
                        connected_at: Self::current_timestamp(),
                        last_ping: 0,
                        version: None,
                        height: None,
                        is_outbound: false,
                    };

                    // Add peer to connections
                    {
                        let mut peers_guard = peers.write().await;
                        peers_guard.insert(addr, peer_connection);
                    }

                    // Handle this connection
                    let connection_peers = Arc::clone(&peers);
                    let connection_sender = message_sender.clone();
                    tokio::spawn(async move {
                        Self::handle_peer_connection(stream, addr, connection_peers, connection_sender).await;
                    });
                },
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Handle messages from peers
    async fn handle_messages(
        mut rx: mpsc::UnboundedReceiver<(SocketAddr, NetworkMessage)>,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
        blockchain_db: Arc<AsyncBlockchainDatabase>,
        stats: Arc<RwLock<NetworkStats>>,
        app_handle: Option<AppHandle>,
    ) {
        while let Some((peer_addr, message)) = rx.recv().await {
            match Self::process_message(peer_addr, message, &peers, &blockchain_db, &stats).await {
                Ok(_) => {
                    debug!("Successfully processed message from {}", peer_addr);
                },
                Err(e) => {
                    warn!("Failed to process message from {}: {}", peer_addr, e);
                }
            }

            // Emit network status update
            if let Some(ref app) = app_handle {
                let stats_guard = stats.read().await;
                if let Err(e) = app.emit("network-stats", &*stats_guard) {
                    debug!("Failed to emit network stats: {}", e);
                }
            }
        }
    }    /// Process a network message
    async fn process_message(
        peer_addr: SocketAddr,
        message: NetworkMessage,
        peers: &Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
        blockchain_db: &Arc<AsyncBlockchainDatabase>,
        stats: &Arc<RwLock<NetworkStats>>,
    ) -> AppResult<()> {
        match message {
            NetworkMessage::Ping { timestamp, nonce } => {
                debug!("Received ping from {} (nonce: {})", peer_addr, nonce);
                // TODO: Send pong response
            },
            NetworkMessage::Pong { timestamp, nonce } => {
                debug!("Received pong from {} (nonce: {})", peer_addr, nonce);
                // Update last ping time
                if let Some(peer) = peers.write().await.get_mut(&peer_addr) {
                    peer.last_ping = Self::current_timestamp();
                }
            },
            NetworkMessage::GetHeight => {
                debug!("Received height request from {}", peer_addr);
                // TODO: Send height response
                let height = blockchain_db.get_block_height().await.unwrap_or(0);
                // TODO: Send Height response back to peer
            },
            NetworkMessage::Height { height } => {
                debug!("Received height {} from {}", height, peer_addr);
                // Update peer height
                if let Some(peer) = peers.write().await.get_mut(&peer_addr) {
                    peer.height = Some(height);
                }
                
                // Update network stats
                let mut stats_guard = stats.write().await;
                if height > stats_guard.network_height {
                    stats_guard.network_height = height;
                }
            },
            NetworkMessage::GetBlocks { version, block_locator_hashes, hash_stop } => {
                info!("Received getblocks request from {} (locator hashes: {})", peer_addr, block_locator_hashes.len());
                
                // Find the best common block and send inventory of subsequent blocks
                if let Ok(start_height) = Self::find_fork_point(blockchain_db, &block_locator_hashes).await {
                    let mut inventory = Vec::new();
                    let max_blocks = 500; // Bitcoin protocol limit
                      for height in (start_height + 1)..=(start_height + max_blocks) {
                        if let Ok(Some(block)) = blockchain_db.get_block_by_height(height).await {
                            let block_hash = block.hash.clone();
                            inventory.push(InventoryItem {
                                item_type: InventoryType::Block,
                                hash: block_hash.clone(),
                            });
                            
                            // Stop if we've reached the hash_stop
                            if let Some(ref stop_hash) = hash_stop {
                                if block_hash == *stop_hash {
                                    break;
                                }
                            }
                        } else {
                            break; // No more blocks
                        }
                    }
                    
                    // TODO: Send Inv message back to peer with the inventory
                    info!("Sending inventory of {} blocks to {}", inventory.len(), peer_addr);
                }
            },
            NetworkMessage::Inv { inventory } => {
                info!("Received inventory of {} items from {}", inventory.len(), peer_addr);
                
                // Check which blocks/transactions we need and request them
                let mut needed_blocks = Vec::new();
                
                for item in inventory {
                    match item.item_type {
                        InventoryType::Block => {
                            // Check if we have this block
                            if blockchain_db.get_block_by_hash(&item.hash).await.is_err() {
                                needed_blocks.push(item);
                            }
                        },
                        InventoryType::Transaction => {
                            // TODO: Check transaction pool and request if needed
                        },
                        _ => {}
                    }
                }
                
                if !needed_blocks.is_empty() {
                    // TODO: Send GetData message to request the blocks we need
                    info!("Requesting {} blocks from {}", needed_blocks.len(), peer_addr);
                }
            },
            NetworkMessage::GetData { inventory } => {
                info!("Received getdata request for {} items from {}", inventory.len(), peer_addr);
                
                // Send the requested blocks/transactions
                for item in inventory {
                    match item.item_type {
                        InventoryType::Block => {
                            if let Ok(Some(block)) = blockchain_db.get_block_by_hash(&item.hash).await {
                                // TODO: Send Block message back to peer
                                info!("Sending block {} to {}", block.hash, peer_addr);
                            }
                        },
                        InventoryType::Transaction => {
                            // TODO: Send transaction if we have it
                        },
                        _ => {}
                    }
                }
            },
            NetworkMessage::GetHeaders { version, block_locator_hashes, hash_stop } => {
                info!("Received getheaders request from {} (locator hashes: {})", peer_addr, block_locator_hashes.len());
                
                // Find fork point and send headers
                if let Ok(start_height) = Self::find_fork_point(blockchain_db, &block_locator_hashes).await {
                    let mut headers = Vec::new();
                    let max_headers = 2000; // Bitcoin protocol limit
                    
                    for height in (start_height + 1)..=(start_height + max_headers) {
                        if let Ok(Some(block)) = blockchain_db.get_block_by_height(height).await {                            headers.push(BlockHeader {
                                hash: block.hash.clone(),
                                previous_hash: block.previous_hash,
                                height: block.height,
                                timestamp: block.timestamp,
                                merkle_root: block.merkle_root,
                                nonce: block.nonce,
                                difficulty: block.difficulty as f64,
                            });
                            
                            if let Some(ref stop_hash) = hash_stop {
                                if block.hash == *stop_hash {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    
                    // TODO: Send Headers message back to peer
                    info!("Sending {} headers to {}", headers.len(), peer_addr);
                }
            },
            NetworkMessage::Headers { headers } => {
                info!("Received {} headers from {}", headers.len(), peer_addr);
                
                // Process headers and request blocks if needed
                for header in headers {
                    // Check if we have this block
                    if blockchain_db.get_block_by_hash(&header.hash).await.is_err() {
                        // We need this block - add to download queue
                        // TODO: Implement block download queue and request the full block
                        info!("Need to download block {} at height {}", header.hash, header.height);
                    }
                }
            },
            NetworkMessage::NewBlock { block } => {
                info!("Received new block {} (height: {}) from {}", block.hash, block.height, peer_addr);
                
                // Store the block
                if let Err(e) = blockchain_db.store_block(&block).await {
                    warn!("Failed to store received block: {}", e);
                } else {
                    let mut stats_guard = stats.write().await;
                    stats_guard.blocks_received += 1;
                    stats_guard.local_height = stats_guard.local_height.max(block.height);
                }
            },
            NetworkMessage::NewTransaction { transaction } => {
                info!("Received new transaction {} from {}", transaction.txid, peer_addr);
                
                // TODO: Add to transaction pool
                let mut stats_guard = stats.write().await;
                stats_guard.transactions_received += 1;
            },
            NetworkMessage::Version { version, services, timestamp, start_height, .. } => {
                info!("Received version message from {} (version: {}, height: {})", peer_addr, version, start_height);
                
                // Update peer info
                if let Some(peer) = peers.write().await.get_mut(&peer_addr) {
                    peer.version = Some(version.to_string());
                    peer.height = Some(start_height);
                }
                
                // TODO: Send Verack response
            },
            NetworkMessage::Verack => {
                info!("Received version acknowledgment from {}", peer_addr);
                // Version handshake complete
            },
            _ => {
                debug!("Received unhandled message type from {}", peer_addr);
            }
        }

        Ok(())
    }

    /// Find the fork point given block locator hashes
    async fn find_fork_point(
        blockchain_db: &Arc<AsyncBlockchainDatabase>,
        block_locator_hashes: &[String],
    ) -> AppResult<u64> {
        // Find the most recent block hash that we have
        for hash in block_locator_hashes {
            if let Ok(Some(block)) = blockchain_db.get_block_by_hash(hash).await {
                return Ok(block.height);
            }
        }
        
        // If no common block found, start from genesis
        Ok(0)
    }    /// Handle individual peer connection
    async fn handle_peer_connection(
        _stream: TcpStream,
        addr: SocketAddr,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
        _message_sender: mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>,
    ) {
        info!("Handling peer connection from {}", addr);
        
        // TODO: Implement actual message reading/writing with the stream
        // For now, simulate some basic interaction
        
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // Remove peer on disconnect
        {
            let mut peers_guard = peers.write().await;
            peers_guard.remove(&addr);
        }
        
        info!("Peer {} disconnected", addr);
    }

    /// Peer discovery loop
    async fn peer_discovery_loop(
        known_addresses: Arc<RwLock<HashSet<PeerAddress>>>,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
        message_sender: mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>,
        is_running: Arc<RwLock<bool>>,
    ) {
        let mut interval = interval(Duration::from_secs(60)); // Try discovery every minute

        loop {
            interval.tick().await;
            
            // Check if service is still running
            {
                let running = is_running.read().await;
                if !*running {
                    break;
                }
            }

            // Try to connect to known addresses
            let addresses: Vec<PeerAddress> = {
                let known = known_addresses.read().await;
                known.iter().take(5).cloned().collect() // Try up to 5 addresses
            };

            for addr in addresses {
                let current_peers = peers.read().await.len();
                if current_peers >= 8 { // Max 8 connections
                    break;
                }

                let socket_addr = SocketAddr::new(addr.ip, addr.port);
                
                // Skip if already connected
                if peers.read().await.contains_key(&socket_addr) {
                    continue;
                }

                // Try to connect
                tokio::spawn(Self::try_connect_to_peer(
                    socket_addr,
                    Arc::clone(&peers),
                    message_sender.clone(),
                ));
            }
        }
    }

    /// Try to connect to a peer
    async fn try_connect_to_peer(
        addr: SocketAddr,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
        message_sender: mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>,
    ) {
        debug!("Attempting to connect to peer {}", addr);

        match timeout(Duration::from_secs(10), TcpStream::connect(addr)).await {
            Ok(Ok(stream)) => {
                info!("Successfully connected to peer {}", addr);
                
                let peer_connection = PeerConnection {
                    address: PeerAddress {
                        ip: addr.ip(),
                        port: addr.port(),
                        last_seen: Self::current_timestamp(),
                        services: 0,
                    },
                    connected_at: Self::current_timestamp(),
                    last_ping: 0,
                    version: None,
                    height: None,
                    is_outbound: true,
                };

                // Add peer to connections
                {
                    let mut peers_guard = peers.write().await;
                    peers_guard.insert(addr, peer_connection);
                }

                // Handle this connection
                Self::handle_peer_connection(stream, addr, peers, message_sender).await;
            },
            Ok(Err(e)) => {
                debug!("Failed to connect to peer {}: {}", addr, e);
            },
            Err(_) => {
                debug!("Connection timeout to peer {}", addr);
            }
        }
    }

    /// Periodic maintenance tasks
    async fn periodic_tasks(
        peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
        stats: Arc<RwLock<NetworkStats>>,
        blockchain_db: Arc<AsyncBlockchainDatabase>,
    ) {
        let mut interval = interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            // Update statistics
            {
                let peers_guard = peers.read().await;
                let mut stats_guard = stats.write().await;
                
                stats_guard.connected_peers = peers_guard.len() as u32;
                
                // Update local height
                if let Ok(height) = blockchain_db.get_block_height().await {
                    stats_guard.local_height = height;
                }
            }

            // TODO: Send periodic pings to peers
            // TODO: Clean up stale peer connections
            // TODO: Request missing blocks if behind network height
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        self.stats.read().await.clone()
    }

    /// Get connected peers
    pub async fn get_peers(&self) -> Vec<PeerConnection> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Broadcast a message to all connected peers
    pub async fn broadcast_message(&self, message: NetworkMessage) -> AppResult<()> {
        let peers = self.peers.read().await;
        
        if let Some(ref sender) = self.message_sender {
            for addr in peers.keys() {
                if let Err(e) = sender.send((*addr, message.clone())) {
                    warn!("Failed to send message to peer {}: {}", addr, e);
                }
            }
        }

        Ok(())
    }

    /// Broadcast a new block to the network
    pub async fn broadcast_block(&self, block: Block) -> AppResult<()> {
        info!("Broadcasting new block {} to network", block.hash);
        self.broadcast_message(NetworkMessage::NewBlock { block }).await
    }

    /// Broadcast a new transaction to the network
    pub async fn broadcast_transaction(&self, transaction: Transaction) -> AppResult<()> {
        info!("Broadcasting new transaction {} to network", transaction.txid);
        self.broadcast_message(NetworkMessage::NewTransaction { transaction }).await
    }    /// Request blocks using Bitcoin-style getblocks message
    pub async fn request_blocks(&self, start_height: u64, _end_height: Option<u64>) -> AppResult<()> {
        info!("Requesting blocks starting from height {}", start_height);
        
        // Get block locator hashes (most recent blocks we have)
        let block_locator_hashes = self.get_block_locator_hashes(start_height).await?;
        
        let message = NetworkMessage::GetBlocks {
            version: 1,
            block_locator_hashes,
            hash_stop: None, // Request all available blocks
        };
        
        // Send to all connected peers
        self.broadcast_message(message).await?;
        
        Ok(())
    }

    /// Request specific blocks by hash using getdata
    pub async fn request_blocks_by_hash(&self, block_hashes: Vec<String>) -> AppResult<()> {
        info!("Requesting {} specific blocks by hash", block_hashes.len());
        
        let inventory: Vec<InventoryItem> = block_hashes
            .into_iter()
            .map(|hash| InventoryItem {
                item_type: InventoryType::Block,
                hash,
            })
            .collect();
        
        let message = NetworkMessage::GetData { inventory };
        self.broadcast_message(message).await?;
        
        Ok(())
    }

    /// Request block headers for headers-first sync
    pub async fn request_headers(&self, start_height: u64) -> AppResult<()> {
        info!("Requesting block headers starting from height {}", start_height);
        
        let block_locator_hashes = self.get_block_locator_hashes(start_height).await?;
        
        let message = NetworkMessage::GetHeaders {
            version: 1,
            block_locator_hashes,
            hash_stop: None,
        };
        
        self.broadcast_message(message).await?;
        
        Ok(())
    }

    /// Get block locator hashes for sync requests (Bitcoin protocol)
    async fn get_block_locator_hashes(&self, start_height: u64) -> AppResult<Vec<String>> {
        let mut locator_hashes = Vec::new();
        let mut step = 1u64;
        let mut height = start_height;
        
        // Add recent blocks with exponential backoff
        while height > 0 && locator_hashes.len() < 10 {
            if let Ok(Some(block)) = self.blockchain_db.get_block_by_height(height).await {
                locator_hashes.push(block.hash);
            }
            
            if height <= step {
                break;
            }
            height -= step;
            
            // Exponential backoff for older blocks
            if locator_hashes.len() >= 10 {
                step *= 2;
            }
        }
        
        // Always include genesis block
        if let Ok(Some(genesis)) = self.blockchain_db.get_block_by_height(0).await {
            if !locator_hashes.contains(&genesis.hash) {
                locator_hashes.push(genesis.hash);
            }
        }
        
        Ok(locator_hashes)
    }    /// Perform initial blockchain sync with peers (with development stub)
    pub async fn sync_blockchain(&self) -> AppResult<()> {
        info!("Starting blockchain synchronization");
        
        let local_height = self.blockchain_db.get_block_height().await.unwrap_or(0);
        let stats = self.stats.read().await;
        let network_height = stats.network_height;
        drop(stats);
        
        // Development mode: if local blockchain is empty, create stub data
        if local_height == 0 {
            info!("Local blockchain is empty, creating development stub with fake transactions");
            
            match self.create_development_blockchain_stub().await {
                Ok(stub_blocks) => {
                    info!("Generated {} stub blocks, storing in local database", stub_blocks.len());
                    
                    // Store all stub blocks in the database
                    for block in stub_blocks {
                        if let Err(e) = self.blockchain_db.store_block(&block).await {
                            error!("Failed to store stub block {}: {}", block.height, e);
                        } else {
                            info!("Stored stub block {} with {} transactions", block.height, block.transactions.len());
                        }
                    }
                      // Update network stats to reflect the new local height
                    let mut stats_guard = self.stats.write().await;
                    stats_guard.local_height = stats_guard.local_height.max(30); // We created 31 blocks (0-30)
                    stats_guard.network_height = stats_guard.network_height.max(30); // Simulate network having these blocks
                    drop(stats_guard);
                    
                    info!("Development blockchain stub created and stored successfully");
                    return Ok(());
                },
                Err(e) => {
                    error!("Failed to create development blockchain stub: {}", e);
                    // Continue with normal sync process as fallback
                }
            }
        }
        
        if network_height <= local_height {
            info!("Blockchain already up to date (local: {}, network: {})", local_height, network_height);
            return Ok(());
        }
        
        info!("Syncing blockchain: local height {} -> network height {}", local_height, network_height);
        
        // Use headers-first sync for efficiency
        self.request_headers(local_height + 1).await?;
        
        // TODO: After receiving headers, request the actual block data
        // This would be done in the message handler when we receive Headers response
        
        Ok(())
    }

    /// Handle incoming block inventory (inv message)
    async fn handle_block_inventory(&self, inventory: Vec<InventoryItem>) -> AppResult<()> {
        let block_hashes: Vec<String> = inventory
            .into_iter()
            .filter(|item| matches!(item.item_type, InventoryType::Block))
            .map(|item| item.hash)
            .collect();
        
        if !block_hashes.is_empty() {
            info!("Received inventory for {} blocks", block_hashes.len());
            
            // Check which blocks we don't have and request them
            let mut needed_hashes = Vec::new();
            for hash in block_hashes {
                if self.blockchain_db.get_block_by_hash(&hash).await.is_err() {
                    needed_hashes.push(hash);
                }
            }
            
            if !needed_hashes.is_empty() {
                info!("Requesting {} missing blocks", needed_hashes.len());
                self.request_blocks_by_hash(needed_hashes).await?;
            }
        }
        
        Ok(())
    }

    /// Send block inventory to peers (announce new blocks)
    pub async fn announce_new_block(&self, block_hash: String) -> AppResult<()> {
        let inventory = vec![InventoryItem {
            item_type: InventoryType::Block,
            hash: block_hash,
        }];
        
        let message = NetworkMessage::Inv { inventory };
        self.broadcast_message(message).await?;
        
        Ok(())
    }

    /// Create development blockchain stub with fake transactions from existing wallets
    async fn create_development_blockchain_stub(&self) -> AppResult<Vec<Block>> {
        info!("Creating development blockchain stub with fake transactions");
        
        // Get existing wallet addresses from config (in a real implementation)
        let wallet_addresses = self.get_existing_wallet_addresses().await;
          if wallet_addresses.is_empty() {
            warn!("No wallet addresses found, creating generic addresses for testing");
            return Ok(self.create_generic_test_blocks().await);        }
        
        // Use the comprehensive blockchain creation for more realistic data
        self.create_comprehensive_development_blockchain(&wallet_addresses).await    }    /// Get existing wallet addresses from app config
    async fn get_existing_wallet_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        
        // Try to get wallet addresses from app handle if available
        if let Some(ref app_handle) = self.app_handle {            // Try to get addresses from config as fallback
            if let Some(config_manager) = app_handle.try_state::<crate::config::ConfigManager>() {
                let config = config_manager.get_config();
                for wallet_info in &config.wallets {
                    addresses.extend(wallet_info.addresses.clone());
                }
                
                if !addresses.is_empty() {
                    info!("Found {} wallet addresses from config", addresses.len());
                    return addresses;
                }
            }
        }
        
        // Fallback to sample addresses for development
        warn!("No wallet addresses found in app state, using sample addresses for testing");
        vec![
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
            "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string(),
            "bc1qrp33g0w5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3".to_string(),
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kw508d6qejxtdg4y5r3zarvary0c5xw7kw5rljs90".to_string(),
            "bc1qm34lsc65zpw79lxes69zkqmk6ee3ewf0j77s3h".to_string(),
        ]
    }
    
    /// Create generic test blocks when no wallet addresses are found
    async fn create_generic_test_blocks(&self) -> Vec<Block> {
        info!("Creating generic test blocks");
        
        let test_addresses = vec![
            "test1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
            "test1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string(),
            "test1qrp33g0w5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3".to_string(),
        ];
        
        let mut blocks = Vec::new();
        let current_timestamp = 1640995200;
        
        // Create just genesis block for now
        let genesis_block = Block {
            height: 0,
            hash: "00000000001234567890abcdef000000000000000000000000000000000000".to_string(),
            previous_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            timestamp: current_timestamp,
            nonce: 0,
            difficulty: 1,
            transactions: vec![self.create_coinbase_transaction(0, &test_addresses[0], 5000000000).await],
            merkle_root: "abc123def456".to_string(),
        };
        blocks.push(genesis_block);
        
        blocks
    }
    
    /// Create a coinbase transaction (mining reward)
    async fn create_coinbase_transaction(&self, height: u64, recipient: &str, amount: u64) -> Transaction {
        Transaction {
            txid: format!("coinbase{:016x}{:08x}", height, amount),
            inputs: vec![TransactionInput {
                previous_txid: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                previous_output_index: 0xffffffff,
                script_sig: format!("coinbase_height_{}", height),
                sequence: 0xffffffff,
            }],
            outputs: vec![TransactionOutput {
                value: amount,
                script_pubkey: format!("OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG", recipient),
                address: recipient.to_string(),
            }],
            timestamp: 1640995200 + (height * 600),
            fee: 0,
        }
    }
      /// Create a regular transaction between two addresses
    async fn create_regular_transaction(
        &self,
        height: u64,
        tx_index: u64,
        from_addr: &str,
        to_addr: &str,
        amount: u64,
        timestamp: u64,
    ) -> Transaction {
        let fee = 1000 + (height * 100); // Gradually increasing fees
        let change = amount / 10; // 10% change back to sender
        
        Transaction {
            txid: format!("tx{:08x}{:04x}{:08x}", height, tx_index, amount),
            inputs: vec![TransactionInput {
                previous_txid: format!("prev{:08x}{:04x}", height.saturating_sub(1), tx_index),
                previous_output_index: 0,
                script_sig: format!("signature_from_{}", from_addr.chars().take(10).collect::<String>()),
                sequence: 0xffffffff,
            }],
            outputs: vec![
                // Main payment to recipient
                TransactionOutput {
                    value: amount,
                    script_pubkey: format!("OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG", 
                                          to_addr.chars().take(20).collect::<String>()),
                    address: to_addr.to_string(),
                },
                // Change back to sender
                TransactionOutput {
                    value: change,
                    script_pubkey: format!("OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG", 
                                          from_addr.chars().take(20).collect::<String>()),
                    address: from_addr.to_string(),
                }
            ],
            timestamp,
            fee,
        }
    }

    /// Create a more comprehensive development blockchain with varied transaction patterns
    async fn create_comprehensive_development_blockchain(&self, wallet_addresses: &[String]) -> AppResult<Vec<Block>> {
        info!("Creating comprehensive development blockchain with {} wallet addresses", wallet_addresses.len());
        
        let mut blocks = Vec::new();
        let mut current_timestamp = 1640995200; // Jan 1, 2022
        let mut wallet_balances: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        
        // Initialize wallet balances
        for addr in wallet_addresses {
            wallet_balances.insert(addr.clone(), 0);
        }
        
        // Create genesis block
        let genesis_recipient = &wallet_addresses[0];
        let genesis_block = Block {
            height: 0,
            hash: "00000000001234567890abcdef000000000000000000000000000000000000".to_string(),
            previous_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            timestamp: current_timestamp,
            nonce: 0,
            difficulty: 1,
            transactions: vec![self.create_coinbase_transaction(0, genesis_recipient, 5000000000).await],
            merkle_root: "abc123def456".to_string(),
        };
        
        // Update balance for genesis
        *wallet_balances.get_mut(genesis_recipient).unwrap() += 5000000000;
        blocks.push(genesis_block);
        
        // Create blocks with varied transaction patterns
        for height in 1..=30 {
            current_timestamp += 600; // 10 minutes between blocks
            let mut transactions = Vec::new();
            
            // Coinbase transaction (mining reward)
            let coinbase_recipient = &wallet_addresses[height as usize % wallet_addresses.len()];
            let coinbase_amount = 5000000000 - (height * 5000000); // Decreasing mining rewards
            transactions.push(self.create_coinbase_transaction(height, coinbase_recipient, coinbase_amount).await);
            *wallet_balances.get_mut(coinbase_recipient).unwrap() += coinbase_amount;
            
            // Regular transactions with realistic patterns
            match height % 5 {
                0 => {
                    // Large transactions (simulating business payments)
                    self.add_large_transactions(&mut transactions, wallet_addresses, &mut wallet_balances, height, current_timestamp).await;
                },
                1 | 2 => {
                    // Small everyday transactions
                    self.add_small_transactions(&mut transactions, wallet_addresses, &mut wallet_balances, height, current_timestamp).await;
                },
                3 => {
                    // Exchange-like transactions (multiple small outputs)
                    self.add_exchange_transactions(&mut transactions, wallet_addresses, &mut wallet_balances, height, current_timestamp).await;
                },
                4 => {
                    // Mixed transaction types
                    self.add_mixed_transactions(&mut transactions, wallet_addresses, &mut wallet_balances, height, current_timestamp).await;
                },
                _ => {}
            }
              let block = Block {
                height,
                hash: format!("000000000{:09x}{:08x}0000000000000000000000000000000", height, current_timestamp),
                previous_hash: blocks.last().unwrap().hash.clone(),
                timestamp: current_timestamp,
                nonce: height * 1337 + (height % 7) * 999,
                difficulty: 1 + (height / 5),
                merkle_root: format!("merkle{:016x}{:04x}", height, transactions.len()),
                transactions,
            };
            
            blocks.push(block);
        }
        
        // Log final wallet balances for verification
        info!("Final wallet balances after {} blocks:", blocks.len());
        for (addr, balance) in &wallet_balances {
            info!("  {}: {} satoshis ({:.8} BTC)", 
                  addr.chars().take(20).collect::<String>(), 
                  balance, 
                  *balance as f64 / 100_000_000.0);
        }
        
        Ok(blocks)
    }

    /// Add large business-style transactions
    async fn add_large_transactions(
        &self,
        transactions: &mut Vec<Transaction>,
        wallet_addresses: &[String],
        _wallet_balances: &mut std::collections::HashMap<String, u64>,
        height: u64,
        timestamp: u64,
    ) {
        let amounts = [500000000, 1000000000, 2000000000]; // 5, 10, 20 BTC
        for (i, &amount) in amounts.iter().enumerate() {
            let from_idx = (height + i as u64) % wallet_addresses.len() as u64;
            let to_idx = (height + i as u64 + 1) % wallet_addresses.len() as u64;
            
            if from_idx != to_idx {
                transactions.push(self.create_regular_transaction(
                    height,
                    i as u64,
                    &wallet_addresses[from_idx as usize],
                    &wallet_addresses[to_idx as usize],
                    amount,
                    timestamp + (i as u64 * 60),
                ).await);
            }
        }
    }

    /// Add small everyday transactions  
    async fn add_small_transactions(
        &self,
        transactions: &mut Vec<Transaction>,
        wallet_addresses: &[String],
        _wallet_balances: &mut std::collections::HashMap<String, u64>,
        height: u64,
        timestamp: u64,
    ) {
        let amounts = [50000, 100000, 250000, 500000]; // 0.0005 to 0.005 BTC
        for (i, &amount) in amounts.iter().enumerate() {
            let from_idx = (height + i as u64 + 2) % wallet_addresses.len() as u64;
            let to_idx = (height + i as u64 + 3) % wallet_addresses.len() as u64;
            
            if from_idx != to_idx {
                transactions.push(self.create_regular_transaction(
                    height,
                    i as u64 + 10,
                    &wallet_addresses[from_idx as usize],
                    &wallet_addresses[to_idx as usize],
                    amount,
                    timestamp + (i as u64 * 90),
                ).await);
            }
        }
    }

    /// Add exchange-style transactions (one input, multiple outputs)
    async fn add_exchange_transactions(
        &self,
        transactions: &mut Vec<Transaction>,
        wallet_addresses: &[String],
        _wallet_balances: &mut std::collections::HashMap<String, u64>,
        height: u64,
        timestamp: u64,
    ) {
        // Create a transaction that sends to multiple addresses (like an exchange payout)
        let from_addr = &wallet_addresses[height as usize % wallet_addresses.len()];
        let amounts = [1000000, 2000000, 5000000]; // Various small amounts
        
        let mut outputs = Vec::new();
        for (i, &amount) in amounts.iter().enumerate() {
            let to_idx = (height + i as u64 + 1) % wallet_addresses.len() as u64;
            outputs.push(TransactionOutput {
                value: amount,
                script_pubkey: format!("OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG", 
                                      wallet_addresses[to_idx as usize].chars().take(20).collect::<String>()),
                address: wallet_addresses[to_idx as usize].clone(),
            });
        }
        
        let tx = Transaction {
            txid: format!("exchange_tx_{:08x}", height),
            inputs: vec![TransactionInput {
                previous_txid: format!("exchange_input_{:08x}", height.saturating_sub(1)),
                previous_output_index: 0,
                script_sig: format!("exchange_signature_{}", from_addr.chars().take(10).collect::<String>()),
                sequence: 0xffffffff,
            }],
            outputs,
            timestamp: timestamp + 200,
            fee: 5000,
        };
        
        transactions.push(tx);
    }

    /// Add mixed transaction types
    async fn add_mixed_transactions(
        &self,
        transactions: &mut Vec<Transaction>,
        wallet_addresses: &[String],
        _wallet_balances: &mut std::collections::HashMap<String, u64>,
        height: u64,
        timestamp: u64,
    ) {
        // Mix of different transaction sizes
        let patterns = [
            (10000000, "large_purchase"),    // 0.1 BTC
            (1000000, "medium_payment"),     // 0.01 BTC  
            (100000, "small_tip"),           // 0.001 BTC
        ];
        
        for (i, (amount, tx_type)) in patterns.iter().enumerate() {
            let from_idx = (height + i as u64) % wallet_addresses.len() as u64;
            let to_idx = (height + i as u64 + 2) % wallet_addresses.len() as u64;
            
            if from_idx != to_idx {
                let mut tx = self.create_regular_transaction(
                    height,
                    i as u64 + 20,
                    &wallet_addresses[from_idx as usize],
                    &wallet_addresses[to_idx as usize],
                    *amount,
                    timestamp + (i as u64 * 120),
                ).await;
                
                // Add transaction type to the ID for identification
                tx.txid = format!("{}_{:08x}_{:04x}", tx_type, height, amount);
                transactions.push(tx);
            }
        }
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            connected_peers: 0,
            total_known_peers: 0,
            blocks_received: 0,
            transactions_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            network_height: 0,
            local_height: 0,
        }
    }
}

/// Thread-safe wrapper for NetworkService
pub struct AsyncNetworkService {
    inner: Arc<RwLock<NetworkService>>,
}

impl AsyncNetworkService {
    /// Create new async network service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>, port: Option<u16>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(NetworkService::new(blockchain_db, port))),
        }
    }

    /// Initialize the service
    pub async fn initialize(&self, app_handle: AppHandle) -> AppResult<()> {
        let mut service = self.inner.write().await;
        service.initialize(app_handle).await
    }

    /// Start the service
    pub async fn start(&self) -> AppResult<()> {
        let mut service = self.inner.write().await;
        service.start().await
    }

    /// Stop the network service
    pub async fn stop(&self) -> AppResult<()> {
        let mut service = self.inner.write().await;
        service.stop().await
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        let service = self.inner.read().await;
        service.get_stats().await
    }

    /// Get connected peers
    pub async fn get_peers(&self) -> Vec<PeerConnection> {
        let service = self.inner.read().await;
        service.get_peers().await
    }

    /// Broadcast a message to all peers
    pub async fn broadcast_message(&self, message: NetworkMessage) -> AppResult<()> {
        let service = self.inner.read().await;
        service.broadcast_message(message).await
    }

    /// Broadcast a new block
    pub async fn broadcast_block(&self, block: Block) -> AppResult<()> {
        let service = self.inner.read().await;
        service.broadcast_block(block).await
    }

    /// Broadcast a new transaction
    pub async fn broadcast_transaction(&self, transaction: Transaction) -> AppResult<()> {
        let service = self.inner.read().await;
        service.broadcast_transaction(transaction).await
    }

    /// Request blocks using Bitcoin-style protocol
    pub async fn request_blocks(&self, start_height: u64, end_height: Option<u64>) -> AppResult<()> {
        let service = self.inner.read().await;
        service.request_blocks(start_height, end_height).await
    }

    /// Request specific blocks by hash
    pub async fn request_blocks_by_hash(&self, block_hashes: Vec<String>) -> AppResult<()> {
        let service = self.inner.read().await;
        service.request_blocks_by_hash(block_hashes).await
    }

    /// Request block headers
    pub async fn request_headers(&self, start_height: u64) -> AppResult<()> {
        let service = self.inner.read().await;
        service.request_headers(start_height).await
    }

    /// Perform blockchain synchronization
    pub async fn sync_blockchain(&self) -> AppResult<()> {
        let service = self.inner.read().await;
        service.sync_blockchain().await
    }

    /// Announce a new block to the network
    pub async fn announce_new_block(&self, block_hash: String) -> AppResult<()> {
        let service = self.inner.read().await;
        service.announce_new_block(block_hash).await
    }
}
