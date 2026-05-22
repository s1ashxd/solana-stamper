use std::collections::BTreeMap;

use smallvec::SmallVec;
use solana_sdk::pubkey::Pubkey;

use crate::compile::MAX_TX_SIZE;
use crate::spec::account::DeriveFn;
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
    pub(crate) buf: [u8; MAX_TX_SIZE],
    pub(crate) len: u16,
    pub(crate) msg_start: u16,
    pub(crate) blockhash_off: u16,
    pub(crate) sig_offs: SmallVec<[u16; 2]>,
    pub(crate) slot_table: BTreeMap<String, PatchSlot>,
    pub(crate) computed: SmallVec<[ComputedSlot; 4]>,
    pub(crate) per_provider_slots: SmallVec<[String; 4]>,
    pub(crate) payer: Pubkey,
    pub(crate) additional_signer_names: SmallVec<[String; 2]>,
}

impl Template {
    #[must_use]
    pub fn payer(&self) -> &Pubkey { &self.payer }

    pub fn slot_names(&self) -> impl Iterator<Item = &str> {
        self.slot_table.keys().map(String::as_str)
    }
}
