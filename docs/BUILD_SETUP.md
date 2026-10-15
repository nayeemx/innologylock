# InnologyLock - Windows 10/11 App, Folder & File Locker
## Build, Environment Setup & Deployment Specification

**Document Version:** 2.0.0  
**Product Name:** InnologyLock  
**Developer / Publisher:** InnologyBD  
**Target Platform:** Windows 10/11 (x64 / ARM64)  

---

## 1. Automated 1-Click Environment Setup Script (`setup_environment.ps1`)

If your personal computer is missing **Rust**, **Node.js**, **Visual Studio C++ Build Tools**, or **WiX Toolchain**, simply run the automated PowerShell setup script included in `D:\projects\code\innologylock\docs\`:

```powershell
# Open PowerShell as Administrator and run:
Set-ExecutionPolicy Unrestricted -Scope Process -Force
.\setup_environment.ps1
```

### What `setup_environment.ps1` automatically does:
1. **Checks Node.js**: Verifies `node` & `npm`. If missing, installs Node.js LTS via `winget`.
2. **Checks Rust Toolchain**: Verifies `cargo` & `rustc`. If missing, automatically downloads `rustup-init.exe` and installs the `stable-x86_64-pc-windows-msvc` toolchain.
3. **Checks Visual C++ Build Tools**: Verifies MSVC compiler files. If missing, automatically triggers `winget install Microsoft.VisualStudio.2022.BuildTools`.
4. **Checks WiX / NSIS Bundlers**: Verifies installer packaging tools. If missing, installs WiX Toolchain via `winget`.

---

## 2. Windows UAC Manifest (`app.manifest`)

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.0.0.0"
    processorArchitecture="*"
    name="com.innologybd.InnologyLock"
    type="win32"
  />
  <description>InnologyLock - Windows 11 App, Folder and File Locker by InnologyBD</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
    <security>
      <requestedPrivileges xmlns="urn:schemas-microsoft-com:asm.v3">
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
    </application>
  </compatibility>
</assembly>
```

---

## 3. Tauri v2 Configuration (`src-tauri/tauri.conf.json`)

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "InnologyLock",
  "version": "1.0.0",
  "identifier": "com.innologybd.InnologyLock",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "plugins": {
    "updater": {
      "pubkey": "<public key from src-tauri/update.key.pub>",
      "endpoints": [
        "https://api.github.com/repos/nayeemx/InnologyLock/releases/latest"
      ]
    }
  },
  "app": {
    "windows": [
      {
        "title": "InnologyLock - Enterprise Desktop Security",
        "width": 1280,
        "height": 800,
        "minWidth": 1024,
        "minHeight": 680,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "transparent": false,
        "center": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https: data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline';"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "createUpdaterArtifacts": true,
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ],
    "resources": [],
    "copyright": "Copyright Â© 2026 InnologyBD. All rights reserved.",
    "shortDescription": "InnologyLock - Enterprise Desktop Security by InnologyBD",
    "longDescription": "Zero-trust enterprise desktop security tool. Application, folder, file & drive locker with Argon2id, AES-256-GCM, TOTP 2FA, and dual-process watchdog. Windows 10/11."
  }
}
```

---

## 4. Signing Key Setup (Required for Auto-Updates)

### Generate Key Pair
```powershell
cd D:\projects\code\innologylock
npx tauri signer generate -w src-tauri\update.key
# Enter your signing password when prompted (never commit it to the repo)
# Public key will be at src-tauri\update.key.pub
```

### Environment Variables for Signed Builds
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content src-tauri\update.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<your-signing-password>"
npx tauri build
```

### Key Files
- **Private key:** `src-tauri/update.key` (KEEP SECRET! git-ignored, never commit)
- **Public key:** `src-tauri/update.key.pub` (goes in tauri.conf.json)
- **Password:** kept ONLY in your password manager + the local-only `signing-keys/README.json` (git-ignored). It is never written in docs or code. If you lose it, you cannot sign updates with this key.
- **Signatures:** `.sig` files generated during signed builds

---

## 5. GitHub Releases Setup (v0.0.35+ â€” CI automates this)

The workflow `.github/workflows/build.yml` now builds the service + monitor, then:
- **Tag push (`v*.*.*`)** â†’ `tauri-apps/tauri-action` creates a **signed GitHub Release** with the installer + `.sig` attached (needs repo secrets `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`).
- **Non-tagged push / workflow_dispatch** â†’ build only, installer uploaded as a CI artifact.

### Release flow (recommended)
1. Bump version to `0.0.35` in `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `package.json`.
2. Commit everything (including updated `.md` files).
3. `git tag v0.0.35 && git push origin main --tags`
4. CI builds, signs, and publishes the release automatically.
5. Verify the release page has `InnologyLock_0.0.35_x64-setup.exe` + `.sig`, and that `latest.json`/updater endpoint resolves.

### Rotating the signing password (do this — old passwords are in git history)

**Password rotation ≠ key rotation.** Two different operations:

| Operation | Effect on installed apps | When to use |
|---|---|---|
| Change password only (same keypair) | None — auto-update keeps working | Password exposed (our case). Recommended. |
| New keypair (`signer generate`) | BREAKS auto-update — every installed copy must be manually reinstalled (embedded pubkey no longer trusts new signatures) | Only if the private key file itself leaked. |

**Secure handling rules (always):**
1. Generate the new password in your password manager (20+ chars). It lives in exactly 3 places: password manager, local-only `signing-keys/README.json` (git-ignored), and GitHub repo secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. NEVER in chat, docs, code, or terminal history.
2. Pass passwords via environment variables, typed with masked input — never as command arguments:
   ```powershell
   $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = Read-Host -MaskInput "Signing password"
   ```
3. After rotating, update the `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repo secret (GitHub → Settings → Secrets → Actions) or the next CI tag build signs with the wrong password and fails.
4. Verify with a test signature before deleting any backup (see below).

**Procedure (password-only rotation):**
1. Back up the current key: `Copy-Item src-tauri\update.key "$env:TEMP\update.key.bak"`.
2. Re-encrypt `src-tauri\update.key` with the new password. `tauri signer` has NO password-change command (`sign` / `generate` only), so this needs a small helper script (minisign re-encrypt: decrypt with old password → re-encrypt with new, same keypair). Ask the AI to write + run it when ready — it takes the old/new passwords via env vars and never writes them anywhere.
3. Verify: `npx tauri signer sign` a throwaway file with the new password and confirm the `.sig` verifies against the pubkey in `tauri.conf.json`. Then delete `$env:TEMP\update.key.bak`.
4. Update `signing-keys/README.json` + password manager + the GitHub secret. Done — old history passwords are dead.

### Manual fallback (if CI is not used)
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content src-tauri\update.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<your-signing-password>"
npx tauri build
# then create the release with gh CLI, attaching the installer + .sig
```

---

*End of BUILD_SETUP.md (InnologyLock v2.0.0)*
