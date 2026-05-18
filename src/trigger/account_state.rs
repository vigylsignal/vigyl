use sha2::{Digest, Sha256};

use crate::{Result, VigylError};

/// The values needed to encode `AccountState` into the on-chain trigger buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccountStateTrigger {
    pub watched_account: [u8; 32],
    pub expected_hash: [u8; 32],
    pub data_offset: u16,
    pub data_len: u16,
}

impl AccountStateTrigger {
    /// Encode the trigger into the 128-byte buffer described in docs/anchor-spec.md §2.2.
    pub fn encode(&self) -> [u8; 128] {
        let mut buf = [0u8; 128];
        buf[0..32].copy_from_slice(&self.watched_account);
        buf[32..64].copy_from_slice(&self.expected_hash);
        buf[64..66].copy_from_slice(&self.data_offset.to_le_bytes());
        buf[66..68].copy_from_slice(&self.data_len.to_le_bytes());
        buf
    }

    /// Decode the trigger back from the 128-byte buffer. Reserved bytes are ignored.
    pub fn decode(buf: &[u8; 128]) -> Self {
        let mut watched = [0u8; 32];
        watched.copy_from_slice(&buf[0..32]);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&buf[32..64]);
        let offset = u16::from_le_bytes([buf[64], buf[65]]);
        let len = u16::from_le_bytes([buf[66], buf[67]]);
        Self {
            watched_account: watched,
            expected_hash: hash,
            data_offset: offset,
            data_len: len,
        }
    }
}

/// SHA256 the requested slice of an account's data.
///
/// `data_len == 0` hashes the entire account data (matches Anchor spec semantics).
pub fn hash_slice(data: &[u8], offset: u16, len: u16) -> Result<[u8; 32]> {
    let start = offset as usize;
    if start > data.len() {
        return Err(VigylError::OutOfRange("offset > data.len()"));
    }
    let end = if len == 0 {
        data.len()
    } else {
        start + len as usize
    };
    let end = end.min(data.len());
    let mut hasher = Sha256::new();
    hasher.update(&data[start..end]);
    Ok(hasher.finalize().into())
}

pub fn account_changed(previous: &[u8; 32], current: &[u8; 32]) -> bool {
    previous != current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encode_decode() {
        let t = AccountStateTrigger {
            watched_account: [3u8; 32],
            expected_hash: [7u8; 32],
            data_offset: 8,
            data_len: 32,
        };
        let buf = t.encode();
        let back = AccountStateTrigger::decode(&buf);
        assert_eq!(back, t);
    }

    #[test]
    fn zero_len_hashes_whole_data() {
        let data = vec![1u8, 2, 3, 4, 5];
        let a = hash_slice(&data, 0, 0).unwrap();
        let b = hash_slice(&data, 0, 5).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn out_of_range_offset_rejected() {
        let data = vec![1u8; 4];
        let err = hash_slice(&data, 10, 1).unwrap_err();
        assert_eq!(err, VigylError::OutOfRange("offset > data.len()"));
    }
}
