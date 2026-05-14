pub mod account_state;
pub mod cron;
pub mod price_threshold;
pub mod slot_epoch;

pub use account_state::AccountStateTrigger;
pub use cron::CronSchedule;
pub use price_threshold::PriceThresholdTrigger;
pub use slot_epoch::SlotEpochTrigger;

/// Discriminant tags used on `Job::trigger_type`.
pub const CRON: u8 = 0;
pub const ACCOUNT_STATE: u8 = 1;
pub const PRICE_THRESHOLD: u8 = 2;
pub const SLOT_EPOCH: u8 = 3;
