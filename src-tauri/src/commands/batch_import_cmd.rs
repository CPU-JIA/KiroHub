// 批量导入命令 - 高并发版本
// 使用 futures::stream 实现并发控制

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;
use crate::account::Account;
use crate::auth::{refresh_token_desktop, get_usage_limits_desktop};
use crate::codewhisperer_client::CodeWhispererClient;
use crate::providers::{AuthProvider, IdcProvider, RefreshMetadata};
use crate::kiro::get_machine_id;

/// 单条导入项
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportItem {
    pub refresh_token: String,
    pub provider: Option<String>,
    // IdC 账号需要的字段
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub region: Option<String>,
}

/// 单条导入结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportItemResult {
    pub index: usize,
    pub success: bool,
    pub email: Option<String>,
    pub error: Option<String>,
}

/// 批量导入结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportResult {
    pub total: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub results: Vec<ImportItemResult>,
}

/// 处理单个 Social 账号导入（无状态，不访问 AppState）
async fn process_social_account(
    refresh_token: &str,
    provider: Option<String>,
) -> Result<(Account, serde_json::Value), String> {
    let refresh_result = refresh_token_desktop(refresh_token).await?;
    let access_token = refresh_result.access_token;
    let new_refresh_token = refresh_result.refresh_token;

    let usage_call = get_usage_limits_desktop(&access_token).await;
    let (usage_result, ban_reason) = match &usage_call {
        Ok(usage) => (Some(usage.clone()), None),
        Err(e) if e.starts_with("BANNED:") => (None, Some(e.strip_prefix("BANNED:").unwrap_or("UNKNOWN").to_string())),
        Err(_) => (None, None),
    };
    let usage_data = serde_json::to_value(&usage_result).unwrap_or(serde_json::Value::Null);
    let is_banned = ban_reason.is_some();

    let email = usage_result.as_ref()
        .and_then(|u| u.user_info.as_ref())
        .and_then(|u| u.email.clone())
        .unwrap_or_else(|| "unknown@kiro.dev".to_string());
    let user_id = usage_result.as_ref()
        .and_then(|u| u.user_info.as_ref())
        .and_then(|u| u.user_id.clone());

    let idp = provider.unwrap_or_else(|| {
        if email.contains("gmail") { "Google".to_string() }
        else if email.contains("github") { "GitHub".to_string() }
        else { "Google".to_string() }
    });

    let mut account = Account::new(email.clone(), format!("Kiro {} 账号", idp));
    account.access_token = Some(access_token);
    account.refresh_token = Some(new_refresh_token);
    account.provider = Some(idp);
    account.user_id = user_id;
    account.usage_data = Some(usage_data.clone());
    account.status = if is_banned { "已封禁".to_string() } else { "正常".to_string() };

    Ok((account, usage_data))
}

/// 处理单个 IdC 账号导入（无状态，不访问 AppState）
async fn process_idc_account(
    refresh_token: &str,
    client_id: String,
    client_secret: String,
    region: Option<String>,
) -> Result<(Account, serde_json::Value), String> {
    let region = region.unwrap_or_else(|| "us-east-1".to_string());
    let metadata = RefreshMetadata {
        client_id: Some(client_id.clone()),
        client_secret: Some(client_secret.clone()),
        region: Some(region.clone()),
        ..Default::default()
    };

    let idc_provider = IdcProvider::new("BuilderId", &region, None);
    let auth_result = idc_provider.refresh_token(refresh_token, metadata).await?;

    let machine_id = get_machine_id();
    let cw_client = CodeWhispererClient::new(&machine_id);
    let usage_call = cw_client.get_usage_limits(&auth_result.access_token).await;
    let (usage, is_banned) = match &usage_call {
        Ok(u) => (Some(u.clone()), false),
        Err(e) if e.starts_with("BANNED:") => (None, true),
        Err(_) => (None, false),
    };
    let usage_data = serde_json::to_value(&usage).unwrap_or(serde_json::Value::Null);

    let email = usage.as_ref()
        .and_then(|u| u.user_info.as_ref())
        .and_then(|u| u.email.clone())
        .unwrap_or_else(|| "builderid@kiro.dev".to_string());
    let user_id = usage.as_ref()
        .and_then(|u| u.user_info.as_ref())
        .and_then(|u| u.user_id.clone());

    use sha2::{Digest, Sha256};
    let start_url = "https://view.awsapps.com/start";
    let mut hasher = Sha256::new();
    hasher.update(start_url.as_bytes());
    let client_id_hash = hex::encode(hasher.finalize());

    let expires_at = chrono::Local::now() + chrono::Duration::seconds(auth_result.expires_in);

    let mut account = Account::new(email.clone(), "Kiro BuilderId 账号".to_string());
    account.access_token = Some(auth_result.access_token);
    account.refresh_token = Some(auth_result.refresh_token);
    account.provider = Some("BuilderId".to_string());
    account.user_id = user_id;
    account.expires_at = Some(expires_at.format("%Y/%m/%d %H:%M:%S").to_string());
    account.client_id = Some(client_id);
    account.client_secret = Some(client_secret);
    account.region = Some(region);
    account.client_id_hash = Some(client_id_hash);
    account.id_token = auth_result.id_token;
    account.sso_session_id = auth_result.sso_session_id;
    account.usage_data = Some(usage_data.clone());
    account.status = if is_banned { "已封禁".to_string() } else { "正常".to_string() };

    Ok((account, usage_data))
}

/// 批量导入账号（高并发版本）
///
/// # 参数
/// - items: 导入项列表
/// - concurrency: 并发数（建议 5-10，默认 5）
#[tauri::command]
pub async fn batch_import_accounts(
    state: State<'_, AppState>,
    items: Vec<ImportItem>,
    concurrency: Option<usize>,
) -> Result<BatchImportResult, String> {
    let concurrency = concurrency.unwrap_or(5).min(20).max(1); // 限制 1-20
    let total = items.len();

    println!("[BatchImport] Starting batch import: {} items, concurrency: {}", total, concurrency);

    // 为每个 item 附加索引
    let indexed_items: Vec<(usize, ImportItem)> = items.into_iter().enumerate().collect();

    // 并发处理所有导入项
    let results: Vec<(ImportItemResult, Option<Account>)> = stream::iter(indexed_items)
        .map(|(index, item)| async move {
            let is_idc = item.client_id.is_some() && item.client_secret.is_some();

            let result = if is_idc {
                process_idc_account(
                    &item.refresh_token,
                    item.client_id.clone().unwrap(),
                    item.client_secret.clone().unwrap(),
                    item.region.clone(),
                ).await
            } else {
                process_social_account(&item.refresh_token, item.provider.clone()).await
            };

            match result {
                Ok((account, _)) => {
                    let item_result = ImportItemResult {
                        index,
                        success: true,
                        email: Some(account.email.clone()),
                        error: None,
                    };
                    (item_result, Some(account))
                }
                Err(e) => {
                    let item_result = ImportItemResult {
                        index,
                        success: false,
                        email: None,
                        error: Some(e.chars().take(100).collect()),
                    };
                    (item_result, None)
                }
            }
        })
        .buffer_unordered(concurrency) // 并发执行
        .collect()
        .await;

    // 分离结果和账号
    let mut item_results = Vec::with_capacity(total);
    let mut accounts_to_save = Vec::new();

    for (result, account_opt) in results {
        item_results.push(result);
        if let Some(account) = account_opt {
            accounts_to_save.push(account);
        }
    }

    // 按 index 排序结果
    item_results.sort_by_key(|r| r.index);

    let success_count = item_results.iter().filter(|r| r.success).count();
    let failed_count = total - success_count;

    // 批量保存到存储（单次锁定）
    if !accounts_to_save.is_empty() {
        let mut store = state.store.lock().map_err(|e| format!("锁定存储失败: {}", e))?;

        for account in accounts_to_save {
            // 按 email + provider 去重
            if let Some(existing) = store.accounts.iter_mut().find(|a|
                a.email == account.email && a.provider == account.provider
            ) {
                // 更新现有账号
                existing.access_token = account.access_token;
                existing.refresh_token = account.refresh_token;
                existing.user_id = account.user_id;
                existing.expires_at = account.expires_at;
                existing.client_id = account.client_id;
                existing.client_secret = account.client_secret;
                existing.region = account.region;
                existing.client_id_hash = account.client_id_hash;
                existing.id_token = account.id_token;
                existing.sso_session_id = account.sso_session_id;
                existing.usage_data = account.usage_data;
                existing.status = account.status;
            } else {
                // 新增账号
                store.accounts.insert(0, account);
            }
        }

        store.save_to_file();
    }

    println!("[BatchImport] Completed: {}/{} success", success_count, total);

    Ok(BatchImportResult {
        total,
        success_count,
        failed_count,
        results: item_results,
    })
}
