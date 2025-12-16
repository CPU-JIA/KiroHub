// Deep Link 回调处理
// 处理 kiro://kiro.kiroAgent/authenticate-success?code=xxx&state=xxx 格式的 OAuth 回调

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};

/// State 有效期：5 分钟
const STATE_VALIDITY_SECONDS: i64 = 300;

/// HMAC 密钥（从机器ID派生）
fn get_hmac_key() -> Vec<u8> {
    let machine_id = crate::kiro::get_machine_id();
    // 直接用 machine_id 的 hash 作为 HMAC 密钥
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b"kirohub-state-hmac-key");
    hasher.finalize().to_vec()
}

/// 生成带 HMAC 签名的 state
#[allow(dead_code)]
pub fn generate_secure_state() -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let nonce = uuid::Uuid::new_v4().to_string();
    let payload = format!("{}:{}", timestamp, nonce);

    // 计算 HMAC
    let key = get_hmac_key();
    let mut mac = Hmac::<Sha256>::new_from_slice(&key).expect("HMAC key init failed");
    mac.update(payload.as_bytes());
    let hmac_result = mac.finalize();
    let hmac_bytes = hmac_result.into_bytes();

    // 格式: timestamp:nonce:hmac
    let state = format!("{}:{}", payload, BASE64.encode(hmac_bytes));
    state
}

/// 验证 state 的 HMAC 签名和时间戳
fn validate_state_signature(state: &str) -> Result<(), String> {
    // 解析格式: timestamp:nonce:hmac
    let parts: Vec<&str> = state.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Invalid state format".to_string());
    }

    let hmac_encoded = parts[0];
    let payload = parts[1];

    // 解析 payload
    let payload_parts: Vec<&str> = payload.split(':').collect();
    if payload_parts.len() != 2 {
        return Err("Invalid state payload format".to_string());
    }

    let timestamp_str = payload_parts[0];
    let timestamp = timestamp_str.parse::<i64>()
        .map_err(|_| "Invalid timestamp".to_string())?;

    // 检查时间戳未过期
    let now = chrono::Utc::now().timestamp();
    if (now - timestamp).abs() > STATE_VALIDITY_SECONDS {
        return Err(format!("State expired (age: {} seconds)", now - timestamp));
    }

    // 验证 HMAC
    let hmac_bytes = BASE64.decode(hmac_encoded)
        .map_err(|_| "Invalid HMAC encoding".to_string())?;

    let key = get_hmac_key();
    let mut mac = Hmac::<Sha256>::new_from_slice(&key)
        .map_err(|_| "HMAC key init failed".to_string())?;
    mac.update(payload.as_bytes());

    mac.verify_slice(&hmac_bytes)
        .map_err(|_| "HMAC verification failed - possible tampering".to_string())?;

    Ok(())
}

/// OAuth 回调结果
#[derive(Debug, Clone)]
pub struct OAuthCallbackResult {
    pub code: String,
    pub state: String,
}

// 类型别名，简化复杂类型
type CallbackReceiver = Receiver<Result<OAuthCallbackResult, String>>;
type CallbackSender = Sender<Result<OAuthCallbackResult, String>>;
type PendingSender = (String, CallbackSender);

/// Deep Link OAuth 回调等待器
pub struct DeepLinkCallbackWaiter {
    result_rx: Arc<Mutex<Option<CallbackReceiver>>>,
    timeout: Duration,
}

impl DeepLinkCallbackWaiter {
    /// 获取 redirect_uri (使用 Kiro 官方协议)
    pub fn get_redirect_uri() -> String {
        "kiro://kiro.kiroAgent/authenticate-success".to_string()
    }

    /// 等待回调结果
    pub fn wait_for_callback(&self) -> Result<OAuthCallbackResult, String> {
        let rx = self.result_rx.lock().unwrap().take()
            .ok_or("Callback channel already consumed")?;

        match rx.recv_timeout(self.timeout) {
            Ok(result) => result,
            Err(_) => Err("OAuth callback timeout (5 minutes)".to_string()),
        }
    }
}

/// 全局回调发送器存储
static PENDING_SENDER: std::sync::OnceLock<Mutex<Option<PendingSender>>> = std::sync::OnceLock::new();

/// 注册一个新的回调等待器，返回接收端
pub fn register_waiter(state: &str) -> DeepLinkCallbackWaiter {
    let (tx, rx) = mpsc::channel();

    // 存储发送端
    let storage = PENDING_SENDER.get_or_init(|| Mutex::new(None));
    *storage.lock().unwrap() = Some((state.to_string(), tx));

    DeepLinkCallbackWaiter {
        result_rx: Arc::new(Mutex::new(Some(rx))),
        timeout: Duration::from_secs(300),
    }
}

/// 注册带安全验证的回调等待器（推荐使用）
#[allow(dead_code)]
pub fn register_secure_waiter() -> (String, DeepLinkCallbackWaiter) {
    let state = generate_secure_state();
    let waiter = register_waiter(&state);
    (state, waiter)
}

/// 处理 deep link URL（由 main.rs 调用）
pub fn handle_deep_link(url: &str) -> bool {
    println!("[DeepLink] Processing URL: {}", url);
    
    let storage = match PENDING_SENDER.get() {
        Some(s) => s,
        None => {
            println!("[DeepLink] No pending sender");
            return false;
        }
    };
    
    let mut guard = storage.lock().unwrap();
    let (expected_state, tx) = match guard.take() {
        Some(s) => s,
        None => {
            println!("[DeepLink] No pending waiter");
            return false;
        }
    };
    
    // 解析 URL
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            println!("[DeepLink] URL parse error: {}", e);
            let _ = tx.send(Err(format!("Invalid URL: {}", e)));
            return false;
        }
    };

    // 检查是否是 kiro:// 协议
    if parsed.scheme() != "kiro" {
        println!("[DeepLink] Not kiro:// scheme");
        *guard = Some((expected_state, tx)); // 放回去
        return false;
    }

    // 提取参数
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();
    
    // 检查错误
    if let Some(error) = params.get("error") {
        let desc = params.get("error_description")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown error".to_string());
        println!("[DeepLink] OAuth error: {} - {}", error, desc);
        let _ = tx.send(Err(format!("OAuth error: {} - {}", error, desc)));
        return true;
    }
    
    let code = match params.get("code") {
        Some(c) => c.to_string(),
        None => {
            println!("[DeepLink] Missing code parameter");
            let _ = tx.send(Err("Missing code parameter".to_string()));
            return true;
        }
    };

    let state = match params.get("state") {
        Some(s) => s.to_string(),
        None => {
            println!("[DeepLink] Missing state parameter");
            let _ = tx.send(Err("Missing state parameter".to_string()));
            return true;
        }
    };

    // 验证 state
    if state != expected_state {
        println!("[DeepLink] State mismatch: expected {}, got {}", expected_state, state);
        let _ = tx.send(Err("State mismatch - possible CSRF attack".to_string()));
        return true;
    }

    // 验证 state 签名（如果是新格式）
    if let Err(e) = validate_state_signature(&state) {
        println!("[DeepLink] State validation failed: {}", e);
        let _ = tx.send(Err(format!("State validation failed: {}", e)));
        return true;
    }

    #[cfg(debug_assertions)]
    println!("[DeepLink] Callback success, code length: {}", code.len());

    let _ = tx.send(Ok(OAuthCallbackResult { code, state }));
    true
}
