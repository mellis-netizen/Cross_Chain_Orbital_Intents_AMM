#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;

use stylus_sdk::{alloy_primitives::{U256, Address, B256}, prelude::*, ArbResult};
use alloy_sol_types::sol;

sol! {
    event IntentCreated(bytes32 indexed intentId, address indexed user, uint256 timestamp);
    event IntentMatched(bytes32 indexed intentId, address indexed solver, uint256 timestamp);
    event IntentExecuted(bytes32 indexed intentId, address indexed solver, bool success);
    event IntentCancelled(bytes32 indexed intentId, address indexed user);
    event SolverRegistered(address indexed solver, uint256 stake);
    event SolverSlashed(address indexed solver, uint256 amount, bytes32 intentId);
}

#[derive(SolidityError)]
pub enum IntentsError {
    IntentNotFound(IntentNotFound),
    IntentExpired(IntentExpired),
    UnauthorizedSolver(UnauthorizedSolver),
    InsufficientStake(InsufficientStake),
    IntentAlreadyMatched(IntentAlreadyMatched),
    IntentNotMatched(IntentNotMatched),
    ExecutionFailed(ExecutionFailed),
    InvalidIntent(InvalidIntent),
}

sol! {
    error IntentNotFound();
    error IntentExpired();
    error UnauthorizedSolver();
    error InsufficientStake();
    error IntentAlreadyMatched();
    error IntentNotMatched();
    error ExecutionFailed();
    error InvalidIntent();
}

sol_storage! {
    #[entrypoint]
    pub struct IntentsEngine {
        mapping(bytes32 => Intent) intents;
        mapping(address => Solver) solvers;
        mapping(address => uint256) user_nonces;
        mapping(bytes32 => IntentExecution) executions;
        
        uint256 min_solver_stake;
        uint256 intent_fee;
        uint256 slash_percentage;
        address fee_recipient;
        address owner;
    }

    pub struct Intent {
        address user;
        uint256 source_chain_id;
        uint256 dest_chain_id;
        address source_token;
        address dest_token;
        uint256 source_amount;
        uint256 min_dest_amount;
        uint256 deadline;
        uint256 nonce;
        bytes32 data_hash;
        IntentStatus status;
    }

    pub struct IntentExecution {
        address solver;
        uint256 matched_at;
        uint256 executed_at;
        uint256 dest_amount;
        bytes32 proof_hash;
        bool verified;
    }

    pub struct Solver {
        uint256 stake;
        uint256 reputation_score;
        uint256 successful_intents;
        uint256 failed_intents;
        uint256 last_active;
        bool is_registered;
    }

    pub enum IntentStatus {
        Created,
        Matched,
        Executed,
        Cancelled,
        Failed
    }
}

#[public]
impl IntentsEngine {
    pub fn initialize(
        &mut self,
        owner: Address,
        fee_recipient: Address,
        min_stake: U256,
        intent_fee: U256,
        slash_percentage: U256,
    ) -> ArbResult {
        self.owner.set(owner);
        self.fee_recipient.set(fee_recipient);
        self.min_solver_stake.set(min_stake);
        self.intent_fee.set(intent_fee);
        self.slash_percentage.set(slash_percentage);
        Ok(())
    }

    pub fn create_intent(
        &mut self,
        source_chain_id: U256,
        dest_chain_id: U256,
        source_token: Address,
        dest_token: Address,
        source_amount: U256,
        min_dest_amount: U256,
        deadline: U256,
        data: Vec<u8>,
    ) -> Result<B256, IntentsError> {
        if deadline <= U256::from(block::timestamp()) {
            return Err(IntentsError::IntentExpired(IntentExpired {}));
        }

        if source_amount == U256::ZERO || min_dest_amount == U256::ZERO {
            return Err(IntentsError::InvalidIntent(InvalidIntent {}));
        }

        let user = msg::sender();
        let nonce = self.user_nonces.get(user);
        self.user_nonces.setter(user).set(nonce + U256::from(1));

        let intent_id = self.compute_intent_id(
            user,
            source_chain_id,
            dest_chain_id,
            source_token,
            dest_token,
            source_amount,
            min_dest_amount,
            deadline,
            nonce,
        );

        let mut intent = self.intents.setter(intent_id);
        intent.user.set(user);
        intent.source_chain_id.set(source_chain_id);
        intent.dest_chain_id.set(dest_chain_id);
        intent.source_token.set(source_token);
        intent.dest_token.set(dest_token);
        intent.source_amount.set(source_amount);
        intent.min_dest_amount.set(min_dest_amount);
        intent.deadline.set(deadline);
        intent.nonce.set(nonce);
        intent.data_hash.set(keccak256(data));
        intent.status.set(IntentStatus::Created);

        evm::log(IntentCreated {
            intentId: intent_id,
            user,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(intent_id)
    }

    pub fn match_intent(&mut self, intent_id: B256) -> Result<(), IntentsError> {
        let solver = msg::sender();
        let solver_info = self.solvers.get(solver);
        
        if !solver_info.is_registered.get() || solver_info.stake.get() < self.min_solver_stake.get() {
            return Err(IntentsError::UnauthorizedSolver(UnauthorizedSolver {}));
        }

        let intent = self.intents.get(intent_id);
        if matches!(intent.status.get(), IntentStatus::Created) {
            return Err(IntentsError::IntentNotFound(IntentNotFound {}));
        }

        if intent.deadline.get() <= U256::from(block::timestamp()) {
            return Err(IntentsError::IntentExpired(IntentExpired {}));
        }

        if matches!(intent.status.get(), IntentStatus::Matched) {
            return Err(IntentsError::IntentAlreadyMatched(IntentAlreadyMatched {}));
        }

        self.intents.setter(intent_id).status.set(IntentStatus::Matched);
        
        let mut execution = self.executions.setter(intent_id);
        execution.solver.set(solver);
        execution.matched_at.set(U256::from(block::timestamp()));

        evm::log(IntentMatched {
            intentId: intent_id,
            solver,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(())
    }

    pub fn execute_intent(
        &mut self,
        intent_id: B256,
        dest_amount: U256,
        proof: Vec<u8>,
    ) -> Result<(), IntentsError> {
        let solver = msg::sender();
        let execution = self.executions.get(intent_id);
        
        if execution.solver.get() != solver {
            return Err(IntentsError::UnauthorizedSolver(UnauthorizedSolver {}));
        }

        let intent = self.intents.get(intent_id);
        if !matches!(intent.status.get(), IntentStatus::Matched) {
            return Err(IntentsError::IntentNotMatched(IntentNotMatched {}));
        }

        if dest_amount < intent.min_dest_amount.get() {
            return Err(IntentsError::ExecutionFailed(ExecutionFailed {}));
        }

        self.intents.setter(intent_id).status.set(IntentStatus::Executed);
        
        let mut execution_mut = self.executions.setter(intent_id);
        execution_mut.executed_at.set(U256::from(block::timestamp()));
        execution_mut.dest_amount.set(dest_amount);
        execution_mut.proof_hash.set(keccak256(proof));
        execution_mut.verified.set(true);

        let mut solver_info = self.solvers.setter(solver);
        solver_info.successful_intents.set(solver_info.successful_intents.get() + U256::from(1));
        solver_info.reputation_score.set(
            solver_info.reputation_score.get() + U256::from(10)
        );
        solver_info.last_active.set(U256::from(block::timestamp()));

        evm::log(IntentExecuted {
            intentId: intent_id,
            solver,
            success: true,
        });

        Ok(())
    }

    pub fn cancel_intent(&mut self, intent_id: B256) -> Result<(), IntentsError> {
        let intent = self.intents.get(intent_id);
        
        if intent.user.get() != msg::sender() {
            return Err(IntentsError::InvalidIntent(InvalidIntent {}));
        }

        if !matches!(intent.status.get(), IntentStatus::Created) {
            return Err(IntentsError::IntentAlreadyMatched(IntentAlreadyMatched {}));
        }

        self.intents.setter(intent_id).status.set(IntentStatus::Cancelled);

        evm::log(IntentCancelled {
            intentId: intent_id,
            user: msg::sender(),
        });

        Ok(())
    }

    pub fn register_solver(&mut self, stake_amount: U256) -> Result<(), IntentsError> {
        if stake_amount < self.min_solver_stake.get() {
            return Err(IntentsError::InsufficientStake(InsufficientStake {}));
        }

        let solver = msg::sender();
        let mut solver_info = self.solvers.setter(solver);
        
        solver_info.stake.set(solver_info.stake.get() + stake_amount);
        solver_info.is_registered.set(true);
        solver_info.last_active.set(U256::from(block::timestamp()));

        evm::log(SolverRegistered {
            solver,
            stake: solver_info.stake.get(),
        });

        Ok(())
    }

    pub fn slash_solver(&mut self, solver: Address, intent_id: B256) -> Result<(), IntentsError> {
        if msg::sender() != self.owner.get() {
            return Err(IntentsError::UnauthorizedSolver(UnauthorizedSolver {}));
        }

        let mut solver_info = self.solvers.setter(solver);
        let slash_amount = solver_info.stake.get() * self.slash_percentage.get() / U256::from(100);
        
        solver_info.stake.set(solver_info.stake.get() - slash_amount);
        solver_info.failed_intents.set(solver_info.failed_intents.get() + U256::from(1));
        solver_info.reputation_score.set(
            solver_info.reputation_score.get().saturating_sub(U256::from(20))
        );

        if solver_info.stake.get() < self.min_solver_stake.get() {
            solver_info.is_registered.set(false);
        }

        evm::log(SolverSlashed {
            solver,
            amount: slash_amount,
            intentId: intent_id,
        });

        Ok(())
    }

    fn compute_intent_id(
        &self,
        user: Address,
        source_chain_id: U256,
        dest_chain_id: U256,
        source_token: Address,
        dest_token: Address,
        source_amount: U256,
        min_dest_amount: U256,
        deadline: U256,
        nonce: U256,
    ) -> B256 {
        keccak256((
            user,
            source_chain_id,
            dest_chain_id,
            source_token,
            dest_token,
            source_amount,
            min_dest_amount,
            deadline,
            nonce,
        ).abi_encode())
    }

    pub fn get_intent(&self, intent_id: B256) -> Intent {
        self.intents.get(intent_id)
    }

    pub fn get_execution(&self, intent_id: B256) -> IntentExecution {
        self.executions.get(intent_id)
    }

    pub fn get_solver(&self, solver: Address) -> Solver {
        self.solvers.get(solver)
    }
}