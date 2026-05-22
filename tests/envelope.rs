use smallvec::SmallVec;
use tx_stamper::envelope::spec::{BodyEncoding, BodyPlaceholder, ContentLengthSpec, EnvelopeSpec, UserSlot};
use tx_stamper::envelope::template::EnvelopeTemplate;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use tx_stamper::signer::{KeypairSigner, Signer};
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::{MessageVersion, TemplateSpec};
use tx_stamper::template::Template;

fn make_stamped() -> tx_stamper::stamped::StampedTx {
    let signer = KeypairSigner::from_bytes(&[5u8; 32]);
    let payer = signer.pubkey();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
        InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")),
    );
    let tpl = Template::compile(spec).unwrap();
    tpl.stamp()
        .set("recipient", Pubkey::new_unique())
        .set("amount", 1_000u64)
        .blockhash(Hash::new_from_array([1u8; 32]))
        .sign(&signer)
        .unwrap()
}

#[test]
fn envelope_spec_construction() {
    let body = BodyPlaceholder {
        sentinel: {
            let mut sv: SmallVec<[u8; 16]> = SmallVec::new();
            sv.extend_from_slice(b"<<BODY>>");
            sv
        },
        max_len: 2048,
        encoding: BodyEncoding::Base64,
    };
    let cl = ContentLengthSpec {
        sentinel: {
            let mut sv: SmallVec<[u8; 16]> = SmallVec::new();
            sv.extend_from_slice(b"<<CL>>     ");
            sv
        },
        width: 6,
    };
    let mut user_slot_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    user_slot_sentinel.extend_from_slice(b"<<UUID>>");
    let spec = EnvelopeSpec {
        bytes: b"POST /v1/tx HTTP/1.1\r\n".to_vec(),
        body,
        content_length: Some(cl),
        user_slots: smallvec::smallvec![UserSlot { name: "uuid", sentinel: user_slot_sentinel }],
    };
    assert_eq!(spec.bytes.len(), 22);
    assert!(spec.content_length.is_some());
    assert_eq!(spec.user_slots.len(), 1);
}

#[test]
fn envelope_compile_records_body_offset() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let buf = b"prefix-<<BODY>>-suffix".to_vec();
    let spec = EnvelopeSpec {
        bytes: buf,
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 100,
            encoding: BodyEncoding::Binary,
        },
        content_length: None,
        user_slots: SmallVec::new(),
    };
    let env = EnvelopeTemplate::compile(spec).unwrap();
    assert!(env.body_max() >= 8);
    assert!(env.body_offset() > 0);
}

#[test]
fn envelope_compile_missing_body_sentinel_errors() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<MISSING>>");
    let spec = EnvelopeSpec {
        bytes: b"plain-buf".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 100,
            encoding: BodyEncoding::Binary,
        },
        content_length: None,
        user_slots: SmallVec::new(),
    };
    assert!(matches!(
        EnvelopeTemplate::compile(spec).err().unwrap(),
        tx_stamper::error::StamperError::EnvelopeBodyMissing
    ));
}

#[test]
fn envelope_splice_base64_body() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let stamped = make_stamped();
    let estimated_b64_len = stamped.as_bytes().len().div_ceil(3) * 4;
    let spec = EnvelopeSpec {
        bytes: b"BODY=<<BODY>>;END".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: estimated_b64_len + 8,
            encoding: BodyEncoding::Base64,
        },
        content_length: None,
        user_slots: SmallVec::new(),
    };
    let mut env = EnvelopeTemplate::compile(spec).unwrap();
    let wire = env.splice(&stamped).unwrap();
    let prefix = &wire[..5];
    assert_eq!(prefix, b"BODY=");
    let end = &wire[wire.len() - 4..];
    assert_eq!(end, b";END");
    let b64_part = &wire[5..wire.len() - 4];
    let decoded = base64_simd::STANDARD.decode_to_vec(b64_part).unwrap();
    assert_eq!(decoded, stamped.as_bytes());
}

#[test]
fn envelope_splice_body_too_large_errors() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let stamped = make_stamped();
    let spec = EnvelopeSpec {
        bytes: b"<<BODY>>".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 50,
            encoding: BodyEncoding::Base64,
        },
        content_length: None,
        user_slots: SmallVec::new(),
    };
    let mut env = EnvelopeTemplate::compile(spec).unwrap();
    let err = env.splice(&stamped).err().unwrap();
    assert!(matches!(err, tx_stamper::error::StamperError::BodyTooLarge { .. }));
}

#[test]
fn envelope_content_length_written_correctly() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let mut cl_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    cl_sentinel.extend_from_slice(b"<<CL>>     ");
    let stamped = make_stamped();
    let spec = EnvelopeSpec {
        bytes: b"CL:<<CL>>     |B:<<BODY>>;".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 4096,
            encoding: BodyEncoding::Base64,
        },
        content_length: Some(ContentLengthSpec {
            sentinel: cl_sentinel,
            width: 11,
        }),
        user_slots: SmallVec::new(),
    };
    let mut env = EnvelopeTemplate::compile(spec).unwrap();
    let wire = env.splice(&stamped).unwrap();
    let cl_segment = std::str::from_utf8(&wire[3..14]).unwrap();
    let cl_value: usize = cl_segment.trim().parse().unwrap();
    let stamped_b64_len = stamped.as_bytes().len().div_ceil(3) * 4;
    assert_eq!(cl_value, stamped_b64_len);
}

#[test]
fn envelope_content_length_overflow_errors() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let mut cl_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    cl_sentinel.extend_from_slice(b"CL");
    let stamped = make_stamped();
    let spec = EnvelopeSpec {
        bytes: b"CL|<<BODY>>".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 4096,
            encoding: BodyEncoding::Base64,
        },
        content_length: Some(ContentLengthSpec {
            sentinel: cl_sentinel,
            width: 1,
        }),
        user_slots: SmallVec::new(),
    };
    let mut env = EnvelopeTemplate::compile(spec).unwrap();
    let err = env.splice(&stamped).err().unwrap();
    assert!(matches!(err, tx_stamper::error::StamperError::ContentLengthOverflow { .. }));
}

#[test]
fn envelope_user_slot_set_and_splice() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let mut uuid_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    uuid_sentinel.extend_from_slice(b"<<UUID>>");
    let stamped = make_stamped();
    let spec = EnvelopeSpec {
        bytes: b"auth=<<UUID>>;body=<<BODY>>;".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 4096,
            encoding: BodyEncoding::Binary,
        },
        content_length: None,
        user_slots: smallvec::smallvec![UserSlot { name: "uuid", sentinel: uuid_sentinel }],
    };
    let mut env = EnvelopeTemplate::compile(spec).unwrap();
    env.set_user("uuid", b"deadbeef").unwrap();
    let wire = env.splice(&stamped).unwrap();
    assert!(wire.windows(8).any(|w| w == b"deadbeef"));
}

#[test]
fn envelope_user_slot_value_too_large_errors() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let mut uuid_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    uuid_sentinel.extend_from_slice(b"<<U>>");
    let spec = EnvelopeSpec {
        bytes: b"<<U>>+<<BODY>>".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 4096,
            encoding: BodyEncoding::Binary,
        },
        content_length: None,
        user_slots: smallvec::smallvec![UserSlot { name: "u", sentinel: uuid_sentinel }],
    };
    let mut env = EnvelopeTemplate::compile(spec).unwrap();
    let err = env.set_user("u", b"too-long-for-5-byte-sentinel").err().unwrap();
    assert!(matches!(err, tx_stamper::error::StamperError::UserSlotOverflow { .. }));
}

#[test]
fn envelope_user_slot_unknown_errors() {
    let mut body_sentinel: SmallVec<[u8; 16]> = SmallVec::new();
    body_sentinel.extend_from_slice(b"<<BODY>>");
    let spec = EnvelopeSpec {
        bytes: b"<<BODY>>".to_vec(),
        body: BodyPlaceholder {
            sentinel: body_sentinel,
            max_len: 100,
            encoding: BodyEncoding::Binary,
        },
        content_length: None,
        user_slots: SmallVec::new(),
    };
    let mut env = EnvelopeTemplate::compile(spec).unwrap();
    let err = env.set_user("nope", b"x").err().unwrap();
    assert!(matches!(err, tx_stamper::error::StamperError::UnknownUserSlot { .. }));
}
