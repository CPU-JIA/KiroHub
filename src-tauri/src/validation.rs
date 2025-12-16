// 输入验证模块
use regex::Regex;
use crate::constants::*;

/// 验证邮箱格式
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("邮箱不能为空".to_string());
    }

    // 基本的邮箱格式验证
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|e| format!("邮箱验证正则错误: {}", e))?;

    if !email_regex.is_match(email) {
        return Err("邮箱格式无效".to_string());
    }

    // 检查邮箱长度
    if email.len() > 254 {
        return Err("邮箱长度超过限制".to_string());
    }

    Ok(())
}

/// 验证 provider 是否在白名单中
pub fn validate_provider(provider: &str) -> Result<(), String> {
    match provider {
        PROVIDER_GOOGLE | PROVIDER_GITHUB | PROVIDER_BUILDER_ID | PROVIDER_ENTERPRISE => Ok(()),
        _ => Err(format!("不支持的 provider: {}", provider))
    }
}

/// 验证 token 格式（基本检查）
pub fn validate_token(token: &str, token_type: &str) -> Result<(), String> {
    if token.is_empty() {
        return Err(format!("{} 不能为空", token_type));
    }

    // 检查最小长度
    if token.len() < 20 {
        return Err(format!("{} 长度过短", token_type));
    }

    // 检查最大长度
    if token.len() > 10000 {
        return Err(format!("{} 长度过长", token_type));
    }

    // 对于特定类型的 token 进行格式检查
    if token_type == "RefreshToken" {
        // Kiro refresh token 通常以 "aor-" 开头
        if !token.starts_with("aor-") && !token.starts_with("arn:") {
            // 宽松检查，只警告不阻止
            #[cfg(debug_assertions)]
            eprintln!("Warning: RefreshToken 格式可能不正确");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user@kiro.dev").is_ok());
        assert!(validate_email("invalid@").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_validate_provider() {
        assert!(validate_provider(PROVIDER_GOOGLE).is_ok());
        assert!(validate_provider(PROVIDER_GITHUB).is_ok());
        assert!(validate_provider(PROVIDER_BUILDER_ID).is_ok());
        assert!(validate_provider("Unknown").is_err());
    }

    #[test]
    fn test_validate_token() {
        assert!(validate_token("aor-1234567890abcdefghij", "RefreshToken").is_ok());
        assert!(validate_token("short", "AccessToken").is_err());
        assert!(validate_token("", "RefreshToken").is_err());
    }
}
