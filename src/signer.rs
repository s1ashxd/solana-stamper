use ed25519_dalek::{Signer as DalekSigner, SigningKey};
use solana_sdk::pubkey::Pubkey;

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
