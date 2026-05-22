use std::collections::BTreeMap;

use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;

use crate::error::StamperError;
use crate::spec::slot::SlotKind;

#[derive(Debug, Clone)]
pub enum SlotValue {
    Pubkey([u8; 32]),
    Hash([u8; 32]),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

impl SlotValue {
    #[must_use]
    pub fn kind(&self) -> SlotKind {
        match self {
            Self::Pubkey(_) => SlotKind::Pubkey,
            Self::Hash(_) => SlotKind::Hash,
            Self::U8(_) => SlotKind::U8,
            Self::U16(_) => SlotKind::U16,
            Self::U32(_) => SlotKind::U32,
            Self::U64(_) => SlotKind::U64,
            Self::U128(_) => SlotKind::U128,
        }
    }
}

impl From<Pubkey> for SlotValue {
    fn from(p: Pubkey) -> Self {
        Self::Pubkey(p.to_bytes())
    }
}

impl From<[u8; 32]> for SlotValue {
    fn from(b: [u8; 32]) -> Self {
        Self::Pubkey(b)
    }
}

impl From<Hash> for SlotValue {
    fn from(h: Hash) -> Self {
        Self::Hash(h.to_bytes())
    }
}

impl From<u8> for SlotValue {
    fn from(n: u8) -> Self {
        Self::U8(n)
    }
}
impl From<u16> for SlotValue {
    fn from(n: u16) -> Self {
        Self::U16(n)
    }
}
impl From<u32> for SlotValue {
    fn from(n: u32) -> Self {
        Self::U32(n)
    }
}
impl From<u64> for SlotValue {
    fn from(n: u64) -> Self {
        Self::U64(n)
    }
}
impl From<u128> for SlotValue {
    fn from(n: u128) -> Self {
        Self::U128(n)
    }
}

#[derive(Default)]
pub struct ResolvedSlots {
    inner: BTreeMap<String, Pubkey>,
}

impl ResolvedSlots {
    pub fn try_pubkey(&self, name: &str) -> Result<Pubkey, StamperError> {
        self.inner
            .get(name)
            .copied()
            .ok_or_else(|| StamperError::MissingSlotValue {
                name: name.to_string(),
                required_by: Vec::new(),
            })
    }

    pub fn insert(&mut self, name: impl Into<String>, value: Pubkey) {
        self.inner.insert(name.into(), value);
    }
}
