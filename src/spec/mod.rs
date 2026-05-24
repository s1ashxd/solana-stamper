use std::collections::HashSet;

use smallvec::SmallVec;
use solana_sdk::pubkey::Pubkey;

pub mod account;
pub mod data;
pub mod instruction;
pub mod lookup;
pub mod prefix;
pub mod slot;

use crate::spec::instruction::InstructionSpec;
use crate::spec::lookup::LookupTable;
use crate::spec::prefix::PrefixOptions;

#[derive(Debug, Clone, Copy)]
pub enum MessageVersion {
    V0,
}

#[derive(Clone)]
pub struct TemplateSpec {
    pub payer: Pubkey,
    pub version: MessageVersion,
    pub additional_signers: SmallVec<[&'static str; 2]>,
    pub prefix: PrefixOptions,
    pub ixs: Vec<InstructionSpec>,
    pub luts: Vec<LookupTable>,
}

impl TemplateSpec {
    #[must_use]
    pub fn new(payer: Pubkey, version: MessageVersion) -> Self {
        Self {
            payer,
            version,
            additional_signers: SmallVec::new(),
            prefix: PrefixOptions::default(),
            ixs: Vec::new(),
            luts: Vec::new(),
        }
    }

    #[must_use]
    pub fn prefix(mut self, p: PrefixOptions) -> Self {
        self.prefix = p;
        self
    }

    #[must_use]
    pub fn additional_signer(mut self, name: &'static str) -> Self {
        self.additional_signers.push(name);
        self
    }

    #[must_use]
    pub fn ix(mut self, i: InstructionSpec) -> Self {
        self.ixs.push(i);
        self
    }

    #[must_use]
    pub fn lut(mut self, l: LookupTable) -> Self {
        self.luts.push(l);
        self
    }

    #[must_use]
    pub fn static_addresses(&self) -> Vec<Pubkey> {
        let mut out = Vec::new();
        let mut seen = HashSet::<Pubkey>::new();

        macro_rules! push {
            ($pk:expr) => {
                if seen.insert($pk) {
                    out.push($pk);
                }
            };
        }

        if self.prefix.advance_nonce.is_some() {
            push!(solana_system_interface::program::ID);
        }
        if self.prefix.compute_budget.is_some() {
            push!(solana_compute_budget_interface::id());
        }
        if self.prefix.tip_transfer.is_some() {
            push!(solana_system_interface::program::ID);
        }

        for ix in &self.ixs {
            push!(ix.program);
            for acc in &ix.accounts {
                if let crate::spec::account::Acc::Fixed(pk, _) = acc {
                    push!(*pk);
                }
            }
        }

        out
    }

    #[must_use]
    pub fn addresses_missing_from(&self, existing: &[Pubkey]) -> Vec<Pubkey> {
        let existing_set: HashSet<&Pubkey> = existing.iter().collect();
        self.static_addresses()
            .into_iter()
            .filter(|pk| !existing_set.contains(pk))
            .collect()
    }
}
