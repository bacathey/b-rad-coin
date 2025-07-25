//! Transaction Fee Estimation Service
//! Provides intelligent fee estimation based on network conditions and mempool state

use crate::mempool_service::AsyncMempoolService;
use crate::blockchain_database::AsyncBlockchainDatabase;
use crate::errors::*;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Fee estimation target (in blocks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeeTarget {
    NextBlock = 1,    // High priority - next block
    Fast = 3,         // Fast - within 3 blocks
    Normal = 6,       // Normal - within 6 blocks
    Slow = 12,        // Slow - within 12 blocks
}

/// Fee rate recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub target: FeeTarget,
    pub fee_rate: u64,        // satoshis per byte
    pub confidence: f32,      // confidence level (0.0-1.0)
    pub estimated_time: u64,  // estimated confirmation time in seconds
}

/// Historical fee data for a block
#[derive(Debug, Clone)]
struct BlockFeeData {
    pub height: u64,
    pub timestamp: u64,
    pub min_fee_rate: u64,
    pub max_fee_rate: u64,
    pub median_fee_rate: u64,
    pub tx_count: usize,
}

/// Fee estimation service
pub struct FeeEstimator {
    mempool: Option<AsyncMempoolService>,
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    historical_data: Arc<RwLock<VecDeque<BlockFeeData>>>,
    max_history_blocks: usize,
}

impl FeeEstimator {
    /// Create new fee estimator
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            mempool: None,
            blockchain_db,
            historical_data: Arc::new(RwLock::new(VecDeque::new())),
            max_history_blocks: 100, // Keep 100 blocks of history
        }
    }

    /// Set mempool for current state analysis
    pub fn set_mempool(&mut self, mempool: AsyncMempoolService) {
        self.mempool = Some(mempool);
    }

    /// Estimate fee for different confirmation targets
    pub async fn estimate_fees(&self) -> AppResult<Vec<FeeEstimate>> {
        let mut estimates = Vec::new();

        // Get current mempool state
        let mempool_stats = if let Some(ref mempool) = self.mempool {
            Some(mempool.get_stats().await)
        } else {
            None
        };

        // Get historical data
        let historical = self.historical_data.read().await;
        
        for &target in &[FeeTarget::NextBlock, FeeTarget::Fast, FeeTarget::Normal, FeeTarget::Slow] {
            let estimate = self.calculate_fee_for_target(target, &mempool_stats, &historical).await?;
            estimates.push(estimate);
        }

        Ok(estimates)
    }

    /// Calculate fee estimate for specific target
    async fn calculate_fee_for_target(
        &self,
        target: FeeTarget,
        mempool_stats: &Option<crate::mempool_service::MempoolStats>,
        historical: &VecDeque<BlockFeeData>,
    ) -> AppResult<FeeEstimate> {
        let target_blocks = target as u64;
        
        // Base fee rate (minimum network fee)
        let mut base_fee_rate = 1000; // 1000 satoshis per byte minimum

        // Adjust based on mempool congestion
        if let Some(stats) = mempool_stats {
            if stats.transaction_count > 1000 {
                // High congestion
                base_fee_rate = (stats.max_fee_rate * 110) / 100; // 10% above max
            } else if stats.transaction_count > 500 {
                // Medium congestion
                base_fee_rate = (stats.max_fee_rate + stats.avg_fee_rate) / 2;
            } else if stats.transaction_count > 100 {
                // Low congestion
                base_fee_rate = stats.avg_fee_rate;
            }
        }

        // Adjust based on historical data
        if !historical.is_empty() {
            let recent_blocks: Vec<_> = historical.iter().take(target_blocks as usize).collect();
            if !recent_blocks.is_empty() {
                let avg_historical = recent_blocks.iter()
                    .map(|b| b.median_fee_rate)
                    .sum::<u64>() / recent_blocks.len() as u64;
                
                // Weight historical vs current data
                base_fee_rate = (base_fee_rate * 70 + avg_historical * 30) / 100;
            }
        }

        // Apply target multiplier
        let multiplier = match target {
            FeeTarget::NextBlock => 1.5,  // 50% premium for next block
            FeeTarget::Fast => 1.2,       // 20% premium for fast
            FeeTarget::Normal => 1.0,     // Normal rate
            FeeTarget::Slow => 0.8,       // 20% discount for slow
        };

        let estimated_fee_rate = (base_fee_rate as f64 * multiplier) as u64;

        // Calculate confidence based on data availability
        let confidence = if mempool_stats.is_some() && !historical.is_empty() {
            0.9 // High confidence with both current and historical data
        } else if mempool_stats.is_some() || !historical.is_empty() {
            0.7 // Medium confidence with partial data
        } else {
            0.5 // Low confidence with minimal data
        };

        // Estimate confirmation time (60 seconds per block average)
        let estimated_time = target_blocks * 60;

        Ok(FeeEstimate {
            target,
            fee_rate: estimated_fee_rate.max(1000), // Ensure minimum fee
            confidence,
            estimated_time,
        })
    }

    /// Update historical data with new block
    pub async fn update_with_new_block(&self, block_height: u64) -> AppResult<()> {
        // Get block from database
        if let Some(block) = self.blockchain_db.get_block_by_height(block_height).await? {
            // Analyze transactions in block for fee data
            let mut fee_rates = Vec::new();
            
            for tx in &block.transactions {
                // Skip coinbase transaction
                if !tx.inputs.is_empty() {
                    // Calculate fee rate for this transaction
                    let tx_size = self.estimate_transaction_size(tx)?;
                    if tx_size > 0 {
                        let fee_rate = (tx.fee * 1000) / tx_size as u64; // Convert to sat/byte
                        fee_rates.push(fee_rate);
                    }
                }
            }

            if !fee_rates.is_empty() {
                fee_rates.sort();
                
                let block_fee_data = BlockFeeData {
                    height: block_height,
                    timestamp: block.timestamp,
                    min_fee_rate: fee_rates[0],
                    max_fee_rate: fee_rates[fee_rates.len() - 1],
                    median_fee_rate: fee_rates[fee_rates.len() / 2],
                    tx_count: fee_rates.len(),
                };

                // Add to historical data
                let mut historical = self.historical_data.write().await;
                historical.push_front(block_fee_data);
                
                // Keep only recent history
                while historical.len() > self.max_history_blocks {
                    historical.pop_back();
                }

                info!("Updated fee estimation data for block {} (median: {} sat/byte)", 
                      block_height, fee_rates[fee_rates.len() / 2]);
            }
        }

        Ok(())
    }

    /// Estimate transaction size (simplified)
    fn estimate_transaction_size(&self, tx: &crate::blockchain_database::Transaction) -> AppResult<usize> {
        // Simple size estimation based on inputs/outputs
        let base_size = 10; // Base transaction overhead
        let input_size = tx.inputs.len() * 150; // ~150 bytes per input
        let output_size = tx.outputs.len() * 35; // ~35 bytes per output
        
        Ok(base_size + input_size + output_size)
    }

    /// Get recommended fee for transaction size
    pub async fn get_recommended_fee(&self, tx_size_bytes: usize, target: FeeTarget) -> AppResult<u64> {
        let estimates = self.estimate_fees().await?;
        
        if let Some(estimate) = estimates.iter().find(|e| e.target == target) {
            Ok(estimate.fee_rate * tx_size_bytes as u64)
        } else {
            // Fallback to reasonable default
            let default_rate = match target {
                FeeTarget::NextBlock => 5000,
                FeeTarget::Fast => 3000,
                FeeTarget::Normal => 2000,
                FeeTarget::Slow => 1000,
            };
            Ok(default_rate * tx_size_bytes as u64)
        }
    }
}

/// Async wrapper for fee estimator
pub struct AsyncFeeEstimator {
    inner: Arc<RwLock<FeeEstimator>>,
}

impl AsyncFeeEstimator {
    /// Create new async fee estimator
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(FeeEstimator::new(blockchain_db))),
        }
    }

    /// Set mempool service
    pub async fn set_mempool(&self, mempool: AsyncMempoolService) {
        let mut estimator = self.inner.write().await;
        estimator.set_mempool(mempool);
    }

    /// Get fee estimates
    pub async fn estimate_fees(&self) -> AppResult<Vec<FeeEstimate>> {
        let estimator = self.inner.read().await;
        estimator.estimate_fees().await
    }

    /// Update with new block
    pub async fn update_with_new_block(&self, block_height: u64) -> AppResult<()> {
        let estimator = self.inner.read().await;
        estimator.update_with_new_block(block_height).await
    }

    /// Get recommended fee
    pub async fn get_recommended_fee(&self, tx_size_bytes: usize, target: FeeTarget) -> AppResult<u64> {
        let estimator = self.inner.read().await;
        estimator.get_recommended_fee(tx_size_bytes, target).await
    }
}

impl Clone for AsyncFeeEstimator {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
