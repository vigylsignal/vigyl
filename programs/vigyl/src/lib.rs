//! Anchor 0.31 mirror of the VIGYL keeper network on-chain state.
//!
//! The full state and instruction handlers live in the private deploy repository.
//! This crate exposes the account layouts and the enum tags so external SDKs and
//! auditors can pin against a stable schema without waiting on the deploy.

use anchor_lang::prelude::*;

declare_id!("V1gy1KpR1RJt1n1eN3tw1RkPr1grM11111111111111");

#[program]
pub mod vigyl {
    use super::*;

    pub fn ping(_ctx: Context<Ping>) -> Result<()> {
        msg!("vigyl program v{}", VERSION);
        Ok(())
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Accounts)]
pub struct Ping<'info> {
    pub caller: Signer<'info>,
}

/// Global configuration shared across the program.
#[account]
#[derive(Default)]
pub struct Config {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub vigyl_mint: Pubkey,
    pub min_keeper_bond: u64,
    pub execution_fee_bps: u16,
    pub slash_burn_bps: u16,
    pub slash_owner_bps: u16,
    pub job_registration_fee: u64,
    pub bond_unlock_seconds: i64,
    pub execution_timeout_slots: u64,
    pub bump: u8,
}

/// Aggregate counters for the whole registry.
#[account]
#[derive(Default)]
pub struct JobRegistry {
    pub total_jobs: u64,
    pub total_keepers: u32,
    pub total_executions: u64,
    pub total_slashes: u32,
    pub total_bonded: u64,
    pub bump: u8,
}

/// A single job's on-chain record.
#[account]
pub struct Job {
    pub owner: Pubkey,
    pub job_index: u64,
    pub trigger_type: u8,
    pub trigger_data: [u8; 128],
    pub target_program: Pubkey,
    pub target_ix_data: [u8; 512],
    pub target_accounts_hash: [u8; 32],
    pub budget_lamports: u64,
    pub max_priority_fee_micro_lamports: u64,
    pub execution_count: u64,
    pub failure_count: u64,
    pub next_run_slot: u64,
    pub assigned_keeper: Pubkey,
    pub assigned_at_slot: u64,
    pub is_paused: bool,
    pub bump: u8,
}

/// A keeper's bond record.
#[account]
#[derive(Default)]
pub struct KeeperBond {
    pub keeper: Pubkey,
    pub bond_amount: u64,
    pub bonded_at_slot: u64,
    pub active_jobs: u32,
    pub total_executions: u64,
    pub total_slashes: u32,
    pub is_unbonding: bool,
    pub unbond_request_slot: u64,
    pub bump: u8,
}

/// An execution proof issued after a successful (or explicitly failed) run.
#[account]
pub struct ExecutionProof {
    pub job: Pubkey,
    pub keeper: Pubkey,
    pub execution_index: u64,
    pub submitted_at_slot: u64,
    pub assigned_at_slot: u64,
    pub latency_slots: u32,
    pub priority_fee_used: u64,
    pub tx_signature: [u8; 64],
    pub success: bool,
    pub bump: u8,
}

/// Trigger discriminant tags kept in sync with the off-chain crate.
pub const TRIGGER_CRON: u8 = 0;
pub const TRIGGER_ACCOUNT_STATE: u8 = 1;
pub const TRIGGER_PRICE_THRESHOLD: u8 = 2;
pub const TRIGGER_SLOT_EPOCH: u8 = 3;

/// Vigyl error surface exposed as `#[error_code]`.
#[error_code]
pub enum VigylError {
    #[msg("integer overflow")]
    Overflow,
    #[msg("integer underflow")]
    Underflow,
    #[msg("unauthorized")]
    Unauthorized,
    #[msg("job is paused")]
    JobPaused,
    #[msg("job has insufficient budget")]
    JobBudgetInsufficient,
    #[msg("job not due yet")]
    JobNotDue,
    #[msg("keeper is below the minimum bond")]
    KeeperUnderBonded,
    #[msg("keeper still has active jobs")]
    KeeperHasActiveJobs,
    #[msg("unbond delay not reached")]
    UnbondNotReady,
    #[msg("job is already assigned")]
    AlreadyAssigned,
    #[msg("signer is not the assigned keeper")]
    NotAssignedKeeper,
    #[msg("invalid trigger type")]
    InvalidTriggerType,
    #[msg("invalid trigger data")]
    InvalidTriggerData,
    #[msg("priority fee exceeds job cap")]
    PriorityFeeExceedsCap,
    #[msg("keeper is not slashable yet")]
    SlashNotEligible,
}
