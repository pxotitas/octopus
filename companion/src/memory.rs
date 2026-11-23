use std::cell::Cell;
use std::time::Duration;

use gtk::glib;

const M_TRIM_THRESHOLD: i32 = -1;
const M_MMAP_THRESHOLD: i32 = -4;
const M_ARENA_MAX: i32 = -8;

extern "C" {
    fn mallopt(param: i32, value: i32) -> i32;
    fn malloc_trim(pad: usize) -> i32;
}

/// Keeps glibc from holding freed memory: one arena per two threads instead
/// of eight per core, and big buffers (textures, listings) straight to mmap.
pub fn tune() {
    unsafe {
        mallopt(M_ARENA_MAX, 2);
        mallopt(M_TRIM_THRESHOLD, 256 * 1024);
        mallopt(M_MMAP_THRESHOLD, 256 * 1024);
    }
}

thread_local! {
    static PENDING: Cell<bool> = const { Cell::new(false) };
}

/// Gives the freed heap back to the system a moment after a panel closes.
pub fn release_soon() {
    if PENDING.with(|p| p.replace(true)) {
        return;
    }
    glib::timeout_add_local_once(Duration::from_secs(2), || {
        PENDING.with(|p| p.set(false));
        unsafe {
            malloc_trim(0);
        }
    });
}
