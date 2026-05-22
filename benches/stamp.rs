use criterion::{Criterion, criterion_group, criterion_main};
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use tx_stamper::signer::{KeypairSigner, PrecomputedSigner, Signer};
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::{MessageVersion, TemplateSpec};
use tx_stamper::template::Template;

fn bench_stamp_keypair(c: &mut Criterion) {
    let signer = KeypairSigner::from_bytes(&[3u8; 32]);
    let payer = signer.pubkey();
    let recipient = Pubkey::new_unique();
    let spec =
        TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = Template::compile(spec).unwrap();
    let blockhash = Hash::new_from_array([5u8; 32]);
    c.bench_function("stamp_keypair", |b| {
        b.iter(|| {
            tpl.stamp()
                .set("recipient", recipient)
                .set("amount", 1_000u64)
                .blockhash(blockhash)
                .sign(&signer)
                .unwrap()
        });
    });
}

fn bench_stamp_precomputed(c: &mut Criterion) {
    let signer = PrecomputedSigner::new(&[3u8; 32], 4096);
    let payer = signer.pubkey();
    let recipient = Pubkey::new_unique();
    let spec =
        TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = Template::compile(spec).unwrap();
    let blockhash = Hash::new_from_array([5u8; 32]);
    c.bench_function("stamp_precomputed", |b| {
        b.iter(|| {
            tpl.stamp()
                .set("recipient", recipient)
                .set("amount", 1_000u64)
                .blockhash(blockhash)
                .sign(&signer)
                .unwrap()
        });
    });
}

criterion_group!(benches, bench_stamp_keypair, bench_stamp_precomputed);
criterion_main!(benches);
