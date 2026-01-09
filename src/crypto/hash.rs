use serde::Serialize;
use sha2::{Digest, Sha256};

/// Generic SHA-256 hashing with integrated postcard serialization.
/// Works with any type that implements Serialize.
pub fn sha256<T: Serialize>(data: &T) -> [u8; 32] {
    let bytes = postcard::to_allocvec(data).expect("serialization must be deterministic");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

/// Raw SHA-256 for byte arrays (no serialization)
pub fn sha256_raw(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
