#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Context menu: right-click â†’ "Lock with InnologyLock" or "Unlock with InnologyLock"
    // Writes action to a file in the vault dir; the running app picks it up.
    if let Some(pos) = args.iter().position(|a| a == "--context-menu-lock") {
        if let Some(path) = args.get(pos + 1) {
            let result = innologylock::shell_context::dispatch_context_action("lock", path);
            match &result {
                Ok(msg) => eprintln!("[InnologyLock] {}", msg),
                Err(e) => eprintln!("[InnologyLock] error: {}", e),
            }
            return;
        }
    }

    if let Some(pos) = args.iter().position(|a| a == "--context-menu-unlock") {
        if let Some(path) = args.get(pos + 1) {
            let result = innologylock::shell_context::dispatch_context_action("unlock", path);
            match &result {
                Ok(msg) => eprintln!("[InnologyLock] {}", msg),
                Err(e) => eprintln!("[InnologyLock] error: {}", e),
            }
            return;
        }
    }

    innologylock::run();
}
