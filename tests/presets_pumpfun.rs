#![cfg(feature = "pumpfun")]

use solana_sdk::pubkey::Pubkey;
use tx_stamper::protocols::common::TokenProgram;
use tx_stamper::protocols::pumpfun::buy_v2_spec;
use tx_stamper::template::Template;

#[test]
fn pumpfun_buy_v2_spec_compiles() {
    let payer = Pubkey::new_unique();
    let spec = buy_v2_spec(payer, TokenProgram::Legacy);
    let tpl = Template::compile(spec).unwrap();
    let names: Vec<&str> = tpl.slot_names().collect();
    for expected in [
        "mint",
        "bonding_curve",
        "associated_bonding_curve",
        "creator_vault",
        "bonding_curve_v2",
        "user_vol",
        "sol_amount",
        "min_tokens_out",
        "cu_limit",
        "cu_price",
        "tip_account",
        "tip_lamports",
    ] {
        assert!(names.contains(&expected), "missing slot {expected}, got: {names:?}");
    }
}

use tx_stamper::protocols::pumpfun::sell_v2_spec;

#[test]
fn pumpfun_sell_v2_spec_compiles() {
    let payer = Pubkey::new_unique();
    let spec = sell_v2_spec(payer, TokenProgram::Legacy);
    let tpl = Template::compile(spec).unwrap();
    let names: Vec<&str> = tpl.slot_names().collect();
    for expected in [
        "mint",
        "bonding_curve",
        "associated_bonding_curve",
        "creator_vault",
        "bonding_curve_v2",
        "token_amount",
        "min_sol_out",
        "cu_limit",
        "cu_price",
        "tip_account",
        "tip_lamports",
    ] {
        assert!(names.contains(&expected), "missing slot {expected}, got: {names:?}");
    }
    assert!(!names.contains(&"user_vol"), "sell should not have user_vol slot");
}

#[test]
fn pumpfun_buy_stamp_end_to_end() {
    use smallvec::smallvec;
    use solana_sdk::hash::Hash;
    use solana_sdk::message::VersionedMessage;
    use solana_sdk::pubkey::Pubkey;
    use tx_stamper::signer::{KeypairSigner, Signer};
    use tx_stamper::stamp::bundle::PerProviderValues;
    use tx_stamper::template::Template;

    let signer = KeypairSigner::from_bytes(&[7u8; 32]);
    let payer = signer.pubkey();
    let spec = tx_stamper::protocols::pumpfun::buy_v2_spec(
        payer,
        tx_stamper::protocols::common::TokenProgram::Legacy,
    );
    let tpl = Template::compile(spec).unwrap();

    let mint = Pubkey::new_unique();
    let bonding_curve = Pubkey::new_unique();
    let abc = Pubkey::new_unique();
    let creator_vault = Pubkey::new_unique();
    let bcv2 = Pubkey::new_unique();
    let user_vol = Pubkey::new_unique();
    let tip_acc = Pubkey::new_unique();

    let provider = PerProviderValues {
        slots: smallvec![
            ("tip_account".to_string(), tip_acc.into()),
            ("tip_lamports".to_string(), 100_000u64.into()),
        ],
    };

    let bundle = tpl
        .stamp_bundle([provider])
        .set("mint", mint)
        .set("bonding_curve", bonding_curve)
        .set("associated_bonding_curve", abc)
        .set("creator_vault", creator_vault)
        .set("bonding_curve_v2", bcv2)
        .set("user_vol", user_vol)
        .set("sol_amount", 1_000_000u64)
        .set("min_tokens_out", 1u64)
        .set("cu_limit", 200_000u32)
        .set("cu_price", 100_000u64)
        .blockhash(Hash::new_from_array([42u8; 32]))
        .sign(&signer)
        .unwrap();

    let stamped = bundle.reconstruct(0);

    let tx: solana_sdk::transaction::VersionedTransaction =
        bincode::deserialize(stamped.as_bytes()).unwrap();
    let VersionedMessage::V0(msg) = &tx.message else {
        panic!("expected V0 message")
    };
    assert!(msg.account_keys.contains(&mint), "mint not in account_keys");
    assert!(msg.account_keys.contains(&tip_acc), "tip_acc not in account_keys");

    let serialized = msg.serialize();
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&payer.to_bytes()).unwrap();
    let sig =
        ed25519_dalek::Signature::from_bytes(tx.signatures[0].as_ref().try_into().unwrap());
    assert!(vk.verify_strict(&serialized, &sig).is_ok(), "signature did not verify");
}
