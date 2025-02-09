use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_float};
use std::ptr;
use crate::{WingConsole, NodeType, NodeUnit, WingResponse};

// Opaque type wrappers
#[repr(C)]
pub struct WingDiscoveryInfoHandle {
    info: Vec<crate::DiscoveryInfo>
}

#[repr(C)]
pub struct WingConsoleHandle {
    console: WingConsole,
}

#[repr(C)]
pub struct ResponseHandle {
    response: WingResponse
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub enum ResponseType {
    End = 0,
    NodeDefinition = 1,
    NodeData = 2,
}

#[no_mangle]
pub extern "C" fn wing_string_destroy(handle: *const c_char) {
    unsafe {
        drop(CString::from_raw(handle as *mut c_char));
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_scan(stop_on_first: c_int) -> *mut WingDiscoveryInfoHandle {
    let results = WingConsole::scan(stop_on_first != 0);
    if let Ok(results) = results {
        Box::into_raw(Box::new(WingDiscoveryInfoHandle { info: results }))
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_destroy(handle: *mut WingDiscoveryInfoHandle) {
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_count(handle: *const WingDiscoveryInfoHandle) -> c_int {
    unsafe {
        (*handle).info.len() as c_int
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_ip(handle: *const WingDiscoveryInfoHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(&info.ip[..]).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_name(handle: *const WingDiscoveryInfoHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(&info.name[..]).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_model(handle: *const WingDiscoveryInfoHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(&info.model[..]).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_serial(handle: *const WingDiscoveryInfoHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(&info.serial[..]).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_discover_get_firmware(handle: *const WingDiscoveryInfoHandle, index: c_int) -> *const c_char {
    unsafe {
        let info = &(*handle).info[index as usize];
        CString::new(&info.firmware[..]).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn wing_console_connect(ip: *const c_char) -> *mut WingConsoleHandle {
    let ip = unsafe { CStr::from_ptr(ip).to_str() };
    if let Ok(ip) = ip {
        match WingConsole::connect(ip) {
            Ok(console) => Box::into_raw(Box::new(WingConsoleHandle { console })),
            Err(_) => ptr::null_mut()
        }
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn wing_console_destroy(handle: *mut WingConsoleHandle) {
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[no_mangle]
pub extern "C" fn wing_console_read(handle: *mut WingConsoleHandle) -> *mut ResponseHandle {
    unsafe {
        if let Ok(response) = (*handle).console.read() {
            Box::into_raw(Box::new(ResponseHandle { response }))
        } else {
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_response_destroy(handle: *mut ResponseHandle) {
    unsafe {
        drop(Box::from_raw(handle));
    }
}


#[no_mangle]
pub extern "C" fn wing_console_set_string(handle: *mut WingConsoleHandle, id: i32, value: *const c_char) -> c_int {
    unsafe {
        if let Ok(value) = CStr::from_ptr(value).to_str() {
            if (*handle).console.set_string(id, value).is_ok() {
                0
            } else {
                -1
            }
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_console_set_float(handle: *mut WingConsoleHandle, id: i32, value: c_float) -> c_int {
    unsafe {
        if (*handle).console.set_float(id, value).is_ok() {
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_console_set_int(handle: *mut WingConsoleHandle, id: i32, value: c_int) -> c_int {
    unsafe {
        if (*handle).console.set_int(id, value).is_ok() {
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_console_request_node_definition(handle: *mut WingConsoleHandle, id: i32) -> c_int {
    unsafe {
        if (*handle).console.request_node_definition(id).is_ok() {
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_console_request_node_data(handle: *mut WingConsoleHandle, id: i32) -> c_int {
    unsafe {
        if (*handle).console.request_node_data(id).is_ok() {
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_response_get_type(handle: *const ResponseHandle) -> ResponseType {
    match unsafe { &(*handle).response } {
        WingResponse::RequestEnd => ResponseType::End,
        WingResponse::NodeDef(_) => ResponseType::NodeDefinition,
        WingResponse::NodeData(_, _) => ResponseType::NodeData,
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_get_string(handle: *const ResponseHandle) -> *const c_char {
    unsafe {
        if let WingResponse::NodeData(_, data) = &(*handle).response {
            let s = data.get_string();
            CString::new(&s[..]).unwrap().into_raw()
        } else {
            ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_get_float(handle: *const ResponseHandle) -> c_float {
    unsafe {
        if let WingResponse::NodeData(_, data) = &(*handle).response {
            data.get_float()
        } else {
            0.0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_get_int(handle: *const ResponseHandle) -> c_int {
    unsafe {
        if let WingResponse::NodeData(_, data) = &(*handle).response {
            data.get_int()
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_has_string(handle: *const ResponseHandle) -> c_int {
    unsafe {
        if let WingResponse::NodeData(_, data) = &(*handle).response {
            if data.has_string() { 1 } else { 0 }
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_has_float(handle: *const ResponseHandle) -> c_int {
    unsafe {
        if let WingResponse::NodeData(_, data) = &(*handle).response {
            if data.has_float() { 1 } else { 0 }
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_data_has_int(handle: *const ResponseHandle) -> c_int {
    unsafe {
        if let WingResponse::NodeData(_, data) = &(*handle).response {
            if data.has_int() { 1 } else { 0 }
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_name_to_id(name: *const c_char) -> i32 {
    unsafe {
        if let Ok(name_str) = CStr::from_ptr(name).to_str() {
            WingConsole::name_to_id(name_str).unwrap_or(0)
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_id_to_name(id: i32) -> *const c_char {
    let name = WingConsole::id_to_name(id);
    if let Some(name) = name {
        CString::new(name).unwrap().into_raw()
    } else {
        ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_parent_id(def: *const ResponseHandle) -> i32 {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            def.parent_id
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_id(def: *const ResponseHandle) -> i32 {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            def.id
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_index(def: *const ResponseHandle) -> u16 {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            def.index
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_type(def: *const ResponseHandle) -> NodeType {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            def.node_type
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_unit(def: *const ResponseHandle) -> NodeUnit {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            def.unit
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_name(def: *const ResponseHandle) -> *const c_char {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            CString::new(&def.name[..]).unwrap().into_raw()
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_long_name(def: *const ResponseHandle) -> *const c_char {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            CString::new(&def.long_name[..]).unwrap().into_raw()
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_min_float(def: *const ResponseHandle, ret: *mut c_float) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(min_float) = def.min_float {
                *ret = min_float;
                1
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_max_float(def: *const ResponseHandle, ret: *mut c_float) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(max_float) = def.max_float {
                *ret = max_float;
                1
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_steps(def: *const ResponseHandle, ret: *mut c_int) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(steps) = def.steps {
                *ret = steps;
                1
            } else {
                0
            }
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_min_int(def: *const ResponseHandle, ret: *mut c_int) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(min_int) = def.min_int {
                *ret = min_int;
                1
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_max_int(def: *const ResponseHandle, ret: *mut c_int) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(max_int) = def.max_int {
                *ret = max_int;
                1
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_max_string_len(def: *const ResponseHandle, ret: *mut c_int) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(max_string_len) = def.max_string_len {
                *ret = max_string_len as i32;
                1
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_string_enum_count(def: *const ResponseHandle) -> usize {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(string_enum) = &def.string_enum {
                string_enum.len()
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_float_enum_count(def: *const ResponseHandle) -> usize {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(float_enum) = &def.float_enum {
                float_enum.len()
            } else {
                0
            }
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_float_enum_item(def: *const ResponseHandle, index: usize, ret: *mut c_float) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(item) = &def.float_enum {
                if let Some(item) = item.get(index) {
                    *ret = item.item;
                    1
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_float_enum_long_item(def: *const ResponseHandle, index: usize, ret: *mut *mut c_char) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(item) = &def.float_enum {
                if let Some(item) = item.get(index) {
                    *ret = CString::new(&item.long_item[..]).unwrap().into_raw();
                    1
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_node_definition_get_string_enum_item(def: *const ResponseHandle, index: usize, ret: *mut *mut c_char) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(item) = &def.string_enum {
                if let Some(item) = item.get(index) {
                    *ret = CString::new(&item.item[..]).unwrap().into_raw();
                    1
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}
#[no_mangle]
pub extern "C" fn wing_node_definition_get_string_enum_long_item(def: *const ResponseHandle, index: usize, ret: *mut *mut c_char) -> c_int {
    unsafe {
        if let WingResponse::NodeDef(def) = &(*def).response {
            if let Some(item) = &def.string_enum {
                if let Some(item) = item.get(index) {
                    *ret = CString::new(&item.long_item[..]).unwrap().into_raw();
                    1
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            panic!("Invalid response type");
        }
    }
}
