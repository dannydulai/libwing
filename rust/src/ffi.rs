use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_float, c_void};
use std::slice;
use std::ptr;
use crate::{WingConsole, NodeDefinition, NodeData, NodeType, NodeUnit};

// Opaque type wrappers
#[repr(C)]
pub struct WingDiscoverHandle {
    info: Vec<crate::DiscoveryInfo>
}

#[repr(C)]
pub struct WingConsoleHandle {
    console: WingConsole,
    request_end_cb: Option<extern "C" fn(*mut c_void)>,
    request_end_data: *mut c_void,
    node_def_cb: Option<extern "C" fn(*mut NodeDefinitionHandle, *mut c_void)>,
    node_def_data: *mut c_void,
    node_data_cb: Option<extern "C" fn(u32, *mut NodeDataHandle, *mut c_void)>,
    node_data_data: *mut c_void,
}

#[repr(C)]
pub struct NodeDefinitionHandle {
    def: NodeDefinition
}

#[repr(C)]
pub struct NodeDataHandle {
    data: NodeData
}

#[no_mangle]
pub extern "C" fn wing_discover_scan(stop_on_first: c_int) -> *mut WingDiscoverHandle {
    let results = WingConsole::scan(stop_on_first != 0);
    let handle = Box::new(WingDiscoverHandle { info: results });
    Box::into_raw(handle)
}

#[no_mangle]
pub extern "C" fn wing_discover_destroy(handle: *mut WingDiscoverHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_count(handle: *const WingDiscoverHandle) -> c_int {
    unsafe {
        (*handle).info.len() as c_int
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_ip(handle: *const WingDiscoverHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(info.ip.clone()).unwrap().into_raw()
    }
}

// Similar getters for name, model, serial, firmware...

#[no_mangle]
pub extern "C" fn wing_console_connect(ip: *const c_char) -> *mut WingConsoleHandle {
    let ip_str = unsafe { CStr::from_ptr(ip).to_string_lossy().into_owned() };
    match WingConsole::connect(&ip_str) {
        Ok(console) => {
            let handle = Box::new(WingConsoleHandle {
                console,
                request_end_cb: None,
                request_end_data: ptr::null_mut(),
                node_def_cb: None,
                node_def_data: ptr::null_mut(),
                node_data_cb: None,
                node_data_data: ptr::null_mut(),
            });
            Box::into_raw(handle)
        }
        Err(_) => ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn wing_console_destroy(handle: *mut WingConsoleHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// Implement remaining FFI functions...
