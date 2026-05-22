use solana_sdk::signature::Signature;

use crate::compile::MAX_TX_SIZE;

pub struct StampedTx {
    pub(crate) buf: [u8; MAX_TX_SIZE],
    pub(crate) len: u16,
    pub(crate) sig_off: u16,
}

impl StampedTx {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..usize::from(self.len)]
    }

    #[must_use]
    pub fn signature(&self) -> Signature {
        let off = usize::from(self.sig_off);
        let mut s = [0u8; 64];
        s.copy_from_slice(&self.buf[off..off + 64]);
        Signature::from(s)
    }

    #[must_use]
    pub fn to_base64(&self) -> String {
        base64_simd::STANDARD.encode_to_string(self.as_bytes())
    }

    pub fn write_base64_into(&self, out: &mut String) {
        out.clear();
        out.push_str(&self.to_base64());
    }

    #[must_use]
    pub fn signature_base58(&self) -> String {
        let sig = self.signature();
        bs58::encode(sig.as_ref()).into_string()
    }
}
