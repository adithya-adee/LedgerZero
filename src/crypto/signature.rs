use crate::core::types::{Address, PublicKeyBytes, Signature, TransactionHash};
use crate::crypto::hash::sha256_raw;
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};

/// Derives an address from a public key using SHA-256
pub fn address_from_pubkey(pubkey: &PublicKeyBytes) -> Address {
    sha256_raw(pubkey)
}

/// Verifies a signature against a transaction hash
/// Note: Takes the hash directly, not the transaction object
pub fn verify_signature(
    tx_hash: &TransactionHash,
    sig_bytes: &Signature,
    pubkey: &PublicKeyBytes,
) -> bool {
    // Parse the public key bytes into a VerifyingKey
    let verifying_key = match VerifyingKey::from_bytes(pubkey) {
        Ok(vk) => vk,
        Err(_) => return false,
    };

    // Convert our signature bytes to ed25519_dalek::Signature
    let signature = Ed25519Signature::from_bytes(sig_bytes);

    // Verify the signature against the transaction hash
    verifying_key.verify(tx_hash, &signature).is_ok()
}
