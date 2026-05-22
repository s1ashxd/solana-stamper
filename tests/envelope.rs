use smallvec::SmallVec;
use tx_stamper::envelope::spec::{BodyEncoding, BodyPlaceholder, ContentLengthSpec, EnvelopeSpec, UserSlot};

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
