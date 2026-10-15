use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::state::LockedItemsState;

static FILTER_STOP: AtomicBool = AtomicBool::new(false);

pub fn signal_stop() { FILTER_STOP.store(true, Ordering::Relaxed); }
pub fn is_stopping() -> bool { FILTER_STOP.load(Ordering::Relaxed) }

fn dos_to_nt_path(dos_path: &str) -> String {
    if dos_path.len() < 2 { return dos_path.to_string(); }
    let drive = dos_path.chars().next().unwrap_or('C');
    let rest = &dos_path[2..];
    format!("\\??\\{}:{}", drive, rest)
}

unsafe fn open_filter_device() -> Result<windows_sys::Win32::Foundation::HANDLE, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{CreateFileW, OPEN_EXISTING};
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;

    let device_wide: Vec<u16> = OsStr::new(r"\\.\InnLockDev")
        .encode_wide().chain(std::iter::once(0)).collect();

    let handle = CreateFileW(
        device_wide.as_ptr(),
        0x80000000 | 0x40000000, // GENERIC_READ | GENERIC_WRITE
        3, // FILE_SHARE_READ | FILE_SHARE_WRITE
        std::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        std::ptr::null_mut() as _,
    );
    if handle == INVALID_HANDLE_VALUE {
        return Err(format!("Cannot open filter device: {}", std::io::Error::last_os_error()));
    }
    Ok(handle)
}

fn send_ioctl(ioctl: u32, data: Option<&[u8]>) -> Result<(), String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::IO::DeviceIoControl;

    unsafe {
        let handle = open_filter_device()?;
        let mut bytes_returned: u32 = 0;
        let input_ptr = data.map(|d| d.as_ptr() as *const std::ffi::c_void).unwrap_or(std::ptr::null());
        let input_len = data.map(|d| d.len() as u32).unwrap_or(0);

        let ok = DeviceIoControl(
            handle, ioctl,
            input_ptr, input_len,
            std::ptr::null_mut(), 0,
            &mut bytes_returned, std::ptr::null_mut(),
        );
        CloseHandle(handle);
        if ok == 0 {
            return Err(format!("DeviceIoControl failed: {}", std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

pub fn add_filter_path(path: &str) -> Result<(), String> {
    let nt_path = dos_to_nt_path(path);
    let wide: Vec<u16> = nt_path.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = unsafe { std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2) };
    send_ioctl(0x80002000, Some(bytes))
}

pub fn remove_filter_path(path: &str) -> Result<(), String> {
    let nt_path = dos_to_nt_path(path);
    let wide: Vec<u16> = nt_path.encode_utf16().chain(std::iter::once(0)).collect();
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

pub fn filter_bridge_thread(_locked: Arc<Mutex<LockedItemsState>>) {
    log("Filter bridge thread started");
    loop {
        if is_stopping() { break; }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    log("Filter bridge thread stopped");
}

fn log(msg: &str) {
    use std::io::Write;
    let _ = std::fs::OpenOptions::new()
        .create(true).append(true)
        .open(r"C:\ProgramData\InnologyBD\InnologyLock\svc.log")
        .and_then(|mut f| writeln!(f, "[FILTER] {}", msg));
}
