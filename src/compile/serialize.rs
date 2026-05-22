use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::{AddressLookupTableAccount, VersionedMessage, v0::Message as V0Message};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

use crate::compile::MAX_TX_SIZE;
use crate::error::StamperError;

pub fn serialize_placeholder_tx(
    payer: &Pubkey,
    blockhash: Hash,
    ixs: &[Instruction],
    luts: &[AddressLookupTableAccount],
    signature_count: usize,
) -> Result<Vec<u8>, StamperError> {
    let msg = V0Message::try_compile(payer, ixs, luts, blockhash)
        .map_err(|e| StamperError::V0Compile(e.to_string()))?;
    let signatures = vec![Signature::from([0xAA; 64]); signature_count];
    let tx = VersionedTransaction {
        signatures,
        message: VersionedMessage::V0(msg),
    };
    let bytes = bincode::serialize(&tx).map_err(|e| StamperError::V0Compile(e.to_string()))?;
    if bytes.len() > MAX_TX_SIZE {
        return Err(StamperError::TransactionTooLarge { size: bytes.len() });
    }
    Ok(bytes)
}
