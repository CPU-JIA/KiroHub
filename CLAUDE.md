# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KiroHub is a desktop application built with **Tauri 2** (Rust backend + React frontend) for managing Kiro IDE authentication accounts with multi-provider support.

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
cargo clippy               # Lint Rust code
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
│  ├── hooks/              │  ├── account.rs    (Data model)  │
│  └── locales/            │  └── state.rs      (App state)   │
└──────────────────────────┴──────────────────────────────────┘
```

### Frontend (src/)
- **React 18** with Vite 5, no TypeScript
- **Tailwind CSS** for styling
- **i18next** for i18n (zh-CN, en-US, ru-RU)
- **State**: React Context API only (ThemeContext, DialogContext)
- **IPC**: `invoke()` from `@tauri-apps/api` to call Rust commands

### Backend (src-tauri/)
- **Tauri 2** with Rust
- **Authentication Providers** (`providers/`):
  - `SocialProvider` - Google/GitHub via AWS SSO
  - `IdcProvider` - BuilderId/SSO with client credentials
  - `WebOAuthProvider` - Web-based OAuth in WebView
- **Commands** (`commands/`): All frontend-callable functions
- **Deep Link**: `kiro://` protocol for OAuth callbacks

### Key Data Flow
```
Frontend invoke('command_name', args)
    ↓
Tauri IPC
    ↓
commands/*.rs (handler functions)
    ↓
providers/*.rs (auth logic) / account.rs (data ops)
    ↓
Returns Result<T, String> serialized to JSON
```

## Key Files

| Purpose | Frontend | Backend |
|---------|----------|---------|
| Entry point | `src/main.jsx` | `src-tauri/src/main.rs` |
| App state | `src/contexts/` | `src-tauri/src/state.rs` |
| Account CRUD | `src/components/AccountManager/hooks/useAccounts.js` | `src-tauri/src/commands/account_cmd.rs` |
| Auth flow | `src/components/Login.jsx` | `src-tauri/src/commands/auth_cmd.rs` |
| MCP config | `src/components/MCPManager/` | `src-tauri/src/commands/mcp_cmd.rs` |
| Settings | `src/components/Settings.jsx` | `src-tauri/src/commands/app_settings_cmd.rs` |

## Adding New Features

### New Tauri Command
1. Add handler in `src-tauri/src/commands/` (e.g., `my_cmd.rs`)
2. Export in `src-tauri/src/commands/mod.rs`
3. Register in `src-tauri/src/main.rs` `.invoke_handler()`
4. Call from frontend: `invoke('my_command', { args })`

### New React Component
1. Create in `src/components/`
2. Use existing contexts: `useTheme()`, `useDialog()`, `useTranslation()`
3. Add i18n keys to `src/locales/*.json`

### New Auth Provider
1. Implement `AuthProvider` trait in `src-tauri/src/providers/`
2. Register in `src-tauri/src/providers/factory.rs`

## Configuration Files

- `src-tauri/tauri.conf.json` - Tauri app config (window, plugins, bundle)
- `vite.config.js` - Frontend build config
- `tailwind.config.js` - Tailwind CSS config
- `src/i18n.jsx` - i18next configuration

## Data Storage

- Accounts: `~/.kirohub/accounts.json`
- App settings: Managed by `app_settings_cmd.rs`
- i18n preference: localStorage

## Tauri Plugins Used

- `tauri-plugin-shell` - Execute system commands
- `tauri-plugin-deep-link` - Handle `kiro://` protocol
- `tauri-plugin-updater` - Auto-update from GitHub releases
- `tauri-plugin-dialog` - Native dialogs
- `tauri-plugin-fs` - File system operations
- `tauri-plugin-opener` - Open external URLs

## Platform Notes

- **Windows**: NSIS installer, requires WebView2 (built-in on Win11)
- **macOS**: DMG bundle, minimum 10.15
- Build targets configured in `tauri.conf.json` bundle section
