use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::{Result, VigylError};

/// The record a keeper posts after it lands the target instruction.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionProof {
    pub job: [u8; 32],
    pub keeper: [u8; 32],
    pub execution_index: u64,
    pub assigned_at_slot: u64,
    pub submitted_at_slot: u64,
    pub latency_slots: u32,
    pub priority_fee_used: u64,
    #[serde(with = "BigArray")]
    pub tx_signature: [u8; 64],
    pub success: bool,
}

impl ExecutionProof {
    pub fn new(
        job: [u8; 32],
        keeper: [u8; 32],
        execution_index: u64,
        assigned_at_slot: u64,
        submitted_at_slot: u64,
        priority_fee_used: u64,
        tx_signature: [u8; 64],
        success: bool,
    ) -> Result<Self> {
        let latency = latency_slots(assigned_at_slot, submitted_at_slot);
        Ok(Self {
            job,
            keeper,
            execution_index,
            assigned_at_slot,
            submitted_at_slot,
            latency_slots: latency.try_into().map_err(|_| VigylError::Overflow)?,
            priority_fee_used,
            tx_signature,
            success,
        })
    }
}

/// Clamp latency to `0..=u32::MAX`.
pub fn latency_slots(assigned_slot: u64, submitted_slot: u64) -> u64 {
    submitted_slot.saturating_sub(assigned_slot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_latency_clamped_to_zero() {
        assert_eq!(latency_slots(100, 50), 0);
    }

    #[test]
    fn latency_matches_arithmetic() {
        assert_eq!(latency_slots(100, 137), 37);
    }

    #[test]
    fn proof_carries_signature_bytes() {
        let sig = [7u8; 64];
        let proof = ExecutionProof::new([0u8; 32], [1u8; 32], 0, 10, 15, 4200, sig, true).unwrap();
        assert_eq!(proof.latency_slots, 5);
        assert_eq!(proof.tx_signature[..], sig[..]);
    }
}
