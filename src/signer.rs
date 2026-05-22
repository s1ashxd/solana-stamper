use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use arc_swap::ArcSwap;
use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
use curve25519_dalek::scalar::Scalar;
use ed25519_dalek::hazmat::ExpandedSecretKey;
use ed25519_dalek::{Signer as DalekSigner, SigningKey};
use sha2::{Digest, Sha512};
use solana_sdk::pubkey::Pubkey;
use zeroize::Zeroizing;

pub trait Signer {
    fn pubkey(&self) -> Pubkey;

    fn sign(&self, message: &[u8]) -> [u8; 64];
}

pub struct KeypairSigner {
    signing_key: SigningKey,
    pubkey: Pubkey,
}

impl KeypairSigner {
    #[must_use]
    pub fn from_bytes(secret: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(secret);
        let pubkey = Pubkey::new_from_array(signing_key.verifying_key().to_bytes());
        Self { signing_key, pubkey }
    }
}

impl Signer for KeypairSigner {
    fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.signing_key.sign(message).to_bytes()
    }
}

struct NonceEntry {
    r_bytes: [u8; 32],
    r_scalar: Scalar,
}

struct PoolState {
    nonces: Vec<NonceEntry>,
    cursor: AtomicUsize,
}

impl PoolState {
    fn new(nonces: Vec<NonceEntry>) -> Self {
        Self { nonces, cursor: AtomicUsize::new(0) }
    }
}

pub struct PrecomputedSigner {
    secret: Zeroizing<[u8; 32]>,
    scalar: Scalar,
    hash_prefix: [u8; 32],
    pubkey_bytes: [u8; 32],
    pubkey: Pubkey,
    pool: ArcSwap<PoolState>,
}

impl PrecomputedSigner {
    #[must_use]
    pub fn new(secret: &[u8; 32], pool_size: usize) -> Self {
        let signing_key = SigningKey::from_bytes(secret);
        let pubkey_bytes = signing_key.verifying_key().to_bytes();
        let pubkey = Pubkey::new_from_array(pubkey_bytes);
        let expanded = ExpandedSecretKey::from(secret);
        let scalar = expanded.scalar;
        let hash_prefix = expanded.hash_prefix;
        let pool = PoolState::new(generate_nonces(&hash_prefix, pool_size));
        Self {
            secret: Zeroizing::new(*secret),
            scalar,
            hash_prefix,
            pubkey_bytes,
            pubkey,
            pool: ArcSwap::from_pointee(pool),
        }
    }

    pub fn refill(&self, count: usize) {
        let new_pool = PoolState::new(generate_nonces(&self.hash_prefix, count));
        self.pool.store(Arc::new(new_pool));
    }

    #[must_use]
    pub fn pool_remaining(&self) -> usize {
        let pool = self.pool.load();
        pool.nonces.len().saturating_sub(pool.cursor.load(Ordering::Acquire))
    }

    fn try_sign_with_pool(&self, message: &[u8]) -> Option<[u8; 64]> {
        let pool = self.pool.load();
        let idx = pool.cursor.fetch_add(1, Ordering::AcqRel);
        let entry = pool.nonces.get(idx)?;
        let mut hasher = Sha512::new();
        hasher.update(entry.r_bytes);
        hasher.update(self.pubkey_bytes);
        hasher.update(message);
        let k = Scalar::from_hash(hasher);
        let s = entry.r_scalar + k * self.scalar;
        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(&entry.r_bytes);
        sig[32..].copy_from_slice(s.as_bytes());
        Some(sig)
    }

    fn deterministic(&self, message: &[u8]) -> [u8; 64] {
        let signing_key = SigningKey::from_bytes(&self.secret);
        signing_key.sign(message).to_bytes()
    }
}

impl Drop for PrecomputedSigner {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.scalar = Scalar::ZERO;
        self.hash_prefix.zeroize();
    }
}

impl Signer for PrecomputedSigner {
    fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.try_sign_with_pool(message).unwrap_or_else(|| self.deterministic(message))
    }
}

fn generate_nonces(hash_prefix: &[u8; 32], count: usize) -> Vec<NonceEntry> {
    let mut out = Vec::with_capacity(count);
    let mut counter = [0u8; 32];
    for i in 0..count {
        counter[..8].copy_from_slice(&(i as u64).to_le_bytes());
        let mut hasher = Sha512::new();
        hasher.update(hash_prefix);
        hasher.update(counter);
        let r_scalar = Scalar::from_hash(hasher);
        let r_point = ED25519_BASEPOINT_TABLE * &r_scalar;
        let r_bytes = r_point.compress().to_bytes();
        out.push(NonceEntry { r_bytes, r_scalar });
    }
    out
}
