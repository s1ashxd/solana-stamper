use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

use crate::spec::slot::SlotKind;

#[derive(Debug, Error)]
pub enum StamperError {
    #[error("duplicate slot name: {name}")]
    DuplicateSlotName { name: String },

    #[error("computed slot {computed} depends on undeclared slot {missing}")]
    MissingDependency { computed: String, missing: String },

    #[error("cyclic dependency in computed slots: {cycle:?}")]
    CyclicComputed { cycle: Vec<String> },

    #[error("computed slot {computed} cannot depend on per-provider slot {dep}")]
    ComputedDependsOnPerProvider { computed: String, dep: String },

    #[error("payer mismatch in instruction {ix}")]
    PayerMismatch { ix: usize },

    #[error("sentinel for slot {name} not found in serialized buffer")]
    SentinelNotFound { name: String },

    #[error("sentinel collision between slot {a} and slot {b}")]
    MarkerCollision { a: String, b: String },

    #[error("fixed account pubkey {pk} matches a sentinel pattern")]
    FixedPubkeyHitsSentinel { pk: Pubkey },

    #[error("transaction too large after serialization: {size}")]
    TransactionTooLarge { size: usize },

    #[error("V0 message compile failed: {0}")]
    V0Compile(String),

    #[error("lookup table {idx}: {reason}")]
    InvalidLookupTable { idx: usize, reason: &'static str },

    #[error("missing value for slot {name} (required by {required_by:?})")]
    MissingSlotValue {
        name: String,
        required_by: Vec<String>,
    },

    #[error("wrong type for slot {name}: expected {expected:?}, got {got:?}")]
    WrongSlotType {
        name: String,
        expected: SlotKind,
        got: SlotKind,
    },

    #[error("slot {name} is per-provider; use stamp_bundle() instead of stamp()")]
    PerProviderInSingleStamp { name: String },

    #[error("computed slot {name} failed")]
    ComputeFailed {
        name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("signer pubkey mismatch at index {index}: expected {expected}, got {got}")]
    SignerMismatch {
        expected: Pubkey,
        got: Pubkey,
        index: usize,
    },

    #[error("missing additional signer {name}")]
    MissingAdditionalSigner { name: String },

    #[error("provider input {idx}: missing per-provider slot {name}")]
    BundleMissingPerProviderSlot { idx: usize, name: String },

    #[error("provider index out of bounds: {idx} >= {len}")]
    ProviderOutOfBounds { idx: usize, len: usize },
}
