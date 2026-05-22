pub struct MarkerAllocator {
    pubkey: u8,
    hash: u8,
    signer: u8,
    u128_n: u16,
    u64_n: u16,
    u32_n: u16,
    u16_n: u16,
    u8_n: u16,
}

impl MarkerAllocator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pubkey: 0,
            hash: 0,
            signer: 0,
            u128_n: 0,
            u64_n: 0,
            u32_n: 0,
            u16_n: 0,
            u8_n: 0,
        }
    }

    pub fn pubkey_sentinel(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.fill(0xC0);
        out[31] = self.pubkey;
        self.pubkey = self.pubkey.checked_add(1).expect("too many pubkey slots");
        out
    }

    pub fn hash_sentinel(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.fill(0xB0);
        out[31] = self.hash;
        self.hash = self.hash.checked_add(1).expect("too many hash slots");
        out
    }

    pub fn signer_sentinel(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.fill(0xD0);
        out[31] = self.signer;
        self.signer = self.signer.checked_add(1).expect("too many signer slots");
        out
    }

    pub fn u128_magic(&mut self) -> u128 {
        let n = u128::from(self.u128_n);
        self.u128_n = self.u128_n.checked_add(1).expect("too many u128 slots");
        0xC0DE_C0DE_C0DE_C0DE_C0DE_C0DE_C0DE_0000_u128 | n
    }

    pub fn u64_magic(&mut self) -> u64 {
        let n = u64::from(self.u64_n);
        self.u64_n = self.u64_n.checked_add(1).expect("too many u64 slots");
        0xFADE_CAFE_B0BA_0000 | n
    }

    pub fn u32_magic(&mut self) -> u32 {
        let n = u32::from(self.u32_n);
        self.u32_n = self.u32_n.checked_add(1).expect("too many u32 slots");
        0xDEAD_BE00 | n
    }

    pub fn u16_magic(&mut self) -> u16 {
        let n = self.u16_n;
        self.u16_n = self.u16_n.checked_add(1).expect("too many u16 slots");
        0xBEE0 | n
    }

    pub fn u8_magic(&mut self) -> u8 {
        let n = self.u8_n;
        self.u8_n = self.u8_n.checked_add(1).expect("u8 magic counter overflow");
        u8::try_from(n).expect("u8 counter exceeds u8 range") | 0xE0
    }
}

impl Default for MarkerAllocator {
    fn default() -> Self {
        Self::new()
    }
}
