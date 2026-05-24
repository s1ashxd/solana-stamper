use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct LookupTable {
    pub address: Pubkey,
    pub addresses: Vec<Pubkey>,
}
