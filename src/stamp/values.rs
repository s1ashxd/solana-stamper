use std::collections::BTreeMap;
use solana_sdk::pubkey::Pubkey;

#[derive(Default)]
pub struct ResolvedSlots {
    inner: BTreeMap<String, Pubkey>,
}

impl ResolvedSlots {
    pub fn pubkey(&self, name: &str) -> Pubkey {
        *self.inner.get(name).expect("slot not resolved")
    }

    pub fn insert(&mut self, name: impl Into<String>, value: Pubkey) {
        self.inner.insert(name.into(), value);
    }
}
