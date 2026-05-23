use crate::{Result, VigylError};

/// Priority-fee levels the daemon may attempt, coarsely mapping to Helius bands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriorityFeeBands {
    pub p50: u32,
    pub p75: u32,
    pub p95: u32,
}

impl PriorityFeeBands {
    pub const DEFAULT: Self = Self {
        p50: 1_500,
        p75: 4_000,
        p95: 25_000,
    };

    /// Pick the compute-unit price for `attempt`.
    ///
    /// * `attempt = 0` -> `p75`
    /// * `attempt >= 1` -> `p95`, growing linearly per retry but capped by `max`
    pub fn choose(&self, attempt: u32, max_micro_lamports: u64) -> u32 {
        let base = if attempt == 0 { self.p75 } else { self.p95 };
        let scaled = (base as u64).saturating_mul(1 + attempt as u64);
        scaled.min(max_micro_lamports).min(u32::MAX as u64) as u32
    }
}

/// Convert `micro_lamports/CU * cu_limit` to a rounded-up lamport fee.
pub fn priority_fee_lamports(micro_lamports_per_cu: u32, compute_unit_limit: u32) -> u64 {
    let product = (micro_lamports_per_cu as u128) * (compute_unit_limit as u128);
    ((product + 999_999) / 1_000_000) as u64
}

/// Full transaction cost = priority fee + base signature fee (5000 lamports).
pub fn total_cost(micro_lamports_per_cu: u32, compute_unit_limit: u32) -> u64 {
    priority_fee_lamports(micro_lamports_per_cu, compute_unit_limit).saturating_add(5_000)
}

/// Guard: enforce the job's per-CU cap before broadcasting.
pub fn enforce_cap(applied_cu_price: u32, cap: u64) -> Result<()> {
    if (applied_cu_price as u64) > cap {
        return Err(VigylError::OutOfRange("priority_fee > job.max_priority_fee"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_walks_bands() {
        let bands = PriorityFeeBands::DEFAULT;
        assert_eq!(bands.choose(0, 1_000_000), bands.p75);
        assert!(bands.choose(1, 1_000_000) >= bands.p95);
    }

    #[test]
    fn choose_clamps_to_cap() {
        let bands = PriorityFeeBands::DEFAULT;
        assert_eq!(bands.choose(0, 2_000), 2_000);
    }

    #[test]
    fn fee_rounds_up() {
        assert_eq!(priority_fee_lamports(1_000, 200_000), 200);
        assert_eq!(priority_fee_lamports(1_500, 400_000), 600);
    }

    #[test]
    fn cap_enforced() {
        assert!(enforce_cap(4_000, 5_000).is_ok());
        assert!(enforce_cap(6_000, 5_000).is_err());
    }
}
