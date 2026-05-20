use crate::{Result, VigylError};

/// Direction of the price crossing that fires the trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriceDirection {
    Above,
    Below,
}

impl PriceDirection {
    pub fn tag(self) -> u8 {
        match self {
            Self::Above => 0,
            Self::Below => 1,
        }
    }

    pub fn from_tag(tag: u8) -> Result<Self> {
        match tag {
            0 => Ok(Self::Above),
            1 => Ok(Self::Below),
            _ => Err(VigylError::InvalidTrigger("unknown direction tag")),
        }
    }
}

/// A single Pyth observation as consumed by the trigger evaluator.
#[derive(Clone, Copy, Debug)]
pub struct PythObservation {
    pub price_e6: i64,
    pub confidence_pct: u32,
    pub publish_time_seconds: i64,
}

/// The encoded price-threshold trigger the daemon and program both understand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriceThresholdTrigger {
    pub pyth_feed: [u8; 32],
    pub threshold_price_e6: i64,
    pub direction: PriceDirection,
    pub max_confidence_pct: u32,
    pub min_publish_time_seconds: i64,
}

impl PriceThresholdTrigger {
    /// Encode into docs/anchor-spec.md §2.3.
    pub fn encode(&self) -> [u8; 128] {
        let mut buf = [0u8; 128];
        buf[0..32].copy_from_slice(&self.pyth_feed);
        buf[32..40].copy_from_slice(&self.threshold_price_e6.to_le_bytes());
        buf[40] = self.direction.tag();
        buf[41..49].copy_from_slice(&(self.max_confidence_pct as u64).to_le_bytes());
        buf[49..57].copy_from_slice(&self.min_publish_time_seconds.to_le_bytes());
        buf
    }

    pub fn decode(buf: &[u8; 128]) -> Result<Self> {
        let mut feed = [0u8; 32];
        feed.copy_from_slice(&buf[0..32]);
        let threshold = i64::from_le_bytes(buf[32..40].try_into().unwrap());
        let direction = PriceDirection::from_tag(buf[40])?;
        let confidence = u64::from_le_bytes(buf[41..49].try_into().unwrap());
        let min_publish = i64::from_le_bytes(buf[49..57].try_into().unwrap());
        Ok(Self {
            pyth_feed: feed,
            threshold_price_e6: threshold,
            direction,
            max_confidence_pct: confidence as u32,
            min_publish_time_seconds: min_publish,
        })
    }

    pub fn should_fire(&self, observation: PythObservation, now_seconds: i64) -> bool {
        if observation.confidence_pct > self.max_confidence_pct {
            return false;
        }
        if now_seconds.saturating_sub(observation.publish_time_seconds)
            > self.min_publish_time_seconds
        {
            return false;
        }
        match self.direction {
            PriceDirection::Above => observation.price_e6 >= self.threshold_price_e6,
            PriceDirection::Below => observation.price_e6 <= self.threshold_price_e6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PriceThresholdTrigger {
        PriceThresholdTrigger {
            pyth_feed: [1u8; 32],
            threshold_price_e6: 200_000_000,
            direction: PriceDirection::Above,
            max_confidence_pct: 50,
            min_publish_time_seconds: 60,
        }
    }

    #[test]
    fn above_fires_when_price_crosses() {
        let t = base();
        let now = 1_000_000;
        let obs = PythObservation {
            price_e6: 201_000_000,
            confidence_pct: 10,
            publish_time_seconds: now - 5,
        };
        assert!(t.should_fire(obs, now));
    }

    #[test]
    fn stale_price_does_not_fire() {
        let t = base();
        let now = 1_000_000;
        let obs = PythObservation {
            price_e6: 500_000_000,
            confidence_pct: 10,
            publish_time_seconds: now - 3_600,
        };
        assert!(!t.should_fire(obs, now));
    }

    #[test]
    fn high_confidence_interval_rejected() {
        let t = base();
        let now = 1_000_000;
        let obs = PythObservation {
            price_e6: 500_000_000,
            confidence_pct: 90,
            publish_time_seconds: now,
        };
        assert!(!t.should_fire(obs, now));
    }

    #[test]
    fn roundtrip_encode_decode() {
        let t = base();
        let buf = t.encode();
        let back = PriceThresholdTrigger::decode(&buf).unwrap();
        assert_eq!(back, t);
    }
}
