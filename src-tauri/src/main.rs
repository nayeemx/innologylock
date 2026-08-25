#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Context menu: right-click → "Lock with OmniLock" or "Unlock with OmniLock"
    // Writes action to a file in the vault dir; the running app picks it up.
    if let Some(pos) = args.iter().position(|a| a == "--context-menu-lock") {
        if let Some(path) = args.get(pos + 1) {
            let result = omnilock::shell_context::dispatch_context_action("lock", path);
            match &result {
                Ok(msg) => eprintln!("[omnilock] {}", msg),
                Err(e) => eprintln!("[omnilock] error: {}", e),
            }
            return;
        }
    }

    if let Some(pos) = args.iter().position(|a| a == "--context-menu-unlock") {
        if let Some(path) = args.get(pos + 1) {
            let result = omnilock::shell_context::dispatch_context_action("unlock", path);
            match &result {
                Ok(msg) => eprintln!("[omnilock] {}", msg),
                Err(e) => eprintln!("[omnilock] error: {}", e),
            }
            return;
        }
    }

    omnilock::run();
}
