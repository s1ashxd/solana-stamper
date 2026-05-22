use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub enum DataPiece {
    Bytes(SmallVec<[u8; 16]>),
    U8Slot(&'static str),
    U16Slot(&'static str),
    U32Slot(&'static str),
    U64Slot(&'static str),
    PubkeySlot(&'static str),
}

#[derive(Debug, Clone, Default)]
pub struct DataSpec(pub SmallVec<[DataPiece; 8]>);

impl DataSpec {
    #[must_use]
    pub fn bytes(b: &[u8]) -> Self {
        let mut sv: SmallVec<[u8; 16]> = SmallVec::new();
        sv.extend_from_slice(b);
        Self(smallvec::smallvec![DataPiece::Bytes(sv)])
    }

    #[must_use]
    pub fn disc(d: &[u8; 8]) -> Self {
        Self::bytes(d)
    }

    #[must_use]
    pub fn u8_slot(mut self, n: &'static str) -> Self {
        self.0.push(DataPiece::U8Slot(n));
        self
    }

    #[must_use]
    pub fn u16_slot(mut self, n: &'static str) -> Self {
        self.0.push(DataPiece::U16Slot(n));
        self
    }

    #[must_use]
    pub fn u32_slot(mut self, n: &'static str) -> Self {
        self.0.push(DataPiece::U32Slot(n));
        self
    }

    #[must_use]
    pub fn u64_slot(mut self, n: &'static str) -> Self {
        self.0.push(DataPiece::U64Slot(n));
        self
    }

    #[must_use]
    pub fn pubkey_slot(mut self, n: &'static str) -> Self {
        self.0.push(DataPiece::PubkeySlot(n));
        self
    }
}
