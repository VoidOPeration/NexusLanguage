use std::vec::Vec;

type NxIntList = Vec<i32>;

pub extern "C" fn nx_arraylist_int_new() -> *mut NxIntList {
    Box::into_raw(Box::new(Vec::new()))
}

pub extern "C" fn nx_arraylist_int_push(list: *mut NxIntList, val: i32) {
    unsafe {
        (*list).push(val);
    }
}

pub extern "C" fn nx_arraylist_int_get(list: *const NxIntList, index: usize) -> i32 {
    unsafe {
        match (&*list).get(index) {
            Some(v) => *v,
            None => 0,
        }
    }
}

pub extern "C" fn nx_arraylist_int_free(list: *mut NxIntList) {
    unsafe {
        drop(Box::from_raw(list));
    }
}