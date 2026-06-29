use std::collections::HashMap;

#[allow(dead_code)]
pub struct NxErrorMap {
    map: HashMap<String, String>
}

#[no_mangle]
pub extern "C" fn nx_errormap_new() -> *mut NxErrorMap {
    Box::into_raw(Box::new(NxErrorMap { map: HashMap::new() }))
}