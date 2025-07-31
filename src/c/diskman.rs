use std::ffi::c_char;

unsafe extern "C" {
    pub fn du(path: *const c_char, fsym: bool) -> u64;
}
