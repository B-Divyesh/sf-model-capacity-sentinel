use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Clone)]
pub struct SecretBox(Aes256Gcm);

impl SecretBox {
    pub fn from_data_dir(data_dir: &Path) -> Result<Self> {
        let key = if let Ok(raw) = std::env::var("SENTINEL_MASTER_KEY") {
            Sha256::digest(raw.as_bytes()).to_vec()
        } else {
            let path = data_dir.join("master.key");
            if path.exists() {
                fs::read(&path).context("read encryption key")?
            } else {
                fs::create_dir_all(data_dir)?;
                let mut key = vec![0u8; 32];
                OsRng.fill_bytes(&mut key);
                fs::write(&path, &key).context("persist encryption key")?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
                }
                key
            }
        };
        if key.len() != 32 {
            anyhow::bail!("encryption key must be 32 bytes")
        }
        Ok(Self(Aes256Gcm::new_from_slice(&key).expect("valid key")))
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let encrypted = self
            .0
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
            .map_err(|_| anyhow::anyhow!("secret encryption failed"))?;
        let mut payload = nonce.to_vec();
        payload.extend(encrypted);
        Ok(STANDARD.encode(payload))
    }

    pub fn decrypt(&self, encoded: &str) -> Result<String> {
        let payload = STANDARD
            .decode(encoded)
            .context("decode encrypted secret")?;
        if payload.len() < 13 {
            anyhow::bail!("invalid encrypted secret")
        }
        let plain = self
            .0
            .decrypt(Nonce::from_slice(&payload[..12]), &payload[12..])
            .map_err(|_| anyhow::anyhow!("secret decryption failed"))?;
        String::from_utf8(plain).context("secret is not UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encrypted_secrets_round_trip_without_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let box_ = SecretBox::from_data_dir(dir.path()).unwrap();
        let cipher = box_.encrypt("sk-example").unwrap();
        assert!(!cipher.contains("sk-example"));
        assert_eq!(box_.decrypt(&cipher).unwrap(), "sk-example");
    }
}
