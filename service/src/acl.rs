use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::*;
use windows_sys::Win32::Security::Authorization::*;
use windows_sys::Win32::System::Threading::*;

fn to_wide(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

unsafe fn enable_privilege(name: &str) -> Result<(), String> {
    let mut h_token = std::ptr::null_mut();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut h_token) == 0 {
        return Err("OpenProcessToken failed".to_string());
    }
    let name_w = to_wide(name);
    let mut luid: LUID = std::mem::zeroed();
    if LookupPrivilegeValueW(std::ptr::null_mut(), name_w.as_ptr(), &mut luid) == 0 {
        CloseHandle(h_token);
        return Err(format!("LookupPrivilegeValue({}) failed", name));
    }
    let mut tp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES { Luid: luid, Attributes: SE_PRIVILEGE_ENABLED }],
    };
    let ret = AdjustTokenPrivileges(h_token, 0, &mut tp, std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
        std::ptr::null_mut(), std::ptr::null_mut());
    let gle = GetLastError();
    if ret == 0 || gle != 0 {
        CloseHandle(h_token);
        return Err(format!("AdjustTokenPrivileges({}) failed: err={}", name, gle));
    }
    CloseHandle(h_token);
    Ok(())
}

unsafe fn current_user_sid_buf() -> Result<Vec<u8>, String> {
    let mut h_token = std::ptr::null_mut();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut h_token) == 0 {
        return Err("OpenProcessToken failed".to_string());
    }
    let mut len: u32 = 0;
    GetTokenInformation(h_token, TokenUser, std::ptr::null_mut(), 0, &mut len);
    let mut buf = vec![0u8; len as usize];
    if GetTokenInformation(h_token, TokenUser, buf.as_mut_ptr() as _, len, &mut len) == 0 {
        CloseHandle(h_token);
        return Err("GetTokenInformation failed".to_string());
    }
    CloseHandle(h_token);
    Ok(buf)
}

fn backup_dir() -> std::path::PathBuf {
    let base = std::env::var("PROGRAMDATA")
        .unwrap_or_else(|_| r"C:\ProgramData".to_string());
    std::path::PathBuf::from(base).join("InnologyBD").join("InnologyLock").join("acl_backups")
}

fn backup_path_for(path: &str) -> std::path::PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hex = format!("{:016x}", hasher.finish());
    backup_dir().join(format!("{}.bin", hex))
}

/// Read the raw DACL security descriptor bytes for a path.
/// Returns (owner_sid_bytes, dacl_bytes).
fn read_current_dacl(path: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    unsafe {
        let path_wide = to_wide(path);
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        let ret = GetNamedSecurityInfoW(
            path_wide.as_ptr(), SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(),
            &mut sd,
        );
        if ret != 0 {
            return Err(format!("GetNamedSecurityInfoW failed: err={}", ret));
        }

        // Extract owner SID
        let mut owner_sid: PSID = std::ptr::null_mut();
        let mut owner_defaulted: i32 = 0;
        GetSecurityDescriptorOwner(sd, &mut owner_sid, &mut owner_defaulted);

        // Extract DACL
        let mut dacl_present: i32 = 0;
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let mut dacl_defaulted: i32 = 0;
        GetSecurityDescriptorDacl(sd, &mut dacl_present, &mut dacl, &mut dacl_defaulted);

        // Extract owner SID bytes
        let owner_bytes = if !owner_sid.is_null() {
            // Get SID length from sub authority count
            let _revision = *(owner_sid as *const u8);
            let sub_count = *((owner_sid as *const u8).offset(1));
            let len = 8 + (sub_count as usize) * 4;
            std::slice::from_raw_parts(owner_sid as *const u8, len).to_vec()
        } else {
            Vec::new()
        };

        // Extract DACL bytes
        let dacl_bytes = if dacl_present != 0 && !dacl.is_null() {
            let dacl_size = (*dacl).AclSize as usize;
            std::slice::from_raw_parts(dacl as *const u8, dacl_size).to_vec()
        } else {
            Vec::new()
        };

        LocalFree(sd as *mut _);
        Ok((owner_bytes, dacl_bytes))
    }
}

/// Save current DACL to backup file before locking.
fn save_dacl_backup(path: &str) -> Result<(), String> {
    let (owner_bytes, dacl_bytes) = read_current_dacl(path)?;
    let dir = backup_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create backup dir: {}", e))?;
    let bp = backup_path_for(path);
    // Format: [4] owner_len (u32 LE) | [owner_len] owner_sid | [rest] dacl_bytes
    let mut data = Vec::with_capacity(4 + owner_bytes.len() + dacl_bytes.len());
    data.extend_from_slice(&(owner_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(&owner_bytes);
    data.extend_from_slice(&dacl_bytes);
    std::fs::write(&bp, &data).map_err(|e| format!("Cannot write DACL backup: {}", e))
}

/// Restore DACL from backup file.
fn restore_dacl_from_backup(path: &str) -> Result<(), String> {
    let bp = backup_path_for(path);
    if !bp.exists() {
        return Err(format!("No DACL backup found for {}", path));
    }
    let data = std::fs::read(&bp).map_err(|e| format!("Cannot read DACL backup: {}", e))?;
    if data.len() < 4 {
        return Err("DACL backup too short".to_string());
    }
    let owner_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + owner_len {
        return Err("DACL backup truncated".to_string());
    }
    let owner_bytes = &data[4..4 + owner_len];
    let dacl_bytes = &data[4 + owner_len..];

    unsafe {
        let path_wide = to_wide(path);
        let _ = enable_privilege("SeTakeOwnershipPrivilege");
        let _ = enable_privilege("SeRestorePrivilege");

        // Restore owner if we have it
        if !owner_bytes.is_empty() {
            let owner_sid = owner_bytes.as_ptr() as PSID;
            SetNamedSecurityInfoW(
                path_wide.as_ptr(), SE_FILE_OBJECT, OWNER_SECURITY_INFORMATION,
                owner_sid, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(),
            );
        }

        // Restore DACL
        if !dacl_bytes.is_empty() {
            let dacl_ptr = dacl_bytes.as_ptr() as *mut ACL;
            let ret = SetNamedSecurityInfoW(
                path_wide.as_ptr(), SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(), std::ptr::null_mut(), dacl_ptr, std::ptr::null_mut(),
            );
            if ret != 0 {
                return Err(format!("SetNamedSecurityInfoW (restore DACL) failed: err={}", ret));
            }
        }
    }

    // Clean up backup
    let _ = std::fs::remove_file(&bp);
    Ok(())
}

/// Safe ACL lock: add a DENY ACE for the current user.
/// Does NOT change ownership. Does NOT replace existing DACL.
/// Saves a backup so the original DACL can always be restored.
pub fn apply_lock(path: &str) -> Result<(), String> {
    if !std::path::Path::new(path).exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // If backup already exists, verify the DENY ACE is still present.
    // If someone removed it (e.g. via icacls), re-apply.
    if backup_path_for(path).exists() {
        if is_deny_ace_present(path)? {
            return Ok(());
        }
        // Backup exists but DENY ACE is missing — re-apply below
    }

    // Step 1: Save current DACL to backup BEFORE modifying anything
    save_dacl_backup(path)?;

    unsafe {
        let path_wide = to_wide(path);

        // Step 2: Get current user SID
        let buf = current_user_sid_buf()?;
        let token_user = &*(buf.as_ptr() as *const TOKEN_USER);

        // Step 3: Create a DENY ACE for the current user (GENERIC_ALL, inherits to object and children)
        let mut deny_ea: EXPLICIT_ACCESS_W = std::mem::zeroed();
        deny_ea.grfAccessPermissions = GENERIC_ALL;
        deny_ea.grfAccessMode = DENY_ACCESS;
        deny_ea.grfInheritance = SUB_CONTAINERS_AND_OBJECTS_INHERIT;
        deny_ea.Trustee.TrusteeForm = TRUSTEE_IS_SID;
        deny_ea.Trustee.TrusteeType = TRUSTEE_IS_USER;
        deny_ea.Trustee.ptstrName = token_user.User.Sid as *mut u16;

        // Step 4: Merge the DENY ACE into the existing DACL
        let mut existing_dacl: *mut ACL = std::ptr::null_mut();
        let mut dacl_present: i32 = 0;
        let mut dacl_defaulted: i32 = 0;
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        let ret = GetNamedSecurityInfoW(
            path_wide.as_ptr(), SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(),
            &mut sd,
        );
        if ret != 0 {
            return Err(format!("GetNamedSecurityInfoW failed: err={}", ret));
        }
        if !sd.is_null() {
            GetSecurityDescriptorDacl(sd, &mut dacl_present, &mut existing_dacl, &mut dacl_defaulted);
        }

        let mut new_dacl: *mut ACL = std::ptr::null_mut();
        let acl_ref = if dacl_present != 0 && !existing_dacl.is_null() {
            existing_dacl as *const ACL
        } else {
            std::ptr::null()
        };
        let ret = SetEntriesInAclW(1, &mut deny_ea, acl_ref, &mut new_dacl);
        if !sd.is_null() { LocalFree(sd as *mut _); }
        if ret != 0 {
            return Err(format!("SetEntriesInAclW (merge deny) failed: {}", ret));
        }

        // Step 5: Apply the merged DACL
        let ret = SetNamedSecurityInfoW(
            path_wide.as_ptr(), SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(), std::ptr::null_mut(), new_dacl, std::ptr::null_mut(),
        );
        LocalFree(new_dacl as *mut _);

        if ret != 0 {
            // Rollback: restore original DACL
            let _ = restore_dacl_from_backup(path);
            return Err(format!("SetNamedSecurityInfoW (apply deny) failed: err={}", ret));
        }
    }

    Ok(())
}

/// Check if a DENY ACE for the current user is present in the DACL.
fn is_deny_ace_present(path: &str) -> Result<bool, String> {
    unsafe {
        let path_wide = to_wide(path);
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        let ret = GetNamedSecurityInfoW(
            path_wide.as_ptr(), SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(),
            &mut sd,
        );
        if ret != 0 || sd.is_null() {
            return Ok(false);
        }

        let mut dacl_present: i32 = 0;
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let mut dacl_defaulted: i32 = 0;
        GetSecurityDescriptorDacl(sd, &mut dacl_present, &mut dacl, &mut dacl_defaulted);

        if dacl_present == 0 || dacl.is_null() {
            LocalFree(sd as *mut _);
            return Ok(false);
        }

        // Get current user SID
        let buf = current_user_sid_buf()?;
        let token_user = &*(buf.as_ptr() as *const TOKEN_USER);
        let user_sid = token_user.User.Sid;

        // Walk the ACL looking for a DENY ACE matching the current user
        let ace_count = (*dacl).AceCount;
        let ace_ptr = (dacl as *const u8).offset(std::mem::size_of::<ACL>() as isize) as *const ACE_HEADER;
        let mut found = false;
        for i in 0..ace_count {
            let ace = &*ace_ptr.offset(i as isize);
            if ace.AceType == 0x01 { // ACCESS_DENIED_ACE_TYPE
                // ACCESS_ALLOWED_ACE / ACCESS_DENIED_ACE have the same layout after AceHeader
                let sid_ptr = (ace as *const ACE_HEADER).offset(std::mem::size_of::<ACE_HEADER>() as isize + 4) as *const u8;
                let sid_len = *((ace as *const ACE_HEADER).offset(1) as *const u16);
                // Get the sub authority count from the SID
                let sub_count = *sid_ptr.offset(1);
                let total_len = 8 + (sub_count as usize) * 4;
                if total_len == sid_len as usize && EqualSid(sid_ptr as PSID, user_sid) != 0 {
                    found = true;
                    break;
                }
            }
        }

        LocalFree(sd as *mut _);
        Ok(found)
    }
}

/// Safe ACL unlock: restore original DACL from backup.
pub fn remove_lock(path: &str) -> Result<(), String> {
    if !std::path::Path::new(path).exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let bp = backup_path_for(path);
    if bp.exists() {
        restore_dacl_from_backup(path)?;
    }

    // Recursively reset ACLs on child files/folders if it's a directory
    let p = std::path::Path::new(path);
    if p.is_dir() {
        remove_children_recursive(p).ok();
    }

    Ok(())
}

fn remove_children_recursive(dir: &std::path::Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    unsafe { remove_files_recursive(dir) }
}

unsafe fn remove_files_recursive(dir: &std::path::Path) -> Result<(), String> {
    use windows_sys::Win32::Storage::FileSystem::{
        FindFirstFileW, FindNextFileW, FindClose, WIN32_FIND_DATAW,
        FILE_ATTRIBUTE_DIRECTORY,
    };

    let search_wide = format!(r"{}\*", dir.to_string_lossy());
    let search_w: Vec<u16> = to_wide(&search_wide);

    let mut find_data: WIN32_FIND_DATAW = std::mem::zeroed();
    let handle = FindFirstFileW(search_w.as_ptr(), &mut find_data);

    if handle == INVALID_HANDLE_VALUE {
        let entries = std::fs::read_dir(dir).map_err(|e| format!("Cannot read directory: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Error reading entry: {}", e))?;
            let child = entry.path();
            let path_str = child.to_string_lossy().to_string();
            if child.is_dir() {
                remove_lock(&path_str).ok();
            } else if child.is_file() {
                remove_lock(&path_str).ok();
            }
        }
        return Ok(());
    }

    loop {
        let name = String::from_utf16_lossy(
            &find_data.cFileName[..find_data.cFileName.iter().position(|&c| c == 0).unwrap_or(find_data.cFileName.len())]
        );

        if name != "." && name != ".." {
            let child_path = dir.join(&name);
            let attrs = find_data.dwFileAttributes;
            let path_str = child_path.to_string_lossy().to_string();

            if attrs & FILE_ATTRIBUTE_DIRECTORY != 0 {
                remove_lock(&path_str).ok();
            } else {
                remove_lock(&path_str).ok();
            }
        }

        let mut next_data: WIN32_FIND_DATAW = std::mem::zeroed();
        if FindNextFileW(handle, &mut next_data) == 0 {
            break;
        }
        find_data = next_data;
    }

    FindClose(handle);
    Ok(())
}

/// Check if a path is locked (has a DACL backup).
pub fn verify_lock(path: &str) -> bool {
    backup_path_for(path).exists()
}

pub fn lock_drive(drive_letter: &str) -> Result<(), String> {
    let root = format!("{}:\\", drive_letter);
    apply_lock(&root)
}

pub fn unlock_drive(drive_letter: &str) -> Result<(), String> {
    let root = format!("{}:\\", drive_letter);
    remove_lock(&root)
}
