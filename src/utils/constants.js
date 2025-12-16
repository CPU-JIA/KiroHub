// 账号状态常量
export const ACCOUNT_STATUS = {
  NORMAL: '正常',
  BANNED: '已封禁',
  BANNED_ALT: '封禁',  // 旧版本兼容
  VALID: '有效'
}

// 检查账号是否已封禁
export const isAccountBanned = (status) => {
  return status === ACCOUNT_STATUS.BANNED || status === ACCOUNT_STATUS.BANNED_ALT
}

// 检查账号是否正常
export const isAccountNormal = (status) => {
  return status === ACCOUNT_STATUS.NORMAL || status === ACCOUNT_STATUS.VALID
}

// Provider 类型常量
export const PROVIDER_TYPES = {
  GOOGLE: 'Google',
  GITHUB: 'Github',
  BUILDER_ID: 'BuilderId',
  ENTERPRISE: 'Enterprise'
}

// 认证方法常量
export const AUTH_METHODS = {
  SOCIAL: 'social',
  IDC: 'IdC',
  WEB_OAUTH: 'web_oauth'
}

// AWS 区域常量
export const AWS_REGIONS = [
  'us-east-1',
  'us-west-2',
  'ap-northeast-1',
  'eu-west-1'
]

export const DEFAULT_AWS_REGION = 'us-east-1'

// 默认配置值
export const DEFAULTS = {
  PROFILE_ARN: 'arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK',
  REFRESH_THRESHOLD_MS: 5 * 60 * 1000,  // 提前 5 分钟刷新
  AUTO_REFRESH_INTERVAL_MINUTES: 50
}
