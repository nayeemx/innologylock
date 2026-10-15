# InnologyLock

Windows 10/11 desktop security application by **InnologyBD**. Application, folder, file & drive locker with Argon2id key derivation, AES-256-GCM authenticated encryption, RFC 6238 TOTP 2FA, and a self-monitoring watchdog daemon.

## Quick Start

```powershell
# Check TypeScript
npx tsc --noEmit

# Check Rust
cd src-tauri && cargo check && cd ..

# Full build (outputs MSI, NSIS exe, portable zip)
# For signed builds (required for auto-updates):
$env:TAURI_SIGNING_PRIVATE_KEY="<key content>"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD="<your-signing-password>"
npx tauri build
```

## Architecture

- **Frontend:** React 18 / TypeScript / Tailwind CSS (dark glassmorphism, oklch tokens)
- **Backend:** Rust (Tauri v2) â€” modules: auth, vault, totp, process_guard, system_presets, installer_guard, panic_hotkey, biometric (Windows Hello fingerprint prompt + DPAPI token), file_locker (AES-256-GCM), vault_storage (encrypted file storage), drive_locker, watchdog, models
- **Vault:** `%APPDATA%\InnologyBD\InnologyLock\` (survives reinstalls)
- **Target:** Windows 10/11 x64, requires admin (UAC manifest)

## Key Features

| Feature | Status |
|:---|:---|
| Vault encrypt/decrypt (Argon2id + AES-256-GCM) | âœ… |
| Password reset via security question (2FA wiped) | âœ… |
| Application process lock (SHA-256 hash verified) | âœ… |
| System preset lockdown (all 6 presets) | âœ… |
| Installer guard (blocks MSI/setup executables) | âœ… |
| File & folder locking via AES-256-GCM encryption (`.InnologyLock`) | âœ… v0.0.35 |
| Hidden lock blobs (invisible in Explorer) | âœ… v0.0.38 |
| Vault Storage â€” encrypted private file storage (add/extract/delete) | âœ… v0.0.36 |
| Stale-unlock prompt after restart (re-lock all / keep unlocked) | âœ… v0.0.36 |
| Legacy service ACL state purged at boot | âœ… v0.0.36 |
| Biometric login parity (auto-lock + USB-removal wiring) | âœ… v0.0.36 |
| Biometric availability checks enrolled fingerprints | âœ… v0.0.36 |
| ACL Recovery (repair files damaged by the old DACL lock) | âœ… v0.0.35 |
| Drive locking (NoDrives + encryption; design concern â€” see AGENTS.md) | âš ï¸ |
| Fingerprint login via direct Windows Biometric Framework (no Windows Hello) | âœ… v0.0.35 |
| Widget fingerprint unlock (Windows Hello in widget) | âœ… v0.0.38 |
| Widget auto-relaunch app after unlock | âœ… v0.0.38 |
| Widget temp-unlock + auto re-lock on close | âœ… v0.0.35 |
| `.InnologyLock` double-click â†’ unlock widget | âœ… v0.0.35 |
| Context menu lock/unlock (right-click in Explorer) | âœ… v0.0.39 |
| Hidden Items page (view all encrypted blobs) | âœ… v0.0.39 |
| Lock on screen lock (Win+L re-locks items) | âœ… v0.0.39 |
| App data lock (lock AppData folders for an app) | âœ… v0.0.39 |
| Lock event notifications (toast on app kill) | âœ… v0.0.39 |
| Signed GitHub Releases via tag push (CI) | âœ… v0.0.35 |
| TOTP 2FA (RFC 6238, base32, QR setup) | âœ… |
| Panic hotkey (Win+Alt+L) | âœ… |
| Auto-lock timer | âœ… |
| Self-monitoring watchdog (no external guard binary) | âœ… |
| Vault versioning + transparent migration | âœ… |
| Auto-update via GitHub Releases (signed) | âœ… |
| Modular frontend (18 component files) | âœ… |

## Auto-Update

InnologyLock supports automatic updates via GitHub Releases:

1. Open InnologyLock â†’ Go to Security page
2. Click "Check for Updates"
3. If new version available â†’ click "Install Update & Restart"
4. App downloads, installs, and restarts

For developers: updates must be signed with the Tauri signing key (`src-tauri/update.key`).

## Documentation

| File | Purpose |
|:---|:---|
| `AGENTS.md` | AI agent starter â€” reading order, DO NOT rules, health check |
| `CHANGELOG.md` | Version history |
| `docs/DEV_LOG.md` | Issues encountered and solutions |
| `docs/ARCHITECTURE.md` | System design and technical specification |
| `docs/INSTRUCTION.md` | How to add new features step-by-step |
| `docs/SECURITY.md` | Encryption model and threat analysis |
| `docs/BUILD_SETUP.md` | Environment setup |

## Build Output

```
src-tauri/target/release/
â”œâ”€â”€ InnologyLock.exe                          # Portable
â”œâ”€â”€ bundle/
â”‚   â”œâ”€â”€ msi/InnologyLock_1.0.0_x64_en-US.msi
â”‚   â”œâ”€â”€ msi/InnologyLock_1.0.0_x64_en-US.msi.sig
â”‚   â”œâ”€â”€ nsis/InnologyLock_1.0.0_x64-setup.exe
â”‚   â””â”€â”€ nsis/InnologyLock_1.0.0_x64-setup.exe.sig
â””â”€â”€ dist/InnologyLock_1.0.0_x64.zip          # All-in-one package
```

## Dev Environment

- Node.js v26+ / npm 11+
- Rust 1.97+ / Cargo 1.97+
- MSVC 2022
- Windows 10/11 x64
