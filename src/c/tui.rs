use std::ffi::{c_char, CString};

unsafe extern "C" {
    pub fn initTUI();
    pub fn termTUI();
    pub fn runTUI() -> *const c_char;
    pub fn setHostnames(hostnames: *mut *const c_char, count: i32);
}

pub fn update_tui_hostnames(hostnames: &Vec<String>) {
    let c_strings: Vec<CString> = hostnames
        .iter()
        .map(|s| CString::new(s.as_str()).expect("CString::new failed"))
        .collect();
    let mut pointers: Vec<*const c_char> = c_strings.iter().map(|cs| cs.as_ptr()).collect();
    unsafe {
        setHostnames(pointers.as_mut_ptr(), pointers.len() as i32);
    }
}
