//! Vigyl keeper network primitives.
//!
//! This crate hosts the pieces that both the Anchor program and the off-chain keeper
//! daemon consume: the trigger encodings, the bond-weighted rotation, and the
//! execution-proof shape. The Anchor program in `programs/vigyl` re-exports the
//! account layouts, and the TypeScript SDK re-encodes the same buffers in
//! `ts-sdk/src/`. Keeping the encoding logic here means both sides can be tested
//! independently without needing a running validator.

pub mod job;
pub mod keeper;
pub mod proof;
pub mod slash;
pub mod executor;
pub mod trigger;

pub use job::{Job, JobBudget, JobStatus};
pub use keeper::{KeeperBond, KeeperCandidate, weighted_leader};
pub use proof::{ExecutionProof, latency_slots};
pub use slash::{SlashOutcome, split_slash};

/// Version tag for on-chain / off-chain compatibility checks.
pub const PROTOCOL_VERSION: u16 = 1;

/// Fixed size of the trigger data buffer on `Job::trigger_data`.
pub const TRIGGER_DATA_LEN: usize = 128;

/// Fixed size of the target instruction data buffer on `Job::target_ix_data`.
pub const TARGET_IX_DATA_LEN: usize = 512;

/// Common error type used across the crate.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VigylError {
    #[error("integer overflow")]
    Overflow,
    #[error("integer underflow")]
    Underflow,
    #[error("value out of range: {0}")]
    OutOfRange(&'static str),
    #[error("invalid trigger encoding: {0}")]
    InvalidTrigger(&'static str),
    #[error("invalid cron expression: {0}")]
    InvalidCron(String),
    #[error("target instruction data too large: {0} bytes")]
    TargetTooLarge(usize),
    #[error("execution timeout not reached")]
    TimeoutNotReached,
    #[error("bond below minimum")]
    UnderBonded,
}

pub type Result<T> = core::result::Result<T, VigylError>;
