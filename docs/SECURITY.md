# InnologyLock - Security Specification

**Document Version:** 2.2.0

---

## 1. Encryption

### Vault Encryption
- **Algorithm:** AES-256-GCM (authenticated encryption)
- **Key Derivation:** Argon2id (m=65536, t=3, p=1, output=32 bytes)
- **Key:** Argon2id(password, per-vault random salt) â€” NOT SHA-256(password)
- **Nonce:** Random 96-bit nonce generated per encryption
- **Header Magic:** `OMNI` (4 bytes) for format identification

### Recovery Data Encryption
- **Algorithm:** AES-256-GCM
- **Key:** SHA-256(security_answer_lowercased_trimmed) â€” used to encrypt/decrypt the master password
- **Storage:** Separate file (`vault.recovery`), not inside the vault

### Password Verification
- **Algorithm:** Argon2id (same params as encryption)
- **Storage:** `VaultConfig.password_hash` (32 bytes)
- **Salt:** `VaultConfig.password_salt` (16 bytes)

### TOTP
- **Algorithm:** RFC 6238 (TOTP)
- **Hash:** SHA-1
- **Digits:** 6
- **Period:** 30 seconds
- **Tolerance:** 1 time-step (past and future)
- **Secret Encoding:** Base32 (RFC 4648) â€” compatible with all major authenticator apps

---

## 2. System Protection

### File/Folder Locking (v0.0.35 â€” AES-256-GCM encryption, replaced ACL)
- **Method:** Authenticated encryption in place (AES-256-GCM, `aes-gcm` crate). No NTFS ACL modification.
- **Key:** `VaultConfig.file_encryption_key` â€” 32 random bytes inside the encrypted vault, never rotated on password change.
- **Lock:** `original` â†’ `original.InnologyLock` (`OLCK` magic + version + 96-bit nonce + original path + ciphertext + 16-byte tag), original deleted. Blob is hidden (`FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM`) so it's invisible in Explorer by default.
- **Folder:** recursive encrypt; the folder stays browsable.
- **Unlock:** decrypt via the embedded original path; blob deleted.
- **Security property:** contents are unreadable without the vault key. If the vault is lost, locked files are unrecoverable â€” keep a vault backup.
- **Legacy ACL damage:** `scan_acl_damage` / `bulk_recover_acl` / `force_unlock` repair files broken by the pre-0.0.35 ACL lock (owner=SYSTEM). They also purge the service's persisted re-lock list.
- **Boot sweep (v0.0.36):** on app startup the service's persisted legacy ACL `locked_items` are purged (`get_locked_items()` + `notify_force_remove_locked_item`), so the legacy daemon can never re-apply owner=SYSTEM locks at boot.
- **Stale-unlock reconciliation (v0.0.36):** after login `cmd_verify_locked_state` compares the vault's `locked_files`/`locked_folders` against disk; entries with no `.InnologyLock` blob (temp-unlocked, app exited before re-lock) are surfaced for the user to **re-lock** (`cmd_relock_entries`) or **keep unlocked** (`cmd_forget_unlocked_entries`). No silent re-encryption â€” folders in active use are never re-encrypted without consent.

### Vault Storage (v0.0.36)
- **Purpose:** private encrypted storage of user files (`%APPDATA%\InnologyBD\InnologyLock\storage\<id>.vaultfile`).
- **Blob format:** `OVLF` magic | version 1 | 96-bit nonce | name_len (u32 LE) | original name (UTF-8) | size (u64 LE) | AES-256-GCM ciphertext + 16-byte tag. Key = the vault's `file_encryption_key`.
- **Manifest:** `vault_files.enc` â€” AES-256-GCM encrypted (`OVMF` magic + nonce + ciphertext JSON `[{id,name,size,added_at}]`) so **stored file names never leak on disk**.
- **Guarantees:** original file deleted only after the blob is written; extraction refuses to overwrite existing files and verifies size; blob + manifest entry removed only after a successful extraction write; symlinks and protected paths refused.
- **Recovery trade-off:** if the vault is lost, stored files are unrecoverable â€” same as locked files. Keep a vault backup.

### Drive Locking
- **Visibility:** `NoDrives` registry value hides drive in Windows Explorer (bitmask: bit 0 = A:, bit 1 = B:, â€¦)
- **Lock (v0.0.35):** NoDrives hide only. Whole-drive recursive encryption was removed as a data hazard; direct path access still works â€” see AGENTS.md for the open design question.
- **Legacy:** pre-0.0.35 used DACL on the drive root (owner=SYSTEM) â€” destructive; no longer sent to the service

### System Presets
| Preset | Registry Key | Value |
|:---|:---|:---|
| Task Manager | `HKCU\...\Policies\System` | `DisableTaskMgr=1` |
| Control Panel | `HKCU\...\Policies\Explorer` | `NoControlPanel=1` |
| Registry Editor | `HKCU\...\Policies\System` | `DisableRegistryTools=1` |
| PowerShell | `HKCU\...\Policies\Microsoft\Windows\PowerShell` | `EnableScripts=0` |
| CMD | `HKCU\...\Policies\Microsoft\Windows\System` | `DisableCMD=1` |
| System Restore | `HKLM\...\Policies\Microsoft\Windows NT\SystemRestore` | `DisableSR=1` |

---

## 3. Password Reset (2FA Wipe on Reset)

1. User answers security question
2. Answer hash used as key to decrypt `vault.recovery` â†’ recovers old master password
3. Old master password decrypts `vault.enc` â†’ full vault config loaded
4. Vault is re-encrypted with new Argon2id(password, new_salt) key
5. 2FA is **disabled** (totp_enabled = false, totp_secret cleared)
6. Recovery data is re-encrypted with same answer hash but new password
7. User must log in with new password â€” no 2FA until re-enabled

---

## 4. Panic Lock (Win+Alt+L)

1. Hotkey triggers `PANIC_ACTIVE` atomic flag in Rust backend
2. Frontend detects flag and blanks the screen content
3. Session vault config is cleared from memory
4. User must re-authenticate to restore access

---

## 5. Data Locations

| Data | Path | Encrypted? |
|:---|:---|:---|
| Vault | `%APPDATA%\InnologyBD\InnologyLock\vault.enc` | Yes (AES-256-GCM) |
| Recovery | `%APPDATA%\InnologyBD\InnologyLock\vault.recovery` | Yes (AES-256-GCM) |
| Meta | `%APPDATA%\InnologyBD\InnologyLock\vault.meta` | No (plaintext) |
| Locked item | `path.InnologyLock` next to original | Yes (AES-256-GCM, `OLCK` blob) |
| Locked items index | `%APPDATA%\InnologyBD\InnologyLock\locked_items.json` | No (paths + display names only) |
| Vault storage blobs | `%APPDATA%\InnologyBD\InnologyLock\storage\*.vaultfile` | Yes (AES-256-GCM, `OVLF` blob) |
| Vault storage manifest | `%APPDATA%\InnologyBD\InnologyLock\vault_files.enc` | Yes (AES-256-GCM, `OVMF`) |

---

*End of SECURITY.md (InnologyLock v2.2.0)*
