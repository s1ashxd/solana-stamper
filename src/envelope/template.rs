use std::collections::BTreeMap;
use std::ops::Range;

use crate::compile::scan::find_unique;
use crate::envelope::spec::{BodyEncoding, EnvelopeSpec};
use crate::error::StamperError;
use crate::stamped::StampedTx;

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

        let (mut cl_off, cl_width) = if let Some(ref cl) = spec.content_length {
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
            let delta = u32::try_from(max_len - sentinel_len).expect("delta fits");
            let shift_threshold = u32::try_from(body_off + sentinel_len).expect("threshold fits");
            if let Some(ref mut off) = cl_off
                && *off >= shift_threshold
            {
                *off += delta;
            }
            for range in user_slots.values_mut() {
                if range.start >= shift_threshold {
                    range.start += delta;
                    range.end += delta;
                }
            }
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

    pub fn splice(&mut self, tx: &StampedTx) -> Result<&[u8], StamperError> {
        let body_off = self.body_off as usize;
        let body_max = self.body_max as usize;
        let body_region = &mut self.buf[body_off..body_off + body_max];

        let encoded_len = match self.encoding {
            BodyEncoding::Binary => {
                let src = tx.as_bytes();
                if src.len() > body_max {
                    return Err(StamperError::BodyTooLarge { encoded: src.len(), body_max });
                }
                body_region[..src.len()].copy_from_slice(src);
                src.len()
            }
            BodyEncoding::Base64 => {
                let src = tx.as_bytes();
                let needed = src.len().div_ceil(3) * 4;
                if needed > body_max {
                    return Err(StamperError::BodyTooLarge { encoded: needed, body_max });
                }
                base64_simd::STANDARD
                    .encode(src, base64_simd::Out::from_slice(body_region))
                    .len()
            }
        };

        let suffix_start = body_off + encoded_len;
        let suffix_end = body_off + body_max + self.suffix_len as usize;
        let suffix_src_start = body_off + body_max;
        self.buf.copy_within(suffix_src_start..suffix_end, suffix_start);

        let real_len = suffix_start + self.suffix_len as usize;

        if let Some(cl_off) = self.cl_off {
            let cl_off_usize = if cl_off as usize >= suffix_src_start {
                suffix_start + (cl_off as usize - suffix_src_start)
            } else {
                cl_off as usize
            };
            let cl_width = self.cl_width as usize;
            let body_len_value = encoded_len;
            let digits = format!("{body_len_value}");
            if digits.len() > cl_width {
                return Err(StamperError::ContentLengthOverflow {
                    value: body_len_value,
                    width: self.cl_width,
                });
            }
            let pad_len = cl_width - digits.len();
            for i in 0..pad_len {
                self.buf[cl_off_usize + i] = b' ';
            }
            self.buf[cl_off_usize + pad_len..cl_off_usize + cl_width].copy_from_slice(digits.as_bytes());
        }

        Ok(&self.buf[..real_len])
    }

    pub fn set_user(&mut self, name: &str, value: &[u8]) -> Result<&mut Self, StamperError> {
        let (start, end) = {
            let range = self.user_slots.get(name).ok_or_else(|| StamperError::UnknownUserSlot {
                name: name.to_string(),
            })?;
            (range.start, range.end)
        };
        let sentinel_len = (end - start) as usize;
        if value.len() > sentinel_len {
            return Err(StamperError::UserSlotOverflow {
                name: name.to_string(),
                value: value.len(),
                sentinel: sentinel_len,
            });
        }
        let start = start as usize;
        self.buf[start..start + value.len()].copy_from_slice(value);
        for i in value.len()..sentinel_len {
            self.buf[start + i] = b' ';
        }
        Ok(self)
    }

    pub fn splice_into<'a>(&self, tx: &StampedTx, out: &'a mut Vec<u8>) -> Result<&'a [u8], StamperError> {
        let body_off = self.body_off as usize;
        let body_max = self.body_max as usize;
        let suffix_len = self.suffix_len as usize;

        out.clear();
        out.extend_from_slice(&self.buf[..body_off]);

        let encoded_len = match self.encoding {
            BodyEncoding::Binary => {
                let src = tx.as_bytes();
                if src.len() > body_max {
                    return Err(StamperError::BodyTooLarge { encoded: src.len(), body_max });
                }
                out.extend_from_slice(src);
                src.len()
            }
            BodyEncoding::Base64 => {
                let src = tx.as_bytes();
                let needed = src.len().div_ceil(3) * 4;
                if needed > body_max {
                    return Err(StamperError::BodyTooLarge { encoded: needed, body_max });
                }
                let start = out.len();
                out.resize(start + needed, 0);
                let written = base64_simd::STANDARD
                    .encode(src, base64_simd::Out::from_slice(&mut out[start..]))
                    .len();
                out.truncate(start + written);
                written
            }
        };

        out.extend_from_slice(&self.buf[body_off + body_max..body_off + body_max + suffix_len]);
        let _ = encoded_len;
        Ok(&out[..])
    }
}
