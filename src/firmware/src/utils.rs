#[cfg(target_arch = "xtensa")]
pub fn get_free_heap_size() -> u32 {
    unsafe { esp_idf_sys::esp_get_free_heap_size() }
}

#[cfg(not(target_arch = "xtensa"))]
pub fn get_free_heap_size() -> u32 {
    1024 * 1024 // Mock 1MB free heap for testing
}

#[cfg(target_arch = "xtensa")]
pub fn get_timer_ms() -> i64 {
    unsafe { esp_idf_sys::esp_timer_get_time() / 1000 }
}

#[cfg(not(target_arch = "xtensa"))]
pub fn get_timer_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
