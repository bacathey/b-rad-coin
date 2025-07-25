//! Network Monitoring and Diagnostics Service
//! Provides comprehensive network health monitoring and diagnostic tools

use crate::network_service::{AsyncNetworkService, PeerConnection, PeerScore};
use crate::errors::*;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::interval;

/// Network diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnostics {
    pub connection_health: ConnectionHealth,
    pub peer_analysis: Vec<PeerAnalysis>,
    pub bandwidth_stats: BandwidthStats,
    pub sync_status: SyncStatus,
    pub network_issues: Vec<NetworkIssue>,
}

/// Connection health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHealth {
    pub total_peers: usize,
    pub healthy_peers: usize,
    pub average_ping: u64,
    pub connection_stability: f32, // 0.0-1.0
    pub overall_score: u8,         // 0-100
}

/// Individual peer analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAnalysis {
    pub address: String,
    pub score: i32,
    pub ping_ms: u64,
    pub blocks_contributed: u32,
    pub transactions_contributed: u32,
    pub uptime_percentage: f32,
    pub reliability_grade: String, // A, B, C, D, F
    pub issues: Vec<String>,
}

/// Bandwidth usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub average_message_size: f32,
    pub bandwidth_utilization: f32, // 0.0-1.0
}

/// Blockchain synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_synced: bool,
    pub local_height: u64,
    pub network_height: u64,
    pub blocks_behind: u64,
    pub sync_progress: f32, // 0.0-1.0
    pub estimated_sync_time: Option<u64>, // seconds
}

/// Network issue detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub recommendation: String,
    pub first_detected: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueCategory {
    Connectivity,
    Performance,
    Synchronization,
    PeerQuality,
    Security,
}

/// Network monitor service
pub struct NetworkMonitor {
    network_service: Option<AsyncNetworkService>,
    diagnostics_history: Arc<RwLock<Vec<NetworkDiagnostics>>>,
    bandwidth_tracker: Arc<RwLock<BandwidthTracker>>,
    issue_tracker: Arc<RwLock<HashMap<String, NetworkIssue>>>,
}

/// Bandwidth tracking helper
#[derive(Debug, Default)]
struct BandwidthTracker {
    bytes_sent: u64,
    bytes_received: u64,
    messages_sent: u64,
    messages_received: u64,
    last_reset: u64,
}

impl NetworkMonitor {
    /// Create new network monitor
    pub fn new() -> Self {
        Self {
            network_service: None,
            diagnostics_history: Arc::new(RwLock::new(Vec::new())),
            bandwidth_tracker: Arc::new(RwLock::new(BandwidthTracker::default())),
            issue_tracker: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set network service for monitoring
    pub fn set_network_service(&mut self, network_service: AsyncNetworkService) {
        self.network_service = Some(network_service);
    }

    /// Start monitoring loop
    pub async fn start_monitoring(&self) -> AppResult<()> {
        let mut interval = interval(Duration::from_secs(30)); // Monitor every 30 seconds
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.collect_diagnostics().await {
                warn!("Failed to collect network diagnostics: {}", e);
            }
            
            if let Err(e) = self.detect_issues().await {
                warn!("Failed to detect network issues: {}", e);
            }
        }
    }

    /// Collect comprehensive network diagnostics
    pub async fn collect_diagnostics(&self) -> AppResult<NetworkDiagnostics> {
        let connection_health = self.assess_connection_health().await?;
        let peer_analysis = self.analyze_peers().await?;
        let bandwidth_stats = self.get_bandwidth_stats().await;
        let sync_status = self.get_sync_status().await?;
        let network_issues = self.get_current_issues().await;

        let diagnostics = NetworkDiagnostics {
            connection_health,
            peer_analysis,
            bandwidth_stats,
            sync_status,
            network_issues,
        };

        // Store in history
        let mut history = self.diagnostics_history.write().await;
        history.push(diagnostics.clone());
        
        // Keep only last 100 diagnostic snapshots
        let history_len = history.len();
        if history_len > 100 {
            history.drain(0..history_len - 100);
        }

        Ok(diagnostics)
    }

    /// Assess overall connection health
    async fn assess_connection_health(&self) -> AppResult<ConnectionHealth> {
        if let Some(ref network) = self.network_service {
            let stats = network.get_stats().await;
            
            // Analyze peer health (placeholder - would need peer access)
            let total_peers = stats.connected_peers as usize;
            let healthy_peers = (total_peers as f32 * 0.8) as usize; // Assume 80% healthy for now
            let average_ping = 150; // Placeholder
            
            // Calculate connection stability based on peer count and uptime
            let stability = if total_peers >= 8 {
                0.9
            } else if total_peers >= 4 {
                0.7
            } else if total_peers >= 1 {
                0.5
            } else {
                0.0
            };

            // Overall score calculation
            let score = ((healthy_peers as f32 / total_peers.max(1) as f32) * 40.0
                + (stability * 30.0)
                + if average_ping < 200 { 30.0 } else { 15.0 }) as u8;

            Ok(ConnectionHealth {
                total_peers,
                healthy_peers,
                average_ping,
                connection_stability: stability,
                overall_score: score.min(100),
            })
        } else {
            Err(AppError::Generic("Network service not available".to_string()))
        }
    }

    /// Analyze individual peer performance
    async fn analyze_peers(&self) -> AppResult<Vec<PeerAnalysis>> {
        // Placeholder implementation - would need access to peer details
        let mut analyses = Vec::new();
        
        // This would iterate through actual peers and analyze each one
        for i in 0..3 { // Simulated peers
            let analysis = PeerAnalysis {
                address: format!("192.168.1.{}:8333", 100 + i),
                score: 850 - (i as i32 * 50),
                ping_ms: 120 + (i * 30),
                blocks_contributed: 50 - (i as u32 * 10),
                transactions_contributed: 200 - (i as u32 * 50),
                uptime_percentage: 95.0 - (i as f32 * 5.0),
                reliability_grade: match i {
                    0 => "A".to_string(),
                    1 => "B".to_string(),
                    _ => "C".to_string(),
                },
                issues: if i == 2 {
                    vec!["High latency".to_string()]
                } else {
                    vec![]
                },
            };
            analyses.push(analysis);
        }
        
        Ok(analyses)
    }

    /// Get bandwidth usage statistics
    async fn get_bandwidth_stats(&self) -> BandwidthStats {
        let tracker = self.bandwidth_tracker.read().await;
        
        let avg_message_size = if tracker.messages_sent + tracker.messages_received > 0 {
            (tracker.bytes_sent + tracker.bytes_received) as f32
                / (tracker.messages_sent + tracker.messages_received) as f32
        } else {
            0.0
        };

        BandwidthStats {
            bytes_sent: tracker.bytes_sent,
            bytes_received: tracker.bytes_received,
            messages_sent: tracker.messages_sent,
            messages_received: tracker.messages_received,
            average_message_size: avg_message_size,
            bandwidth_utilization: 0.3, // Placeholder
        }
    }

    /// Get blockchain synchronization status
    async fn get_sync_status(&self) -> AppResult<SyncStatus> {
        if let Some(ref network) = self.network_service {
            let stats = network.get_stats().await;
            
            let local_height = stats.local_height;
            let network_height = stats.network_height;
            let blocks_behind = network_height.saturating_sub(local_height);
            
            let is_synced = blocks_behind <= 1;
            let sync_progress = if network_height > 0 {
                (local_height as f32 / network_height as f32).min(1.0)
            } else {
                1.0
            };

            let estimated_sync_time = if blocks_behind > 0 {
                Some(blocks_behind * 60) // Assume 1 minute per block
            } else {
                None
            };

            Ok(SyncStatus {
                is_synced,
                local_height,
                network_height,
                blocks_behind,
                sync_progress,
                estimated_sync_time,
            })
        } else {
            Err(AppError::Generic("Network service not available".to_string()))
        }
    }

    /// Get current network issues
    async fn get_current_issues(&self) -> Vec<NetworkIssue> {
        let issues = self.issue_tracker.read().await;
        issues.values().cloned().collect()
    }

    /// Detect network issues automatically
    async fn detect_issues(&self) -> AppResult<()> {
        let mut new_issues = Vec::new();
        
        // Check connection count
        if let Some(ref network) = self.network_service {
            let stats = network.get_stats().await;
            
            if stats.connected_peers < 2 {
                new_issues.push(NetworkIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::Connectivity,
                    description: "Low peer count - fewer than 2 connections".to_string(),
                    recommendation: "Check network connectivity and firewall settings".to_string(),
                    first_detected: Self::current_timestamp(),
                });
            }

            if stats.network_height > stats.local_height + 10 {
                new_issues.push(NetworkIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Synchronization,
                    description: "Node is significantly behind network".to_string(),
                    recommendation: "Allow time for synchronization to complete".to_string(),
                    first_detected: Self::current_timestamp(),
                });
            }
        }

        // Update issue tracker
        let mut tracker = self.issue_tracker.write().await;
        for issue in new_issues {
            let key = format!("{}_{}", issue.category.clone() as u8, issue.description.len());
            tracker.insert(key, issue);
        }

        Ok(())
    }

    /// Record bandwidth usage
    pub async fn record_bandwidth(&self, bytes_sent: u64, bytes_received: u64) {
        let mut tracker = self.bandwidth_tracker.write().await;
        tracker.bytes_sent += bytes_sent;
        tracker.bytes_received += bytes_received;
        tracker.messages_sent += if bytes_sent > 0 { 1 } else { 0 };
        tracker.messages_received += if bytes_received > 0 { 1 } else { 0 };
    }

    /// Get diagnostic history
    pub async fn get_diagnostic_history(&self) -> Vec<NetworkDiagnostics> {
        let history = self.diagnostics_history.read().await;
        history.clone()
    }

    /// Current timestamp helper
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Async wrapper for network monitor
pub struct AsyncNetworkMonitor {
    inner: Arc<RwLock<NetworkMonitor>>,
}

impl AsyncNetworkMonitor {
    /// Create new async network monitor
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(NetworkMonitor::new())),
        }
    }

    /// Set network service
    pub async fn set_network_service(&self, network_service: AsyncNetworkService) {
        let mut monitor = self.inner.write().await;
        monitor.set_network_service(network_service);
    }

    /// Start monitoring
    pub async fn start_monitoring(&self) -> AppResult<()> {
        let monitor = self.inner.read().await;
        monitor.start_monitoring().await
    }

    /// Collect diagnostics
    pub async fn collect_diagnostics(&self) -> AppResult<NetworkDiagnostics> {
        let monitor = self.inner.read().await;
        monitor.collect_diagnostics().await
    }

    /// Record bandwidth
    pub async fn record_bandwidth(&self, bytes_sent: u64, bytes_received: u64) {
        let monitor = self.inner.read().await;
        monitor.record_bandwidth(bytes_sent, bytes_received).await;
    }

    /// Get diagnostic history
    pub async fn get_diagnostic_history(&self) -> Vec<NetworkDiagnostics> {
        let monitor = self.inner.read().await;
        monitor.get_diagnostic_history().await
    }
}

impl Clone for AsyncNetworkMonitor {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
