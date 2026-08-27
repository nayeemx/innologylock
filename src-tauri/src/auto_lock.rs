use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, Once, OnceLock};

static AUTO_LOCK_ENABLED: AtomicBool = AtomicBool::new(false);
static AUTO_LOCK_MINUTES: AtomicU32 = AtomicU32::new(5);
static RUNNING: AtomicBool = AtomicBool::new(false);
static ONCE: Once = Once::new();

/// When true, locking the Windows screen (Win+L) triggers InnologyLock to re-lock
/// all temporarily unlocked items.
static LOCK_ON_SCREEN_LOCK: AtomicBool = AtomicBool::new(false);

#[repr(C)]
struct LASTINPUTINFO {
    cb_size: u32,
    dw_time: u32,
}

pub fn set_auto_lock_minutes(minutes: u32) {
    AUTO_LOCK_MINUTES.store(minutes, Ordering::SeqCst);
    AUTO_LOCK_ENABLED.store(minutes > 0, Ordering::SeqCst);
}

pub fn get_auto_lock_minutes() -> u32 {
    AUTO_LOCK_MINUTES.load(Ordering::SeqCst)
}

pub fn set_lock_on_screen_lock(enabled: bool) {
    LOCK_ON_SCREEN_LOCK.store(enabled, Ordering::SeqCst);
}

pub fn get_lock_on_screen_lock() -> bool {
    LOCK_ON_SCREEN_LOCK.load(Ordering::SeqCst)
}

// Callback invoked on idle lock trigger (used to re-lock folders before workstation lock)
static ON_IDLE_LOCK: OnceLock<Mutex<Option<Box<dyn Fn() + Send>>>> = OnceLock::new();

pub fn set_on_idle_lock_callback(cb: Box<dyn Fn() + Send>) {
    if let Ok(mut guard) = ON_IDLE_LOCK.get_or_init(|| Mutex::new(None)).lock() {
        *guard = Some(cb);
    }
}

pub fn start_auto_lock_monitor() {
    ONCE.call_once(|| {
        std::thread::spawn(|| {
            RUNNING.store(true, Ordering::SeqCst);
            loop {
                if !RUNNING.load(Ordering::SeqCst) {
                    break;
                }

                if AUTO_LOCK_ENABLED.load(Ordering::SeqCst) {
                    let minutes = AUTO_LOCK_MINUTES.load(Ordering::SeqCst);
                    if minutes == 0 {
                        // Immediate mode - skip sleep
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        continue;
                    }

                    let idle_secs = get_idle_time_secs();
                    let threshold = minutes as u64 * 60;

                    if idle_secs >= threshold {
                        do_lock_workstation();
                        // Reset after locking
                        AUTO_LOCK_ENABLED.store(false, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_secs(60));
                        AUTO_LOCK_ENABLED.store(minutes > 0, Ordering::SeqCst);
                    }
                }

                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });

        // Start screen lock listener thread
        start_screen_lock_listener();
    });
}

/// Start a hidden window that listens for WM_WTSSESSION_CHANGE messages.
/// When Windows locks the screen (SESSION_LOCK), we re-lock all temp-unlocked items.
fn start_screen_lock_listener() {
    std::thread::spawn(|| {
        unsafe {
            use std::ffi::c_void;

            extern "system" {
                fn GetModuleHandleA(name: *const u8) -> isize;
                fn GetProcAddress(module: isize, name: *const u8) -> usize;
                fn RegisterClassExA(cls: *const u8) -> u16;
                fn CreateWindowExA(
                    ex_style: u32, class: *const u8, title: *const u8,
                    style: u32, x: i32, y: i32, w: i32, h: i32,
                    parent: isize, menu: isize, instance: isize, param: *mut c_void,
                ) -> isize;
                fn DefWindowProcA(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
                fn GetMessageA(msg: *mut u8, hwnd: isize, min: u32, max: u32) -> i32;
                fn TranslateMessage(msg: *const u8) -> i32;
                fn DispatchMessageA(msg: *const u8) -> isize;
            }

            const WS_OVERLAPPED: u32 = 0;
            const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
            const WM_WTSSESSION_CHANGE: u32 = 0x02B1;
            const SESSION_LOCK: usize = 0x7;

            #[repr(C)]
            struct WNDCLASSEXA {
                cb_size: u32,
                style: u32,
                lpfn_wnd_proc: usize,
                cb_cls_extra: i32,
                cb_wnd_extra: i32,
                h_instance: isize,
                h_icon: isize,
                h_cursor: isize,
                hbr_background: isize,
                lpsz_menu_name: *const u8,
                lpsz_class_name: *const u8,
                h_icon_sm: isize,
            }

            unsafe extern "system" fn wnd_proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
                if msg == WM_WTSSESSION_CHANGE {
                    if wparam == SESSION_LOCK {
                        crate::logger::log("SCREEN_LOCK", "Windows screen locked - re-locking items");
                        crate::process_guard::relock_all_unlocked_folders();
                    }
                }
                DefWindowProcA(hwnd, msg, wparam, lparam)
            }

            let class_name = b"InnologyLockScreenLock\0";
            let mut cls = WNDCLASSEXA {
                cb_size: std::mem::size_of::<WNDCLASSEXA>() as u32,
                style: 0,
                lpfn_wnd_proc: wnd_proc as *const () as usize,
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: GetModuleHandleA(0 as *const u8),
                h_icon: 0,
                h_cursor: 0,
                hbr_background: 0,
                lpsz_menu_name: std::ptr::null(),
                lpsz_class_name: class_name.as_ptr(),
                h_icon_sm: 0,
            };

            RegisterClassExA(&mut cls as *mut _ as *const u8);

            let hwnd = CreateWindowExA(
                0, class_name.as_ptr(), b"InnologyLock\0".as_ptr(),
                WS_OVERLAPPED, CW_USEDEFAULT, CW_USEDEFAULT, 0, 0,
                0, 0, GetModuleHandleA(0 as *const u8), std::ptr::null_mut(),
            );

            if hwnd == 0 { return; }

            // Use WTSRegisterSessionNotification if available
            type WtsRegisterSessionNotificationFn = unsafe extern "system" fn(isize, u32) -> i32;
            let wts_lib = GetModuleHandleA(b"wtsapi32.dll\0".as_ptr());
            if wts_lib != 0 {
                let addr = GetProcAddress(wts_lib, b"WTSRegisterSessionNotification\0".as_ptr());
                if addr != 0 {
                    let register: WtsRegisterSessionNotificationFn = std::mem::transmute(addr);
                    register(hwnd, 0); // NOTIFY_FOR_THIS_SESSION = 0
                }
            }

            // Message loop
            let mut msg = [0u8; 48];
            loop {
                let ret = GetMessageA(&mut msg as *mut _ as *mut u8, 0, 0, 0);
                if ret == 0 || ret == -1 {
                    break;
                }
                TranslateMessage(&msg as *const _ as *const u8);
                DispatchMessageA(&msg as *const _ as *const u8);
            }
        }
    });
}

fn get_idle_time_secs() -> u64 {
    unsafe {
        extern "system" {
            fn GetModuleHandleA(name: *const u8) -> isize;
            fn GetProcAddress(module: isize, name: *const u8) -> usize;
            fn GetTickCount() -> u32;
        }

        let user32 = GetModuleHandleA(b"user32.dll\0".as_ptr());
        if user32 == 0 { return 0; }

        let addr = GetProcAddress(user32, b"GetLastInputInfo\0".as_ptr());
        if addr == 0 { return 0; }

        type GetLastInputInfoFn = unsafe extern "system" fn(*mut LASTINPUTINFO) -> i32;
        let get_last_input: GetLastInputInfoFn = std::mem::transmute(addr);

        let mut info = LASTINPUTINFO {
            cb_size: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dw_time: 0,
        };

        if get_last_input(&mut info) == 0 {
            return 0;
        }

        let tick = GetTickCount();
        let elapsed = tick.wrapping_sub(info.dw_time);
        (elapsed / 1000) as u64
    }
}

fn do_idle_relock() {
    if let Ok(guard) = ON_IDLE_LOCK.get_or_init(|| Mutex::new(None)).lock() {
        if let Some(ref cb) = *guard {
            cb();
        }
    }
}

fn do_lock_workstation() {
    do_idle_relock();
    unsafe {
        extern "system" {
            fn GetModuleHandleA(name: *const u8) -> isize;
            fn GetProcAddress(module: isize, name: *const u8) -> usize;
        }

        let user32 = GetModuleHandleA(b"user32.dll\0".as_ptr());
        if user32 == 0 { return; }

        let addr = GetProcAddress(user32, b"LockWorkStation\0".as_ptr());
        if addr == 0 { return; }

        type LockWorkStationFn = unsafe extern "system" fn() -> i32;
        let lock_fn: LockWorkStationFn = std::mem::transmute(addr);
        lock_fn();
    }
}

pub fn stop_auto_lock_monitor() {
    RUNNING.store(false, Ordering::SeqCst);
}
