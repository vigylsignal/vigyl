use serde::{Deserialize, Serialize};

use crate::{Result, VigylError, TARGET_IX_DATA_LEN, TRIGGER_DATA_LEN};

/// The subset of [`Job`] state the off-chain daemon needs to compute the next fire.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobBudget {
    pub budget_lamports: u64,
    pub max_priority_fee_micro_lamports: u64,
    pub max_failures_before_pause: u32,
    pub current_failures: u32,
}

impl JobBudget {
    pub fn charge(mut self, cost: u64) -> Result<Self> {
        self.budget_lamports = self
            .budget_lamports
            .checked_sub(cost)
            .ok_or(VigylError::Underflow)?;
        Ok(self)
    }

    pub fn record_failure(mut self) -> Self {
        self.current_failures = self.current_failures.saturating_add(1);
        self
    }

    pub fn should_pause_after_failure(&self) -> bool {
        self.current_failures >= self.max_failures_before_pause
    }
}

/// Lifecycle of a job as tracked on-chain and mirrored off-chain.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Active,
    Paused,
    Cancelled,
}

/// Represents a scheduled job in the format shared between on-chain and off-chain code.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Job {
    pub owner: [u8; 32],
    pub job_index: u64,
    pub trigger_type: u8,
    pub trigger_data: Vec<u8>,
    pub target_program: [u8; 32],
    pub target_ix_data: Vec<u8>,
    pub target_accounts_hash: [u8; 32],
    pub budget: JobBudget,
    pub execution_count: u64,
    pub next_run_slot: u64,
    pub assigned_keeper: Option<[u8; 32]>,
    pub assigned_at_slot: u64,
    pub status: JobStatus,
}

impl Job {
    pub fn new(
        owner: [u8; 32],
        job_index: u64,
        trigger_type: u8,
        trigger_data: Vec<u8>,
        target_program: [u8; 32],
        target_ix_data: Vec<u8>,
        target_accounts_hash: [u8; 32],
        budget: JobBudget,
    ) -> Result<Self> {
        if trigger_data.len() > TRIGGER_DATA_LEN {
            return Err(VigylError::InvalidTrigger("trigger_data > 128 bytes"));
        }
        if target_ix_data.len() > TARGET_IX_DATA_LEN {
            return Err(VigylError::TargetTooLarge(target_ix_data.len()));
        }
        if trigger_type > 3 {
            return Err(VigylError::InvalidTrigger("trigger_type > 3"));
        }
        Ok(Self {
            owner,
            job_index,
            trigger_type,
            trigger_data,
            target_program,
            target_ix_data,
            target_accounts_hash,
            budget,
            execution_count: 0,
            next_run_slot: 0,
            assigned_keeper: None,
            assigned_at_slot: 0,
            status: JobStatus::Active,
        })
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, JobStatus::Active)
    }

    pub fn is_due(&self, current_slot: u64) -> bool {
        self.is_active()
            && self.assigned_keeper.is_none()
            && current_slot >= self.next_run_slot
    }

    pub fn assign(&mut self, keeper: [u8; 32], slot: u64) {
        self.assigned_keeper = Some(keeper);
        self.assigned_at_slot = slot;
    }

    pub fn clear_assignment(&mut self) {
        self.assigned_keeper = None;
        self.assigned_at_slot = 0;
    }

    pub fn record_success(&mut self, cost: u64) -> Result<()> {
        self.execution_count = self
            .execution_count
            .checked_add(1)
            .ok_or(VigylError::Overflow)?;
        self.budget = self.budget.charge(cost)?;
        self.clear_assignment();
        Ok(())
    }

    pub fn record_failure(&mut self, cost: u64) -> Result<()> {
        self.budget = self.budget.charge(cost)?;
        self.budget = self.budget.record_failure();
        if self.budget.should_pause_after_failure() {
            self.status = JobStatus::Paused;
        }
        self.clear_assignment();
        Ok(())
    }

    pub fn pause(&mut self) {
        self.status = JobStatus::Paused;
    }

    pub fn resume(&mut self) {
        if matches!(self.status, JobStatus::Paused) {
            self.status = JobStatus::Active;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zeros() -> [u8; 32] {
        [0u8; 32]
    }

    fn budget() -> JobBudget {
        JobBudget {
            budget_lamports: 1_000_000,
            max_priority_fee_micro_lamports: 5_000,
            max_failures_before_pause: 3,
            current_failures: 0,
        }
    }

    #[test]
    fn success_decrements_budget_and_counts() {
        let mut job = Job::new(zeros(), 0, 0, vec![0u8; 40], zeros(), vec![], zeros(), budget()).unwrap();
        job.record_success(1000).unwrap();
        assert_eq!(job.execution_count, 1);
        assert_eq!(job.budget.budget_lamports, 999_000);
    }

    #[test]
    fn repeated_failures_pause_the_job() {
        let mut job = Job::new(zeros(), 0, 0, vec![0u8; 40], zeros(), vec![], zeros(), budget()).unwrap();
        job.record_failure(100).unwrap();
        job.record_failure(100).unwrap();
        assert!(job.is_active());
        job.record_failure(100).unwrap();
        assert!(!job.is_active());
    }

    #[test]
    fn trigger_data_over_128_bytes_rejected() {
        let err = Job::new(zeros(), 0, 0, vec![0u8; 129], zeros(), vec![], zeros(), budget()).unwrap_err();
        assert_eq!(err, VigylError::InvalidTrigger("trigger_data > 128 bytes"));
    }
}
