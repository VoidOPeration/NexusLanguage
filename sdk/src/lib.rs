pub mod system;
pub mod scanner;
pub mod iostream;
pub mod thread;
pub mod fstream;
pub mod path;
pub mod streams;
pub mod types;
pub mod advance;
pub mod net;
pub mod utils;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub unsafe fn c_str_to_rust(ptr: *const c_char) -> &'static str {
    if ptr.is_null() { return ""; }
    CStr::from_ptr(ptr).to_str().unwrap_or("")
}

pub fn rust_str_to_c(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn nx_alloc(size: usize) -> *mut u8 {
    unsafe { std::alloc::alloc(std::alloc::Layout::from_size_align(size, 8).unwrap()) }
}

#[no_mangle]
pub extern "C" fn nx_free(ptr: *mut u8, size: usize) {
    unsafe { std::alloc::dealloc(ptr, std::alloc::Layout::from_size_align(size, 8).unwrap()) }
}