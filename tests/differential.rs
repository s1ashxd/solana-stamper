use ed25519_dalek::Signer as DalekSigner;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::{VersionedMessage, v0::Message as V0Message};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

use tx_stamper::signer::{KeypairSigner, Signer};
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::{MessageVersion, TemplateSpec};
use tx_stamper::template::Template;

#[test]
fn transfer_byte_equal_to_manual_build() {
    let signer = KeypairSigner::from_bytes(&[42u8; 32]);
    let payer = signer.pubkey();
    let recipient = Pubkey::new_unique();
    let amount = 1_000_u64;
    let blockhash = Hash::new_from_array([0x77; 32]);
    let program_id = Pubkey::default();

    let mut data = vec![0u8; 12];
    data[..4].copy_from_slice(&2u32.to_le_bytes());
    data[4..].copy_from_slice(&amount.to_le_bytes());
    let sdk_instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(recipient, false),
        ],
        data,
    };
    let sdk_msg = V0Message::try_compile(&payer, &[sdk_instruction], &[], blockhash).unwrap();
    let sdk_msg_bytes = sdk_msg.serialize();
    let dalek_signing = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
    let sdk_sig: [u8; 64] = dalek_signing.sign(&sdk_msg_bytes).to_bytes();
    let sdk_versioned = VersionedTransaction {
        signatures: vec![Signature::from(sdk_sig)],
        message: VersionedMessage::V0(sdk_msg),
    };
    let manual_bytes = bincode::serialize(&sdk_versioned).unwrap();

    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(program_id)
        .account(Acc::payer())
        .account(Acc::slot_w("recipient"))
        .data(DataSpec::bytes(&2u32.to_le_bytes()).u64_slot("amount")));
    let tpl = Template::compile(spec).unwrap();
    let stamped = tpl
        .stamp()
        .set("recipient", recipient)
        .set("amount", amount)
        .blockhash(blockhash)
        .sign(&signer)
        .unwrap();

    assert_eq!(stamped.as_bytes(), manual_bytes.as_slice());
}
