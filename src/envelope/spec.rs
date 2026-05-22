use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyEncoding {
    Binary,
    Base64,
}

#[derive(Debug, Clone)]
pub struct BodyPlaceholder {
    pub sentinel: SmallVec<[u8; 16]>,
    pub max_len: usize,
    pub encoding: BodyEncoding,
}

#[derive(Debug, Clone)]
pub struct ContentLengthSpec {
    pub sentinel: SmallVec<[u8; 16]>,
    pub width: u8,
}

#[derive(Debug, Clone)]
pub struct UserSlot {
    pub name: &'static str,
    pub sentinel: SmallVec<[u8; 16]>,
}

#[derive(Debug, Clone)]
pub struct EnvelopeSpec {
    pub bytes: Vec<u8>,
    pub body: BodyPlaceholder,
    pub content_length: Option<ContentLengthSpec>,
    pub user_slots: SmallVec<[UserSlot; 4]>,
}
