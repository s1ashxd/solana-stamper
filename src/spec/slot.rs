#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Pubkey,
    U8,
    U16,
    U32,
    U64,
    Hash,
}

#[macro_export]
macro_rules! slot {
    ($name:literal) => { $name };
}
