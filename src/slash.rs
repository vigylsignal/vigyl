use crate::{Result, VigylError};

/// Bond split when a keeper misses its execution window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlashOutcome {
    pub burned: u64,
    pub owner_reward: u64,
    pub keeper_leftover: u64,
}

/// Split `bond` between burn and owner reward according to the config bps values.
pub fn split_slash(bond: u64, burn_bps: u16, owner_bps: u16) -> Result<SlashOutcome> {
    if (burn_bps as u32).saturating_add(owner_bps as u32) > 10_000 {
        return Err(VigylError::OutOfRange("burn_bps + owner_bps > 10000"));
    }
    let burned = mul_bps(bond, burn_bps)?;
    let owner = mul_bps(bond, owner_bps)?;
    let total = burned.checked_add(owner).ok_or(VigylError::Overflow)?;
    let leftover = bond.checked_sub(total).ok_or(VigylError::Underflow)?;
    Ok(SlashOutcome {
        burned,
        owner_reward: owner,
        keeper_leftover: leftover,
    })
}

fn mul_bps(value: u64, bps: u16) -> Result<u64> {
    let product = (value as u128)
        .checked_mul(bps as u128)
        .ok_or(VigylError::Overflow)?;
    Ok((product / 10_000) as u64)
}

/// Whether `slash_keeper` is currently eligible against a given job.
pub fn slash_eligible(
    assigned_at_slot: u64,
    current_slot: u64,
    execution_timeout_slots: u64,
) -> bool {
    current_slot > assigned_at_slot.saturating_add(execution_timeout_slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_split_burns_and_pays() {
        let outcome = split_slash(1_000, 5_000, 5_000).unwrap();
        assert_eq!(outcome.burned, 500);
        assert_eq!(outcome.owner_reward, 500);
        assert_eq!(outcome.keeper_leftover, 0);
    }

    #[test]
    fn slash_leaves_remainder_when_bps_sum_below_100() {
        let outcome = split_slash(1_000, 3_000, 3_000).unwrap();
        assert_eq!(outcome.burned, 300);
        assert_eq!(outcome.owner_reward, 300);
        assert_eq!(outcome.keeper_leftover, 400);
    }

    #[test]
    fn oversized_bps_rejected() {
        let err = split_slash(1_000, 6_000, 6_000).unwrap_err();
        assert_eq!(err, VigylError::OutOfRange("burn_bps + owner_bps > 10000"));
    }

    #[test]
    fn slash_only_after_timeout() {
        assert!(!slash_eligible(100, 120, 50));
        assert!(!slash_eligible(100, 150, 50));
        assert!(slash_eligible(100, 151, 50));
    }
}
