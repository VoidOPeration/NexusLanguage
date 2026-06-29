// Убрали asm, так как он требует nightly
// pub use std::arch::asm; 

#[no_mangle]
pub extern "C" fn nx_advance_write_mem(addr: usize, val: u8) {
    unsafe { std::ptr::write_volatile(addr as *mut u8, val); }
}