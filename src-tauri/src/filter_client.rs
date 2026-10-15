use std::sync::{Arc, OnceLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use tauri::Emitter;

static FILTER_STOP: AtomicBool = AtomicBool::new(false);
pub fn signal_stop() { FILTER_STOP.store(true, Ordering::Relaxed); }

#[repr(C, packed(8))]
#[derive(Debug, Clone, Copy)]
pub struct FilterAccessRequest {
    pub request_id: u64,
    pub process_id: u32,
    pub process_name: [u16; 64],
    pub file_path: [u16; 512],
}

#[repr(C, packed(8))]
#[derive(Debug, Clone, Copy)]
pub struct FilterAccessReply {
    pub request_id: u64,
    pub allow: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FilterMessageHeader {
    message_id: u32,
    reserved: u32,
    reply_length: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FilterReplyHeader {
    message_id: u32,
    status: i32,
}

type FnConnect = unsafe extern "system" fn(*const u16, u32, *const std::ffi::c_void, u32, *mut std::ffi::c_void, *mut *mut std::ffi::c_void) -> i32;
type FnGetMsg = unsafe extern "system" fn(*mut std::ffi::c_void, *mut u8, u32, *mut u32) -> i32;
type FnReply = unsafe extern "system" fn(*mut std::ffi::c_void, *const FilterReplyHeader, u32) -> i32;
type FnClose = unsafe extern "system" fn(*mut std::ffi::c_void) -> i32;

#[derive(Clone)]
struct FltUserApis {
    connect: FnConnect,
    get_message: FnGetMsg,
    reply: FnReply,
    close: FnClose,
}

static FLT_APIS: OnceLock<Arc<Mutex<Option<FltUserApis>>>> = OnceLock::new();

unsafe fn load_fltuser_apis() -> Result<(), String> {
    let apis = FLT_APIS.get_or_init(|| Arc::new(Mutex::new(None)));
    {
        let guard = apis.lock().unwrap();
        if guard.is_some() { return Ok(()); }
    }
    let dll_name: Vec<u16> = "fltLib.dll\0".encode_utf16().collect();
    let h = windows_sys::Win32::System::LibraryLoader::LoadLibraryW(dll_name.as_ptr());
    if h.is_null() {
        return Err(format!("LoadLibraryW(fltLib.dll) failed: {}", std::io::Error::last_os_error()));
    }
    let gp = windows_sys::Win32::System::LibraryLoader::GetProcAddress;
    let connect: FnConnect = std::mem::transmute(gp(h, "FilterConnectCommunicationPort\0".as_ptr() as _));
    let get_msg: FnGetMsg = std::mem::transmute(gp(h, "FilterGetMessage\0".as_ptr() as _));
    let reply_fn: FnReply = std::mem::transmute(gp(h, "FilterReplyMessage\0".as_ptr() as _));
    let close_fn: FnClose = std::mem::transmute(gp(h, "CloseHandle\0".as_ptr() as _));
    *apis.lock().unwrap() = Some(FltUserApis { connect, get_message: get_msg, reply: reply_fn, close: close_fn });
    log("fltLib.dll loaded");
    Ok(())
}

pub struct PendingFilterRequest {
    pub request_id: u64,
    pub message_id: u32,
    pub process_id: u32,
    pub file_path: String,
    pub display_name: String,
    pub timestamp: std::time::Instant,
}

pub static PENDING_FILTER_REQUESTS: OnceLock<Arc<Mutex<Vec<PendingFilterRequest>>>> = OnceLock::new();

pub fn get_pending_requests() -> Arc<Mutex<Vec<PendingFilterRequest>>> {
    PENDING_FILTER_REQUESTS.get_or_init(|| Arc::new(Mutex::new(Vec::new()))).clone()
}

fn build_reply_buf(message_id: u32, request_id: u64, allow: bool) -> Vec<u8> {
    let reply = FilterAccessReply { request_id, allow: if allow { 1 } else { 0 } };
    let reply_hdr = FilterReplyHeader { message_id, status: 0 };
    let reply_size = std::mem::size_of::<FilterReplyHeader>() + std::mem::size_of::<FilterAccessReply>();
    let mut reply_buf = vec![0u8; reply_size];
    unsafe {
        std::ptr::copy_nonoverlapping(&reply_hdr, reply_buf.as_mut_ptr() as *mut FilterReplyHeader, 1);
        std::ptr::copy_nonoverlapping(&reply, reply_buf.as_mut_ptr().add(std::mem::size_of::<FilterReplyHeader>()) as *mut FilterAccessReply, 1);
    }
    reply_buf
}

pub fn start_filter_thread(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        log("Filter client thread starting...");
        if let Err(e) = unsafe { load_fltuser_apis() } {
            log(&format!("Failed to load fltuser: {}", e));
            return;
        }
        let apis = FLT_APIS.get().unwrap().lock().unwrap().clone().unwrap();
        let mut port: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut connected = false;
        let mut retries = 0;

        loop {
            if FILTER_STOP.load(Ordering::Relaxed) { return; }
            if !connected {
                let port_name: Vec<u16> = "\\InnologyLockFilterPort2\0".encode_utf16().collect();
                let status = unsafe { (apis.connect)(port_name.as_ptr(), 0, std::ptr::null(), 0, std::ptr::null_mut(), &mut port) };
                if status >= 0 {
                    connected = true;
                    retries = 0;
                    log("Connected to filter port");
                } else {
                    retries += 1;
                    if retries > 60 {
                        log(&format!("Giving up connecting to filter port after {} retries", retries));
                        return;
                    }
                    if retries % 10 == 1 {
                        log(&format!("Filter port connect retry {}", retries));
                    }
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    continue;
                }
            }
            let msg_size = std::mem::size_of::<FilterMessageHeader>() + std::mem::size_of::<FilterAccessRequest>();
            let mut buf = vec![0u8; msg_size];
            let mut bytes_returned: u32 = 0;
            let status = unsafe { (apis.get_message)(port, buf.as_mut_ptr(), msg_size as u32, &mut bytes_returned) };
            if status < 0 {
                log(&format!("FilterGetMessage failed: 0x{:08X}", status as u32));
                connected = false;
                unsafe { (apis.close)(port); }
                port = std::ptr::null_mut();
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            let msg_hdr = unsafe { &*(buf.as_ptr() as *const FilterMessageHeader) };
            let request = unsafe { *((buf.as_ptr() as usize + std::mem::size_of::<FilterMessageHeader>()) as *const FilterAccessRequest) };
            let path = unsafe {
                let len = request.file_path.iter().position(|&c| c == 0).unwrap_or(512);
                String::from_utf16_lossy(&request.file_path[..len])
            };
            let display = std::path::Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());

            log(&format!("Access blocked: msg_id={} req_id={} path={}", msg_hdr.message_id, request.request_id, path));

            let pending = PendingFilterRequest {
                request_id: request.request_id,
                message_id: msg_hdr.message_id,
                process_id: request.process_id,
                file_path: path.clone(),
                display_name: display.clone(),
                timestamp: std::time::Instant::now(),
            };
            {
                let mut queue = get_pending_requests();
                queue.lock().unwrap().push(pending);
            }

            let _ = app_handle.emit("filter-access-request", serde_json::json!({
                "request_id": request.request_id,
                "message_id": msg_hdr.message_id,
                "file_path": path,
                "display_name": display,
            }));
            if let Some(w) = app_handle.get_webview_window("widget") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    });
}

pub fn reply_to_driver(message_id: u32, request_id: u64, allow: bool) -> Result<(), String> {
    let apis_guard = FLT_APIS.get().ok_or("Filter APIs not loaded")?.lock().unwrap();
    let apis = apis_guard.as_ref().ok_or("Filter APIs not loaded")?;
    let port_name: Vec<u16> = "\\InnologyLockFilterPort2\0".encode_utf16().collect();
    let mut port: *mut std::ffi::c_void = std::ptr::null_mut();
    let status = unsafe { (apis.connect)(port_name.as_ptr(), 0, std::ptr::null(), 0, std::ptr::null_mut(), &mut port) };
    if status < 0 {
        return Err(format!("FilterConnect for reply failed: 0x{:08X}", status as u32));
    }
    let reply_buf = build_reply_buf(message_id, request_id, allow);
    let reply_status = unsafe { (apis.reply)(port, reply_buf.as_ptr() as *const FilterReplyHeader, reply_buf.len() as u32) };
    unsafe { (apis.close)(port); }
    if reply_status < 0 {
        return Err(format!("FilterReplyMessage failed: 0x{:08X}", reply_status as u32));
    }
    log(&format!("Reply sent: msg_id={} allow={}", message_id, allow));
    Ok(())
}

pub fn deny_all_pending() {
    let queue = get_pending_requests();
    let mut pending: Vec<PendingFilterRequest> = {
        let mut q = queue.lock().unwrap();
        std::mem::take(&mut *q)
    };
    for req in pending.drain(..) {
        log(&format!("Auto-deny: msg_id={} path={}", req.message_id, req.file_path));
        if let Err(e) = reply_to_driver(req.message_id, req.request_id, false) {
            log(&format!("Auto-deny reply failed: {}", e));
        }
    }
}

pub fn add_filter_path(path: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    let dos_path = if path.len() >= 2 && path.as_bytes()[1] == b':' {
        let drive = path.chars().next().unwrap_or('C');
        format!("\\??\\{}:{}", drive, &path[2..])
    } else {
        path.to_string()
    };
    let wide: Vec<u16> = std::ffi::OsStr::new(&dos_path).encode_wide().chain(std::iter::once(0)).collect();
    let bytes = unsafe { std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2) };
    send_ioctl(0x80002000, Some(bytes))
}

pub fn remove_filter_path(path: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    let dos_path = if path.len() >= 2 && path.as_bytes()[1] == b':' {
        let drive = path.chars().next().unwrap_or('C');
        format!("\\??\\{}:{}", drive, &path[2..])
    } else {
        path.to_string()
    };
    let wide: Vec<u16> = std::ffi::OsStr::new(&dos_path).encode_wide().chain(std::iter::once(0)).collect();
    let bytes = unsafe { std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2) };
    send_ioctl(0x80002004, Some(bytes))
}

pub fn clear_filter_paths() -> Result<(), String> {
    send_ioctl(0x80002008, None)
}

pub fn set_filter_enabled(enabled: bool) -> Result<(), String> {
    let val: i32 = if enabled { 1 } else { 0 };
    let bytes = unsafe { std::slice::from_raw_parts(&val as *const i32 as *const u8, 4) };
    send_ioctl(0x8000200C, Some(bytes))
}

fn send_ioctl(ioctl: u32, data: Option<&[u8]>) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    unsafe {
        let device_wide: Vec<u16> = std::ffi::OsStr::new(r"\\.\InnLockDev")
            .encode_wide().chain(std::iter::once(0)).collect();
        let handle = windows_sys::Win32::Storage::FileSystem::CreateFileW(
            device_wide.as_ptr(),
            0x80000000 | 0x40000000,
            3,
            std::ptr::null_mut(),
            windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING,
            0,
            std::ptr::null_mut() as _,
        );
        if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(format!("Cannot open filter device: {}", std::io::Error::last_os_error()));
        }
        let mut bytes_returned: u32 = 0;
        let input_ptr = data.map(|d| d.as_ptr() as *const std::ffi::c_void).unwrap_or(std::ptr::null());
        let input_len = data.map(|d| d.len() as u32).unwrap_or(0);
        let ok = windows_sys::Win32::System::IO::DeviceIoControl(
            handle, ioctl,
            input_ptr, input_len,
            std::ptr::null_mut(), 0,
            &mut bytes_returned, std::ptr::null_mut(),
        );
        windows_sys::Win32::Foundation::CloseHandle(handle);
        if ok == 0 {
            return Err(format!("DeviceIoControl failed: {}", std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

fn log(msg: &str) {
    use std::io::Write;
    let _ = std::fs::OpenOptions::new()
        .create(true).append(true)
        .open(r"C:\ProgramData\InnologyBD\InnologyLock\app.log")
        .and_then(|mut f| writeln!(f, "[FILTER] {}", msg));
}
