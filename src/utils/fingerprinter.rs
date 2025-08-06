use sha2::{Digest, Sha256};

pub fn username_from_fingerprint(fingerprint: &str) -> String {
  let hash = Sha256::digest(fingerprint);
  format!("user_{}", hex::encode(hash)[..6].to_owned())
}
