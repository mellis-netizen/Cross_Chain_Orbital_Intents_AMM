use crate::{intent::*, Result, EngineError};
use ethers::types::{H256, U256};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct EngineState {
    intents: RwLock<HashMap<H256, IntentState>>,
    executions: RwLock<HashMap<H256, IntentExecution>>,
}

struct IntentState {
    intent: Intent,
    status: IntentStatus,
    created_at: u64,
    updated_at: u64,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            intents: RwLock::new(HashMap::new()),
            executions: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn add_intent(&self, intent: Intent) -> Result<H256> {
        let intent_id = intent.compute_id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut intents = self.intents.write().await;
        
        if intents.contains_key(&intent_id) {
            return Err(EngineError::InvalidIntent("Intent already exists".to_string()));
        }
        
        intents.insert(intent_id, IntentState {
            intent,
            status: IntentStatus::Pending,
            created_at: now,
            updated_at: now,
        });
        
        Ok(intent_id)
    }
    
    pub async fn get_intent_status(&self, intent_id: H256) -> Result<IntentStatus> {
        let intents = self.intents.read().await;
        
        intents.get(&intent_id)
            .map(|state| state.status)
            .ok_or_else(|| EngineError::InvalidIntent("Intent not found".to_string()))
    }
    
    pub async fn update_intent_status(&self, intent_id: H256, status: IntentStatus) -> Result<()> {
        let mut intents = self.intents.write().await;
        
        let state = intents.get_mut(&intent_id)
            .ok_or_else(|| EngineError::InvalidIntent("Intent not found".to_string()))?;
        
        state.status = status;
        state.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(())
    }
    
    pub async fn add_execution(&self, execution: IntentExecution) -> Result<()> {
        let mut executions = self.executions.write().await;
        executions.insert(execution.intent_id, execution);
        Ok(())
    }
    
    pub async fn complete_intent(&self, intent_id: H256, dest_amount: U256) -> Result<()> {
        self.update_intent_status(intent_id, IntentStatus::Executed).await?;
        
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.get_mut(&intent_id) {
            execution.status = ExecutionStatus::Completed;
            execution.dest_amount = Some(dest_amount);
            execution.executed_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
        }
        
        Ok(())
    }
    
    pub async fn fail_intent(&self, intent_id: H256) -> Result<()> {
        self.update_intent_status(intent_id, IntentStatus::Failed).await?;
        
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.get_mut(&intent_id) {
            execution.status = ExecutionStatus::Failed;
        }
        
        Ok(())
    }
    
    pub async fn get_pending_intents(&self) -> Vec<(H256, Intent)> {
        let intents = self.intents.read().await;
        
        intents.iter()
            .filter(|(_, state)| state.status == IntentStatus::Pending)
            .map(|(id, state)| (*id, state.intent.clone()))
            .collect()
    }
}