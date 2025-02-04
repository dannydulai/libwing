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

#[no_mangle]
pub extern "C" fn wing_discover_get_name(handle: *const WingDiscoverHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(info.name.clone()).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_model(handle: *const WingDiscoverHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(info.model.clone()).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_serial(handle: *const WingDiscoverHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(info.serial.clone()).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_firmware(handle: *const WingDiscoverHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(info.firmware.clone()).unwrap().into_raw()
    }
}

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

#[no_mangle]
pub extern "C" fn wing_console_set_request_end_callback(
    handle: *mut WingConsoleHandle,
    callback: Option<extern "C" fn(*mut c_void)>,
    user_data: *mut c_void,
) {
    unsafe {
        let console = &mut (*handle);
        console.request_end_cb = callback;
        console.request_end_data = user_data;
    }
}

#[no_mangle]
pub extern "C" fn wing_console_set_node_definition_callback(
    handle: *mut WingConsoleHandle,
    callback: Option<extern "C" fn(*mut NodeDefinitionHandle, *mut c_void)>,
    user_data: *mut c_void,
) {
    unsafe {
        let console = &mut (*handle);
        console.node_def_cb = callback;
        console.node_def_data = user_data;
    }
}

#[no_mangle]
pub extern "C" fn wing_console_set_node_data_callback(
    handle: *mut WingConsoleHandle,
    callback: Option<extern "C" fn(u32, *mut NodeDataHandle, *mut c_void)>,
    user_data: *mut c_void,
) {
    unsafe {
        let console = &mut (*handle);
        console.node_data_cb = callback;
        console.node_data_data = user_data;
    }
}

#[no_mangle]
pub extern "C" fn wing_console_read(handle: *mut WingConsoleHandle) {
    unsafe {
        (*handle).console.read().unwrap_or(());
    }
}

#[no_mangle]
pub extern "C" fn wing_console_set_string(
    handle: *mut WingConsoleHandle,
    id: u32,
    value: *const c_char,
) {
    unsafe {
        if let Ok(s) = CStr::from_ptr(value).to_str() {
            let _ = (*handle).console.set_string(id, s);
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_console_set_float(
    handle: *mut WingConsoleHandle,
    id: u32,
    value: c_float,
) {
    unsafe {
        let _ = (*handle).console.set_float(id, value);
    }
}

#[no_mangle]
pub extern "C" fn wing_console_set_int(
    handle: *mut WingConsoleHandle,
    id: u32,
    value: c_int,
) {
    unsafe {
        let _ = (*handle).console.set_int(id, value as i32);
    }
}

#[no_mangle]
pub extern "C" fn wing_console_request_node_definition(
    handle: *mut WingConsoleHandle,
    id: u32,
) {
    unsafe {
        let _ = (*handle).console.request_node_definition(id);
    }
}

#[no_mangle]
pub extern "C" fn wing_console_request_node_data(
    handle: *mut WingConsoleHandle,
    id: u32,
) {
    unsafe {
        let _ = (*handle).console.request_node_data(id);
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_destroy(handle: *mut NodeDefinitionHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_destroy(handle: *mut NodeDataHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_get_string(
    handle: *const NodeDataHandle,
    buffer: *mut c_char,
    buffer_size: usize,
) {
    unsafe {
        let s = (*handle).data.get_string();
        let c_str = CString::new(s).unwrap_or_default();
        libc::strncpy(buffer, c_str.as_ptr(), buffer_size - 1);
        *buffer.add(buffer_size - 1) = 0;
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_get_float(handle: *const NodeDataHandle) -> c_float {
    unsafe {
        (*handle).data.get_float()
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_get_int(handle: *const NodeDataHandle) -> c_int {
    unsafe {
        (*handle).data.get_int()
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_has_string(handle: *const NodeDataHandle) -> c_int {
    unsafe {
        (*handle).data.has_string() as c_int
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_has_float(handle: *const NodeDataHandle) -> c_int {
    unsafe {
        (*handle).data.has_float() as c_int
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_has_int(handle: *const NodeDataHandle) -> c_int {
    unsafe {
        (*handle).data.has_int() as c_int
    }
}
