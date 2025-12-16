# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KiroHub is a desktop application built with **Tauri 2** (Rust backend + React frontend) for managing Kiro IDE authentication accounts with multi-provider support.

**Current Version**: v1.3.0
**Security Grade**: A+ (5/5)
**Code Quality**: 5/5 (Perfect - Zero Warnings)

## Build & Development Commands

```bash
# Frontend development
npm run dev                 # Start Vite dev server (port 1420)
npm run build              # Build frontend to dist/

# Tauri development
npm run tauri dev          # Run app in dev mode (frontend + backend)
npm run tauri build        # Build production installer

# i18n
npm run extract            # Extract i18n strings (Lingui)
npm run compile            # Compile i18n catalogs

# Rust backend (from src-tauri/)
cargo build --release      # Build Rust backend
cargo check                # Type check without building
cargo clippy               # Lint Rust code (must be 0 warnings)
cargo test                 # Run unit tests
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri 2 Application                      │
├──────────────────────────┬──────────────────────────────────┤
│   Frontend (React 18)    │      Backend (Rust/Tauri)        │
│                          │                                  │
│  src/                    │  src-tauri/src/                  │
│  ├── components/         │  ├── commands/     (IPC handlers)│
│  ├── contexts/           │  ├── providers/    (Auth logic)  │
│  │   ├── AppSettings     │  ├── crypto.rs     (Encryption)  │
│  │   ├── Theme           │  ├── validation.rs (Input checks)│
│  │   └── Dialog          │  ├── constants.rs  (Constants)   │
│  ├── utils/              │  ├── account.rs    (Data model)  │
│  └── locales/            │  └── state.rs      (App state)   │
└──────────────────────────┴──────────────────────────────────┘
```

### Frontend (src/)
- **React 18** with Vite 5, no TypeScript
- **Tailwind CSS** for styling
- **i18next** for i18n (zh-CN, en-US, ru-RU) - 95% coverage
- **State**: React Context API (ThemeContext, DialogContext, AppSettingsContext)
- **IPC**: `invoke()` from `@tauri-apps/api` to call Rust commands
- **Virtual List**: @tanstack/react-virtual for 50+ accounts

### Backend (src-tauri/)
- **Tauri 2.9** with Rust
- **Security Modules**:
  - `crypto.rs` - ChaCha20-Poly1305 encryption
  - `validation.rs` - Input validation with unit tests
  - `constants.rs` - Centralized constants
- **Authentication Providers** (`providers/`):
  - `SocialProvider` - Google/GitHub via AWS SSO
  - `IdcProvider` - BuilderId/SSO with client credentials
  - `WebOAuthProvider` - Web-based OAuth in WebView
- **Commands** (`commands/`): All frontend-callable functions
- **Deep Link**: `kiro://` protocol with HMAC validation

### Key Data Flow
```
Frontend invoke('command_name', args)
    ↓
Tauri IPC (Optimized with caching)
    ↓
commands/*.rs (handler functions)
    ↓
providers/*.rs (auth logic) / account.rs (encrypted storage)
    ↓
Returns Result<T, String> serialized to JSON
```

## Key Files

| Purpose | Frontend | Backend |
|---------|----------|---------|
| Entry point | `src/main.jsx` | `src-tauri/src/main.rs` |
| App state | `src/contexts/AppSettingsContext.jsx` | `src-tauri/src/state.rs` |
| Encryption | N/A | `src-tauri/src/crypto.rs` |
| Validation | N/A | `src-tauri/src/validation.rs` |
| Account CRUD | `src/components/AccountManager/hooks/useAccounts.js` | `src-tauri/src/commands/account_cmd.rs` |
| Auth flow | `src/components/Login.jsx` | `src-tauri/src/commands/auth_cmd.rs` |
| MCP config | `src/components/MCPManager/` | `src-tauri/src/commands/mcp_cmd.rs` |
| Settings | `src/components/Settings.jsx` | `src-tauri/src/commands/app_settings_cmd.rs` |

## Security Architecture

### Encryption (crypto.rs)
- **Algorithm**: ChaCha20-Poly1305 (AEAD)
- **Key Derivation**: PBKDF2-HMAC-SHA256 (100,000 rounds)
- **Key Source**: Machine ID (automatic, user-transparent)
- **Encrypted Fields**: access_token, refresh_token, csrf_token, session_token, client_secret, id_token

### Storage Format
```json
{
  "id": "...",
  "email": "user@example.com",
  "accessTokenEnc": {
    "ciphertext": "base64-encrypted-data",
    "nonce": "base64-random-nonce"
  },
  "version": 2
}
```

### State Validation (deep_link_handler.rs)
- HMAC-SHA256 signature
- Timestamp validation (5-minute expiration)
- Format: `timestamp:nonce:hmac`

## Adding New Features

### New Tauri Command
1. Add handler in `src-tauri/src/commands/` (e.g., `my_cmd.rs`)
2. Export in `src-tauri/src/commands/mod.rs`
3. Register in `src-tauri/src/main.rs` `.invoke_handler()`
4. Call from frontend: `invoke('my_command', { args })`
5. **Important**: Use proper error handling (no unwrap()), return Result<T, String>

### New React Component
1. Create in `src/components/`
2. Use existing contexts: `useTheme()`, `useDialog()`, `useTranslation()`, `useAppSettings()`
3. Add i18n keys to `locales/*.json`
4. Consider React.memo for list items

### New Auth Provider
1. Implement `AuthProvider` trait in `src-tauri/src/providers/`
2. Register in `src-tauri/src/providers/factory.rs`
3. Use constants from `constants.rs`

### Adding Sensitive Fields
1. Add encrypted field to `SecureAccount` in `account.rs`
2. Update `to_secure()` and `from_secure()` methods
3. Use `encrypt_optional()` and `decrypt_optional()` from `crypto.rs`

## Configuration Files

- `src-tauri/tauri.conf.json` - Tauri app config (window, plugins, bundle, CSP)
- `vite.config.js` - Frontend build config
- `tailwind.config.js` - Tailwind CSS config
- `src/i18n.jsx` - i18next configuration

## Data Storage

- Accounts: `~/.kirohub/accounts.json` (encrypted V2 format)
- App settings: Managed by `app_settings_cmd.rs`
- i18n preference: localStorage
- File permissions: 0600 on Unix, default on Windows

## Tauri Plugins Used

- `tauri-plugin-shell` (>=2.2.1) - Execute system commands (CVE fixed)
- `tauri-plugin-deep-link` (>=2) - Handle `kiro://` protocol
- `tauri-plugin-updater` (>=2.9) - Auto-update from GitHub releases
- `tauri-plugin-dialog` (>=2.4) - Native dialogs
- `tauri-plugin-fs` (>=2.4) - File system operations
- `tauri-plugin-opener` (>=2.5) - Open external URLs

## Code Standards

### Zero Warnings Policy
- **ALL warnings must be fixed before commit**
- Run `cargo check`, `cargo clippy`, `npm run build`
- Use `#[allow(dead_code)]` only for future-use functions

### Security Requirements
- No sensitive data in logs (use `#[cfg(debug_assertions)]`)
- All external inputs must be validated (use `validation.rs`)
- All file paths must be validated (check `steering.rs` for examples)
- Use constants from `constants.rs`, never hardcode
- Proper error handling: Result<T, String>, no unwrap()

### Performance Best Practices
- Use RwLock for read-heavy data (AuthState, AccountStore)
- Cache IPC calls when possible (see AppSettingsContext)
- Use React.memo for list components
- Enable virtual list for large datasets (50+ items)

## Platform Notes

- **Windows**: NSIS installer, requires WebView2 (built-in on Win11)
- **macOS**: DMG bundle, minimum 10.15, Universal binary
- Build targets configured in `tauri.conf.json` bundle section
- CSP is enabled - ensure all scripts are from 'self' or whitelisted

## Testing

### Unit Tests
```bash
cd src-tauri
cargo test                 # Run Rust unit tests
```

### Security Tests
See `SECURITY_TESTS.md` for comprehensive security test cases:
- Path traversal attacks
- Input validation
- Encryption/Decryption
- HMAC validation

### Performance Benchmarks
- App startup: < 2s
- List render (20): < 100ms
- List render (100): < 400ms
- Account switch: < 350ms

## Important Notes

- **Encryption**: Data encrypted with machine ID. Changing machines requires re-login.
- **Backwards Compatibility**: V1 (plaintext) accounts auto-migrate to V2 (encrypted) on save.
- **File Permissions**: Accounts file is 0600 on Unix, ensure proper ACL on Windows.
- **Virtual List**: Automatically enabled for 50+ accounts, threshold configurable.
- **IPC Caching**: AppSettings are cached globally, use `useAppSettings()` hook instead of invoke().

## Version History

- **v1.3.0**: i18n + Virtual List
- **v1.2.0**: IPC Optimization + RwLock
- **v1.1.0**: Security Hardening + Zero Warnings
- **v1.0.1**: Fork Baseline
