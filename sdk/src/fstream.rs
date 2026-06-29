use std::fs::File;
use std::ffi::CStr;
// Убрали Read, Write

#[allow(dead_code)]
pub struct NxFile {
    file: File,
}

#[no_mangle]
pub extern "C" fn nx_fstream_open(path: *const i8, mode: *const i8) -> *mut NxFile {
    unsafe {
        let path = CStr::from_ptr(path).to_str().unwrap();
        let mode = CStr::from_ptr(mode).to_str().unwrap();
        
        let file = if mode == "w" { File::create(path).ok() }
                   else { File::open(path).ok() };
                   
        match file {
            Some(f) => Box::into_raw(Box::new(NxFile { file: f })),
            None => std::ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn nx_fstream_close(f: *mut NxFile) {
    unsafe { drop(Box::from_raw(f)); }
}