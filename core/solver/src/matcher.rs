use crate::{Result, SolverError, SolverConfig, SolverQuote};
use crate::reputation::ReputationManager;
use ethers::{
    prelude::*,
    types::{H256, U256, Address},
};
use intents_engine::intent::Intent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct IntentMatcher {
    matched_intents: RwLock<HashMap<H256, MatchedIntent>>,
    pending_auctions: RwLock<HashMap<H256, IntentAuction>>,
    reputation_manager: Arc<ReputationManager>,
}

#[derive(Clone)]
struct MatchedIntent {
    intent: Intent,
    matched_at: u64,
    expected_profit: U256,
    winning_solver: Address,
    winning_quote: SolverQuote,
}

/// Competitive auction for intent matching
#[derive(Clone)]
pub struct IntentAuction {
    pub intent_id: H256,
    pub intent: Intent,
    pub quotes: Vec<SolverQuote>,
    pub started_at: u64,
    pub deadline: u64,
    pub minimum_quotes: usize,
}

impl IntentMatcher {
    pub fn new(reputation_manager: Arc<ReputationManager>) -> Self {
        Self {
            matched_intents: RwLock::new(HashMap::new()),
            pending_auctions: RwLock::new(HashMap::new()),
            reputation_manager,
        }
    }

    /// Start competitive auction for intent
    pub async fn start_auction(
        &self,
        intent_id: H256,
        intent: Intent,
        auction_duration: u64,
    ) -> Result<()> {
        let mut auctions = self.pending_auctions.write().await;

        if auctions.contains_key(&intent_id) {
            return Err(SolverError::ExecutionFailed(
                "Auction already started".to_string()
            ));
        }

        let now = current_timestamp();
        auctions.insert(intent_id, IntentAuction {
            intent_id,
            intent,
            quotes: Vec::new(),
            started_at: now,
            deadline: now + auction_duration,
            minimum_quotes: 2, // Require at least 2 quotes for competition
        });

        Ok(())
    }

    /// Submit quote for intent auction
    pub async fn submit_quote(
        &self,
        intent_id: H256,
        quote: SolverQuote,
    ) -> Result<()> {
        // Verify solver eligibility
        let intent_amount = {
            let auctions = self.pending_auctions.read().await;
            let auction = auctions.get(&intent_id)
                .ok_or(SolverError::ExecutionFailed("Auction not found".to_string()))?;
            auction.intent.source_amount
        };

        if !self.reputation_manager.is_eligible(quote.solver, intent_amount).await {
            return Err(SolverError::ExecutionFailed(
                "Solver not eligible".to_string()
            ));
        }

        let mut auctions = self.pending_auctions.write().await;

        if let Some(auction) = auctions.get_mut(&intent_id) {
            // Check if auction is still open
            if current_timestamp() > auction.deadline {
                return Err(SolverError::ExecutionFailed(
                    "Auction expired".to_string()
                ));
            }

            // Check if solver already submitted
            if auction.quotes.iter().any(|q| q.solver == quote.solver) {
                return Err(SolverError::ExecutionFailed(
                    "Quote already submitted".to_string()
                ));
            }

            auction.quotes.push(quote);
            Ok(())
        } else {
            Err(SolverError::ExecutionFailed("Auction not found".to_string()))
        }
    }

    /// Finalize auction and select winning solver
    pub async fn finalize_auction(&self, intent_id: H256) -> Result<Address> {
        let mut auctions = self.pending_auctions.write().await;

        let auction = auctions.remove(&intent_id)
            .ok_or(SolverError::ExecutionFailed("Auction not found".to_string()))?;

        // Check if auction deadline passed
        if current_timestamp() < auction.deadline {
            return Err(SolverError::ExecutionFailed(
                "Auction not yet expired".to_string()
            ));
        }

        // Check minimum quotes
        if auction.quotes.len() < auction.minimum_quotes {
            return Err(SolverError::ExecutionFailed(
                "Insufficient quotes".to_string()
            ));
        }

        // Select winner based on best output and reputation
        let winner = self.select_best_solver(&auction).await?;

        // Store matched intent
        let mut matched = self.matched_intents.write().await;
        matched.insert(intent_id, MatchedIntent {
            intent: auction.intent.clone(),
            matched_at: current_timestamp(),
            expected_profit: winner.profit,
            winning_solver: winner.solver,
            winning_quote: winner.clone(),
        });

        Ok(winner.solver)
    }

    /// Select best solver using multi-criteria decision
    async fn select_best_solver(&self, auction: &IntentAuction) -> Result<SolverQuote> {
        let mut best_score = 0.0;
        let mut best_quote: Option<SolverQuote> = None;

        for quote in &auction.quotes {
            // Get solver reputation
            let reputation = self.reputation_manager
                .get_reputation(quote.solver)
                .await
                .ok_or(SolverError::ExecutionFailed("Solver not found".to_string()))?;

            // Calculate multi-criteria score
            let score = self.calculate_quote_score(quote, &reputation, &auction.intent);

            if score > best_score {
                best_score = score;
                best_quote = Some(quote.clone());
            }
        }

        best_quote.ok_or(SolverError::ExecutionFailed("No valid quotes".to_string()))
    }

    /// Calculate quote score based on multiple criteria
    fn calculate_quote_score(
        &self,
        quote: &SolverQuote,
        reputation: &crate::reputation::SolverReputation,
        intent: &Intent,
    ) -> f64 {
        const OUTPUT_WEIGHT: f64 = 0.4;
        const REPUTATION_WEIGHT: f64 = 0.3;
        const SPEED_WEIGHT: f64 = 0.2;
        const CONFIDENCE_WEIGHT: f64 = 0.1;

        // Output score: normalized by intent amount
        let output_ratio = quote.dest_amount.as_u128() as f64
            / intent.min_dest_amount.as_u128() as f64;
        let output_score = (output_ratio - 1.0).min(1.0).max(0.0);

        // Reputation score: normalized to 0-1
        let reputation_score = reputation.score as f64 / 10000.0;

        // Speed score: inverse of execution time
        let speed_score = 1.0 / (1.0 + (quote.execution_time_estimate as f64 / 60.0));

        // Confidence score: as provided
        let confidence_score = quote.confidence;

        output_score * OUTPUT_WEIGHT +
        reputation_score * REPUTATION_WEIGHT +
        speed_score * SPEED_WEIGHT +
        confidence_score * CONFIDENCE_WEIGHT
    }

    /// Get all active auctions
    pub async fn get_active_auctions(&self) -> Vec<H256> {
        let auctions = self.pending_auctions.read().await;
        auctions.keys().copied().collect()
    }
    
    pub async fn match_intent(
        &self,
        intent_id: H256,
        intent: &Intent,
        config: &SolverConfig,
    ) -> Result<()> {
        // Check if already matched
        let matched = self.matched_intents.read().await;
        if matched.contains_key(&intent_id) {
            return Err(SolverError::ExecutionFailed("Intent already matched".to_string()));
        }
        drop(matched);
        
        // Calculate expected profit
        let expected_profit = self.calculate_expected_profit(intent, config).await?;
        
        // Store matched intent
        let mut matched = self.matched_intents.write().await;
        matched.insert(intent_id, MatchedIntent {
            intent: intent.clone(),
            matched_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expected_profit,
            winning_solver: config.address,
            winning_quote: SolverQuote {
                solver: config.address,
                dest_amount: intent.min_dest_amount,
                profit: expected_profit,
                execution_time_estimate: 60,
                confidence: 0.9,
            },
        });
        
        Ok(())
    }
    
    pub async fn get_matched_intent(&self, intent_id: H256) -> Option<Intent> {
        let matched = self.matched_intents.read().await;
        matched.get(&intent_id).map(|m| m.intent.clone())
    }
    
    pub async fn remove_matched_intent(&self, intent_id: H256) {
        let mut matched = self.matched_intents.write().await;
        matched.remove(&intent_id);
    }
    
    async fn calculate_expected_profit(
        &self,
        intent: &Intent,
        config: &SolverConfig,
    ) -> Result<U256> {
        // Simple profit calculation
        // TODO: Implement sophisticated profit estimation
        let min_profit = intent.source_amount * U256::from(config.min_profit_bps) / U256::from(10000);
        Ok(min_profit)
    }
    
    pub async fn cleanup_expired(&self) {
        let now = current_timestamp();

        // Clean up expired matched intents
        let mut matched = self.matched_intents.write().await;
        matched.retain(|_, m| m.intent.deadline > now);

        // Clean up expired auctions
        let mut auctions = self.pending_auctions.write().await;
        auctions.retain(|_, a| a.deadline > now && a.intent.deadline > now);
    }

    /// Get winning quote for matched intent
    pub async fn get_winning_quote(&self, intent_id: H256) -> Option<SolverQuote> {
        let matched = self.matched_intents.read().await;
        matched.get(&intent_id).map(|m| m.winning_quote.clone())
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}