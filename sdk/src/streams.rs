use std::sync::mpsc;

#[allow(dead_code)]
pub struct NxChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

// Example: Int channel
#[no_mangle]
pub extern "C" fn nx_stream_create_int() -> *mut NxChannel<i32> {
    let (tx, rx) = mpsc::channel();
    Box::into_raw(Box::new(NxChannel { sender: tx, receiver: rx }))
}
// ... implementation for send/recv