use criterion::{Criterion, criterion_group, criterion_main};
use smallvec::SmallVec;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use tx_stamper::envelope::spec::{BodyEncoding, BodyPlaceholder, EnvelopeSpec};
use tx_stamper::envelope::template::EnvelopeTemplate;
use tx_stamper::signer::{KeypairSigner, Signer};
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::{MessageVersion, TemplateSpec};
use tx_stamper::template::Template;

fn bench_splice_base64(c: &mut Criterion) {
    let signer = KeypairSigner::from_bytes(&[3u8; 32]);
    let payer = signer.pubkey();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
        InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = Template::compile(spec).unwrap();
    let stamped = tpl.stamp()
        .set("recipient", Pubkey::new_unique())
        .set("amount", 1u64)
        .blockhash(Hash::new_from_array([1u8; 32]))
        .sign(&signer).unwrap();

    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let env_spec = EnvelopeSpec {
        bytes: b"PRE=<<BODY>>=POST".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 4096,
            encoding: BodyEncoding::Base64,
        },
        content_length: None,
        user_slots: SmallVec::new(),
    };
    let env = EnvelopeTemplate::compile(env_spec).unwrap();
    let mut out: Vec<u8> = Vec::with_capacity(4096);
    c.bench_function("envelope_splice_base64", |b| {
        b.iter(|| {
            env.splice_into(&stamped, &mut out).unwrap().len()
        });
    });
}

criterion_group!(benches, bench_splice_base64);
criterion_main!(benches);
