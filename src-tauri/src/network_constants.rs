//! Network constants and seed nodes for BradCoin
//! Based on Bitcoin's DNS seeds and network discovery

use crate::network_service::PeerAddress;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{SystemTime, UNIX_EPOCH};

/// Bitcoin default port
pub const BITCOIN_DEFAULT_PORT: u16 = 8333;

/// Bitcoin protocol version
pub const BITCOIN_PROTOCOL_VERSION: u32 = 70015;

/// Bitcoin mainnet DNS seed domains
/// These are the official Bitcoin DNS seeds used for peer discovery
pub const BITCOIN_DNS_SEEDS: &[&str] = &[
    "seed.bitcoin.sipa.be",
    "dnsseed.bluematt.me", 
    "dnsseed.bitcoin.dashjr.org",
    "seed.bitcoinstats.com",
    "seed.bitcoin.jonasschnelli.ch",
    "seed.btc.petertodd.org",
    "seed.bitcoin.sprovoost.nl",
    "dnsseed.emzy.de",
];

/// Testnet DNS seed domains
pub const TESTNET_DNS_SEEDS: &[&str] = &[
    "testnet-seed.bitcoin.jonasschnelli.ch",
    "seed.tbtc.petertodd.org",
    "seed.testnet.bitcoin.sprovoost.nl",
    "testnet-seed.bluematt.me",
];

/// Well-known Bitcoin node IP addresses (fallback)
/// These are long-running, reliable Bitcoin nodes
pub const BITCOIN_SEED_NODES: &[(IpAddr, u16)] = &[
    // Bitnodes.io known stable nodes
    (IpAddr::V4(Ipv4Addr::new(13, 107, 5, 53)), 8333),
    (IpAddr::V4(Ipv4Addr::new(18, 189, 156, 23)), 8333),
    (IpAddr::V4(Ipv4Addr::new(34, 215, 113, 31)), 8333),
    (IpAddr::V4(Ipv4Addr::new(47, 180, 14, 6)), 8333),
    (IpAddr::V4(Ipv4Addr::new(52, 1, 164, 206)), 8333),
    
    // IPv6 nodes
    (IpAddr::V6(Ipv6Addr::new(0x2001, 0x19f0, 0x6c01, 0x1c9c, 0x5400, 0x02ff, 0xfe71, 0x0bc6)), 8333),
    (IpAddr::V6(Ipv6Addr::new(0x2a01, 0x4f8, 0x212, 0x346c, 0, 0, 0, 2)), 8333),
];

/// BradCoin specific seed nodes (for when we have our own network)
pub const BRADCOIN_SEED_NODES: &[(IpAddr, u16)] = &[
    // Development/testnet nodes
    (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333),
    (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8334),
    (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8335),
];

/// Network service flags (Bitcoin protocol)
pub const NODE_NETWORK: u64 = 1 << 0;          // Full node, can serve blocks
pub const NODE_GETUTXO: u64 = 1 << 1;          // Can serve UTXO queries
pub const NODE_BLOOM: u64 = 1 << 2;            // Supports bloom filters
pub const NODE_WITNESS: u64 = 1 << 3;          // Supports witness transactions
pub const NODE_XTHIN: u64 = 1 << 4;            // Supports Xtreme Thinblocks
pub const NODE_NETWORK_LIMITED: u64 = 1 << 10; // Pruned node, limited blocks

/// Protocol version constants
pub const PROTOCOL_VERSION: u32 = 70015;
pub const MIN_PROTOCOL_VERSION: u32 = 31800;
pub const USER_AGENT: &str = "/BradCoin:0.2.3/";

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

/// Get seed nodes for network discovery
pub fn get_seed_nodes(use_bitcoin_network: bool) -> Vec<PeerAddress> {
    let nodes = if use_bitcoin_network {
        BITCOIN_SEED_NODES
    } else {
        BRADCOIN_SEED_NODES
    };
    
    nodes
        .iter()
        .map(|(ip, port)| create_peer_address(*ip, *port, NODE_NETWORK))
        .collect()
}

/// Get DNS seeds for network discovery
pub fn get_dns_seeds(use_bitcoin_network: bool) -> Vec<&'static str> {
    if use_bitcoin_network {
        BITCOIN_DNS_SEEDS.to_vec()
    } else {
        // For now, BradCoin uses the same discovery mechanism
        // In the future, we would have our own DNS seeds
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_nodes_creation() {
        let bitcoin_nodes = get_seed_nodes(true);
        assert!(!bitcoin_nodes.is_empty());
        
        let bradcoin_nodes = get_seed_nodes(false);
        assert!(!bradcoin_nodes.is_empty());
    }

    #[test]
    fn test_dns_seeds() {
        let bitcoin_dns = get_dns_seeds(true);
        assert!(!bitcoin_dns.is_empty());
        assert!(bitcoin_dns.contains(&"seed.bitcoin.sipa.be"));
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
