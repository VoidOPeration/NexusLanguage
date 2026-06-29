use std::os::raw::c_char;
use crate::{c_str_to_rust, rust_str_to_c};

/// Exits the program with code.
#[no_mangle]
pub extern "C" fn nx_system_exit(code: i32) {
    std::process::exit(code);
}

/// Gets environment variable.
#[no_mangle]
pub extern "C" fn nx_system_get_env(key: *const c_char) -> *mut c_char {
    unsafe {
        let key = c_str_to_rust(key);
        match std::env::var(key) {
            Ok(val) => rust_str_to_c(&val),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

/// Executes a shell command.
#[no_mangle]
pub extern "C" fn nx_system_exec(cmd: *const c_char) -> i32 {
    unsafe {
        let cmd = c_str_to_rust(cmd);
        // Basic implementation
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map(|s| s.code().unwrap_or(-1))
            .unwrap_or(-1)
    }
}