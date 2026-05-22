use tx_stamper::error::StamperError;

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
