use std::io::{self, Write};
// Убрали неиспользуемый c_str_to_rust

pub extern "C" fn nx_iostream_print(ptr: *const i8, len: usize) {
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    io::stdout().write_all(slice).unwrap();
}

pub extern "C" fn nx_iostream_println(ptr: *const i8, len: usize) {
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    io::stdout().write_all(slice).unwrap();
    io::stdout().write_all(b"\n").unwrap();
}