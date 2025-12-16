use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;
use crate::crypto::{EncryptedData, encrypt_optional, decrypt_optional};
use crate::kiro::get_machine_id;

/// 用于序列化到文件的加密账号结构（V2 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecureAccount {
    // 非敏感字段（明文）
    pub id: String,
    pub email: String,
    pub label: String,
    pub status: String,
    pub added_at: String,
    pub expires_at: Option<String>,
    pub provider: Option<String>,
    pub user_id: Option<String>,
    pub region: Option<String>,
    pub client_id_hash: Option<String>,
    pub profile_arn: Option<String>,
    pub usage_data: Option<serde_json::Value>,
    pub client_id: Option<String>,  // client_id 不敏感，保持明文便于调试
    pub sso_session_id: Option<String>,  // sso_session_id 也不敏感

    // 敏感字段（加密存储）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_enc: Option<EncryptedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_enc: Option<EncryptedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csrf_token_enc: Option<EncryptedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token_enc: Option<EncryptedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret_enc: Option<EncryptedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token_enc: Option<EncryptedData>,

    // 版本标记
    #[serde(default)]
    pub version: u32,  // 2 = 加密版本
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub email: String,
    pub label: String,
    pub status: String,
    pub added_at: String,
    // 认证信息
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub csrf_token: Option<String>,
    pub session_token: Option<String>,
    pub expires_at: Option<String>,
    // 账号信息
    pub provider: Option<String>,
    pub user_id: Option<String>,
    // IdC 专用
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub region: Option<String>,
    pub client_id_hash: Option<String>,
    pub sso_session_id: Option<String>,
    pub id_token: Option<String>,
    // Social 专用
    pub profile_arn: Option<String>,
    // 原始 usage API 响应
    pub usage_data: Option<serde_json::Value>,
}


impl Account {
    pub fn new(email: String, label: String) -> Self {
        let now: DateTime<Local> = Local::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            label,
            status: "正常".to_string(),
            added_at: now.format("%Y/%m/%d %H:%M:%S").to_string(),
            access_token: None,
            refresh_token: None,
            csrf_token: None,
            session_token: None,
            expires_at: None,
            provider: None,
            user_id: None,
            client_id: None,
            client_secret: None,
            region: None,
            client_id_hash: None,
            sso_session_id: None,
            id_token: None,
            profile_arn: None,
            usage_data: None,
        }
    }

    /// 转换为加密格式用于存储
    fn to_secure(&self, machine_id: &str) -> Result<SecureAccount, String> {
        Ok(SecureAccount {
            id: self.id.clone(),
            email: self.email.clone(),
            label: self.label.clone(),
            status: self.status.clone(),
            added_at: self.added_at.clone(),
            expires_at: self.expires_at.clone(),
            provider: self.provider.clone(),
            user_id: self.user_id.clone(),
            region: self.region.clone(),
            client_id_hash: self.client_id_hash.clone(),
            profile_arn: self.profile_arn.clone(),
            usage_data: self.usage_data.clone(),
            client_id: self.client_id.clone(),
            sso_session_id: self.sso_session_id.clone(),

            // 加密敏感字段
            access_token_enc: encrypt_optional(&self.access_token, machine_id)?,
            refresh_token_enc: encrypt_optional(&self.refresh_token, machine_id)?,
            csrf_token_enc: encrypt_optional(&self.csrf_token, machine_id)?,
            session_token_enc: encrypt_optional(&self.session_token, machine_id)?,
            client_secret_enc: encrypt_optional(&self.client_secret, machine_id)?,
            id_token_enc: encrypt_optional(&self.id_token, machine_id)?,

            version: 2,
        })
    }

    /// 从加密格式恢复
    fn from_secure(secure: SecureAccount, machine_id: &str) -> Result<Self, String> {
        Ok(Account {
            id: secure.id,
            email: secure.email,
            label: secure.label,
            status: secure.status,
            added_at: secure.added_at,
            expires_at: secure.expires_at,
            provider: secure.provider,
            user_id: secure.user_id,
            region: secure.region,
            client_id_hash: secure.client_id_hash,
            profile_arn: secure.profile_arn,
            usage_data: secure.usage_data,
            client_id: secure.client_id,
            sso_session_id: secure.sso_session_id,

            // 解密敏感字段
            access_token: decrypt_optional(&secure.access_token_enc, machine_id)?,
            refresh_token: decrypt_optional(&secure.refresh_token_enc, machine_id)?,
            csrf_token: decrypt_optional(&secure.csrf_token_enc, machine_id)?,
            session_token: decrypt_optional(&secure.session_token_enc, machine_id)?,
            client_secret: decrypt_optional(&secure.client_secret_enc, machine_id)?,
            id_token: decrypt_optional(&secure.id_token_enc, machine_id)?,
        })
    }
}

pub struct AccountStore {
    pub accounts: Vec<Account>,
    file_path: PathBuf,
}

impl AccountStore {
    pub fn new() -> Self {
        let file_path = Self::get_storage_path();
        let accounts = Self::load_from_file(&file_path);
        Self { accounts, file_path }
    }

    fn get_storage_path() -> PathBuf {
        let data_dir = dirs::data_dir().unwrap_or_else(|| {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
        });
        data_dir.join(".kirohub").join("accounts.json")
    }

    fn load_from_file(path: &PathBuf) -> Vec<Account> {
        if let Ok(content) = std::fs::read_to_string(path) {
            let machine_id = get_machine_id();

            // 尝试解析为加密版本（V2）
            if let Ok(secure_accounts) = serde_json::from_str::<Vec<SecureAccount>>(&content) {
                // 解密所有账号
                secure_accounts
                    .into_iter()
                    .filter_map(|secure| {
                        match Account::from_secure(secure, &machine_id) {
                            Ok(account) => Some(account),
                            Err(e) => {
                                eprintln!("Failed to decrypt account: {}", e);
                                None
                            }
                        }
                    })
                    .collect()
            } else {
                // 尝试解析为旧版明文格式（V1）
                if let Ok(legacy_accounts) = serde_json::from_str::<Vec<Account>>(&content) {
                    #[cfg(debug_assertions)]
                    println!("[AccountStore] Detected legacy plaintext format, will migrate to encrypted format on next save");

                    // 返回明文账号，下次保存时会自动加密
                    legacy_accounts
                } else {
                    eprintln!("[AccountStore] Failed to parse accounts.json, using empty list");
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }

    pub fn save_to_file(&self) {
        let machine_id = get_machine_id();

        // 转换所有账号为加密格式
        let secure_accounts: Vec<SecureAccount> = self
            .accounts
            .iter()
            .filter_map(|account| {
                match account.to_secure(&machine_id) {
                    Ok(secure) => Some(secure),
                    Err(e) => {
                        eprintln!("Failed to encrypt account {}: {}", account.email, e);
                        None
                    }
                }
            })
            .collect();

        // 序列化并保存
        if let Some(parent) = self.file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match serde_json::to_string_pretty(&secure_accounts) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.file_path, json) {
                    eprintln!("Failed to write accounts file: {}", e);
                }

                // 设置文件权限（仅所有者可读写）
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = std::fs::Permissions::from_mode(0o600);
                    if let Err(e) = std::fs::set_permissions(&self.file_path, permissions) {
                        eprintln!("Failed to set file permissions: {}", e);
                    }
                }

                // Windows 平台文件保护（基本）
                #[cfg(windows)]
                {
                    // Windows 文件默认权限已足够，不需要特殊设置
                    // TODO: 未来可添加更严格的 ACL 设置
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize accounts: {}", e);
            }
        }
    }

    pub fn get_all(&self) -> Vec<Account> {
        self.accounts.clone()
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let len_before = self.accounts.len();
        self.accounts.retain(|a| a.id != id);
        let deleted = self.accounts.len() < len_before;
        if deleted {
            self.save_to_file();
        }
        deleted
    }

    pub fn delete_many(&mut self, ids: &[String]) -> usize {
        let len_before = self.accounts.len();
        self.accounts.retain(|a| !ids.contains(&a.id));
        let deleted = len_before - self.accounts.len();
        if deleted > 0 {
            self.save_to_file();
        }
        deleted
    }

    pub fn import_from_json(&mut self, json: &str) -> Result<usize, String> {
        match serde_json::from_str::<Vec<Account>>(json) {
            Ok(imported) => {
                let count = imported.len();
                for account in imported {
                    if !self.accounts.iter().any(|a| a.id == account.id) {
                        self.accounts.push(account);
                    }
                }
                self.save_to_file();
                Ok(count)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn export_to_json(&self) -> String {
        serde_json::to_string_pretty(&self.accounts).unwrap_or_default()
    }
}
