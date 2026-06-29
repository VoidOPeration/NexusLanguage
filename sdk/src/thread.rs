use std::thread;

// Opaque handle to a thread
pub struct NxThread {
    handle: Option<thread::JoinHandle<()>>,
}

#[no_mangle]
pub extern "C" fn nx_thread_spawn(cb: extern "C" fn()) -> *mut NxThread {
    let handle = thread::spawn(move || {
        cb();
    });
    Box::into_raw(Box::new(NxThread { handle: Some(handle) }))
}

#[no_mangle]
pub extern "C" fn nx_thread_join(handle: *mut NxThread) {
    unsafe {
        if let Some(h) = (*handle).handle.take() {
            h.join().unwrap();
        }
    }
}