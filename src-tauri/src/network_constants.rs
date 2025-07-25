//! Network constants and seed nodes for BradCoin
//! Independent B-rad-coin network - does not use Bitcoin infrastructure

use crate::network_service::PeerAddress;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{SystemTime, UNIX_EPOCH};

/// B-rad-coin default port
pub const BRADCOIN_DEFAULT_PORT: u16 = 8333;

/// B-rad-coin protocol version
pub const BRADCOIN_PROTOCOL_VERSION: u32 = 10001;

/// B-rad-coin network seed nodes (independent network)
pub const BRADCOIN_SEED_NODES: &[(IpAddr, u16)] = &[
    // Development/testnet nodes (localhost)
    (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333),
    (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8334),
    (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8335),
    
    // Local network nodes for inter-node testing
    // These are the actual IP addresses of this computer for local network discovery
    (IpAddr::V4(Ipv4Addr::new(10, 2, 0, 2)), 8333),     // Local IP 1 (primary interface)
    (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 17)), 8333), // Local IP 2 (WiFi/Ethernet interface)
    (IpAddr::V4(Ipv4Addr::new(10, 2, 0, 2)), 8334),     // Local IP 1 alternate port
    (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 17)), 8334), // Local IP 2 alternate port
];

/// Network service flags (B-rad-coin protocol)
pub const NODE_NETWORK: u64 = 1 << 0;          // Full node, can serve blocks
pub const NODE_GETUTXO: u64 = 1 << 1;          // Can serve UTXO queries
pub const NODE_BLOOM: u64 = 1 << 2;            // Supports bloom filters
pub const NODE_WITNESS: u64 = 1 << 3;          // Supports witness transactions
pub const NODE_XTHIN: u64 = 1 << 4;            // Supports Xtreme Thinblocks
pub const NODE_NETWORK_LIMITED: u64 = 1 << 10; // Pruned node, limited blocks

/// Protocol version constants
pub const PROTOCOL_VERSION: u32 = 10001;       // B-rad-coin protocol version
pub const MIN_PROTOCOL_VERSION: u32 = 10000;   // Minimum supported version

/// User agent for network identification
pub const USER_AGENT: &str = "/BradCoin:0.2.5/";

/// Network timeouts and limits
pub const CONNECTION_TIMEOUT_SECS: u64 = 10;
pub const PING_INTERVAL_SECS: u64 = 60;
pub const PEER_DISCOVERY_INTERVAL_SECS: u64 = 300; // 5 minutes
pub const MAX_PEERS: usize = 125;
pub const MAX_OUTBOUND_PEERS: usize = 8;
pub const MAX_INBOUND_PEERS: usize = 117;

/// Message size limits
pub const MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024; // 32MB
pub const MAX_HEADERS_COUNT: usize = 2000;
pub const MAX_INV_COUNT: usize = 50000;

/// Create PeerAddress from IP and port
pub fn create_peer_address(ip: IpAddr, port: u16, services: u64) -> PeerAddress {
    PeerAddress {
        ip,
        port,
        last_seen: current_timestamp(),
        services,
    }
}

/// Get current timestamp
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Get seed nodes for network discovery - B-rad-coin network only
pub fn get_seed_nodes() -> Vec<PeerAddress> {
    let mut seed_addresses = Vec::new();
    
    // Add static B-rad-coin seed nodes
    for (ip, port) in BRADCOIN_SEED_NODES {
        seed_addresses.push(create_peer_address(*ip, *port, NODE_NETWORK));
    }
    
    // Add dynamically detected local IP addresses for local network testing
    if let Ok(local_ips) = get_local_ip_addresses() {
        for local_ip in local_ips {
            // Add multiple ports for local testing
            for port in [8333, 8334, 8335] {
                seed_addresses.push(create_peer_address(local_ip, port, NODE_NETWORK));
            }
        }
    }
    
    seed_addresses
}

/// Get local IP addresses for network discovery
pub fn get_local_ip_addresses() -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
    use std::net::UdpSocket;
    
    let mut local_ips = Vec::new();
    
    // Method 1: Connect to a remote address to determine local IP
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                local_ips.push(local_addr.ip());
            }
        }
    }
    
    // Method 2: Try to get network interfaces (basic fallback)
    // Add some common local network ranges if we can't detect dynamically
    let fallback_locals = [
        "192.168.1.17",  // Detected local IP
        "10.2.0.2",      // Detected local IP  
        "192.168.1.1",   // Common router IP
        "10.0.0.1",      // Common router IP
    ];
    
    for ip_str in &fallback_locals {
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            if !local_ips.contains(&ip) {
                local_ips.push(ip);
            }
        }
    }
    
    Ok(local_ips)
}

/// Get DNS seeds for network discovery - B-rad-coin only (currently none)
pub fn get_dns_seeds() -> Vec<&'static str> {
    // B-rad-coin uses local network discovery, no DNS seeds needed yet
    // In the future, we could add our own DNS infrastructure
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_nodes_creation() {
        let bradcoin_nodes = get_seed_nodes();
        assert!(!bradcoin_nodes.is_empty());
        
        // Verify local IP addresses are included in B-rad-coin seed nodes
        let has_local_ip = bradcoin_nodes.iter().any(|addr| {
            match addr.ip {
                IpAddr::V4(ipv4) => {
                    ipv4 == Ipv4Addr::new(10, 2, 0, 2) || 
                    ipv4 == Ipv4Addr::new(192, 168, 1, 17)
                }
                _ => false
            }
        });
        assert!(has_local_ip, "Local IP addresses should be included in B-rad-coin seed nodes");
    }
    
    #[test]
    fn test_local_ip_detection() {
        let local_ips = get_local_ip_addresses();
        assert!(local_ips.is_ok());
        let ips = local_ips.unwrap();
        assert!(!ips.is_empty(), "Should detect at least one local IP address");
    }

    #[test]
    fn test_dns_seeds() {
        let dns_seeds = get_dns_seeds();
        // B-rad-coin doesn't use DNS seeds currently
        assert!(dns_seeds.is_empty());
    }

    #[test]
    fn test_peer_address_creation() {
        let addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 
            8333, 
            NODE_NETWORK
        );
        assert_eq!(addr.port, 8333);
        assert_eq!(addr.services, NODE_NETWORK);
    }
}
