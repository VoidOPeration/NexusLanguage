use std::io;

/// Reads a line from stdin into a buffer.
/// Returns length read.
#[no_mangle]
pub extern "C" fn nx_scanner_read_line(buf: *mut u8, max_len: usize) -> usize {
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() { return 0; }
    
    let bytes = input.trim().as_bytes();
    let len = bytes.len().min(max_len);
    
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
    }
    len
}