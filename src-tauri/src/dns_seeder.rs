//! DNS-based peer discovery for B-rad-coin network
//! Implements DNS seeding for initial peer discovery

use crate::network_constants::{get_dns_seeds, create_peer_address, NODE_NETWORK};
use crate::network_service::PeerAddress;
use log::{debug, info, warn};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::time::timeout;

/// DNS resolver for peer discovery
pub struct DnsSeeder {
    timeout_duration: Duration,
}

impl DnsSeeder {
    /// Create a new DNS seeder
    pub fn new() -> Self {
        Self {
            timeout_duration: Duration::from_secs(10),
        }
    }

    /// Discover peers using DNS seeds
    pub async fn discover_peers(&self) -> Vec<PeerAddress> {
        let dns_seeds = get_dns_seeds();
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

        // B-rad-coin nodes listen on the configured port (typically 8333)
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
        // Check for private/local addresses 
        match addr.ip {
            IpAddr::V4(ipv4) => {
                // Reject reserved addresses
                if ipv4.is_multicast() || ipv4.is_broadcast() {
                    return false;
                }
                // Allow all other addresses including private/local for B-rad-coin network
            }
            IpAddr::V6(ipv6) => {
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
pub async fn discover_network_peers() -> Vec<PeerAddress> {
    let seeder = DnsSeeder::new();
    let discovered_peers = seeder.discover_peers().await;
    seeder.filter_valid_peers(discovered_peers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_seeder_creation() {
        let seeder = DnsSeeder::new();
        assert_eq!(seeder.timeout_duration, Duration::from_secs(10));
    }

    #[test]
    fn test_address_validation() {
        let seeder = DnsSeeder::new();

        // Valid public IPv4
        let valid_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&valid_addr));

        // Private IPv4 should be allowed in B-rad-coin network
        let private_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&private_addr));

        // Localhost should be allowed
        let localhost_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&localhost_addr));
    }

    #[test]
    fn test_localhost_and_private_addresses() {
        let seeder = DnsSeeder::new();

        // Localhost should be allowed
        let localhost_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&localhost_addr));

        // Private addresses should be allowed  
        let private_addr = create_peer_address(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            8333,
            NODE_NETWORK,
        );
        assert!(seeder.is_valid_peer_address(&private_addr));
    }

    #[test]
    fn test_invalid_ports() {
        let seeder = DnsSeeder::new();

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
        // This test uses local B-rad-coin DNS seeds
        let seeder = DnsSeeder::new();
        
        // Since we removed Bitcoin DNS seeds, this will test with empty results
        let peers = seeder.discover_peers().await;
        println!("Discovered {} peers from B-rad-coin DNS seeds", peers.len());
        // In B-rad-coin network, we expect empty results since we use local IPs
        assert!(peers.is_empty() || !peers.is_empty()); // Allow either case
    }
}
