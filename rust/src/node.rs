#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct StringEnumItem {
    pub item: String,
    pub longitem: String,
}

#[derive(Debug, Clone)]
pub struct FloatEnumItem {
    pub item: f32,
    pub longitem: String,
}

#[derive(Debug, Clone)]
pub struct NodeDefinition {
    pub id: u32,
    pub parent_id: u32,
    pub index: u16,
    pub name: String,
    pub long_name: String,
    pub node_type: NodeType,
    pub unit: NodeUnit,
    pub read_only: bool,
    pub min_float: f32,
    pub max_float: f32,
    pub steps: u32,
    pub min_int: i32,
    pub max_int: i32,
    pub max_string_len: u16,
    pub string_enum: Vec<StringEnumItem>,
    pub float_enum: Vec<FloatEnumItem>,
}

#[derive(Debug, Clone)]
pub struct NodeData {
    string_value: Option<String>,
    float_value: Option<f32>,
    int_value: Option<i32>,
}

impl NodeData {
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

    pub fn with_int(i: i32) -> Self {
        Self {
            string_value: None,
            float_value: None,
            int_value: Some(i),
        }
    }

    pub fn get_string(&self) -> String {
        self.string_value.clone().unwrap_or_default()
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

impl NodeDefinition {
    pub fn get_type(&self) -> NodeType {
        self.node_type
    }

    pub fn get_unit(&self) -> NodeUnit {
        self.unit
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn init_map(path_to_map_file: &str) -> crate::Result<()> {
        // TODO: Implement map file loading
        Ok(())
    }

    pub fn node_name_to_id(fullname: &str) -> u32 {
        // TODO: Implement name to ID lookup
        0
    }

    pub fn node_id_to_name(id: u32) -> String {
        // TODO: Implement ID to name lookup
        String::new()
    }
}
