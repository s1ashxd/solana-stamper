use smallvec::SmallVec;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;

use crate::compile::MAX_TX_SIZE;
use crate::error::StamperError;
use crate::signer::Signer;
use crate::stamp::patch::{
    patch_hash, patch_pubkey, patch_sig, patch_u8, patch_u16, patch_u32, patch_u64, patch_u128,
};
use crate::stamp::values::{ResolvedSlots, SlotValue};
use crate::stamped::StampedTx;
use crate::template::{PatchKind, Template};

pub struct PerProviderValues {
    pub slots: SmallVec<[(String, SlotValue); 2]>,
}

pub struct ProviderDiff {
    pub patches: SmallVec<[(u16, SlotValue); 4]>,
    pub signature: [u8; 64],
}

pub struct StampedBundle {
    pub(crate) base: Box<[u8; MAX_TX_SIZE]>,
    pub(crate) base_len: u16,
    pub(crate) diffs: SmallVec<[ProviderDiff; 8]>,
    pub(crate) sig_off: u16,
}

impl StampedBundle {
    #[must_use]
    pub fn len(&self) -> usize {
        self.diffs.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diffs.is_empty()
    }

    #[must_use]
    pub fn reconstruct(&self, idx: usize) -> StampedTx {
        let mut buf = *self.base;
        let diff = &self.diffs[idx];
        for (off, value) in &diff.patches {
            apply_value(&mut buf, usize::from(*off), value);
        }
        patch_sig(&mut buf, usize::from(self.sig_off), &diff.signature);
        StampedTx {
            buf,
            len: self.base_len,
            sig_off: self.sig_off,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = StampedTx> + '_ {
        (0..self.diffs.len()).map(|i| self.reconstruct(i))
    }
}

fn apply_value(buf: &mut [u8], offset: usize, value: &SlotValue) {
    match value {
        SlotValue::Pubkey(b) => patch_pubkey(buf, offset, b),
        SlotValue::Hash(b) => patch_hash(buf, offset, b),
        SlotValue::U8(n) => patch_u8(buf, offset, *n),
        SlotValue::U16(n) => patch_u16(buf, offset, *n),
        SlotValue::U32(n) => patch_u32(buf, offset, *n),
        SlotValue::U64(n) => patch_u64(buf, offset, *n),
        SlotValue::U128(n) => patch_u128(buf, offset, *n),
    }
}

pub struct BundleBuilder<'t> {
    template: &'t Template,
    common: std::collections::BTreeMap<String, SlotValue>,
    per_provider: Vec<PerProviderValues>,
    blockhash: Option<[u8; 32]>,
}

impl Template {
    #[must_use]
    pub fn stamp_bundle<I>(&self, providers: I) -> BundleBuilder<'_>
    where
        I: IntoIterator<Item = PerProviderValues>,
    {
        BundleBuilder {
            template: self,
            common: std::collections::BTreeMap::new(),
            per_provider: providers.into_iter().collect(),
            blockhash: None,
        }
    }
}

impl BundleBuilder<'_> {
    #[must_use]
    pub fn set(mut self, name: &str, v: impl Into<SlotValue>) -> Self {
        self.common.insert(name.to_string(), v.into());
        self
    }

    #[must_use]
    pub fn blockhash(mut self, h: Hash) -> Self {
        self.blockhash = Some(h.to_bytes());
        self
    }

    pub fn sign<S: Signer>(self, primary: &S) -> Result<StampedBundle, StamperError> {
        let Self {
            template,
            common,
            per_provider,
            blockhash,
        } = self;

        if primary.pubkey() != *template.payer() {
            return Err(StamperError::SignerMismatch {
                expected: *template.payer(),
                got: primary.pubkey(),
                index: 0,
            });
        }
        let blockhash = blockhash.ok_or_else(|| StamperError::MissingSlotValue {
            name: "blockhash".into(),
            required_by: Vec::new(),
        })?;

        let mut base = template.buf;
        patch_hash(&mut base, usize::from(template.blockhash_off), &blockhash);

        let mut resolved = ResolvedSlots::default();
        for (name, value) in &common {
            if let SlotValue::Pubkey(b) = value {
                resolved.insert(name.clone(), Pubkey::new_from_array(*b));
            }
        }
        for computed in template.computed() {
            let pk = (computed.compute)(&resolved, template.payer()).map_err(|source| {
                StamperError::ComputeFailed {
                    name: computed.name.clone(),
                    source: Box::new(source),
                }
            })?;
            resolved.insert(computed.name.clone(), pk);
            for op in &computed.patches {
                patch_pubkey(&mut base, usize::from(op.offset), &pk.to_bytes());
            }
        }

        for (name, value) in &common {
            let slot =
                template
                    .slot_table
                    .get(name)
                    .ok_or_else(|| StamperError::MissingSlotValue {
                        name: name.clone(),
                        required_by: Vec::new(),
                    })?;
            if slot.per_provider {
                return Err(StamperError::PerProviderInSingleStamp { name: name.clone() });
            }
            for op in &slot.patches {
                match (op.kind, value) {
                    (PatchKind::Pubkey32, SlotValue::Pubkey(b)) => {
                        patch_pubkey(&mut base, usize::from(op.offset), b)
                    }
                    (PatchKind::Hash32, SlotValue::Hash(b)) => {
                        patch_hash(&mut base, usize::from(op.offset), b)
                    }
                    (PatchKind::U8, SlotValue::U8(n)) => {
                        patch_u8(&mut base, usize::from(op.offset), *n)
                    }
                    (PatchKind::U16, SlotValue::U16(n)) => {
                        patch_u16(&mut base, usize::from(op.offset), *n)
                    }
                    (PatchKind::U32, SlotValue::U32(n)) => {
                        patch_u32(&mut base, usize::from(op.offset), *n)
                    }
                    (PatchKind::U64, SlotValue::U64(n)) => {
                        patch_u64(&mut base, usize::from(op.offset), *n)
                    }
                    (PatchKind::U128, SlotValue::U128(n)) => {
                        patch_u128(&mut base, usize::from(op.offset), *n)
                    }
                    _ => {
                        return Err(StamperError::WrongSlotType {
                            name: name.clone(),
                            expected: slot.kind,
                            got: value.kind(),
                        });
                    }
                }
            }
        }

        let mut diffs: SmallVec<[ProviderDiff; 8]> = SmallVec::new();
        for (idx, pv) in per_provider.iter().enumerate() {
            let mut local = base;
            let mut patches: SmallVec<[(u16, SlotValue); 4]> = SmallVec::new();
            for slot_name in &template.per_provider_slots {
                let provided = pv.slots.iter().find(|(n, _)| n == slot_name);
                let (name, value) =
                    provided.ok_or_else(|| StamperError::BundleMissingPerProviderSlot {
                        idx,
                        name: slot_name.clone(),
                    })?;
                let slot = template.slot_table.get(name.as_str()).ok_or_else(|| {
                    StamperError::MissingSlotValue {
                        name: name.clone(),
                        required_by: Vec::new(),
                    }
                })?;
                for op in &slot.patches {
                    apply_value(&mut local, usize::from(op.offset), value);
                    patches.push((op.offset, value.clone()));
                }
            }
            let sig = {
                let msg = &local[usize::from(template.msg_start)..usize::from(template.len)];
                primary.sign(msg)
            };
            diffs.push(ProviderDiff {
                patches,
                signature: sig,
            });
        }

        Ok(StampedBundle {
            base: Box::new(base),
            base_len: template.len,
            diffs,
            sig_off: template.sig_offs[0],
        })
    }
}
