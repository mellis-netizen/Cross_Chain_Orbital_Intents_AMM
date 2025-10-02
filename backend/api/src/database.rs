use sqlx::{PgPool, postgres::PgPoolOptions, migrate::MigrateDatabase, Postgres};
use crate::{error::Result, models::*};
use ethers::types::{Address, U256, H256};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::str::FromStr;

// Database connection
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    // Create database if it doesn't exist
    if !Postgres::database_exists(database_url).await.unwrap_or(false) {
        Postgres::create_database(database_url).await
            .map_err(|e| crate::error::internal_error(format!("Failed to create database: {}", e)))?;
    }

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await
        .map_err(|e| crate::error::internal_error(format!("Failed to connect to database: {}", e)))?;

    Ok(pool)
}

// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Create tables if they don't exist
    create_tables(pool).await?;
    Ok(())
}

// Create database tables
async fn create_tables(pool: &PgPool) -> Result<()> {
    // Intents table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS intents (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            intent_id VARCHAR(66) UNIQUE NOT NULL,
            source_chain_id BIGINT NOT NULL,
            dest_chain_id BIGINT NOT NULL,
            source_token VARCHAR(42) NOT NULL,
            dest_token VARCHAR(42) NOT NULL,
            source_amount TEXT NOT NULL,
            min_dest_amount TEXT NOT NULL,
            actual_dest_amount TEXT,
            deadline TIMESTAMPTZ NOT NULL,
            user_address VARCHAR(42) NOT NULL,
            solver_address VARCHAR(42),
            status VARCHAR(20) NOT NULL DEFAULT 'pending',
            execution_tx_hash VARCHAR(66),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            gas_used TEXT,
            fees_paid TEXT,
            error_message TEXT
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| crate::error::internal_error(format!("Failed to create intents table: {}", e)))?;

    // Solvers table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS solvers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            address VARCHAR(42) UNIQUE NOT NULL,
            bond_amount TEXT NOT NULL,
            supported_chains BIGINT[] NOT NULL,
            reputation_score FLOAT NOT NULL DEFAULT 0.5,
            success_count BIGINT NOT NULL DEFAULT 0,
            failure_count BIGINT NOT NULL DEFAULT 0,
            total_volume TEXT NOT NULL DEFAULT '0',
            fee_rate FLOAT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT true,
            is_slashed BOOLEAN NOT NULL DEFAULT false,
            last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            contact_info TEXT
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| crate::error::internal_error(format!("Failed to create solvers table: {}", e)))?;

    // Intent executions table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS intent_executions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            intent_id VARCHAR(66) NOT NULL REFERENCES intents(intent_id),
            solver_address VARCHAR(42) NOT NULL,
            execution_step VARCHAR(50) NOT NULL,
            transaction_hash VARCHAR(66),
            block_number BIGINT,
            gas_used TEXT,
            execution_time_ms BIGINT,
            status VARCHAR(20) NOT NULL,
            error_message TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| crate::error::internal_error(format!("Failed to create intent_executions table: {}", e)))?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_intents_user_address ON intents(user_address)")
        .execute(pool).await.ok();
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_intents_status ON intents(status)")
        .execute(pool).await.ok();
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_intents_created_at ON intents(created_at)")
        .execute(pool).await.ok();
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_solvers_address ON solvers(address)")
        .execute(pool).await.ok();
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_solvers_is_active ON solvers(is_active)")
        .execute(pool).await.ok();

    Ok(())
}

// Intent database operations
pub struct IntentDb;

impl IntentDb {
    pub async fn insert_intent(
        pool: &PgPool,
        intent_id: H256,
        request: &SubmitIntentRequest,
    ) -> Result<IntentRecord> {
        let record = sqlx::query_as::<_, IntentRecord>(r#"
            INSERT INTO intents (
                intent_id, source_chain_id, dest_chain_id, source_token, dest_token,
                source_amount, min_dest_amount, deadline, user_address, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
        "#)
        .bind(format!("{:#x}", intent_id))
        .bind(request.source_chain_id as i64)
        .bind(request.dest_chain_id as i64)
        .bind(format!("{:#x}", request.source_token))
        .bind(format!("{:#x}", request.dest_token))
        .bind(request.source_amount.to_string())
        .bind(request.min_dest_amount.to_string())
        .bind(request.deadline)
        .bind(format!("{:#x}", request.user_address))
        .bind("pending")
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn get_intent_by_id(
        pool: &PgPool,
        intent_id: H256,
    ) -> Result<Option<IntentRecord>> {
        let record = sqlx::query_as::<_, IntentRecord>(
            "SELECT * FROM intents WHERE intent_id = $1"
        )
        .bind(format!("{:#x}", intent_id))
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    pub async fn get_intents_by_user(
        pool: &PgPool,
        user_address: Address,
        pagination: &PaginationParams,
    ) -> Result<(Vec<IntentRecord>, u64)> {
        let offset = (pagination.page.unwrap_or(1) - 1) * pagination.limit.unwrap_or(20);
        let limit = pagination.limit.unwrap_or(20);

        let records = sqlx::query_as::<_, IntentRecord>(r#"
            SELECT * FROM intents 
            WHERE user_address = $1 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
        "#)
        .bind(format!("{:#x}", user_address))
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM intents WHERE user_address = $1"
        )
        .bind(format!("{:#x}", user_address))
        .fetch_one(pool)
        .await?;

        Ok((records, total.0 as u64))
    }

    pub async fn update_intent_status(
        pool: &PgPool,
        intent_id: H256,
        status: &str,
        solver_address: Option<Address>,
        execution_tx_hash: Option<H256>,
        actual_dest_amount: Option<U256>,
        gas_used: Option<U256>,
        fees_paid: Option<U256>,
        error_message: Option<String>,
    ) -> Result<()> {
        sqlx::query(r#"
            UPDATE intents SET 
                status = $2, 
                solver_address = $3,
                execution_tx_hash = $4,
                actual_dest_amount = $5,
                gas_used = $6,
                fees_paid = $7,
                error_message = $8,
                updated_at = NOW()
            WHERE intent_id = $1
        "#)
        .bind(format!("{:#x}", intent_id))
        .bind(status)
        .bind(solver_address.map(|addr| format!("{:#x}", addr)))
        .bind(execution_tx_hash.map(|hash| format!("{:#x}", hash)))
        .bind(actual_dest_amount.map(|amt| amt.to_string()))
        .bind(gas_used.map(|gas| gas.to_string()))
        .bind(fees_paid.map(|fees| fees.to_string()))
        .bind(error_message)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_pending_intents(
        pool: &PgPool,
        limit: u64,
    ) -> Result<Vec<IntentRecord>> {
        let records = sqlx::query_as::<_, IntentRecord>(r#"
            SELECT * FROM intents 
            WHERE status = 'pending' AND deadline > NOW() 
            ORDER BY created_at ASC 
            LIMIT $1
        "#)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}

// Solver database operations
pub struct SolverDb;

impl SolverDb {
    pub async fn register_solver(
        pool: &PgPool,
        request: &SolverRegistrationRequest,
    ) -> Result<SolverRecord> {
        let supported_chains: Vec<i64> = request.supported_chains.iter().map(|&c| c as i64).collect();
        
        let record = sqlx::query_as::<_, SolverRecord>(r#"
            INSERT INTO solvers (
                address, bond_amount, supported_chains, fee_rate
            ) VALUES ($1, $2, $3, $4)
            RETURNING *
        "#)
        .bind(format!("{:#x}", request.solver_address))
        .bind(request.bond_amount.to_string())
        .bind(&supported_chains)
        .bind(request.fee_rate)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn get_solver_by_address(
        pool: &PgPool,
        address: Address,
    ) -> Result<Option<SolverRecord>> {
        let record = sqlx::query_as::<_, SolverRecord>(
            "SELECT * FROM solvers WHERE address = $1"
        )
        .bind(format!("{:#x}", address))
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    pub async fn get_active_solvers(
        pool: &PgPool,
        chain_id: Option<u64>,
    ) -> Result<Vec<SolverRecord>> {
        let query = if let Some(chain_id) = chain_id {
            sqlx::query_as::<_, SolverRecord>(r#"
                SELECT * FROM solvers 
                WHERE is_active = true AND is_slashed = false 
                AND $1 = ANY(supported_chains)
                ORDER BY reputation_score DESC
            "#)
            .bind(chain_id as i64)
        } else {
            sqlx::query_as::<_, SolverRecord>(r#"
                SELECT * FROM solvers 
                WHERE is_active = true AND is_slashed = false 
                ORDER BY reputation_score DESC
            "#)
        };

        let records = query.fetch_all(pool).await?;
        Ok(records)
    }

    pub async fn update_solver_reputation(
        pool: &PgPool,
        address: Address,
        success_count: u64,
        failure_count: u64,
        total_volume: U256,
        reputation_score: f64,
    ) -> Result<()> {
        sqlx::query(r#"
            UPDATE solvers SET 
                success_count = $2,
                failure_count = $3,
                total_volume = $4,
                reputation_score = $5,
                last_activity = NOW()
            WHERE address = $1
        "#)
        .bind(format!("{:#x}", address))
        .bind(success_count as i64)
        .bind(failure_count as i64)
        .bind(total_volume.to_string())
        .bind(reputation_score)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn slash_solver(
        pool: &PgPool,
        address: Address,
        reason: &str,
    ) -> Result<()> {
        sqlx::query(r#"
            UPDATE solvers SET 
                is_slashed = true,
                is_active = false,
                error_message = $2,
                updated_at = NOW()
            WHERE address = $1
        "#)
        .bind(format!("{:#x}", address))
        .bind(reason)
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Helper functions for type conversions
pub fn string_to_h256(s: &str) -> Result<H256> {
    H256::from_str(s).map_err(|e| crate::error::validation_error(format!("Invalid H256: {}", e)))
}

pub fn string_to_address(s: &str) -> Result<Address> {
    Address::from_str(s).map_err(|e| crate::error::validation_error(format!("Invalid address: {}", e)))
}

pub fn string_to_u256(s: &str) -> Result<U256> {
    U256::from_dec_str(s).map_err(|e| crate::error::validation_error(format!("Invalid U256: {}", e)))
}

// Convert database record to API response
pub fn intent_record_to_response(record: IntentRecord) -> Result<IntentResponse> {
    Ok(IntentResponse {
        intent_id: string_to_h256(&record.intent_id)?,
        status: record.status,
        source_chain_id: record.source_chain_id as u64,
        dest_chain_id: record.dest_chain_id as u64,
        source_token: string_to_address(&record.source_token)?,
        dest_token: string_to_address(&record.dest_token)?,
        source_amount: string_to_u256(&record.source_amount)?,
        min_dest_amount: string_to_u256(&record.min_dest_amount)?,
        actual_dest_amount: record.actual_dest_amount
            .map(|s| string_to_u256(&s))
            .transpose()?,
        deadline: record.deadline,
        user_address: string_to_address(&record.user_address)?,
        solver_address: record.solver_address
            .map(|s| string_to_address(&s))
            .transpose()?,
        execution_tx_hash: record.execution_tx_hash
            .map(|s| string_to_h256(&s))
            .transpose()?,
        created_at: record.created_at,
        updated_at: record.updated_at,
        gas_used: record.gas_used
            .map(|s| string_to_u256(&s))
            .transpose()?,
        fees_paid: record.fees_paid
            .map(|s| string_to_u256(&s))
            .transpose()?,
    })
}

pub fn solver_record_to_response(record: SolverRecord) -> Result<SolverResponse> {
    Ok(SolverResponse {
        address: string_to_address(&record.address)?,
        bond_amount: string_to_u256(&record.bond_amount)?,
        supported_chains: record.supported_chains.iter().map(|&c| c as u64).collect(),
        reputation_score: record.reputation_score,
        success_count: record.success_count as u64,
        failure_count: record.failure_count as u64,
        total_volume: string_to_u256(&record.total_volume)?,
        fee_rate: record.fee_rate,
        is_active: record.is_active,
        is_slashed: record.is_slashed,
        last_activity: record.last_activity,
        registered_at: record.registered_at,
    })
}