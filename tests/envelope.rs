use smallvec::SmallVec;
use tx_stamper::envelope::spec::{BodyEncoding, BodyPlaceholder, ContentLengthSpec, EnvelopeSpec, UserSlot};
use tx_stamper::envelope::template::EnvelopeTemplate;

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
