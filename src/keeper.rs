use sha2::{Digest, Sha256};

/// Represents a keeper's on-chain bond entry as far as rotation cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeeperBond {
    pub keeper: [u8; 32],
    pub bond_amount: u64,
    pub active_jobs: u32,
    pub total_executions: u64,
    pub total_slashes: u32,
}

/// A rotation candidate -- the daemon's view of an on-chain [`KeeperBond`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeeperCandidate {
    pub keeper: [u8; 32],
    pub bond_amount: u64,
}

impl KeeperCandidate {
    pub fn from_bond(bond: &KeeperBond) -> Self {
        Self {
            keeper: bond.keeper,
            bond_amount: bond.bond_amount,
        }
    }
}

/// Pick the bond-weighted leader for a given `(job, slot)` pair.
///
/// The rotation is deterministic: everyone who sees the same bond set and the same
/// `job_pubkey` / `slot` derives the same leader, so the daemon can silently back off
/// when it is not the leader without racing the winner on-chain.
///
/// Returns `None` when no candidate meets `min_bond`.
pub fn weighted_leader(
    candidates: &[KeeperCandidate],
    min_bond: u64,
    job_pubkey: &[u8; 32],
    slot: u64,
) -> Option<KeeperCandidate> {
    let eligible: Vec<&KeeperCandidate> =
        candidates.iter().filter(|c| c.bond_amount >= min_bond).collect();
    if eligible.is_empty() {
        return None;
    }
    let total: u128 = eligible.iter().map(|c| c.bond_amount as u128).sum();
    if total == 0 {
        return None;
    }

    let mut hasher = Sha256::new();
    hasher.update(job_pubkey);
    hasher.update(slot.to_le_bytes());
    let digest = hasher.finalize();
    let seed = u64::from_le_bytes(digest[0..8].try_into().unwrap()) as u128;
    let target = seed % total;

    let mut running: u128 = 0;
    for cand in &eligible {
        running += cand.bond_amount as u128;
        if target < running {
            return Some(**cand);
        }
    }
    eligible.last().copied().copied()
}

/// Approximate an unbond delay in slots given seconds (Solana slot ~= 0.4 s).
pub fn slots_from_seconds(seconds: i64) -> u64 {
    if seconds <= 0 {
        return 0;
    }
    ((seconds as f64) / 0.4).ceil() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keeper(byte: u8, bond: u64) -> KeeperCandidate {
        KeeperCandidate {
            keeper: [byte; 32],
            bond_amount: bond,
        }
    }

    #[test]
    fn min_bond_filters_underbonded() {
        let cands = vec![keeper(1, 100), keeper(2, 999), keeper(3, 500)];
        let leader = weighted_leader(&cands, 500, &[7u8; 32], 42).unwrap();
        assert_ne!(leader.keeper, [1u8; 32]);
    }

    #[test]
    fn heavier_bond_wins_more_often() {
        let cands = vec![keeper(1, 100), keeper(2, 900)];
        let mut heavy = 0;
        for slot in 0..1000 {
            let leader = weighted_leader(&cands, 0, &[7u8; 32], slot).unwrap();
            if leader.keeper == [2u8; 32] {
                heavy += 1;
            }
        }
        assert!(heavy > 800);
    }

    #[test]
    fn deterministic_per_slot() {
        let cands = vec![keeper(1, 100), keeper(2, 100), keeper(3, 100)];
        let a = weighted_leader(&cands, 0, &[9u8; 32], 12345);
        let b = weighted_leader(&cands, 0, &[9u8; 32], 12345);
        assert_eq!(a, b);
    }

    #[test]
    fn slot_conversion_matches_400ms() {
        assert_eq!(slots_from_seconds(0), 0);
        assert_eq!(slots_from_seconds(1), 3);
        assert_eq!(slots_from_seconds(4), 10);
        assert_eq!(slots_from_seconds(60), 150);
    }
}
