// 加密模块 - 基于机器ID的自动加密
// 使用 ChaCha20-Poly1305 加密敏感数据

use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Nonce, Key,
};
use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// 固定的 Salt（用于从机器ID派生密钥）
const SALT: &[u8] = b"KiroHub-V1-Salt-2025";
const PBKDF2_ROUNDS: u32 = 100_000;

/// 加密后的数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64 编码的加密数据
    pub ciphertext: String,
    /// Base64 编码的 nonce
    pub nonce: String,
}

/// 从机器ID派生加密密钥
fn derive_key_from_machine_id(machine_id: &str) -> Result<Zeroizing<[u8; 32]>, String> {
    let password = machine_id.as_bytes();
    let mut key = Zeroizing::new([0u8; 32]);

    pbkdf2::<Hmac<Sha256>>(password, SALT, PBKDF2_ROUNDS, &mut *key);

    Ok(key)
}

/// 加密字符串数据
pub fn encrypt_string(plaintext: &str, machine_id: &str) -> Result<EncryptedData, String> {
    if plaintext.is_empty() {
        return Err("Cannot encrypt empty string".to_string());
    }

    // 派生密钥
    let key = derive_key_from_machine_id(machine_id)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&*key)
        .map_err(|e| format!("Cipher creation error: {}", e))?;

    // 生成随机 nonce（12 字节）
    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption error: {}", e))?;

    Ok(EncryptedData {
        ciphertext: BASE64.encode(&ciphertext),
        nonce: BASE64.encode(nonce_bytes),
    })
}

/// 解密字符串数据
pub fn decrypt_string(encrypted: &EncryptedData, machine_id: &str) -> Result<String, String> {
    // 派生密钥
    let key = derive_key_from_machine_id(machine_id)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&*key));

    // 解码 Base64
    let ciphertext = BASE64.decode(&encrypted.ciphertext)
        .map_err(|e| format!("Ciphertext decode error: {}", e))?;
    let nonce_bytes = BASE64.decode(&encrypted.nonce)
        .map_err(|e| format!("Nonce decode error: {}", e))?;

    // 确保 nonce 长度正确
    if nonce_bytes.len() != 12 {
        return Err(format!("Invalid nonce length: {}", nonce_bytes.len()));
    }

    let nonce = Nonce::from_slice(&nonce_bytes);

    // 解密
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption error: {}", e))?;

    String::from_utf8(plaintext_bytes)
        .map_err(|e| format!("UTF-8 decode error: {}", e))
}

/// 加密 Option<String>
pub fn encrypt_optional(value: &Option<String>, machine_id: &str) -> Result<Option<EncryptedData>, String> {
    match value {
        Some(v) if !v.is_empty() => Ok(Some(encrypt_string(v, machine_id)?)),
        _ => Ok(None),
    }
}

/// 解密 Option<EncryptedData>
pub fn decrypt_optional(encrypted: &Option<EncryptedData>, machine_id: &str) -> Result<Option<String>, String> {
    match encrypted {
        Some(e) => Ok(Some(decrypt_string(e, machine_id)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let machine_id = "test-machine-id-12345";
        let plaintext = "aor-secret-refresh-token-abcdefg";

        // 加密
        let encrypted = encrypt_string(plaintext, machine_id).unwrap();
        assert!(!encrypted.ciphertext.is_empty());
        assert!(!encrypted.nonce.is_empty());

        // 解密
        let decrypted = decrypt_string(&encrypted, machine_id).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_machine_id() {
        let machine_id1 = "machine-1";
        let machine_id2 = "machine-2";
        let plaintext = "secret-data";

        // 用 machine_id1 加密
        let encrypted = encrypt_string(plaintext, machine_id1).unwrap();

        // 用 machine_id2 解密应该失败
        let result = decrypt_string(&encrypted, machine_id2);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_optional() {
        let machine_id = "test-machine";

        // Some
        let value = Some("token".to_string());
        let encrypted = encrypt_optional(&value, machine_id).unwrap();
        assert!(encrypted.is_some());

        let decrypted = decrypt_optional(&encrypted, machine_id).unwrap();
        assert_eq!(decrypted, value);

        // None
        let none_value: Option<String> = None;
        let encrypted_none = encrypt_optional(&none_value, machine_id).unwrap();
        assert!(encrypted_none.is_none());
    }

    #[test]
    fn test_empty_string() {
        let machine_id = "test-machine";
        let result = encrypt_string("", machine_id);
        assert!(result.is_err());
    }
}
