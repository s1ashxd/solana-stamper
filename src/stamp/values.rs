use std::collections::BTreeMap;

use solana_sdk::pubkey::Pubkey;

use crate::error::StamperError;

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
