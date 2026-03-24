use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::RngCore, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Invalid password")]
    InvalidPassword,
}

/// Handles AES-256-GCM encryption/decryption with Argon2id key derivation
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    /// Create a new Encryptor from a raw 256-bit key
    pub fn from_key(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("valid key length");
        Self { cipher }
    }

    /// Derive a 256-bit encryption key from a master password using Argon2id
    pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
        let argon2 = Argon2::default();
        let salt_str = SaltString::encode_b64(salt)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

        let hash = argon2
            .hash_password(password.as_bytes(), &salt_str)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

        let hash_output = hash
            .hash
            .ok_or_else(|| CryptoError::KeyDerivationFailed("No hash output".to_string()))?;

        let bytes = hash_output.as_bytes();
        let mut key = [0u8; 32];
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Ok(key)
    }

    /// Generate a random salt for Argon2id
    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; 16];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Hash the password for verification (stored in vault_meta)
    pub fn hash_password(password: &str) -> Result<String, CryptoError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        Ok(hash.to_string())
    }

    /// Verify a password against a stored hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, CryptoError> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Encrypt plaintext bytes. Returns (ciphertext, nonce).
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    /// Decrypt ciphertext bytes using the provided nonce.
    pub fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
        Ok(plaintext)
    }
}

/// Vault state management
pub struct VaultState {
    pub encryptor: Option<Encryptor>,
    pub salt: Option<Vec<u8>>,
}

impl VaultState {
    pub fn new() -> Self {
        Self {
            encryptor: None,
            salt: None,
        }
    }

    /// Set up a new vault with a master password
    pub fn setup(&mut self, password: &str) -> Result<(Vec<u8>, String), CryptoError> {
        let salt = Encryptor::generate_salt();
        let key = Encryptor::derive_key(password, &salt)?;
        let password_hash = Encryptor::hash_password(password)?;

        self.encryptor = Some(Encryptor::from_key(&key));
        self.salt = Some(salt.clone());

        Ok((salt, password_hash))
    }

    /// Unlock an existing vault
    pub fn unlock(
        &mut self,
        password: &str,
        salt: &[u8],
        stored_hash: &str,
    ) -> Result<bool, CryptoError> {
        if !Encryptor::verify_password(password, stored_hash)? {
            return Ok(false);
        }

        let key = Encryptor::derive_key(password, salt)?;
        self.encryptor = Some(Encryptor::from_key(&key));
        self.salt = Some(salt.to_vec());
        Ok(true)
    }

    /// Lock the vault (clear the encryption key from memory)
    pub fn lock(&mut self) {
        self.encryptor = None;
        // Keep salt so we can unlock again
    }

    pub fn is_unlocked(&self) -> bool {
        self.encryptor.is_some()
    }

    pub fn get_encryptor(&self) -> Option<&Encryptor> {
        self.encryptor.as_ref()
    }
}
