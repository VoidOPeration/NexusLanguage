use std::path::Path;
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn nx_path_exists(path: *const c_char) -> bool {
    unsafe { Path::new(CStr::from_ptr(path).to_str().unwrap()).exists() }
}

#[no_mangle]
pub extern "C" fn nx_path_is_dir(path: *const c_char) -> bool {
    unsafe { Path::new(CStr::from_ptr(path).to_str().unwrap()).is_dir() }
}