pub struct MarkerAllocator {
    pubkey_counter: u8,
    hash_counter: u8,
    signer_counter: u8,
    u64_counter: u16,
    u32_counter: u16,
    u16_counter: u16,
    u8_counter: u16,
}

impl MarkerAllocator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pubkey_counter: 0,
            hash_counter: 0,
            signer_counter: 0,
            u64_counter: 0,
            u32_counter: 0,
            u16_counter: 0,
            u8_counter: 0,
        }
    }

    pub fn pubkey_sentinel(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.fill(0xC0);
        out[31] = self.pubkey_counter;
        self.pubkey_counter = self.pubkey_counter.checked_add(1).expect("too many pubkey slots");
        out
    }

    pub fn hash_sentinel(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.fill(0xB0);
        out[31] = self.hash_counter;
        self.hash_counter = self.hash_counter.checked_add(1).expect("too many hash slots");
        out
    }

    pub fn signer_sentinel(&mut self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.fill(0xD0);
        out[31] = self.signer_counter;
        self.signer_counter = self.signer_counter.checked_add(1).expect("too many signer slots");
        out
    }

    pub fn u64_magic(&mut self) -> u64 {
        let n = u64::from(self.u64_counter);
        self.u64_counter = self.u64_counter.checked_add(1).expect("too many u64 slots");
        0xFADE_CAFE_B0BA_0000 | n
    }

    pub fn u32_magic(&mut self) -> u32 {
        let n = u32::from(self.u32_counter);
        self.u32_counter = self.u32_counter.checked_add(1).expect("too many u32 slots");
        0xDEAD_BE00 | n
    }

    pub fn u16_magic(&mut self) -> u16 {
        let n = self.u16_counter;
        self.u16_counter = self.u16_counter.checked_add(1).expect("too many u16 slots");
        0xBEE0 | n
    }

    pub fn u8_magic(&mut self) -> u8 {
        let n = self.u8_counter;
        self.u8_counter = self.u8_counter.checked_add(1).expect("u8 magic counter overflow");
        u8::try_from(n).expect("u8 counter exceeds u8 range") | 0xE0
    }
}

impl Default for MarkerAllocator {
    fn default() -> Self {
        Self::new()
    }
}
