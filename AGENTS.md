# InnologyLock â€” Session Handoff

**Show this file to the AI at the start of every new session.**

## Current State

- **Version**: 0.0.39 (**unreleased** — minifilter driver + folder container lock + 3-attempt lockout). Working tree has uncommitted changes for this feature set.
- **Last Updated**: 2026-10-15
- **2026-10-15 scan session**: external scan verdict confirmed — signing passwords removed from all tracked docs (private key was never committed); rename to InnologyLock finished (zero old-name matches in source; `OLCK`/`OMNI` bytes + biometric entropy kept for compat). See CHANGELOG `[Unreleased]` + DEV_LOG 2026-10-15 entry.
- **Project Name**: **InnologyLock** (rename completed 2026-08-26)
- **Remaining manual steps**: (1) Rename GitHub repo to innologylock (project folder already renamed). (2) Rotate the signing password — both past values are in git history (see 2026-10-15 entries in CHANGELOG + DEV_LOG). (3) Install WDK + build driver (see "Next Session").
- **Build status**: ✅ `cargo check` (src-tauri) — 0 errors. ✅ `cargo check` (service) — 0 errors (1 snake_case warning). ✅ `npx tsc --noEmit` — clean. ⏳ Driver build requires WDK.
- **Runtime status**: App compiles, runs, service starts in standalone mode. **Lock/unlock uses AES-256-GCM encryption (file_locker) + minifilter driver blocks Explorer access to locked paths.**

---

## âš ï¸ CRITICAL: How a Future AI Should Start (Reading Order)

1. Read this file **entirely** (especially "Next Session: Continue from Here").
2. `git status` + `git diff --stat` â€” the 12 modified files ARE the v0.0.35 work. Do NOT revert them.
3. Run `cargo check` (src-tauri) + `npx tsc --noEmit` â€” both must be clean.
4. Read `CHANGELOG.md` â†’ `docs/DEV_LOG.md` â†’ `docs/ARCHITECTURE.md` â†’ `docs/SECURITY.md` for depth.
5. After ANY work (code, docs, research): **update all `.md` files** (AGENTS.md current state, CHANGELOG.md entry, DEV_LOG.md entry, ARCHITECTURE.md if commands/schema changed). This is a standing rule â€” always end a session by updating the .md files so the next session can resume instantly.
6. **STANDING RULE: Record every user prompt in `PROMPTS.md`** â€” Every message the user sends must be logged with a timestamp in `PROMPTS.md`. This ensures session continuity: if a chat session is lost, the next AI reads PROMPTS.md to pick up from the last user prompt. Append each new prompt to the current session's table.

---


## What Was Done in This Session (2026-08-27)

### 1. Safe ACL Lock Rewrite (service/src/acl.rs — COMPLETE REWRITE)
The old `apply_lock` was destructive: set owner=SYSTEM + replaced DACL (only Admins+SYSTEM). User lost access to folders on their office PC.

**New approach:**
- `apply_lock(path)`: saves current DACL + owner to a backup file (`C:\ProgramData\InnologyBD\InnologyLock\acl_backups\<hash>.bin`), then adds a DENY ACE for the current user with `SUB_CONTAINERS_AND_OBJECTS_INHERIT` inheritance. Does NOT change ownership. Does NOT replace existing DACL — merges the DENY ACE into it.
- `remove_lock(path)`: restores original DACL from backup file, deletes backup. Recursively resets child ACLs.
- `verify_lock(path)`: checks if backup file exists (instead of checking owner==SYSTEM).
- Backup format: `[4] owner_len (u32 LE) | [owner_len] owner_sid_bytes | [rest] original_dacl_bytes`

### 2. Tauri App Commands Switched to Service ACL Lock
- `cmd_add_locked_folder`: removed AES encryption (`file_locker::lock_folder`), now calls `service_client::notify_lock_folder(&path)`
- `cmd_remove_locked_folder`: removed AES decryption, now calls `service_client::notify_unlock_item(&path, &password)`
- `cmd_add_locked_file`: removed AES encryption, now calls `service_client::notify_lock_file(&path)`
- `cmd_remove_locked_file`: removed AES decryption, now calls `service_client::notify_unlock_item(&path, &password)`
- All lock commands now return errors from the service pipe (instead of silently ignoring failures)

### 3. service_client.rs — Now Returns Results
- `notify_lock_file`, `notify_lock_folder`, `notify_lock_drive`, `notify_unlock_item` all now return `Result<String, String>` instead of fire-and-forget
- If the service pipe is unreachable, the command returns an error: "Service not reachable — lock not applied"

### 4. VaultPage.tsx — Simplified UI
- Removed "Vault Storage" section (Add/Extract/Delete encrypted files)
- Updated description from "AES-256-GCM" to "ACL LOCK"
- Subtitle now says: "Blocks access via Windows ACL. Locked items show 'Access Denied' in Explorer — unlock with password to restore access."

### 5. Panic Hotkey Fix
- Changed from `Win+Alt+L` (MOD_WIN = 0x0800, restricted by Windows) to `Ctrl+Alt+L` (MOD_CONTROL = 0x0002)
- `MOD_WIN` requires elevated privileges — that's why it always failed with error 5

### 6. Service Lib Name Fixed
- `service/Cargo.toml`: lib name changed from `innologylock-svc` to `InnologyLock-svc` to match binary `use InnologyLock_svc::*` imports
- Warning: snake_case lint warns about `InnologyLock_svc` but is harmless

### 7. Service Binary Rebuilt
- `service/target/debug/InnologyLock-svc.exe` rebuilt with new safe ACL code
- Copied to `src-tauri/resources/InnologyLock-svc.exe`
- `src-tauri/resources/InnologyLock-monitor.exe` also updated

### 1. Async Fix — UI No Longer Freezes During Lock/Unlock
Converted 16 synchronous Tauri commands to async using 	okio::task::spawn_blocking. Previously, encrypting/decrypting files blocked the main thread, freezing the entire app (can't switch pages, can't interact with sidebar). Now heavy I/O runs off the UI thread.

Commands converted: cmd_add_locked_file/folder, cmd_remove_locked_file/folder, cmd_toggle_locked_app, cmd_add_locked_app, cmd_remove_locked_app, cmd_relock_entries, cmd_rescue_unlock, cmd_recover_acl, cmd_force_unlock, cmd_vault_store_file, cmd_vault_extract_file, cmd_vault_delete_file, cmd_backup_vault, cmd_restore_vault, cmd_context_menu_lock/unlock, cmd_list_hidden_blobs.

### 2. Rename to InnologyLock
Replaced every occurrence of the previous working title with "innologylock"/"InnologyLock" across 63 files: source code, configs, docs, Cargo packages, registry keys, file associations, window titles, bundle metadata. Cargo package names normalized to lowercase.
## What Was Done in This Session (2026-08-16) â€” the v0.0.36 feature set

The user asked to scan the project, read every `.md` file, and solve everything still outstanding. All items below are in the working tree (uncommitted).

### 1. Vault Storage â€” private encrypted file storage (backlog user requirement #2, HIGH priority)
- **New module `src-tauri/src/vault_storage.rs`**: `store_file` encrypts a file with the vault's `file_encryption_key` into `%APPDATA%\InnologyBD\InnologyLock\storage\<random-id>.vaultfile` and **deletes the original** only after the blob is written. `extract_file` decrypts to a user-chosen folder (refuses to overwrite existing files, verifies size), `delete_file` removes blob + manifest entry.
- **Blob layout**: `[4] "OVLF" | [1] version | [12] nonce | [4] name_len | [N] name | [8] size (u64 LE) | [*] ciphertext+tag`.
- **Manifest `vault_files.enc`** is itself AES-GCM encrypted (`OVMF` magic + nonce + ciphertext JSON) â€” stored file names never leak on disk.
- Reuses `file_locker::do_encrypt/do_decrypt` and `check_protected_path` (both made `pub`); refuses symlinks and protected paths.
- Commands: `cmd_vault_store_file`, `cmd_vault_list_files`, `cmd_vault_extract_file`, `cmd_vault_delete_file` (all session + file-key gated). Bridge: `vaultStoreFile/vaultListFiles/vaultExtractFile/vaultDeleteFile`. UI: "Vault Storage" section on VaultPage (Add / Extract / Delete, size + date display).

### 2. Temp-unlock no longer dies with the app (AGENTS priority issue #2 â€” SOLVED)
- `cmd_verify_locked_state`: after login, checks every `locked_files`/`locked_folders` entry against disk; any entry whose `.InnologyLock` blob is gone (temp-unlocked, never re-locked â€” e.g. app exited while the item was open) is returned.
- Frontend (App.tsx): after unlock, a full-screen prompt lists the stale items with **Re-lock all** (`cmd_relock_entries` â€” re-encrypts, returns per-item status; failures stay in the list) and **Keep unlocked** (`cmd_forget_unlocked_entries` â€” removes from vault, saves config + summary). No auto-re-encrypt without consent (folders the user may be actively using are never silently re-encrypted).
- Bridge: `verifyLockedState`, `relockEntries`, `forgetUnlockedEntries`.

### 3. Legacy service ACL state purged at boot (AGENTS priority issue #3 â€” SOLVED)
- `service_client::get_locked_items()` added; the app setup hook now purges every persisted item via `notify_force_remove_locked_item`. The legacy service can no longer re-apply owner=SYSTEM ACL locks at boot. Runs once per launch; empty list afterwards â†’ no-op.

### 4. `cmd_biometric_login` parity gap (AGENTS priority issue #4 â€” SOLVED)
- Fingerprint login now also calls `auto_lock::set_auto_lock_minutes(config.auto_lock_minutes)` and `drive_locker::set_usb_removal_callback(...)` (workstation lock on locked-drive removal) â€” identical wiring to `cmd_unlock_session`.
### 5. Biometric login now uses the WINDOWS HELLO prompt â€” hardware-proven (AGENTS priority issue #7 â€” SUPERSEDED)

- **Hardware finding (proven 2026-08-16 on the owner's HP EliteBook 850 G5, Synaptics VFS7552, WBF driver oem27.inf)**: the direct-WBF path CANNOT work on this machine. `WinBioIdentify` blocks forever and the sensor never delivers a single capture to a background app â€” the WBF operational event log records ZERO sensor events during 90s of real finger touches, while the user's own Windows sign-in identifications appear in the same log every time (event 1004, `Microsoft-Windows-Biometrics/Operational`). The driver only serves the Windows Hello pipeline.
- **Old check (`WinBioEnumEnrollments` + SID)**: irrelevant now â€” removed.
- **New implementation** (`src-tauri/src/biometric.rs`, rewritten): `authenticate_biometric` runs PowerShell 5.1 â†’ `Windows.Security.Credentials.UI.UserConsentVerifier.RequestVerificationAsync` via the `System.WindowsRuntimeSystemExtensions.AsTask` pattern â€” the SAME engine as the Windows lock screen. Windows sign-in is untouched; the user touches the sensor, a small Windows prompt confirms, InnologyLock proceeds.
- **CRITICAL gotcha found**: the old v0.0.34 script used the type `UserConsentVerifierResult` which DOES NOT EXIST in WinRT (`UserConsentVerificationResult` is the real name). It failed with "Unable to find type" on real hardware â€” the v0.0.33/34 Hello path was therefore ALSO broken on this machine, which is why "it never worked". The corrected script was proven live: prompt appears, fingerprint accepted, `Verified` returned.
- `check_biometric_available` is back to fast registry/service checks (WbioSrvc running + `WindowsHello\Enabled=0x1` + Bio Credential Provider key).
- `Win32_Devices_BiometricFramework` removed from BOTH `windows-sys` and `windows` feature lists in Cargo.toml (no WBF code remains).

### 6. Docs updated (standing rule)
- AGENTS.md, CHANGELOG.md (0.0.36 entry), docs/DEV_LOG.md, docs/ARCHITECTURE.md (IPC table + vault storage), docs/SECURITY.md, docs/PRD.md, README.md, docs/INSTRUCTION.md (tree + decisions).

---

## What Was Done in This Session (2026-08-24) â€” the v0.0.38 feature set

The user asked to implement Android-inspired lock/unlock features for Windows: hidden files, app lock with auth prompt, fingerprint in widget, auto-relaunch. All items below are in the working tree (uncommitted).

### 1. Android-style app lock with auto-relaunch
- After widget unlock of an app target, `AUTO_RELAUNCH_APP` static stores the exe path. On widget focus-lost, the app is spawned automatically before re-encrypting the exe. No more manual re-launching.
- Widget success message now says "Launching application..." for app targets.

### 2. Fingerprint option in the UnlockWidget
- New `cmd_widget_unlock_biometric` command: calls `biometric::authenticate_biometric` (Windows Hello prompt), loads the DPAPI token for the password, then runs the same unlock flow as `cmd_widget_unlock`.
- `UnlockWidget.tsx` now shows a "Use Fingerprint" button when biometric hardware is available, with an "or" divider between fingerprint and password.
- Bridge: `widgetUnlockBiometric` added to `tauri-bridge.ts`.

### 3. Locked files/folders are invisible in Explorer
- `file_locker.rs`: new `hide_blob` / `show_blob` functions using `SetFileAttributesW` from `Win32::Storage::FileSystem`.
- After writing `.InnologyLock` blobs: `hide_blob` sets `FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM` â€” blobs disappear from Explorer's default view.
- Before reading blobs for unlock: `show_blob` restores `FILE_ATTRIBUTE_NORMAL`.
- Applies to both single files and folder contents (recursive encrypt calls `lock_file` which calls `hide_blob`).

### 4. Restored working tree from catastrophic damage
- `lib.rs` was gutted (1929 â†’ 81 lines) and `vault_storage.rs` was replaced with broken stubs. Both restored from `HEAD` via `git checkout`. Only `process_guard.rs` changes (minor: `add_prompted_folder` + `clear_prompted_folder` made public) were preserved.

---

## What Was Done in the Previous Session (2026-08-07) â€” the v0.0.35 feature set

The user (owner) reported 6 core problems. This session continued the interrupted implementation and **finished + verified** it. Everything below is in the working tree.

### 1. Fingerprint login now uses the Windows Biometric Framework DIRECTLY (no Windows Hello)

**âš ï¸ SUPERSEDED in v0.0.36** â€” hardware testing (2026-08-16) proved this driver ignores background-app captures; the app now uses the Windows Hello prompt instead (see "What Was Done in This Session" item 5). Kept below for history.

User requirement #5: "my computer has fingerprint sensorâ€¦ I don't want it to work via Windows Hello."

- **Old**: `biometric.rs` called PowerShell â†’ `Windows.Security.Credentials.UI.UserConsentVerifier` = literally the Windows Hello dialog.
- **New** (`src-tauri/src/biometric.rs`, fully rewritten): raw WBF C API via `windows-sys` glob imports (same style as `file_locker.rs`):
  - `check_biometric_available()` â†’ `WinBioEnumBiometricUnits` (counts real sensors; no registry/Hello checks)
  - `authenticate_biometric()` â†’ `WinBioOpenSession(FINGERPRINT, POOL_SYSTEM)` â†’ `WinBioIdentify()` (blocks for finger, 30 s timeout via `tokio::time::timeout`) â†’ verifies returned `WINBIO_IDENTITY.Type == WINBIO_ID_TYPE_SID (3)` â†’ `EqualSid` against current user â†’ `WinBioCloseSession`
  - No PowerShell, no Windows Hello dialog. InnologyLock owns the prompt.
- **Cargo.toml**: added `Win32_Devices_BiometricFramework` to **windows-sys** features.
- **Gotcha learned**: windows-sys 0.61 does NOT re-export `WINBIO_TYPE_FINGERPRINT`, `WINBIO_POOL_SYSTEM`, `WINBIO_FLAG_DEFAULT`, `WINBIO_ID_TYPE_SID` as named items â€” they are defined as raw `u32` constants at the top of `biometric.rs` from SDK header values. Do not delete them.
- Frontend: `SecurityPage.tsx` renamed "Biometric Login (Windows Hello)" â†’ "Biometric Login (Fingerprint)".
- **Server-side enforcement** (from v0.0.34, kept): `cmd_biometric_login` verifies the fingerprint inside the command before loading the DPAPI token.

### 2. AES-256-GCM encryption-based locking REPLACES the old ACL locking (user requirement #6 â€” "ACL lock was destructive")

- **Old**: `file_locker.rs` `apply_safe_lock` set NTFS owner = SYSTEM + restricted DACL â†’ files/folders became permanently inaccessible (even to the user). This is what broke the user's already-locked files.
- **New** (`src-tauri/src/file_locker.rs`, rewritten, `lock_file/unlock_file/lock_folder/unlock_folder` now take `key_material: &[u8]`):
  - Locking a file **encrypts it in place**: `original.txt` â†’ `original.txt.InnologyLock`, original deleted.
  - Blob layout: `[4] "OLCK" magic | [1] version=1 | [12] AES-GCM nonce | [4] original path len (u32 LE) | [N] original path (UTF-8) | [*] ciphertext+tag`
  - Folder lock = recursive encrypt of every file inside (folder itself stays browsable). Folder unlock = recursive decrypt.
  - **Key**: 32 random bytes stored in the vault as `VaultConfig.file_encryption_key` (`models.rs`, `auth.rs` generates at setup). It is independent of the master password (survives password changes). Key is loaded into `AppState.file_key` + global `ACTIVE_FILE_KEY` at login.
  - `verify_lock` / `is_file_locked` / `is_folder_locked` detect `.InnologyLock` blobs.
  - Old ACL-damage recovery functions are **kept**: `safe_recover_acl`, `force_unlock`, `scan_acl_damage`, `bulk_recover_acl`.
- **Vault migration**: `cmd_unlock_session` and `cmd_biometric_login` auto-generate `file_encryption_key` for pre-encryption vaults (len != 32 â†’ generate + re-encrypt). **Old-format locks (owner=SYSTEM) are NOT auto-migrated** â€” user fixes them via the ACL Recovery scanner below.

### 3. ACL Recovery: recursive scanner + bulk fixer UI (fixes the user's already-broken files)

- `file_locker.rs`: `scan_acl_damage(root) -> Vec<String>` walks with `FindFirstFile/FindNextFile` (works even under restricted ACLs) and collects every item owned by SYSTEM (S-1-5-18) â€” the old lock's fingerprint. `bulk_recover_acl(paths)` runs `force_unlock` on each.
- `lib.rs`: `cmd_scan_acl_damage`, `cmd_bulk_recover_acl` (both registered). `cmd_force_unlock`/`cmd_recover_acl`/`cmd_bulk_recover_acl` now also call `service_client::notify_force_remove_locked_item` so the service doesn't re-apply the old ACL lock at boot.
- `tauri-bridge.ts`: `scanAclDamage`, `bulkRecoverAcl`.
- `DiagnosticsPage.tsx`: new `AclScannerForm` â€” path input + Scan â†’ list of damaged items â†’ "Fix All N Items" or per-item Fix â†’ âœ…/âŒ results.

### 4. Widget popup: temporary unlock + AUTO-RE-LOCK on close (user requirement #4)

- `lib.rs`: new `WIDGET_TEMP_UNLOCKED` static. `cmd_widget_unlock` no longer removes items from the vault config (item stays logically locked); it lifts the lock temporarily and stores the target.
- Widget window `Focused(false)` event â†’ re-applies `lock_file`/`lock_folder`/`lock_drive` using `get_active_file_key()`, then hides. Close the widget â†’ item re-locks automatically.
- Also added `--open-locked <path>` CLI arg + `.InnologyLock` file association (`shell_context.rs` registers `HKEY_CLASSES_ROOT\.InnologyLock â†’ InnologyLockFile` â†’ `InnologyLock.exe --open-locked "%1"`). Double-clicking a `.InnologyLock` file opens InnologyLock and shows the unlock widget after login.

### 5. GitHub Release pipeline (user complaint: "new version not on GitHub with signature")

- `.github/workflows/build.yml`: builds the service + monitor, then:
  - **Tag push (`v*.*.*`)** â†’ `tauri-apps/tauri-action` creates a **signed GitHub Release** (signing key from repo secrets) with the installer + `.sig` attached.
  - Non-tagged push / workflow_dispatch â†’ build only + artifact upload.
- Repo secrets **are configured** on GitHub: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (set 2026-07-28).
- **âš ï¸ CI bugs fixed this session** (every previous run failed before reaching the release):
  1. "Build service daemon" failed: PowerShell `copy` refuses to overwrite the **committed** `src-tauri/resources/InnologyLock-*.exe` binaries ("item already exists"). Fixed with `shell: bash` + `cp -f`.
  2. tauri-action failed with "Resource not accessible by integration" â€” the default `GITHUB_TOKEN` is read-only. Fixed with `permissions: contents: write` at the workflow top.
  Never reintroduce either.

### 6. Fixes this session applied on top of the interrupted work (IMPORTANT â€” the previous session left these broken)

| # | Issue found | Fix |
|---|---|---|
| a | **Compile errors**: `drive_locker.rs` called `lock_folder/unlock_folder` without the new key param; `lib.rs` `cmd_rescue_unlock` had a stale `unlock_file(&path)` + wrong match arm | `lock_drive`/`unlock_drive` now take `key_material`; all 4 call sites pass the key (`cmd_add_locked_drive`, `cmd_remove_locked_drive`, `cmd_widget_unlock` drive branch, widget auto-relock drive branch). `cmd_rescue_unlock` now takes `State`, calls `require_file_key`, returns the decrypted path |
| b | **File key never set on biometric login**: `cmd_biometric_login` set session/password/config but NOT `state.file_key`/`ACTIVE_FILE_KEY` â†’ every lock/unlock after a fingerprint login would fail with "File encryption key not available" | Added key migration + `state.file_key` set + `set_active_file_key` in `cmd_biometric_login` |
| c | **File key never set on widget unlock**: same bug in `cmd_widget_unlock` (only set the global, not `state.file_key`) | Now sets both |
| d | **Old service would re-inflict ACL damage**: app still called `service_client::notify_lock_file/folder/drive`, which makes the service apply owner=SYSTEM + restricted DACL and persist it for re-application at every boot â€” exactly the destructive behavior the user reported | Removed those notifications from all encryption lock paths (`cmd_add_locked_file/folder/drive`, `cmd_toggle_locked_app`, `cmd_add_locked_app`). ACL recovery commands now purge the service's persisted list |
| e | Unused imports + dead code warnings | Removed unused `Security::Authorization` (biometric.rs) + `Storage::FileSystem` (file_locker.rs) glob imports; `#[allow(dead_code)]` on reserved `backup_path_for` |
| f | **Drive lock would encrypt the WHOLE drive** (data hazard: installed apps, hours of operation) | `drive_locker` no longer calls recursive `lock_folder` on the drive root â€” drive lock = NoDrives hide only (safe). `lock_folder` now **refuses drive roots** so the folder UI can't encrypt `D:\` by accident |
| g | **No critical-path guard** (locking the vault dir would encrypt the key itself; locking the exe bricks the app) | `check_protected_path` in `file_locker.rs` refuses: drive roots, `%APPDATA%\InnologyBD` (vault), and the running exe â€” enforced in `lock_file` + `lock_folder` |
| h | **`unlock_file` would silently overwrite a (re)created original file** (data loss) | Refuses to write when the original already exists |
| i | **Recursive encrypt/decrypt follows junctions/symlinks** â€” could encrypt files OUTSIDE the locked tree (OneDrive redirects etc.) | Skip symlinks/reparse points in `encrypt_dir_recursive` / `decrypt_dir_recursive` |
| j | **`Key::from_slice` panics on wrong-length key** â€” a panic in a command kills the app | `do_encrypt`/`do_decrypt` validate 32-byte key and return an error instead |
| k | **`cmd_scan_acl_damage` / `cmd_bulk_recover_acl` run synchronously** â€” deep walks can freeze the UI | Made `async` + `tokio::task::spawn_blocking` |
| l | **Pre-0.0.35 vault + widget unlock before main-app login** failed (widget only errored on key length instead of migrating) | `cmd_widget_unlock` now runs the same key migration as the other login paths |
| m | `.InnologyLock` double-click flow required manual context-menu install | New `shell_context::register_extension_only()` called in the setup hook on every launch (idempotent) |
| n | Frontend: failed per-item ACL fix vanished from the damaged list | `AclScannerForm.handleFixOne` keeps items whose fix returned an error |

---

## Files Modified in This Session (git status â€” DO NOT REVERT)

```
 M src-tauri/src/biometric.rs
 M src-tauri/src/file_locker.rs
 M src-tauri/src/lib.rs
 M src-tauri/src/service_client.rs
 M src-tauri/src/vault_storage.rs      (NEW â€” vault file storage)
 M src/App.tsx
 M src/components/pages/VaultPage.tsx
 M src/lib/tauri-bridge.ts
```

Plus the documentation updated in this session: `AGENTS.md`, `CHANGELOG.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/BUILD_SETUP.md`, `docs/DEV_LOG.md`, `docs/INSTRUCTION.md`, `docs/PRD.md`, `docs/SECURITY.md`.

---

## Known Errors & Fixes (Reference)

| Error | Root Cause | Fix |
|-------|-----------|-----|
| App window not appearing (v0.0.25+) | `logger::init()` deadlocks on non-reentrant `Mutex` | `drop(guard)` before `log()` call in `init()` |
| All pipe responses fail deser | `SvcResponse` had `#[serde(tag = "type")]` on client, no tag on service | Removed tag in `service_client.rs` |
| icacls /deny silently fails | CLI reports success but never applies deny ACE | Rewrote `acl.rs` to use Win32 `SetNamedSecurityInfoW` |
| **ACL lock permanently destroys access (user #6)** | `apply_safe_lock` set owner=SYSTEM + restricted DACL; Windows ignores WRITE_DAC under DENY-everyone style locks | **Replaced with AES-256-GCM encryption** (`file_locker.rs`) + ACL Recovery scanner to fix old damage |
| **`WINBIO_*` constants not found (cargo E0432)** | windows-sys 0.61 doesn't re-export `WINBIO_TYPE_FINGERPRINT`, `WINBIO_POOL_SYSTEM`, `WINBIO_FLAG_DEFAULT`, `WINBIO_ID_TYPE_SID` | Define them as raw `u32` constants in `biometric.rs` (SDK header values: 0x8, 0x1, 0x0, 3) â€” **MOT since v0.0.36: all WBF code removed** |
| **`unlock_file/lock_folder` missing key arg (cargo E0061)** | encryption rewrite changed signatures; `drive_locker.rs` + `cmd_rescue_unlock` were never updated | Pass `key_material` at every call site (verified by search: 18 call sites) |
| **"File encryption key not available" after biometric/widget login** | `cmd_biometric_login` and `cmd_widget_unlock` never set `state.file_key` (only `cmd_unlock_session` did) | Set `state.file_key` + `set_active_file_key` in both commands |
| **Service re-locks fixed files at boot** | Service persists `locked_items` and re-applies `acl::apply_lock` at startup; ACL recovery fixed the ACL but never purged the list | Recovery commands (`cmd_force_unlock`, `cmd_recover_acl`, `cmd_bulk_recover_acl`) now call `notify_force_remove_locked_item` |
| **CI fails every push at ~1m40s** | "Build service daemon" step: PowerShell `copy` refuses to overwrite the committed `src-tauri/resources/InnologyLock-*.exe` â†’ "An item â€¦ already exists" | `shell: bash` + `cp -f` in `build.yml` |
| **CI release step: "Resource not accessible by integration"** | Default `GITHUB_TOKEN` is read-only; creating a release needs `contents: write` | `permissions: contents: write` at the top of `build.yml` |
| Widget steals focus while user works | Monitor calls `widget.show()+set_focus()` every 2 s poll | `PROMPTED_FOLDERS` static â€” prompt once per folder (v0.0.34) |
| Biometric token loadable without fingerprint | Token was DPAPI-only; Hello prompt was client-side | `authenticate_biometric` runs inside `cmd_biometric_login` before loading token (v0.0.34) |
| **Fingerprint login "never works" on the owner's laptop** | Two stacked failures: v0.0.34 Hello script used the nonexistent `UserConsentVerifierResult` type ("Unable to find type" â€” `UserConsentVerificationResult` is the real name); v0.0.35's direct WBF `WinBioIdentify` is ignored by the Synaptics driver (zero sensor events in the WBF log for background apps) | v0.0.36: Hello prompt with the **corrected type name** â€” proven live (`Verified`); Windows sign-in untouched |
| Weather inaccurate | wttr.in bad for Bangladesh | Open-Meteo API |

---

## Build & Run Commands

```bash
# Dev mode (hot reload)
npm run tauri dev

# Production build (produces installer)
npm run tauri build

# TypeScript check only
npx tsc --noEmit

# Rust checks
cd src-tauri && cargo check && cd ..
cd service && cargo check && cd ..
```

---

## âš ï¸ AUTO-UPDATER SIGNING KEY (CRITICAL â€” UNCHANGED)

**The signing key MUST match the public key embedded in the installed binary.** Never regenerate or overwrite it.

| Field | Value |
|-------|-------|
| **Public key ID** | `EDE58385BDE79B6F` |
| **Public key (base64)** | `RWRvm+e9hYPl7eUOcS2Q3cknhhVt06dE6IRPrbFNNE/CqnEDdfYs12Wy` |
| **Private key file** | `src-tauri/update.key` |
| **Backup copy** | `signing-keys/update-v0.1.0.key` |
| **Password** | (kept in password manager + local-only `signing-keys/README.json` — never commit it) |
| **tauri.conf.json** | `pubkey = "EDE58385BDE79B6F"` |

To sign a release manually (or in CI via secrets `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`):
```powershell
$key = (Get-Content -Raw "src-tauri\update.key").Trim()
$env:TAURI_SIGNING_PRIVATE_KEY = $key
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<your-signing-password>"
cmd /c "echo. | npx tauri signer sign `"path\to\installer.exe`""
```

---

## Architecture (updated for v0.0.36)

```
InnologyLock/
â”œâ”€â”€ src/                      # React frontend (Vite + Tailwind)
â”‚   â”œâ”€â”€ components/
â”‚   â”‚   â”œâ”€â”€ auth/             # LoginScreen, SetupWizard, GitHubConnect
â”‚   â”‚   â”œâ”€â”€ layout/           # Sidebar, TopBar, Footer
â”‚   â”‚   â”œâ”€â”€ pages/            # Dashboard, SystemMonitor, AppLocker, Presets, Vault, Security, Diagnostics, History
â”‚   â”‚   â”œâ”€â”€ shared/           # Field, Toggle, StatusPill, Stat, SectionHeader
â”‚   â”‚   â””â”€â”€ widget/           # UnlockWidget (separate Tauri window)
â”‚   â””â”€â”€ lib/tauri-bridge.ts   # Tauri invoke wrappers
â”œâ”€â”€ src-tauri/                # Rust backend (Tauri)
â”‚   â””â”€â”€ src/
â”‚       â”œâ”€â”€ lib.rs            # Tauri commands, AppState (file_key), UNLOCK_TARGET/WIDGET_TEMP_UNLOCKED/ACTIVE_FILE_KEY
â”‚       â”œâ”€â”€ vault.rs          # Argon2id + AES-256-GCM vault
â”‚       â”œâ”€â”€ auth.rs           # setup/unlock + file_encryption_key generation
â”‚       â”œâ”€â”€ biometric.rs      # Windows Hello (UserConsentVerifier) verification + DPAPI token
â”‚       â”œâ”€â”€ file_locker.rs    # AES-256-GCM encryption locking + ACL damage scanner/recovery
â”‚       â”œâ”€â”€ vault_storage.rs  # vault file storage (OVLF blobs + encrypted OVMF manifest)
â”‚       â”œâ”€â”€ drive_locker.rs   # NoDrives registry + USB-removal monitor
â”‚       â”œâ”€â”€ process_guard.rs  # app kill monitor + Explorer folder monitor (widget prompt, auto-relock)
â”‚       â”œâ”€â”€ shell_context.rs  # context menu + .InnologyLock file association
â”‚       â””â”€â”€ ...               # totp, system_presets, installer_guard, watchdog, system_monitor, etc.
â”œâ”€â”€ service/                  # Windows service (named-pipe ACL daemon â€” LEGACY, state purged at app boot;
â”‚   â”‚                         #   encryption locks bypass it entirely)
â”œâ”€â”€ shared/                   # InnologyLock-shared protocol crate (SvcRequest/SvcResponse/LockedItem)
â”œâ”€â”€ AGENTS.md                 # THIS FILE â€” session handoff
â””â”€â”€ docs/                     # ARCHITECTURE, BUILD_SETUP, DEV_LOG, INSTRUCTION, PRD, SECURITY
```

---

## Versioning Scheme

Patch bumps: 0.0.35 â†’ 0.0.36 â†’ â€¦ â†’ 0.1.0

## Version History (most recent first)

- **0.0.39 (UNRELEASED)** â€” Context menu fix (infinite loop resolved), hidden items page, lock on screen lock (`WTSRegisterSessionNotification`), app data lock, lock event toast notifications.
- **0.0.38 (UNRELEASED)** â€” Android-style app lock with auto-relaunch, fingerprint in UnlockWidget, locked blobs hidden from Explorer (`FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM`).
- **0.0.37 (RELEASED 2026-08-17, CI-built + signed; auto-update published)** â€” **Biometric availability fixed**: `check_biometric_available` now asks `UserConsentVerifier.CheckAvailabilityAsync()` instead of registry keys (`WindowsHello\Enabled` + `Bio\Credential Provider` are absent on Windows 11 â€” that's why v0.0.36 said "unable to detect biometric" on a working sensor). All background commands now spawn with `CREATE_NO_WINDOW` (no console flashes; fixed `cloud_status.rs` + `shell_context.rs` + service daemon start). Installer sha256 `6D5565FF5A993C34B86D1DA1097736461C492E8FA15694F3EE450AE9BBED9B50` on the `v0.0.37` release.
- **0.0.36 (RELEASED 2026-08-16, CI-built + signed; auto-update published)** â€” Vault Storage (encrypted private file storage + encrypted manifest), stale-unlock detection + re-lock/keep prompt after restart, legacy service ACL boot purge, `cmd_biometric_login` parity (auto-lock minutes + USB-removal callback), **biometric login reworked to the Windows Hello prompt** (hardware-proven on the HP EliteBook 850 G5: the Synaptics WBF driver ignores background-app captures, so direct WBF is dead; the corrected `UserConsentVerificationResult` type name makes the Hello path work). Installer sha256 `FF982C26EE5DC96B0C5FC296EAB959EF26E24435BB69C056F0848F5E5CF2AFE8` on the `v0.0.36` release.
- **0.0.35 (RELEASED 2026-08-07, CI-built + signed)** â€” AES-256-GCM encryption-based file/folder locking replaces destructive ACL locking; direct WBF fingerprint (no Windows Hello); ACL damage scanner + bulk recovery UI; widget temp-unlock + auto-relock; `.InnologyLock` file association + `--open-locked`; GitHub Release pipeline fixed and now works on tag push. Installer sha256 `2598219E8238B1EF...7F68CDCC` on the `v0.0.35` release. Runtime E2E testing still outstanding.
- **0.0.34** â€” Widget focus-steal fix (prompt once per folder). Server-side Windows Hello enforcement in `cmd_biometric_login`. Signed installer + updated `latest.json`.
- **0.0.33** â€” File-unlock child ACL fix v3, `force_unlock`, widget capability fix, `sync_vault_to_service` wiring.
- **0.0.29** â€” Deadlock fix in `logger::init()`. Fallback updater endpoints.
- **0.0.25** â€” Diagnostics system + logger.rs (INTRODUCED the deadlock that hid the window).
- â€¦ earlier history in CHANGELOG.md

---

## Next Session: Continue from Here

### Where we left off
v0.0.39 is **unreleased** — minifilter driver + folder container lock + 3-attempt lockout. All features compile clean (cargo check + tsc). Working tree has uncommitted changes for this feature set. **Manual steps remain**: rename GitHub repo, install WDK + build driver.

### What Was Built This Session (2026-09-02)

#### 1. Minifilter Driver (kernel-mode)
- `driver/src/filter.c` + `driver/src/filter.h` — intercepts `IRP_MJ_CREATE` on locked paths
- `driver/inf/innologylock_filt.inf` — driver installation INF
- `driver/build.bat` — WDK build script
- `driver/install.bat` — driver installation script (run as admin)
- `driver/install_wdk.bat` — WDK download helper
- **How it works**: When Explorer tries to open a locked folder, the driver intercepts the request, sends it to the service via filter communication port, service shows unlock modal, user authenticates, driver allows/blocks access.
- **Requires**: WDK installed + test signing mode (`bcdedit /set testsigning on`)

#### 2. Folder-to-Container Lock (file_locker.rs)
- `lock_folder_as_container(path, key)` — folder → single `.InnologyLock` encrypted file (folder deleted)
- `unlock_folder_from_container(blob_path, key)` — `.InnologyLock` → restore original folder
- `is_folder_container(blob_path)` — detect if blob is a folder container
- Container format: `[4] file_count | for each: [4] rel_path_len | rel_path | [4] content_len | content` — encrypted with AES-256-GCM
- **User sees**: Lock a folder → folder disappears → single `.InnologyLock` file appears → double-click → widget → password → folder restored

#### 3. 3-Attempt Lockout (UnlockWidget.tsx)
- After 3 wrong passwords/fingerprints → 30-second cooldown
- Progress bar shows countdown
- App goes to tray during cooldown
- Attempts reset after successful unlock or cooldown

#### 4. Service Filter Bridge (filter_bridge.rs)
- `add_filter_path(path)` — register path with minifilter driver via IOCTL
- `remove_filter_path(path)` — unregister path
- `clear_filter_paths()` — remove all locked paths
- `set_filter_enabled(enabled)` — global enable/disable

#### 5. Lock/Unlock Commands Updated
- `cmd_add_locked_folder` — AES container lock + register with filter driver
- `cmd_remove_locked_folder` — AES container unlock + remove from filter driver
- `cmd_add_locked_file` — AES lock + register with filter driver
- `cmd_remove_locked_file` — AES unlock + remove from filter driver

### Build & Run Commands

```bash
# Dev mode (hot reload)
npm run tauri dev

# Production build (produces installer)
npm run tauri build

# TypeScript check only
npx tsc --noEmit

# Rust checks
cd src-tauri && cargo check && cd ..
cd service && cargo check && cd ..

# Build minifilter driver (requires WDK)
cd driver && build.bat

# Install driver (requires admin + test signing)
cd driver && install.bat
```

### Architecture (Updated)

```
InnologyLock/
├── driver/                     # NEW: Minifilter driver (C + WDK)
│   ├── src/filter.c           # IRP_MJ_CREATE interception
│   ├── src/filter.h           # Shared definitions
│   ├── inf/innologylock_filt.inf
│   ├── build.bat
│   └── install.bat
├── src-tauri/                  # Rust backend (Tauri)
│   └── src/
│       ├── lib.rs             # Tauri commands (lock/unlock + filter registration)
│       ├── file_locker.rs     # AES-256-GCM + folder container lock
│       ├── service_client.rs  # IPC to service + filter path management
│       └── ...
├── service/                    # Windows service (driver ↔ app bridge)
│   └── src/
│       ├── filter_bridge.rs   # NEW: Filter driver communication via IOCTL
│       ├── bin/svc.rs         # Service main + IPC handler
│       └── ...
├── shared/                     # IPC protocol (updated with filter messages)
└── src/                        # React frontend
    └── components/widget/
        └── UnlockWidget.tsx   # 3-attempt lockout + countdown
```

### Flow: Lock a Folder
1. User clicks "Lock Folder" → `cmd_add_locked_folder`
2. `file_locker::lock_folder_as_container` → folder → single `.InnologyLock` file
3. `service_client::add_filter_path` → service → IOCTL → driver registers path
4. `.InnologyLock` file visible in Explorer (NOT hidden — user needs to see it)

### Flow: Open a Locked Folder
1. User double-clicks `.InnologyLock` file in Explorer
2. File association: `InnologyLock.exe --open-locked "path\to\folder.InnologyLock"`
3. App detects container type → shows unlock widget
4. User enters password → `cmd_widget_unlock`
5. `file_locker::unlock_folder_from_container` → folder restored
6. User can now access the folder

### Flow: Minifilter Blocks Access (when driver is loaded)
1. User tries to open a locked path in Explorer
2. Driver intercepts `IRP_MJ_CREATE` → sends request to service
3. Service forwards to Tauri app → shows unlock modal
4. User authenticates → driver allows access
5. If 3 failures → app minimizes to tray

### Suggested next steps (in order)
1. **Install WDK** — download from https://go.microsoft.com/fwlink/?LinkId=2371554, or run `driver/install_wdk.bat` as admin
2. **Build driver** — open "Developer Command Prompt for VS 2022", cd to `driver/`, run `build.bat`
3. **Enable test signing** — `bcdedit /set testsigning on` (reboot required)
4. **Install driver** — run `driver/install.bat` as admin
5. **E2E test on Windows**:
   - Lock a folder → folder becomes `.InnologyLock` file → double-click → widget → password → folder restored
   - Lock a file → `.InnologyLock` blob → double-click → widget → password → file restored
   - 3 wrong passwords → 30-second cooldown → app to tray
   - Right-click context menu lock/unlock
6. **Release v0.0.39** (bump versions, tag, CI build).
7. Update docs as you go (standing rule).