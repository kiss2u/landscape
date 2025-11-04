use libc::{c_char, if_nametoindex};
use std::ffi::CString;

pub fn get_interface_index_by_name(iface_name: &str) -> Option<u32> {
    let c_iface_name = match CString::new(iface_name) {
        Ok(c_iface_name) => c_iface_name,
        Err(e) => {
            tracing::error!("Invalid interface name: {:?}", e);
            return None;
        }
    };

    let index = unsafe { if_nametoindex(c_iface_name.as_ptr() as *const c_char) };

    if index == 0 {
        tracing::error!("Interface '{}' not found", iface_name);
        None
    } else {
        Some(index)
    }
}
