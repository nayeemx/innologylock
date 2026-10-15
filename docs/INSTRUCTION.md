# InnologyLock - Windows 10/11 App, Folder & File Locker
## AI Developer Execution Blueprint & Technical Instruction Guide

**Document Version:** 2.2.0
**Product Name:** InnologyLock
**Developer / Publisher:** InnologyBD
**Target Execution Environment:** Rust 1.97+, Tauri v2, Node.js v26+, Windows SDK (Win32 API), MSVC 2022

---

## 1. START HERE â€” Reading Order for New Sessions
When you open this project after a break, read these in this exact order:

1. **AGENTS.md** (root) â€” Project identity, build commands, DO NOT rules, feature status matrix
2. **CHANGELOG.md** (root) â€” What changed in the current version
3. **docs/DEV_LOG.md** (docs/) â€” Issues encountered and how they were solved, feature tracker
4. **docs/ARCHITECTURE.md** (docs/) â€” System design, IPC commands, migration policy
5. **docs/INSTRUCTION.md** (this file) â€” How to add new features
6. **docs/SECURITY.md** (docs/) â€” Encryption details

---

## 2. Directory Structure (actual as of 2026-07-26)

```
InnologyLock/
â”œâ”€â”€ AGENTS.md                     # AI agent master instructions (START HERE)
â”œâ”€â”€ CHANGELOG.md                  # Version history
â”œâ”€â”€ docs/
â”‚   â”œâ”€â”€ ARCHITECTURE.md           # System architecture (v2.0.0)
â”‚   â”œâ”€â”€ DEV_LOG.md                # Issues + solutions
â”‚   â”œâ”€â”€ INSTRUCTION.md            # This file
â”‚   â”œâ”€â”€ SECURITY.md               # Encryption & threat model
â”‚   â”œâ”€â”€ PRD.md                    # Product requirements
â”‚   â””â”€â”€ BUILD_SETUP.md            # Environment setup script
â”œâ”€â”€ src/
â”‚   â”œâ”€â”€ App.tsx                   # Root routing (~85 lines)
â”‚   â”œâ”€â”€ main.tsx
â”‚   â”œâ”€â”€ index.css                 # oklch design tokens, glass utilities
â”‚   â”œâ”€â”€ index.html
â”‚   â”œâ”€â”€ components/
â”‚   â”‚   â”œâ”€â”€ types.ts              # TabId, SetupStep, shared constants
â”‚   â”‚   â”œâ”€â”€ auth/
â”‚   â”‚   â”‚   â”œâ”€â”€ LoginScreen.tsx   # Login + password reset/recovery
â”‚   â”‚   â”‚   â””â”€â”€ SetupWizard.tsx   # 3-step first-launch wizard
â”‚   â”‚   â”œâ”€â”€ layout/
â”‚   â”‚   â”‚   â”œâ”€â”€ Sidebar.tsx
â”‚   â”‚   â”‚   â”œâ”€â”€ TopBar.tsx
â”‚   â”‚   â”‚   â””â”€â”€ Footer.tsx
â”‚   â”‚   â”œâ”€â”€ pages/
â”‚   â”‚   â”‚   â”œâ”€â”€ AppLockerPage.tsx
â”‚   â”‚   â”‚   â”œâ”€â”€ PresetsPage.tsx
â”‚   â”‚   â”‚   â”œâ”€â”€ VaultPage.tsx
â”‚   â”‚   â”‚   â”œâ”€â”€ SecurityPage.tsx
â”‚   â”‚   â”‚   â”œâ”€â”€ DiagnosticsPage.tsx   # ACL Recovery scanner UI
â”‚   â”‚   â”‚   â”œâ”€â”€ HistoryPage.tsx
â”‚   â”‚   â”‚   â””â”€â”€ SystemMonitorPage.tsx
â”‚   â”‚   â””â”€â”€ shared/
â”‚   â”‚       â”œâ”€â”€ Field.tsx
â”‚   â”‚       â”œâ”€â”€ Toggle.tsx
â”‚   â”‚       â”œâ”€â”€ SectionHeader.tsx
â”‚   â”‚       â”œâ”€â”€ Stat.tsx
â”‚   â”‚       â””â”€â”€ StatusPill.tsx
â”‚   â””â”€â”€ lib/
â”‚       â””â”€â”€ tauri-bridge.ts       # All Tauri IPC wrappers
â””â”€â”€ src-tauri/
    â”œâ”€â”€ Cargo.toml
    â”œâ”€â”€ tauri.conf.json           # Window 1280Ã—800, updater config, CSP allows data: URIs
    â”œâ”€â”€ build.rs
    â”œâ”€â”€ app.manifest              # UAC admin manifest
    â”œâ”€â”€ update.key                # Private signing key (KEEP SECRET!)
    â”œâ”€â”€ update.key.pub            # Public signing key (in tauri.conf.json)
    â””â”€â”€ src/
        â”œâ”€â”€ main.rs               # Tauri runner
        â”œâ”€â”€ lib.rs                # All IPC handlers + app runner
        â”œâ”€â”€ vault.rs              # Argon2id+AES-GCM, recovery, migration
        â”œâ”€â”€ auth.rs               # Setup, unlock, answer verification
        â”œâ”€â”€ totp.rs               # RFC 6238 TOTP with base32 secrets + QR generation
        â”œâ”€â”€ process_guard.rs      # Process enumeration + real suspension
        â”œâ”€â”€ system_presets.rs     # All 6 presets + installer guard
        â”œâ”€â”€ installer_guard.rs    # MSI/setup process killer
        â”œâ”€â”€ panic_hotkey.rs       # Win+Alt+L hotkey via raw FFI
        â”œâ”€â”€ file_locker.rs        # AES-256-GCM encryption locking + ACL damage scanner/recovery
        â”œâ”€â”€ vault_storage.rs      # Vault storage (OVLF blobs + encrypted OVMF manifest)
        â”œâ”€â”€ drive_locker.rs       # NoDrives registry + drive-root encryption (design concern)
        â”œâ”€â”€ biometric.rs          # Windows Hello (UserConsentVerifier) verification + DPAPI token
        â”œâ”€â”€ shell_context.rs      # Context menu + .InnologyLock file association
        â”œâ”€â”€ process_guard.rs      # App-kill monitor + Explorer folder monitor (widget prompt/relock)
        â”œâ”€â”€ watchdog.rs           # Self-monitoring (no guard binary)
        â”œâ”€â”€ models.rs             # All DTOs (VaultConfig, VaultRecoveryData, etc.)
        â””â”€â”€ main.rs               # Backup entry point (see lib.rs)
```

---

## 3. Build & Execution

### TypeScript Check (fast, catches frontend errors)
```powershell
cd D:\projects\code\innologylock
npx tsc --noEmit
```
Must produce zero errors. If there are errors, fix them before proceeding.

### Rust Check (medium, catches backend errors)
```powershell
cd D:\projects\code\innologylock\src-tauri
cargo check
```
Must produce zero errors. If there are errors, read them carefully â€” Rust gives exact file:line for every issue.

### Full Production Build (slow, outputs all bundles)
```powershell
cd D:\projects\code\innologylock
npx tauri build
```
Output products in `src-tauri/target/release/bundle/`:
- `InnologyLock_1.0.0_x64_en-US.msi` â€” Windows installer
- `InnologyLock_1.0.0_x64-setup.exe` â€” NSIS installer
- `InnologyLock_1.0.0_x64_portable.zip` â€” Portable zip

---

## 4. Adding a New Feature (step-by-step)

### Step 1 â€” Rust Backend Module
Create `src-tauri/src/<your_module>.rs`. Implement the functionality with proper error handling (`Result<T, String>` everywhere).

In `src-tauri/src/lib.rs`:
- Add `pub mod <your_module>;` to the module declarations
- Add a `#[tauri::command]` function that calls your module's logic
- Register the command in the `invoke_handler!` macro

If you need a new crate, add it to `src-tauri/Cargo.toml` and run `cargo check` to verify it compiles.

### Step 2 â€” Tauri Bridge
Add typed async function(s) to `src/lib/tauri-bridge.ts` using `invoke()` from `@tauri-apps/api/core`.

Do NOT export unused functions. Every export must have a caller in the frontend.

### Step 3 â€” Frontend Component
Create a React component under the appropriate subdirectory in `src/components/`.

- Import the bridge function(s) you need
- Handle every error with inline UI feedback (an error message div, not just `console.error`)
- After successful operations, show a success message
- If the feature modifies vault state, call `refresh()` after success to reload config

### Step 4 â€” Wire Up in App.tsx
Add the import and render the component in the appropriate tab section of the router in `src/App.tsx`.

### Step 5 â€” Update Documentation (STANDING RULE â€” always do this)
- Update `docs/ARCHITECTURE.md` IPC command table with any new commands
- Add entry to `CHANGELOG.md`
- Add issue/solution entry to `docs/DEV_LOG.md` if any unexpected issues occurred
- Update `AGENTS.md` (Current State + "Next Session: Continue from Here") so the next session can resume instantly
- Update `README.md` feature table if user-facing features changed

### Step 6 â€” Verify Everything Works
```powershell
npx tsc --noEmit && cd src-tauri && cargo check && cd .. && npx tauri build
```
All three commands must succeed with zero errors. Only then is the feature considered done.

---

## 5. Health Check Procedure
After any session or when things feel wrong, run this:

```powershell
cd D:\projects\code\innologylock
echo "=== TypeScript Check ==="
npx tsc --noEmit
echo "=== Rust Check ==="
cd src-tauri && cargo check
echo "=== Full Build ==="
cd .. && npx tauri build
echo "=== All checks complete ==="
```

If Rust check fails, read the error â€” it will tell you the exact file and line. If TypeScript fails, the error message is equally precise.

---

## 6. Key Architectural Decisions
1. **Vault location:** `%APPDATA%\InnologyBD\InnologyLock\` â€” survives MSI/NSIS reinstalls. Instal ler only overwrites Program Files.
2. **Encryption key:** Argon2id(password, salt) â€” per-vault salt in EncryptedVault header. NOT SHA-256(password).
3. **Recovery:** Separate `vault.recovery` file, answer-hash encrypted. NOT in vault.enc.
4. **Versioning:** `EncryptedVault.version: u32` + `vault.meta` tracks vault version. Migration on unlock.
5. **Single process:** No external guard binary. Watchdog thread monitors from within.
6. **All commands return Result<_, String>** â€” errors always contain human-readable messages.
7. **Error feedback:** Every backend call shows success or error in the UI. No silent catch blocks.
8. **Controlled components:** Toggles and inputs with backend state are driven by props, not local useState.
9. **No tokio for app threads:** All background work uses `std::thread::spawn`. tokio is used only for async commands (biometric timeout, GitHub sync, weather).
10. **No hardcoded values:** Daemon status, stats, versions come from backend/state.
11. **File locking = encryption (v0.0.35):** `lock_file(path, key)` encrypts in place. Every session-establishing path (`cmd_unlock_session`, `cmd_biometric_login`, `cmd_widget_unlock`) MUST set `AppState.file_key` + `ACTIVE_FILE_KEY` or lock/unlock commands fail with "File encryption key not available".
12. **Do NOT call `service_client::notify_lock_file/folder/drive` for new locks** â€” the service's ACL daemon is legacy and re-inflicts owner=SYSTEM damage (persisted and re-applied at boot). Use `notify_force_remove_locked_item` after ACL recovery instead.
13. **Biometric login = Windows Hello (v0.0.36):** the direct-WBF path is dead on the owner's hardware (Synaptics driver ignores background-app captures â€” proven via the WBF operational log). `authenticate_biometric` MUST use `UserConsentVerifier.RequestVerificationAsync` (PowerShell 5.1 + `System.WindowsRuntimeSystemExtensions.AsTask`) with the **correct WinRT type name `UserConsentVerificationResult`** â€” the old `UserConsentVerifierResult` does not exist and fails with "Unable to find type". Do not reintroduce WinBio code for this machine.
14. **Vault storage reuses the file key (v0.0.36):** `vault_storage.rs` calls `file_locker::do_encrypt/do_decrypt` + `check_protected_path` (they are `pub`). Keep the OVLF/OVMF formats and the "delete original only after blob written" ordering intact â€” data loss otherwise.
15. **Stale-unlock flow (v0.0.36):** never auto-re-encrypt entries silently. After login show the Re-lock all / Keep unlocked choice (`cmd_verify_locked_state` â†’ `cmd_relock_entries` / `cmd_forget_unlocked_entries`).
16. **Biometric availability (v0.0.37):** `check_biometric_available` MUST use `UserConsentVerifier.CheckAvailabilityAsync()` (WinRT, PowerShell 5.1 + AsTask). Registry checks (`WindowsHello\Enabled`, `Bio\Credential Provider`) are unreliable â€” both keys are absent on Windows 11 machines where Hello works fine (proven on the HP 850 G5), which made v0.0.36 wrongly report "Windows Hello is not available".
17. **Locked blobs are hidden (v0.0.38):** `file_locker.rs` `hide_blob` sets `FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM` on `.InnologyLock` blobs after writing. `show_blob` restores `FILE_ATTRIBUTE_NORMAL` before reading. Never skip the hide/show calls â€” locked items must be invisible in Explorer.
18. **App auto-relaunch (v0.0.38):** after widget unlock of an app target, `AUTO_RELAUNCH_APP` stores the exe path. On widget focus-lost, the app is spawned before re-locking. The widget success message says "Launching application..." for app targets.
19. **Context menu uses file-based IPC (v0.0.39):** `shell_context::dispatch_context_action` writes `action\npath` to `context_action.txt` in the vault dir. A background thread in the setup hook polls this file every 2 seconds and emits a `context-action` Tauri event to the frontend. The old approach (spawning `--lock-action` which re-spawned the same handler) caused an infinite loop. Do NOT re-introduce process-spawning context menu handlers. On Windows 10, entries show directly in the right-click menu. On Windows 11, they appear under "Show more options" (standard behavior for all apps â€” VS Code, Notepad++, 7-Zip do the same).
20. **Lock on screen lock (v0.0.39):** `auto_lock.rs` spawns a hidden window that listens for `WM_WTSSESSION_CHANGE` via `WTSRegisterSessionNotification`. On `SESSION_LOCK` (0x7), calls `relock_all_unlocked_folders()`. Controlled by `VaultConfig.lock_on_screen_lock` + `cmd_set_lock_on_screen_lock`.
21. **Hidden Items page (v0.0.39):** scans `USERPROFILE\Desktop`, `Documents`, `Downloads`, `Pictures`, `Music`, `Videos` for `.InnologyLock` files and vault `storage\` for `.vaultfile` blobs. Shows them in a dedicated "Hidden Items" tab.

---

## 7. DO NOT â€” Rules That Break Things
1. Do not put all code in App.tsx.
2. Do not modify `tauri-bridge.ts` exports without checking `App.tsx` imports.
3. Do not use `SHA-256(password)` for vault encryption â€” use `vault::hash_password()`.
4. Do not add commands to `lib.rs` without also updating `tauri-bridge.ts` AND the calling page component.
5. Do not hardcode daemon status, stats, version strings, or feature flags.
6. Do not use `verifyTotp` or `SessionToken` from `tauri-bridge.ts` â€” dead exports.
7. Do not touch `EncryptedVault` struct without updating `migrate_vault_if_needed()`.
8. Do not reintroduce ACL-based locking for files/folders (destroyed user access â€” see DEV_LOG).
9. Do not remove `windows-sys` features needed by existing commands, and do not reintroduce `Win32_Devices_BiometricFramework` (all WBF code removed in v0.0.36).
10. Do not store recovery data inside vault file â€” it must be in `vault.recovery`.
11. Do not use empty `catch {}` blocks â€” every call must show feedback to the user.
12. Do not name local state variables the same as imported bridge functions.
13. Do not end a session without updating the `.md` files (AGENTS.md / CHANGELOG.md / DEV_LOG.md).
14. Do not start a session without reading `PROMPTS.md` to know the last user prompt and resume from there. Record every new user prompt in `PROMPTS.md` with a timestamp â€” this ensures session continuity.

---

## 8. Troubleshooting
| Symptom | Likely Cause | Fix |
|:---|:---|:---|
| Rust build fails with missing function | windows-sys version doesn't export the API | Use raw FFI with GetProcAddress from user32.dll |
| Portable exe runs slowly | Windows Defender scanning unsigned binary | Add exclusion or code-sign |
| Vault not found | Vault is in AppData, not Program Files | Check `%APPDATA%\InnologyBD\InnologyLock\vault.enc` exists |
| QR code not showing | TOTP secret not generated before QR call | Ensure generateTotpSecret() then generateTotpQr() called in order |
| Toggle not persisting state | Local state shadowing controlled prop | Ensure Toggle receives `on` prop from parent |
| Password reset fails | vault.recovery file missing or answer wrong | Recovery file only created in Setup Wizard |
| Process suspend fails | App not running as Administrator | Requires admin to open process handles for SuspendThread |
| Build takes very long | Normal for first Rust compile subsequent builds are cached | Wait for `Compiling InnologyLock v1.0.0` progress |

---

*End of INSTRUCTION.md (InnologyLock v2.2.0)*

Application nane: InnologyLock
Homepage URL: https://github.com/nayeemx/omilock
Authorization callback URL: http://localhost
Check Enable Device Flmq
Click Register application