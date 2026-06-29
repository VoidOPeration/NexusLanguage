use std::collections::HashMap;
use std::ffi::CStr;
// Убрали CString

type NxHashMap = HashMap<String, String>;

#[no_mangle]
pub extern "C" fn nx_hashmap_new() -> *mut NxHashMap {
    Box::into_raw(Box::new(HashMap::new()))
}

#[no_mangle]
pub extern "C" fn nx_hashmap_insert(map: *mut NxHashMap, key: *const i8, val: *const i8) {
    unsafe {
        let map = &mut *map;
        let k = CStr::from_ptr(key).to_str().unwrap().to_string();
        let v = CStr::from_ptr(val).to_str().unwrap().to_string();
        map.insert(k, v);
    }
}