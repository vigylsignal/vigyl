use crate::{Result, VigylError};

/// The granularity of the slot/epoch trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotEpochGranularity {
    Slot,
    Epoch,
}

impl SlotEpochGranularity {
    pub fn tag(self) -> u8 {
        match self {
            Self::Slot => 0,
            Self::Epoch => 1,
        }
    }

    pub fn from_tag(tag: u8) -> Result<Self> {
        match tag {
            0 => Ok(Self::Slot),
            1 => Ok(Self::Epoch),
            _ => Err(VigylError::InvalidTrigger("unknown granularity tag")),
        }
    }
}

/// Encoded form of the slot/epoch trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotEpochTrigger {
    pub granularity: SlotEpochGranularity,
    pub period_slots: u64,
    pub last_fired_slot: u64,
}

impl SlotEpochTrigger {
    /// Encode into docs/anchor-spec.md §2.4.
    pub fn encode(&self) -> [u8; 128] {
        let mut buf = [0u8; 128];
        buf[0] = self.granularity.tag();
        buf[1..9].copy_from_slice(&self.period_slots.to_le_bytes());
        buf[9..17].copy_from_slice(&self.last_fired_slot.to_le_bytes());
        buf
    }

    pub fn decode(buf: &[u8; 128]) -> Result<Self> {
        let granularity = SlotEpochGranularity::from_tag(buf[0])?;
        let period = u64::from_le_bytes(buf[1..9].try_into().unwrap());
        let last = u64::from_le_bytes(buf[9..17].try_into().unwrap());
        Ok(Self {
            granularity,
            period_slots: period,
            last_fired_slot: last,
        })
    }

    /// Given the current slot / epoch clock, decide whether to fire.
    pub fn should_fire(&self, current_slot: u64, current_epoch: u64, slots_per_epoch: u64) -> bool {
        match self.granularity {
            SlotEpochGranularity::Epoch => {
                let start = current_epoch.saturating_mul(slots_per_epoch);
                current_slot >= start && self.last_fired_slot < start
            }
            SlotEpochGranularity::Slot => {
                if self.period_slots == 0 {
                    false
                } else {
                    current_slot.saturating_sub(self.last_fired_slot) >= self.period_slots
                }
            }
        }
    }

    /// When the trigger will next want to fire.
    pub fn next_run_slot(&self, current_slot: u64, slots_per_epoch: u64) -> u64 {
        match self.granularity {
            SlotEpochGranularity::Epoch => {
                let epoch = current_slot / slots_per_epoch.max(1);
                (epoch + 1).saturating_mul(slots_per_epoch)
            }
            SlotEpochGranularity::Slot => self.last_fired_slot.saturating_add(self.period_slots),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_period_fires_after_gap() {
        let t = SlotEpochTrigger {
            granularity: SlotEpochGranularity::Slot,
            period_slots: 100,
            last_fired_slot: 500,
        };
        assert!(!t.should_fire(599, 0, 432_000));
        assert!(t.should_fire(600, 0, 432_000));
    }

    #[test]
    fn epoch_fires_on_boundary_once() {
        let mut t = SlotEpochTrigger {
            granularity: SlotEpochGranularity::Epoch,
            period_slots: 0,
            last_fired_slot: 431_999,
        };
        assert!(t.should_fire(432_000, 1, 432_000));
        // once the keeper records the fire, the same epoch must not fire again
        t.last_fired_slot = 432_000;
        assert!(!t.should_fire(432_100, 1, 432_000));
    }

    #[test]
    fn roundtrip_encode_decode() {
        let t = SlotEpochTrigger {
            granularity: SlotEpochGranularity::Slot,
            period_slots: 250,
            last_fired_slot: 1_234_567,
        };
        let buf = t.encode();
        let back = SlotEpochTrigger::decode(&buf).unwrap();
        assert_eq!(back, t);
    }
}
