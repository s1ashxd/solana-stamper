use tx_stamper::stamp::patch::{patch_pubkey, patch_u64};

use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use tx_stamper::signer::{KeypairSigner, Signer};
use tx_stamper::spec::{TemplateSpec, MessageVersion};
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::template::Template;

#[test]
fn stamp_simple_transfer() {
    let payer_signer = KeypairSigner::from_bytes(&[9u8; 32]);
    let payer = payer_signer.pubkey();
    let recipient = Pubkey::new_unique();
    let blockhash = Hash::new_from_array([0x33; 32]);

    let mut transfer_data = vec![0u8; 4];
    transfer_data.copy_from_slice(&2u32.to_le_bytes());

    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
        InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&transfer_data).u64_slot("amount")),
    );

    let tpl = Template::compile(spec).unwrap();
    let stamped = tpl
        .stamp()
        .set("recipient", recipient)
        .set("amount", 1_000u64)
        .blockhash(blockhash)
        .sign(&payer_signer)
        .unwrap();

    let tx: solana_sdk::transaction::VersionedTransaction =
        bincode::deserialize(stamped.as_bytes()).unwrap();
    let solana_sdk::message::VersionedMessage::V0(msg) = &tx.message else { panic!() };
    assert_eq!(msg.recent_blockhash, blockhash);
    assert!(msg.account_keys.iter().any(|k| k == &recipient));
}

#[test]
fn patch_pubkey_writes_32_bytes() {
    let mut buf = [0u8; 64];
    let value = [0xAB; 32];
    patch_pubkey(&mut buf, 16, &value);
    for i in 16..48 {
        assert_eq!(buf[i], 0xAB);
    }
    assert_eq!(buf[15], 0);
    assert_eq!(buf[48], 0);
}

#[test]
fn patch_u64_writes_8_le_bytes() {
    let mut buf = [0u8; 16];
    patch_u64(&mut buf, 4, 0xDEADBEEF_CAFEBABE);
    assert_eq!(&buf[4..12], &0xDEADBEEF_CAFEBABE_u64.to_le_bytes());
}
