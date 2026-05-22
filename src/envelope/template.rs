use std::collections::BTreeMap;
use std::ops::Range;

use crate::compile::scan::find_unique;
use crate::envelope::spec::{BodyEncoding, EnvelopeSpec};
use crate::error::StamperError;

#[allow(dead_code)]
pub struct EnvelopeTemplate {
    buf: Vec<u8>,
    body_off: u32,
    body_max: u32,
    encoding: BodyEncoding,
    cl_off: Option<u32>,
    cl_width: u8,
    suffix_len: u32,
    user_slots: BTreeMap<String, Range<u32>>,
}

impl EnvelopeTemplate {
    pub fn compile(mut spec: EnvelopeSpec) -> Result<Self, StamperError> {
        let body_off = find_unique(&spec.bytes, &spec.body.sentinel)
            .map_err(|_| StamperError::EnvelopeBodyMissing)?;

        let (cl_off, cl_width) = if let Some(ref cl) = spec.content_length {
            let off = find_unique(&spec.bytes, &cl.sentinel).map_err(|_| StamperError::EnvelopeBodyMissing)?;
            (Some(u32::try_from(off).expect("cl off fits")), cl.width)
        } else {
            (None, 0)
        };

        let mut user_slots: BTreeMap<String, Range<u32>> = BTreeMap::new();
        for us in &spec.user_slots {
            let off = find_unique(&spec.bytes, &us.sentinel).map_err(|_| StamperError::EnvelopeBodyMissing)?;
            let start = u32::try_from(off).expect("user slot off fits");
            let end = start + u32::try_from(us.sentinel.len()).expect("us len fits");
            user_slots.insert(us.name.to_string(), start..end);
        }

        let sentinel_len = spec.body.sentinel.len();
        let max_len = spec.body.max_len;
        if max_len > sentinel_len {
            let mut padded: Vec<u8> = Vec::with_capacity(spec.bytes.len() + (max_len - sentinel_len));
            padded.extend_from_slice(&spec.bytes[..body_off]);
            padded.extend(std::iter::repeat_n(b' ', max_len));
            padded.extend_from_slice(&spec.bytes[body_off + sentinel_len..]);
            spec.bytes = padded;
        }

        let body_max = u32::try_from(max_len).expect("body_max fits");
        let body_off_u32 = u32::try_from(body_off).expect("body off fits");
        let suffix_len = u32::try_from(spec.bytes.len()).expect("buf fits") - body_off_u32 - body_max;

        Ok(Self {
            buf: spec.bytes,
            body_off: body_off_u32,
            body_max,
            encoding: spec.body.encoding,
            cl_off,
            cl_width,
            suffix_len,
            user_slots,
        })
    }

    #[must_use]
    pub fn body_offset(&self) -> u32 {
        self.body_off
    }

    #[must_use]
    pub fn body_max(&self) -> u32 {
        self.body_max
    }

    #[must_use]
    pub fn suffix_len(&self) -> u32 {
        self.suffix_len
    }
}
