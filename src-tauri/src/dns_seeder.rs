//! DNS-based peer discovery for BradCoin network
//! Implements Bitcoin-style DNS seeding for initial peer discovery

use crate::network_constants::{get_dns_seeds, create_peer_address, NODE_NETWORK};
use crate::network_service::PeerAddress;
use log::{debug, info, warn};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::time::timeout;

/// DNS resolver for peer discovery
pub struct DnsSeeder {
    use_bitcoin_network: bool,
    timeout_duration: Duration,
}

impl DnsSeeder {
    /// Create a new DNS seeder
    pub fn new(use_bitcoin_network: bool) -> Self {
        Self {
            use_bitcoin_network,
            timeout_duration: Duration::from_secs(10),
        }
    }

    /// Discover peers using DNS seeds
    pub async fn discover_peers(&self) -> Vec<PeerAddress> {
        let dns_seeds = get_dns_seeds(self.use_bitcoin_network);
        let mut discovered_peers = Vec::new();

        if dns_seeds.is_empty() {
            debug!("No DNS seeds available for current network configuration");
            return discovered_peers;
        }

        info!("Starting DNS peer discovery with {} seeds", dns_seeds.len());

        for seed in dns_seeds {
            match self.resolve_seed(seed).await {
                Ok(mut peers) => {
                    info!("Discovered {} peers from DNS seed: {}", peers.len(), seed);
                    discovered_peers.append(&mut peers);
                }
                Err(e) => {
                    warn!("Failed to resolve DNS seed {}: {}", seed, e);
                }
            }
        }

        // Remove duplicates based on IP:Port combination
        discovered_peers.sort_by(|a, b| (a.ip, a.port).cmp(&(b.ip, b.port)));
        discovered_peers.dedup_by(|a, b| a.ip == b.ip && a.port == b.port);

        info!("Total unique peers discovered via DNS: {}", discovered_peers.len());
        discovered_peers
    }

    /// Resolve a single DNS seed
    async fn resolve_seed(&self, seed: &str) -> Result<Vec<PeerAddress>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Resolving DNS seed: {}", seed);

        // Bitcoin nodes typically listen on port 8333
        let address_with_port = format!("{}:8333", seed);
        
        // Use timeout for DNS resolution
        let addresses: Vec<SocketAddr> = timeout(self.timeout_duration, async {
            tokio::task::spawn_blocking(move || {
                address_with_port.to_socket_addrs()
                    .map(|addrs| addrs.collect::<Vec<_>>())
            }).await
        }).await???;

        let mut peer_addresses = Vec::new();
        
        for addr in addresses {
            let peer_addr = create_peer_address(
                addr.ip(),
                addr.port(),
                NODE_NETWORK, // Assume full node capabilities
            );
            peer_addresses.push(peer_addr);

            // Limit the number of addresses per seed to avoid overwhelming
            if peer_addresses.len() >= 25 {
                break;
            }
        }

        debug!("Resolved {} addresses from seed {}", peer_addresses.len(), seed);
        Ok(peer_addresses)
    }

    /// Perform A record lookup for additional discovery
    pub async fn resolve_a_records(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Performing A record lookup for: {}", hostname);

        let addresses: Vec<IpAddr> = timeout(self.timeout_duration, async {
            tokio::task::spawn_blocking({
                let hostname = hostname.to_string();
                move || {
                    use std::net::ToSocketAddrs;
                    format!("{}:0", hostname)
                        .to_socket_addrs()
                        .map(|addrs| addrs.map(|addr| addr.ip()).collect::<Vec<_>>())
                }
            }).await
        }).await???;

        debug!("Found {} IP addresses for {}", addresses.len(), hostname);
        Ok(addresses)
    }

    /// Validate that a peer address is reasonable
    pub fn is_valid_peer_address(&self, addr: &PeerAddress) -> bool {
        // Check for private/local addresses that shouldn't be used in production
        match addr.ip {
            IpAddr::V4(ipv4) => {
                // Allow localhost only in development mode
                if ipv4.is_loopback() && self.use_bitcoin_network {
                    return false;
                }
                // Reject private networks in production
                if self.use_bitcoin_network && (ipv4.is_private() || ipv4.is_unspecified()) {
                    return false;
                }
                // Reject reserved addresses
                if ipv4.is_multicast() || ipv4.is_broadcast() {
                    return false;
                }
            }
            IpAddr::V6(ipv6) => {
                // Allow localhost only in development mode  
                if ipv6.is_loopback() && self.use_bitcoin_network {
                    return false;
                }
                // Reject unspecified or multicast
                if ipv6.is_unspecified() || ipv6.is_multicast() {
                    return false;
                }
            }
        }

        // Check port range
        if addr.port == 0 || addr.port > 65535 {
            return false;
        }

        true
    }

    /// Filter discovered peers for validity
    pub fn filter_valid_peers(&self, peers: Vec<PeerAddress>) -> Vec<PeerAddress> {
        let initial_count = peers.len();
        let filtered: Vec<PeerAddress> = peers
            .into_iter()
            .filter(|addr| self.is_valid_peer_address(addr))
            .collect();

        if filtered.len() != initial_count {
            debug!("Filtered {} invalid addresses, {} remaining", 
                   initial_count - filtered.len(), filtered.len());
        }

        filtered
    }
}

/// Perform comprehensive peer discovery
pub async fn discover_network_peers(use_bitcoin_network: bool) -> Vec<PeerAddress> {
    let seeder = DnsSeeder::new(use_bitcoin_network);
    let discovered_peers = seeder.discover_peers().await;
    seeder.filter_valid_peers(discovered_peers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_seeder_creation() {
        let seeder = DnsSeeder::new(true);
        assert!(seeder.use_bitcoin_network);

        let seeder = DnsSeeder::new(false);
        assert!(!seeder.use_bitcoin_network);
    }

    #[test]
    fn test_address_validation() {
        let seeder = DnsSeeder::new(true); // Bitcoin network mode

        // Valid public IPv4
        let valid_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&valid_addr));

        // Invalid private IPv4 in Bitcoin mode
        let private_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(!seeder.is_valid_peer_address(&private_addr));

        // Localhost should be rejected in Bitcoin mode
        let localhost_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(!seeder.is_valid_peer_address(&localhost_addr));
    }

    #[test]
    fn test_development_mode_validation() {
        let seeder = DnsSeeder::new(false); // Development mode

        // Localhost should be allowed in development mode
        let localhost_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&localhost_addr));

        // Private addresses should be allowed in development mode
        let private_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&private_addr));
    }

    #[test]
    fn test_invalid_ports() {
        let seeder = DnsSeeder::new(true);

        // Port 0 should be invalid
        let zero_port = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            0,
            NODE_NETWORK,
        );
        assert!(!seeder.is_valid_peer_address(&zero_port));
    }

    #[tokio::test]
    async fn test_dns_discovery() {
        // This test requires internet connectivity
        let seeder = DnsSeeder::new(true);
        
        // Try to resolve a well-known Bitcoin DNS seed
        match seeder.resolve_seed("seed.bitcoin.sipa.be").await {
            Ok(peers) => {
                println!("Discovered {} peers from DNS seed", peers.len());
                assert!(!peers.is_empty());
            }
            Err(e) => {
                println!("DNS resolution failed (expected in some environments): {}", e);
                // Don't fail the test as this requires internet connectivity
            }
        }
    }
}
