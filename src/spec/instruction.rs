use smallvec::SmallVec;
use solana_sdk::pubkey::Pubkey;

use crate::spec::account::Acc;
use crate::spec::data::DataSpec;

#[derive(Clone)]
pub struct InstructionSpec {
    pub program: Pubkey,
    pub accounts: SmallVec<[Acc; 16]>,
    pub data: DataSpec,
}

impl InstructionSpec {
    #[must_use]
    pub fn new(program: Pubkey) -> Self {
        Self {
            program,
            accounts: SmallVec::new(),
            data: DataSpec::default(),
        }
    }

    #[must_use]
    pub fn account(mut self, a: Acc) -> Self {
        self.accounts.push(a);
        self
    }

    #[must_use]
    pub fn data(mut self, d: DataSpec) -> Self {
        self.data = d;
        self
    }
}
