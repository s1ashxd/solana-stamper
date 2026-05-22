pub mod patch;
pub mod values;

use std::collections::BTreeMap;

use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;

use crate::error::StamperError;
use crate::signer::Signer;
use crate::stamp::patch::{patch_hash, patch_pubkey, patch_sig, patch_u8, patch_u16, patch_u32, patch_u64};
use crate::stamp::values::{ResolvedSlots, SlotValue};
use crate::stamped::StampedTx;
use crate::template::{PatchKind, Template};

pub struct StampBuilder<'t> {
    template: &'t Template,
    values: BTreeMap<String, SlotValue>,
    blockhash: Option<[u8; 32]>,
}

impl Template {
    #[must_use]
    pub fn stamp(&self) -> StampBuilder<'_> {
        StampBuilder { template: self, values: BTreeMap::new(), blockhash: None }
    }
}

impl StampBuilder<'_> {
    #[must_use]
    pub fn set(mut self, name: &str, v: impl Into<SlotValue>) -> Self {
        self.values.insert(name.to_string(), v.into());
        self
    }

    #[must_use]
    pub fn blockhash(mut self, h: Hash) -> Self {
        self.blockhash = Some(h.to_bytes());
        self
    }

    pub fn sign<S: Signer>(self, primary: &S) -> Result<StampedTx, StamperError> {
        self.sign_inner(primary, &[])
    }

    pub fn sign_with(self, primary: &dyn Signer, extras: &[&dyn Signer]) -> Result<StampedTx, StamperError> {
        self.sign_inner(primary, extras)
    }

    fn sign_inner(self, primary: &dyn Signer, extras: &[&dyn Signer]) -> Result<StampedTx, StamperError> {
        let Self { template, values, blockhash } = self;

        if primary.pubkey() != *template.payer() {
            return Err(StamperError::SignerMismatch {
                expected: *template.payer(),
                got: primary.pubkey(),
                index: 0,
            });
        }

        let mut buf = template.buf;

        let bh = blockhash.ok_or_else(|| StamperError::MissingSlotValue {
            name: "blockhash".into(),
            required_by: Vec::new(),
        })?;
        patch_hash(&mut buf, usize::from(template.blockhash_off), &bh);

        let mut resolved: ResolvedSlots = ResolvedSlots::default();
        for (name, value) in &values {
            if let SlotValue::Pubkey(b) = value {
                resolved.insert(name.clone(), Pubkey::new_from_array(*b));
            }
        }
        for computed in template.computed() {
            let pk = (computed.compute)(&resolved, template.payer()).map_err(|source| {
                StamperError::ComputeFailed { name: computed.name.clone(), source: Box::new(source) }
            })?;
            resolved.insert(computed.name.clone(), pk);
            for op in &computed.patches {
                patch_pubkey(&mut buf, usize::from(op.offset), &pk.to_bytes());
            }
        }

        for (name, value) in &values {
            let slot = template
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
                    (PatchKind::Pubkey32, SlotValue::Pubkey(b)) => patch_pubkey(&mut buf, usize::from(op.offset), b),
                    (PatchKind::Hash32, SlotValue::Hash(b)) => patch_hash(&mut buf, usize::from(op.offset), b),
                    (PatchKind::U8, SlotValue::U8(n)) => patch_u8(&mut buf, usize::from(op.offset), *n),
                    (PatchKind::U16, SlotValue::U16(n)) => patch_u16(&mut buf, usize::from(op.offset), *n),
                    (PatchKind::U32, SlotValue::U32(n)) => patch_u32(&mut buf, usize::from(op.offset), *n),
                    (PatchKind::U64, SlotValue::U64(n)) => patch_u64(&mut buf, usize::from(op.offset), *n),
                    _ => {
                        return Err(StamperError::WrongSlotType {
                            name: name.clone(),
                            expected: slot.kind,
                            got: slot.kind,
                        });
                    }
                }
            }
        }

        let msg_bytes: Vec<u8> = buf[usize::from(template.msg_start)..usize::from(template.len)].to_vec();
        let primary_sig = primary.sign(&msg_bytes);
        patch_sig(&mut buf, usize::from(template.sig_offs[0]), &primary_sig);

        for (i, extra) in extras.iter().enumerate() {
            let idx = i + 1;
            let off = template.sig_offs.get(idx).copied().ok_or_else(|| {
                StamperError::MissingAdditionalSigner {
                    name: template.additional_signer_names().get(i).cloned().unwrap_or_default(),
                }
            })?;
            let sig = extra.sign(&msg_bytes);
            patch_sig(&mut buf, usize::from(off), &sig);
        }

        Ok(StampedTx { buf, len: template.len, sig_off: template.sig_offs[0] })
    }
}
