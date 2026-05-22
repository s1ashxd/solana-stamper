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
    for b in &buf[16..48] {
        assert_eq!(*b, 0xAB);
    }
    assert_eq!(buf[15], 0);
    assert_eq!(buf[48], 0);
}

#[test]
fn patch_u64_writes_8_le_bytes() {
    let mut buf = [0u8; 16];
    patch_u64(&mut buf, 4, 0xDEAD_BEEF_CAFE_BABE);
    assert_eq!(&buf[4..12], &0xDEAD_BEEF_CAFE_BABE_u64.to_le_bytes());
}

#[test]
fn stamp_bundle_two_providers_produce_distinct_txs() {
    use smallvec::smallvec;
    use tx_stamper::spec::prefix::{PrefixOptions, TipTransferSlots};
    use tx_stamper::stamp::bundle::PerProviderValues;

    let signer = KeypairSigner::from_bytes(&[3u8; 32]);
    let payer = signer.pubkey();

    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(PrefixOptions::default().with_tip_transfer(TipTransferSlots {
            account_slot: "tip_account",
            lamports_slot: "tip_lamports",
            per_provider: true,
        }))
        .ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = Template::compile(spec).unwrap();

    let providers = vec![
        PerProviderValues {
            slots: smallvec![
                ("tip_account".into(), Pubkey::new_unique().into()),
                ("tip_lamports".into(), 100_000u64.into()),
            ],
        },
        PerProviderValues {
            slots: smallvec![
                ("tip_account".into(), Pubkey::new_unique().into()),
                ("tip_lamports".into(), 200_000u64.into()),
            ],
        },
    ];

    let bundle = tpl.stamp_bundle(providers)
        .set("recipient", Pubkey::new_unique())
        .set("amount", 1_000u64)
        .blockhash(Hash::new_from_array([8u8; 32]))
        .sign(&signer).unwrap();

    assert_eq!(bundle.len(), 2);
    assert!(!bundle.is_empty());
    let tx0 = bundle.reconstruct(0);
    let tx1 = bundle.reconstruct(1);
    assert_ne!(tx0.as_bytes(), tx1.as_bytes());
}

#[test]
fn bundle_reconstruct_equals_individual_stamp() {
    use smallvec::smallvec;
    use tx_stamper::spec::prefix::{PrefixOptions, TipTransferSlots};
    use tx_stamper::stamp::bundle::PerProviderValues;

    let signer = KeypairSigner::from_bytes(&[11u8; 32]);
    let payer = signer.pubkey();
    let recipient = Pubkey::new_unique();
    let tip_a = Pubkey::new_unique();
    let blockhash = Hash::new_from_array([12u8; 32]);

    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(PrefixOptions::default().with_tip_transfer(TipTransferSlots {
            account_slot: "tip_account",
            lamports_slot: "tip_lamports",
            per_provider: true,
        }))
        .ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = Template::compile(spec).unwrap();

    let bundle = tpl.stamp_bundle(vec![PerProviderValues {
        slots: smallvec![
            ("tip_account".into(), tip_a.into()),
            ("tip_lamports".into(), 100_000u64.into()),
        ],
    }])
        .set("recipient", recipient)
        .set("amount", 5_000u64)
        .blockhash(blockhash)
        .sign(&signer).unwrap();

    let from_bundle = bundle.reconstruct(0);

    let spec2 = TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(PrefixOptions::default().with_tip_transfer(TipTransferSlots {
            account_slot: "tip_account",
            lamports_slot: "tip_lamports",
            per_provider: false,
        }))
        .ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl_solo = Template::compile(spec2).unwrap();

    let direct = tpl_solo.stamp()
        .set("recipient", recipient)
        .set("amount", 5_000u64)
        .set("tip_account", tip_a)
        .set("tip_lamports", 100_000u64)
        .blockhash(blockhash)
        .sign(&signer).unwrap();

    assert_eq!(from_bundle.as_bytes(), direct.as_bytes());
}
