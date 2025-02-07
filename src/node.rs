use std::collections::HashMap;

use crate::propmap::NAME_TO_ID;

lazy_static::lazy_static! {
    static ref ID_TO_NAME: HashMap<i32, &'static str> = {
        let mut id2name = HashMap::new();
        if (id2name).is_empty() {
            for (name, id) in NAME_TO_ID.iter() {
                id2name.insert(*id, &name[..]);
            }
        }
        id2name
    };
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub enum NodeType {
    Node = 0,
    LinearFloat = 1,
    LogarithmicFloat = 2,
    FaderLevel = 3,
    Integer = 4,
    StringEnum = 5,
    FloatEnum = 6,
    String = 7,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub enum NodeUnit {
    None = 0,
    Db = 1,
    Percent = 2,
    Milliseconds = 3,
    Hertz = 4,
    Meters = 5,
    Seconds = 6,
    Octaves = 7,
}

pub struct StringEnumItem {
    pub item: String,
    pub long_item: String,
}

pub struct FloatEnumItem {
    pub item: f32,
    pub long_item: String,
}

pub struct WingNodeDef {
    pub id: i32,
    pub parent_id: i32,
    pub index: u16,
    pub name: String,
    pub long_name: String,
    pub node_type: NodeType,
    pub unit: NodeUnit,
    pub read_only: bool,
    pub min_float: Option<f32>,
    pub max_float: Option<f32>,
    pub steps: Option<i32>,
    pub min_int: Option<i32>,
    pub max_int: Option<i32>,
    pub max_string_len: Option<u16>,
    pub string_enum: Option<Vec<StringEnumItem>>,
    pub float_enum: Option<Vec<FloatEnumItem>>,
}

pub struct WingNodeData {
    string_value: Option<String>,
    float_value: Option<f32>,
    int_value: Option<i32>,
}

impl Default for WingNodeData {
    fn default() -> Self {
        Self::new()
    }
}

impl WingNodeData {
    pub fn new() -> Self {
        Self {
            string_value: None,
            float_value: None,
            int_value: None,
        }
    }

    pub fn with_string(s: String) -> Self {
        Self {
            string_value: Some(s),
            float_value: None,
            int_value: None,
        }
    }

    pub fn with_float(f: f32) -> Self {
        Self {
            string_value: None,
            float_value: Some(f),
            int_value: None,
        }
    }

    pub fn with_i32(i: i32) -> Self {
        Self {
            string_value: None,
            float_value: None,
            int_value: Some(i),
        }
    }
    pub fn with_i16(i: i16) -> Self {
        Self {
            string_value: None,
            float_value: None,
            int_value: Some(i as i32),
        }
    }

    pub fn with_i8(i: i8) -> Self {
        Self {
            string_value: None,
            float_value: None,
            int_value: Some(i as i32),
        }
    }

    pub fn get_string(&self) -> String {
        if self.has_string() {
            self.string_value.clone().unwrap()
        } else if self.has_float() {
            self.float_value.unwrap().to_string()
        } else if self.has_int() {
            self.int_value.unwrap().to_string()
        } else {
            String::new()
        }
    }

    pub fn get_float(&self) -> f32 {
        self.float_value.unwrap_or(0.0)
    }

    pub fn get_int(&self) -> i32 {
        self.int_value.unwrap_or(0)
    }

    pub fn has_string(&self) -> bool {
        self.string_value.is_some()
    }

    pub fn has_float(&self) -> bool {
        self.float_value.is_some()
    }

    pub fn has_int(&self) -> bool {
        self.int_value.is_some()
    }
}

impl WingNodeDef {
    pub fn get_type(&self) -> NodeType {
        self.node_type
    }

    pub fn get_unit(&self) -> NodeUnit {
        self.unit
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn node_name_to_id(fullname: &str) -> i32 {
        NAME_TO_ID.get(fullname).copied().unwrap_or(0)
    }

    pub fn node_id_to_name(id: i32) -> Option<&'static str> {
        ID_TO_NAME.get(&id).map(|x| &**x)
    }
}
