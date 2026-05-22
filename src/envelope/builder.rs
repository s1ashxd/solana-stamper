use smallvec::SmallVec;

use crate::envelope::spec::{BodyEncoding, BodyPlaceholder, ContentLengthSpec, EnvelopeSpec, UserSlot};

pub struct EnvelopeSpecBuilder {
    method: &'static str,
    path: String,
    host: Option<String>,
    headers: Vec<(String, String)>,
    body: Option<BodyPlaceholder>,
    content_length: Option<ContentLengthSpec>,
    user_slots: Vec<UserSlot>,
    body_template: Option<String>,
}

impl EnvelopeSpecBuilder {
    #[must_use]
    pub fn http_post(path: &str) -> Self {
        Self {
            method: "POST",
            path: path.to_string(),
            host: None,
            headers: Vec::new(),
            body: None,
            content_length: None,
            user_slots: Vec::new(),
            body_template: None,
        }
    }

    #[must_use]
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn body_json_rpc(mut self, max_body_bytes: usize) -> Self {
        self.body_template = Some(
            r#"{"jsonrpc":"2.0","id":1,"method":"sendTransaction","params":["<<BODY>>",{"encoding":"base64","skipPreflight":true,"maxRetries":0}]}"#.to_string()
        );
        let mut sentinel: SmallVec<[u8; 16]> = SmallVec::new();
        sentinel.extend_from_slice(b"<<BODY>>");
        self.body = Some(BodyPlaceholder {
            sentinel,
            max_len: max_body_bytes,
            encoding: BodyEncoding::Base64,
        });
        let mut cl: SmallVec<[u8; 16]> = SmallVec::new();
        cl.extend_from_slice(b"<<CL>>        ");
        self.content_length = Some(ContentLengthSpec { sentinel: cl, width: 12 });
        self
    }

    #[must_use]
    pub fn user_slot(mut self, name: &'static str, sentinel: &[u8]) -> Self {
        let mut sv: SmallVec<[u8; 16]> = SmallVec::new();
        sv.extend_from_slice(sentinel);
        self.user_slots.push(UserSlot { name, sentinel: sv });
        self
    }

    #[must_use]
    pub fn build(self) -> EnvelopeSpec {
        use std::fmt::Write;
        let mut wire = String::new();
        let _ = write!(wire, "{} {} HTTP/1.1\r\n", self.method, self.path);
        if let Some(h) = &self.host {
            let _ = write!(wire, "Host: {h}\r\n");
        }
        for (k, v) in &self.headers {
            let _ = write!(wire, "{k}: {v}\r\n");
        }
        wire.push_str("Content-Length: <<CL>>        \r\n");
        wire.push_str("\r\n");
        if let Some(t) = &self.body_template {
            wire.push_str(t);
        }
        let bytes = wire.into_bytes();

        EnvelopeSpec {
            bytes,
            body: self.body.expect("body required"),
            content_length: self.content_length,
            user_slots: self.user_slots.into_iter().collect(),
        }
    }
}
