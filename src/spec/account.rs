use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;
use smallvec::SmallVec;

use crate::error::StamperError;
use crate::stamp::values::ResolvedSlots;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountFlags {
    pub writable: bool,
    pub signer: bool,
}

impl AccountFlags {
    pub const READONLY: Self = Self { writable: false, signer: false };
    pub const WRITABLE: Self = Self { writable: true, signer: false };
    pub const SIGNER: Self = Self { writable: false, signer: true };
    pub const SIGNER_WRITABLE: Self = Self { writable: true, signer: true };
}

pub type DeriveFn = Arc<dyn Fn(&ResolvedSlots, &Pubkey) -> Result<Pubkey, StamperError> + Send + Sync>;

#[derive(Clone)]
pub enum Acc {
    Fixed(Pubkey, AccountFlags),
    Payer(AccountFlags),
    Slot {
        name: &'static str,
        flags: AccountFlags,
        per_provider: bool,
    },
    Derived {
        name: &'static str,
        flags: AccountFlags,
        deps: SmallVec<[&'static str; 4]>,
        compute: DeriveFn,
    },
    AdditionalSigner {
        name: &'static str,
        flags: AccountFlags,
    },
}

impl Acc {
    pub fn fixed(pk: Pubkey) -> Self {
        Self::Fixed(pk, AccountFlags::READONLY)
    }

    pub fn payer() -> Self {
        Self::Payer(AccountFlags::SIGNER_WRITABLE)
    }

    pub fn slot(name: &'static str) -> Self {
        Self::Slot { name, flags: AccountFlags::READONLY, per_provider: false }
    }

    pub fn slot_w(name: &'static str) -> Self {
        Self::Slot { name, flags: AccountFlags::WRITABLE, per_provider: false }
    }

    pub fn slot_per_provider(name: &'static str) -> Self {
        Self::Slot { name, flags: AccountFlags::WRITABLE, per_provider: true }
    }

    pub fn derived<F>(name: &'static str, deps: &[&'static str], compute: F) -> Self
    where
        F: Fn(&ResolvedSlots, &Pubkey) -> Result<Pubkey, StamperError> + Send + Sync + 'static,
    {
        Self::Derived {
            name,
            flags: AccountFlags::READONLY,
            deps: deps.iter().copied().collect(),
            compute: Arc::new(compute),
        }
    }

    pub fn signer(name: &'static str) -> Self {
        Self::AdditionalSigner { name, flags: AccountFlags::SIGNER }
    }

    pub fn writable(mut self) -> Self {
        match &mut self {
            Self::Fixed(_, f) | Self::Payer(f) | Self::AdditionalSigner { flags: f, .. } => {
                f.writable = true;
            }
            Self::Slot { flags, .. } | Self::Derived { flags, .. } => {
                flags.writable = true;
            }
        }
        self
    }
}
