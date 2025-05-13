use std::os::raw::{c_void, c_uint};

pub struct Cache {
    // TODO: Felder
    capacity: usize,
}

#[no_mangle]
pub extern "C" fn cache_init(capacity: c_uint) -> *mut Cache {
    let cache = Cache { capacity: capacity as usize };
    Box::into_raw(Box::new(cache)) as *mut Cache
}

/// releases the memory of the cache
#[no_mangle]
pub extern "C" fn cache_destroy(ptr: *mut Cache) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

/// example function
#[no_mangle]
pub extern "C" fn cache_example(ptr: *mut Cache, key: c_uint, value: c_uint) {
    let cache = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    // TODO: implement actual functionality
    println!(
        "Setting key={} to value={} in cache with capacity={}",
        key, value, cache.capacity
    );
}
