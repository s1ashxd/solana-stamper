use tx_stamper::error::StamperError;

#[test]
fn stamp_signature_verifies_against_payer() {
    use solana_sdk::hash::Hash;
    use solana_sdk::message::VersionedMessage;
    use solana_sdk::pubkey::Pubkey;
    use tx_stamper::spec::account::Acc;
    use tx_stamper::spec::data::DataSpec;
    use tx_stamper::spec::instruction::InstructionSpec;
    use tx_stamper::spec::{MessageVersion, TemplateSpec};
    use tx_stamper::template::Template;

    let signer = KeypairSigner::from_bytes(&[5u8; 32]);
    let payer = signer.pubkey();
    let recipient = Pubkey::new_unique();

    let spec =
        TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = Template::compile(spec).unwrap();
    let stamped = tpl
        .stamp()
        .set("recipient", recipient)
        .set("amount", 5_000u64)
        .blockhash(Hash::new_from_array([7u8; 32]))
        .sign(&signer)
        .unwrap();

    let tx: solana_sdk::transaction::VersionedTransaction =
        bincode::deserialize(stamped.as_bytes()).unwrap();
    let VersionedMessage::V0(msg) = &tx.message else {
        panic!()
    };
    let serialized = msg.serialize();
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&payer.to_bytes()).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(tx.signatures[0].as_ref().try_into().unwrap());
    assert!(vk.verify_strict(&serialized, &sig).is_ok());
}

#[test]
fn stamper_error_variants_exist() {
    let e = StamperError::DuplicateSlotName { name: "x".into() };
    assert!(format!("{e}").contains("duplicate"));
    let e = StamperError::TransactionTooLarge { size: 1300 };
    assert!(format!("{e}").contains("1300"));
}

use ed25519_dalek::Verifier;
use tx_stamper::signer::{KeypairSigner, Signer};

#[test]
fn keypair_signer_round_trip() {
    let secret = [42u8; 32];
    let signer = KeypairSigner::from_bytes(&secret);
    let msg = b"hello world";
    let sig = signer.sign(msg);
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&signer.pubkey().to_bytes()).unwrap();
    let dalek_sig = ed25519_dalek::Signature::from_bytes(&sig);
    assert!(vk.verify(msg, &dalek_sig).is_ok());
}

use tx_stamper::signer::PrecomputedSigner;

#[test]
fn precomputed_signer_signs_many() {
    let secret = [7u8; 32];
    let signer = PrecomputedSigner::new(&secret, 8);
    let kp = KeypairSigner::from_bytes(&secret);
    assert_eq!(signer.pubkey(), kp.pubkey());
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&signer.pubkey().to_bytes()).unwrap();
    for i in 0..32u32 {
        let msg = i.to_le_bytes();
        let sig = signer.sign(&msg);
        let dalek_sig = ed25519_dalek::Signature::from_bytes(&sig);
        assert!(vk.verify(&msg, &dalek_sig).is_ok(), "iteration {i}");
    }
}
