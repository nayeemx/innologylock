use serde::{Deserialize, Serialize};

pub const PIPE_NAME: &str = r"\\.\pipe\InnologyLockService";
pub const FILTER_PIPE_NAME: &str = r"\\.\pipe\InnologyLockFilter";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedItem {
    pub item_type: String,
    pub path: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum SvcRequest {
    Ping,
    GetStatus,
    GetLockedItems,
    LockFile { path: String, display_name: String },
    LockFolder { path: String, display_name: String },
    LockDrive { drive_letter: String, display_name: String },
    LockApp { name: String, path: String, display_name: String },
    UnlockItem { path: String, password: String },
    ForceRemoveLockedItem { path: String },
    SyncVault { vault_data: Vec<u8> },
    // Driver communication
    DriverAccessRequest { request_id: u64, process_id: u32, process_name: String, file_path: String },
    DriverReply { request_id: u64, allow: bool },
    // Filter path management
    AddFilterPath { path: String },
    RemoveFilterPath { path: String },
    ClearFilterPaths,
    SetFilterEnabled { enabled: bool },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SvcResponse {
    Pong,
    Ok { message: String },
    Error { message: String },
    Status { running: bool, locked_count: usize },
    LockedItems(Vec<LockedItem>),
    // Driver events forwarded to Tauri app
    AccessRequest { request_id: u64, process_name: String, file_path: String },
}

// Filter access request (matching the C struct layout)
#[derive(Debug, Clone)]
pub struct FilterAccessRequest {
    pub request_id: u64,
    pub process_id: u32,
    pub process_name: [u16; 64],
    pub file_path: [u16; 512],
}

// Filter access reply
#[derive(Debug, Clone)]
pub struct FilterAccessReply {
    pub request_id: u64,
    pub allow: i32,
}
