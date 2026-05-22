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
}
