use smallvec::SmallVec;
use solana_sdk::pubkey::Pubkey;

use crate::spec::account::Acc;

#[derive(Clone, Debug)]
pub enum AddressSource {
    Fixed(Pubkey),
    Slot(&'static str),
}

#[derive(Clone)]
pub struct LookupTableSpec {
    pub address: AddressSource,
    pub keys: SmallVec<[Acc; 64]>,
}
