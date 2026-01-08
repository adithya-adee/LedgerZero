pub use crate::core::transaction::Transaction;
pub use crate::core::types::{Address, PublicKeyBytes, Signature};
pub use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
pub use sha2::{Digest, Sha256};

pub fn address_from_pubkey(pubkey: &PublicKeyBytes) -> Address {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    hasher.finalize().into()
}

pub fn verify_signature(tx: &Transaction, sig_bytes: &Signature, pubkey: &PublicKeyBytes) -> bool {
    let tx_hash = tx.hash();

    // Parse the public key bytes into a VerifyingKey
    let verifying_key = match VerifyingKey::from_bytes(pubkey) {
        Ok(vk) => vk,
        Err(_) => return false,
    };

    // Convert our signature bytes to ed25519_dalek::Signature
    let signature = Ed25519Signature::from_bytes(sig_bytes);

    // Verify the signature against the transaction hash
    verifying_key.verify(&tx_hash, &signature).is_ok()
}
