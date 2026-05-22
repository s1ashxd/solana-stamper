use std::collections::BTreeMap;

use solana_sdk::pubkey::Pubkey;

use crate::compile::markers::MarkerAllocator;
use crate::error::StamperError;
use crate::spec::account::Acc;
use crate::stamp::values::ResolvedSlots;

pub struct ResolveContext {
    payer: Pubkey,
    slot_sentinels: BTreeMap<String, Pubkey>,
}

impl ResolveContext {
    #[must_use]
    pub fn new(payer: Pubkey) -> Self {
        Self { payer, slot_sentinels: BTreeMap::new() }
    }

    #[must_use]
    pub fn payer(&self) -> Pubkey { self.payer }

    #[must_use]
    pub fn slot_sentinels(&self) -> &BTreeMap<String, Pubkey> { &self.slot_sentinels }

    pub fn slot_sentinels_mut(&mut self) -> &mut BTreeMap<String, Pubkey> { &mut self.slot_sentinels }
}

pub fn resolve_account(
    acc: &Acc,
    ctx: &mut ResolveContext,
    alloc: &mut MarkerAllocator,
) -> Result<Pubkey, StamperError> {
    match acc {
        Acc::Fixed(pk, _) => Ok(*pk),
        Acc::Payer(_) => Ok(ctx.payer),
        Acc::Slot { name, .. } => {
            if let Some(pk) = ctx.slot_sentinels.get(*name) {
                return Ok(*pk);
            }
            let pk = Pubkey::new_from_array(alloc.pubkey_sentinel());
            ctx.slot_sentinels.insert((*name).to_string(), pk);
            Ok(pk)
        }
        Acc::AdditionalSigner { name, .. } => {
            if let Some(pk) = ctx.slot_sentinels.get(*name) {
                return Ok(*pk);
            }
            let pk = Pubkey::new_from_array(alloc.signer_sentinel());
            ctx.slot_sentinels.insert((*name).to_string(), pk);
            Ok(pk)
        }
        Acc::Derived { name, deps, compute, .. } => {
            if let Some(pk) = ctx.slot_sentinels.get(*name) {
                return Ok(*pk);
            }
            let mut resolved = ResolvedSlots::default();
            for dep in deps {
                let pk = ctx.slot_sentinels.get(*dep).copied().ok_or_else(|| StamperError::MissingDependency {
                    computed: (*name).to_string(),
                    missing: (*dep).to_string(),
                })?;
                resolved.insert(*dep, pk);
            }
            let payer = ctx.payer;
            let pk = compute(&resolved, &payer).map_err(|source| StamperError::ComputeFailed {
                name: (*name).to_string(),
                source: Box::new(source),
            })?;
            ctx.slot_sentinels.insert((*name).to_string(), pk);
            Ok(pk)
        }
    }
}
