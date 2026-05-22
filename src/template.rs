use std::collections::BTreeMap;

use smallvec::SmallVec;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use crate::compile::MAX_TX_SIZE;
use crate::compile::depgraph::topo_sort;
use crate::compile::markers::MarkerAllocator;
use crate::compile::resolve::{ResolveContext, resolve_account};
use crate::compile::scan::find_all;
use crate::compile::serialize::serialize_placeholder_tx;
use crate::compile::validate::validate_spec;
use crate::error::StamperError;
use crate::spec::TemplateSpec;
use crate::spec::account::Acc;
use crate::spec::account::DeriveFn;
use crate::spec::data::DataPiece;
use crate::spec::prefix::AuthoritySource;
use crate::spec::slot::SlotKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchKind {
    Pubkey32,
    Hash32,
    U8,
    U16,
    U32,
    U64,
}

impl PatchKind {
    #[must_use]
    pub const fn len(self) -> usize {
        match self {
            Self::Pubkey32 | Self::Hash32 => 32,
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 => 4,
            Self::U64 => 8,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PatchOp {
    pub offset: u16,
    pub kind: PatchKind,
}

impl PatchOp {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.kind.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct PatchSlot {
    pub kind: SlotKind,
    pub patches: SmallVec<[PatchOp; 4]>,
    pub per_provider: bool,
}

#[derive(Clone)]
pub struct ComputedSlot {
    pub name: String,
    pub deps: SmallVec<[String; 4]>,
    pub compute: DeriveFn,
    pub patches: SmallVec<[PatchOp; 4]>,
}

pub struct Template {
    #[allow(dead_code)]
    pub(crate) buf: [u8; MAX_TX_SIZE],
    #[allow(dead_code)]
    pub(crate) len: u16,
    #[allow(dead_code)]
    pub(crate) msg_start: u16,
    #[allow(dead_code)]
    pub(crate) blockhash_off: u16,
    #[allow(dead_code)]
    pub(crate) sig_offs: SmallVec<[u16; 2]>,
    pub(crate) slot_table: BTreeMap<String, PatchSlot>,
    #[allow(dead_code)]
    pub(crate) computed: SmallVec<[ComputedSlot; 4]>,
    #[allow(dead_code)]
    pub(crate) per_provider_slots: SmallVec<[String; 4]>,
    pub(crate) payer: Pubkey,
    #[allow(dead_code)]
    pub(crate) additional_signer_names: SmallVec<[String; 2]>,
}

#[allow(clippy::too_many_arguments)]
fn register_sentinel(
    slot_table: &mut BTreeMap<String, PatchSlot>,
    buf: &[u8],
    name: &str,
    needle: &[u8],
    patch_kind: PatchKind,
    slot_kind: SlotKind,
    expected_count: usize,
    per_provider: bool,
) -> Result<(), StamperError> {
    let offsets = find_all(buf, needle);
    if offsets.is_empty() {
        return Err(StamperError::SentinelNotFound {
            name: name.to_string(),
        });
    }
    if offsets.len() != expected_count {
        return Err(StamperError::MarkerCollision {
            a: name.to_string(),
            b: "instruction data".into(),
        });
    }
    let patches: SmallVec<[PatchOp; 4]> = offsets
        .into_iter()
        .map(|o| PatchOp {
            offset: u16::try_from(o).expect("off fits"),
            kind: patch_kind,
        })
        .collect();
    slot_table.insert(
        name.to_string(),
        PatchSlot {
            kind: slot_kind,
            patches,
            per_provider,
        },
    );
    Ok(())
}

impl Template {
    #[must_use]
    pub fn payer(&self) -> &Pubkey {
        &self.payer
    }

    pub fn slot_names(&self) -> impl Iterator<Item = &str> {
        self.slot_table.keys().map(String::as_str)
    }

    #[must_use]
    pub fn additional_signer_names(&self) -> &[String] {
        &self.additional_signer_names
    }

    #[must_use]
    pub fn computed(&self) -> &[ComputedSlot] {
        &self.computed
    }

    #[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
    pub fn compile(spec: TemplateSpec) -> Result<Self, StamperError> {
        validate_spec(&spec)?;

        let mut alloc = MarkerAllocator::new();
        let mut ctx = ResolveContext::new(spec.payer);

        let mut slot_kinds: BTreeMap<String, SlotKind> = BTreeMap::new();
        let mut per_provider_flags: BTreeMap<String, bool> = BTreeMap::new();
        let mut computed_meta: Vec<(String, SmallVec<[String; 4]>, DeriveFn)> = Vec::new();
        let mut additional_signers: SmallVec<[String; 2]> = SmallVec::new();

        let mut u64_sentinels: BTreeMap<String, u64> = BTreeMap::new();
        let mut u32_sentinels: BTreeMap<String, u32> = BTreeMap::new();
        let mut u16_sentinels: BTreeMap<String, u16> = BTreeMap::new();
        let mut u8_sentinels: BTreeMap<String, u8> = BTreeMap::new();
        let mut pubkey_data_sentinels: BTreeMap<String, [u8; 32]> = BTreeMap::new();

        let mut expected_counts: BTreeMap<String, usize> = BTreeMap::new();

        for sig_name in &spec.additional_signers {
            let sentinel = alloc.signer_sentinel();
            ctx.slot_sentinels_mut()
                .insert((*sig_name).to_string(), Pubkey::new_from_array(sentinel));
            additional_signers.push((*sig_name).to_string());
            slot_kinds.insert((*sig_name).to_string(), SlotKind::Pubkey);
            *expected_counts.entry((*sig_name).to_string()).or_insert(0) += 1;
        }

        let blockhash_sentinel = Hash::new_from_array(alloc.hash_sentinel());
        let mut resolved_ixs: Vec<Instruction> = Vec::new();

        if let Some(nonce_cfg) = spec.prefix.advance_nonce {
            let nonce_acc_sentinel = Pubkey::new_from_array(alloc.pubkey_sentinel());
            ctx.slot_sentinels_mut()
                .insert(nonce_cfg.account_slot.to_string(), nonce_acc_sentinel);
            slot_kinds.insert(nonce_cfg.account_slot.to_string(), SlotKind::Pubkey);
            per_provider_flags.insert(nonce_cfg.account_slot.to_string(), false);
            *expected_counts
                .entry(nonce_cfg.account_slot.to_string())
                .or_insert(0) += 1;
            let authority_pk = match nonce_cfg.authority {
                AuthoritySource::Payer => spec.payer,
                AuthoritySource::Fixed(pk) => pk,
                AuthoritySource::Slot(name) => {
                    let sentinel = Pubkey::new_from_array(alloc.pubkey_sentinel());
                    ctx.slot_sentinels_mut().insert(name.to_string(), sentinel);
                    slot_kinds.insert(name.to_string(), SlotKind::Pubkey);
                    per_provider_flags.insert(name.to_string(), false);
                    *expected_counts.entry(name.to_string()).or_insert(0) += 1;
                    sentinel
                }
            };
            resolved_ixs.push(solana_system_interface::instruction::advance_nonce_account(
                &nonce_acc_sentinel,
                &authority_pk,
            ));
        }

        if let Some(cb) = spec.prefix.compute_budget {
            let limit_mag = alloc.u32_magic();
            u32_sentinels.insert(cb.limit_slot.to_string(), limit_mag);
            slot_kinds.insert(cb.limit_slot.to_string(), SlotKind::U32);
            per_provider_flags.insert(cb.limit_slot.to_string(), false);
            *expected_counts
                .entry(cb.limit_slot.to_string())
                .or_insert(0) += 1;
            let price_mag = alloc.u64_magic();
            u64_sentinels.insert(cb.price_slot.to_string(), price_mag);
            slot_kinds.insert(cb.price_slot.to_string(), SlotKind::U64);
            per_provider_flags.insert(cb.price_slot.to_string(), false);
            *expected_counts
                .entry(cb.price_slot.to_string())
                .or_insert(0) += 1;
            resolved_ixs.push(
                solana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_limit(
                    limit_mag,
                ),
            );
            resolved_ixs.push(
                solana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_price(
                    price_mag,
                ),
            );
        }

        if let Some(tip) = spec.prefix.tip_transfer {
            let tip_acc_sentinel = Pubkey::new_from_array(alloc.pubkey_sentinel());
            ctx.slot_sentinels_mut()
                .insert(tip.account_slot.to_string(), tip_acc_sentinel);
            slot_kinds.insert(tip.account_slot.to_string(), SlotKind::Pubkey);
            per_provider_flags.insert(tip.account_slot.to_string(), tip.per_provider);
            *expected_counts
                .entry(tip.account_slot.to_string())
                .or_insert(0) += 1;
            let lam_mag = alloc.u64_magic();
            u64_sentinels.insert(tip.lamports_slot.to_string(), lam_mag);
            slot_kinds.insert(tip.lamports_slot.to_string(), SlotKind::U64);
            per_provider_flags.insert(tip.lamports_slot.to_string(), tip.per_provider);
            *expected_counts
                .entry(tip.lamports_slot.to_string())
                .or_insert(0) += 1;
            resolved_ixs.push(solana_system_interface::instruction::transfer(
                &spec.payer,
                &tip_acc_sentinel,
                lam_mag,
            ));
        }

        for ix in &spec.ixs {
            let mut metas: Vec<AccountMeta> = Vec::new();
            for acc in &ix.accounts {
                let pk = resolve_account(acc, &mut ctx, &mut alloc)?;
                let (writable, signer) = match acc {
                    Acc::Fixed(_, f) | Acc::Payer(f) | Acc::AdditionalSigner { flags: f, .. } => {
                        (f.writable, f.signer)
                    }
                    Acc::Slot {
                        flags,
                        name,
                        per_provider,
                    } => {
                        slot_kinds.insert((*name).to_string(), SlotKind::Pubkey);
                        per_provider_flags.insert((*name).to_string(), *per_provider);
                        *expected_counts.entry((*name).to_string()).or_insert(0) += 1;
                        (flags.writable, flags.signer)
                    }
                    Acc::Derived {
                        flags,
                        name,
                        deps,
                        compute,
                    } => {
                        slot_kinds.insert((*name).to_string(), SlotKind::Pubkey);
                        per_provider_flags.insert((*name).to_string(), false);
                        computed_meta.push((
                            (*name).to_string(),
                            deps.iter().map(|s| (*s).to_string()).collect(),
                            compute.clone(),
                        ));
                        *expected_counts.entry((*name).to_string()).or_insert(0) += 1;
                        (flags.writable, flags.signer)
                    }
                };
                metas.push(if writable {
                    AccountMeta::new(pk, signer)
                } else {
                    AccountMeta::new_readonly(pk, signer)
                });
            }

            let mut bytes: Vec<u8> = Vec::new();
            for piece in &ix.data.0 {
                match piece {
                    DataPiece::Bytes(b) => bytes.extend_from_slice(b),
                    DataPiece::U8Slot(n) => {
                        let mag = *u8_sentinels
                            .entry((*n).to_string())
                            .or_insert_with(|| alloc.u8_magic());
                        slot_kinds.insert((*n).to_string(), SlotKind::U8);
                        per_provider_flags.insert((*n).to_string(), false);
                        *expected_counts.entry((*n).to_string()).or_insert(0) += 1;
                        bytes.push(mag);
                    }
                    DataPiece::U16Slot(n) => {
                        let mag = *u16_sentinels
                            .entry((*n).to_string())
                            .or_insert_with(|| alloc.u16_magic());
                        slot_kinds.insert((*n).to_string(), SlotKind::U16);
                        per_provider_flags.insert((*n).to_string(), false);
                        *expected_counts.entry((*n).to_string()).or_insert(0) += 1;
                        bytes.extend_from_slice(&mag.to_le_bytes());
                    }
                    DataPiece::U32Slot(n) => {
                        let mag = *u32_sentinels
                            .entry((*n).to_string())
                            .or_insert_with(|| alloc.u32_magic());
                        slot_kinds.insert((*n).to_string(), SlotKind::U32);
                        per_provider_flags.insert((*n).to_string(), false);
                        *expected_counts.entry((*n).to_string()).or_insert(0) += 1;
                        bytes.extend_from_slice(&mag.to_le_bytes());
                    }
                    DataPiece::U64Slot(n) => {
                        let mag = *u64_sentinels
                            .entry((*n).to_string())
                            .or_insert_with(|| alloc.u64_magic());
                        slot_kinds.insert((*n).to_string(), SlotKind::U64);
                        per_provider_flags.insert((*n).to_string(), false);
                        *expected_counts.entry((*n).to_string()).or_insert(0) += 1;
                        bytes.extend_from_slice(&mag.to_le_bytes());
                    }
                    DataPiece::PubkeySlot(n) => {
                        let sentinel = *pubkey_data_sentinels
                            .entry((*n).to_string())
                            .or_insert_with(|| alloc.pubkey_sentinel());
                        slot_kinds.insert((*n).to_string(), SlotKind::Pubkey);
                        per_provider_flags.insert((*n).to_string(), false);
                        *expected_counts.entry((*n).to_string()).or_insert(0) += 1;
                        bytes.extend_from_slice(&sentinel);
                    }
                }
            }

            resolved_ixs.push(Instruction {
                program_id: ix.program,
                accounts: metas,
                data: bytes,
            });
        }

        let mut all_pubkey_sentinels: Vec<[u8; 32]> = Vec::new();
        for pk in ctx.slot_sentinels().values() {
            all_pubkey_sentinels.push(pk.to_bytes());
        }
        for bytes in pubkey_data_sentinels.values() {
            all_pubkey_sentinels.push(*bytes);
        }
        all_pubkey_sentinels.push(blockhash_sentinel.to_bytes());

        for ix in &spec.ixs {
            let pk_bytes = ix.program.to_bytes();
            if all_pubkey_sentinels.iter().any(|s| s == &pk_bytes) {
                return Err(StamperError::FixedPubkeyHitsSentinel { pk: ix.program });
            }
            for acc in &ix.accounts {
                if let Acc::Fixed(pk, _) = acc {
                    let pk_bytes = pk.to_bytes();
                    if all_pubkey_sentinels.iter().any(|s| s == &pk_bytes) {
                        return Err(StamperError::FixedPubkeyHitsSentinel { pk: *pk });
                    }
                }
            }
        }

        let sig_count = 1 + additional_signers.len();
        let buf_vec = serialize_placeholder_tx(
            &spec.payer,
            blockhash_sentinel,
            &resolved_ixs,
            &[],
            sig_count,
        )?;

        let mut buf = [0u8; MAX_TX_SIZE];
        buf[..buf_vec.len()].copy_from_slice(&buf_vec);
        let total_len = u16::try_from(buf_vec.len()).expect("len fits in u16");
        let msg_start_usize = 1 + 64 * sig_count;
        let msg_start = u16::try_from(msg_start_usize).expect("msg_start fits in u16");

        let blockhash_bytes = blockhash_sentinel.to_bytes();
        let bh_offsets = find_all(&buf_vec, &blockhash_bytes);
        let blockhash_off =
            u16::try_from(
                *bh_offsets
                    .first()
                    .ok_or_else(|| StamperError::SentinelNotFound {
                        name: "blockhash".into(),
                    })?,
            )
            .expect("bh off fits");

        let mut sig_offs: SmallVec<[u16; 2]> = SmallVec::new();
        for off in find_all(&buf_vec, &[0xAA; 64]) {
            sig_offs.push(u16::try_from(off).expect("sig off fits"));
        }
        if sig_offs.len() != sig_count {
            return Err(StamperError::SentinelNotFound {
                name: "signature".into(),
            });
        }

        let mut slot_table: BTreeMap<String, PatchSlot> = BTreeMap::new();

        let pubkey_sentinels = ctx.slot_sentinels().clone();
        for (name, sentinel) in &pubkey_sentinels {
            let needle = sentinel.to_bytes();
            let expected = expected_counts.get(name).copied().unwrap_or(1);
            let offsets = find_all(&buf_vec, &needle);
            if offsets.is_empty() {
                return Err(StamperError::SentinelNotFound { name: name.clone() });
            }
            if offsets.len() != expected {
                return Err(StamperError::MarkerCollision {
                    a: name.clone(),
                    b: "instruction data".into(),
                });
            }
            let patches: SmallVec<[PatchOp; 4]> = offsets
                .into_iter()
                .map(|o| PatchOp {
                    offset: u16::try_from(o).expect("off fits"),
                    kind: PatchKind::Pubkey32,
                })
                .collect();
            slot_table.insert(
                name.clone(),
                PatchSlot {
                    kind: slot_kinds.get(name).copied().unwrap_or(SlotKind::Pubkey),
                    patches,
                    per_provider: per_provider_flags.get(name).copied().unwrap_or(false),
                },
            );
        }

        for (name, sentinel) in &pubkey_data_sentinels {
            let expected = expected_counts.get(name).copied().unwrap_or(1);
            register_sentinel(
                &mut slot_table,
                &buf_vec,
                name,
                sentinel,
                PatchKind::Pubkey32,
                SlotKind::Pubkey,
                expected,
                false,
            )?;
        }

        for (name, mag) in &u64_sentinels {
            let expected = expected_counts.get(name).copied().unwrap_or(1);
            let per_provider = per_provider_flags.get(name).copied().unwrap_or(false);
            register_sentinel(
                &mut slot_table,
                &buf_vec,
                name,
                &mag.to_le_bytes(),
                PatchKind::U64,
                SlotKind::U64,
                expected,
                per_provider,
            )?;
        }

        for (name, mag) in &u32_sentinels {
            let expected = expected_counts.get(name).copied().unwrap_or(1);
            let per_provider = per_provider_flags.get(name).copied().unwrap_or(false);
            register_sentinel(
                &mut slot_table,
                &buf_vec,
                name,
                &mag.to_le_bytes(),
                PatchKind::U32,
                SlotKind::U32,
                expected,
                per_provider,
            )?;
        }

        for (name, mag) in &u16_sentinels {
            let expected = expected_counts.get(name).copied().unwrap_or(1);
            let per_provider = per_provider_flags.get(name).copied().unwrap_or(false);
            register_sentinel(
                &mut slot_table,
                &buf_vec,
                name,
                &mag.to_le_bytes(),
                PatchKind::U16,
                SlotKind::U16,
                expected,
                per_provider,
            )?;
        }

        for (name, mag) in &u8_sentinels {
            let expected = expected_counts.get(name).copied().unwrap_or(1);
            let per_provider = per_provider_flags.get(name).copied().unwrap_or(false);
            register_sentinel(
                &mut slot_table,
                &buf_vec,
                name,
                &[*mag],
                PatchKind::U8,
                SlotKind::U8,
                expected,
                per_provider,
            )?;
        }

        let computed_graph: BTreeMap<&str, Vec<&str>> = computed_meta
            .iter()
            .map(|(n, deps, _)| (n.as_str(), deps.iter().map(String::as_str).collect()))
            .collect();
        let sorted = topo_sort(&computed_graph);
        let mut computed_ordered: SmallVec<[ComputedSlot; 4]> = SmallVec::new();
        for name in sorted {
            if let Some((_, deps, compute)) = computed_meta.iter().find(|(n, _, _)| n == &name)
                && let Some(slot) = slot_table.get(&name)
            {
                computed_ordered.push(ComputedSlot {
                    name: name.clone(),
                    deps: deps.clone(),
                    compute: compute.clone(),
                    patches: slot.patches.clone(),
                });
            }
        }

        let per_provider_slots: SmallVec<[String; 4]> = slot_table
            .iter()
            .filter(|(_, s)| s.per_provider)
            .map(|(n, _)| n.clone())
            .collect();

        Ok(Self {
            buf,
            len: total_len,
            msg_start,
            blockhash_off,
            sig_offs,
            slot_table,
            computed: computed_ordered,
            per_provider_slots,
            payer: spec.payer,
            additional_signer_names: additional_signers,
        })
    }
}
