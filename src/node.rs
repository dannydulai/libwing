use crate::console::WingConsole;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
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
#[derive(Copy, Clone, PartialEq, Debug)]
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

    pub fn to_description(&self) -> String {
        let fullname = WingConsole::id_to_name(self.id);

        let mut r = String::with_capacity(1000);

        if let Some(fullname) = fullname {
            r.push_str(fullname);
        } else {
            let pname = WingConsole::id_to_name(self.parent_id);
            if let Some(pname) = pname {
                r.push_str(pname);
            } else {
                r.push_str(&format!("<Unknown:{}>", self.parent_id));
            }
            r.push_str(&format!("/<Unknown:{}>", self.id));
        }

        if self.read_only {
            r.push_str(" [read-only]");
        }
        r.push_str(&format!("\n    Id: {}", self.id));
        if self.index != 0 {
            r.push_str(&format!("\n    Index: {}", self.index));
        }
        if !self.name.is_empty() {
            r.push_str(&format!("\n    Name: {}", self.name));
        }
        if !self.long_name.is_empty() {
            r.push_str(&format!("\n    Long Name: {}", self.long_name));
        }

        r.push_str("\n    Type: ");
        match self.node_type {
            NodeType::Node             => { r.push_str("node"); }
            NodeType::LinearFloat      => { r.push_str("linear float"); }
            NodeType::LogarithmicFloat => { r.push_str("log float"); }
            NodeType::Integer          => { r.push_str("integer"); }
            NodeType::String           => { r.push_str("string"); }
            NodeType::FaderLevel       => { r.push_str("fader level (float)"); }
            NodeType::StringEnum       => { r.push_str("string enum"); }
            NodeType::FloatEnum        => { r.push_str("float enum"); }
        }
        if self.unit != NodeUnit::None {
            r.push_str("\n    Unit: ");
            match self.unit {
                NodeUnit::Db           => { r.push_str("dB"); }
                NodeUnit::Percent      => { r.push('%'); }
                NodeUnit::Milliseconds => { r.push_str("ms"); }
                NodeUnit::Hertz        => { r.push_str("Hz"); }
                NodeUnit::Meters       => { r.push_str("meters"); }
                NodeUnit::Seconds      => { r.push_str("seconds"); }
                NodeUnit::Octaves      => { r.push_str("octaves"); }
                _ => {}
            }
        }

        match self.node_type {
            NodeType::LinearFloat | 
            NodeType::LogarithmicFloat |
            NodeType::FaderLevel => {
                if let Some(min_float) = self.min_float { r.push_str(&format!("\n    Minimum: {}", min_float)); }
                if let Some(max_float) = self.max_float { r.push_str(&format!("\n    Maximum: {}", max_float)); }
                if let Some(steps)     = self.steps     { r.push_str(&format!("\n    Steps:   {}", steps)); }
            }
            NodeType::Integer => {
                if let Some(min_int) = self.min_int { r.push_str(&format!("\n    Minimum: {}", min_int)); }
                if let Some(max_int) = self.max_int { r.push_str(&format!("\n    Maximum: {}", max_int)); }
            }
            NodeType::String => {
                if let Some(max_string_len) = self.max_string_len { r.push_str(&format!("\n    Maximum String Length: {}", max_string_len)); }
            }
            NodeType::StringEnum  => {
                if self.string_enum.is_some() {
                    r.push_str("\n    Items:");
                    for item in self.string_enum.as_ref().unwrap() {
                        r.push_str(&format!("\n        • {}", item.item));
                        if !item.long_item.is_empty() {
                            r.push_str(&format!(" ({})", item.long_item));
                        }
                    }
                }
            }
            NodeType::FloatEnum => {
                if self.float_enum.is_some() {
                    r.push_str("\n    Items:");
                    for item in self.float_enum.as_ref().unwrap() {
                        r.push_str(&format!("\n        • {}", item.item));
                        if !item.long_item.is_empty() {
                            r.push_str(&format!(" ({})", item.long_item));
                        }
                    }
                }
            }
            _ => {}
        }
        r
    }

    pub fn to_json(&self) -> jzon::JsonValue {
        let mut json = jzon::object!{
            id: self.id,
        };

        if let Some(fullname) = WingConsole::id_to_name(self.id) {
            json.insert("fullname", fullname).unwrap();
        }

        if self.index != 0 { 
            json.insert("index", self.index).unwrap();
        }
        if !self.name.is_empty() {
            json.insert("name", self.name.clone()).unwrap();
        }
        if !self.long_name.is_empty() {
            json.insert("longname", self.long_name.clone()).unwrap();
        }

        match self.node_type {
            NodeType::Node             => { json.insert("type", "node").unwrap(); }
            NodeType::LinearFloat      => { json.insert("type", "linear float").unwrap(); }
            NodeType::LogarithmicFloat => { json.insert("type", "log float").unwrap(); }
            NodeType::Integer          => { json.insert("type", "integer").unwrap(); }
            NodeType::String           => { json.insert("type", "string").unwrap(); }
            NodeType::FaderLevel       => { json.insert("type", "fader level").unwrap(); }
            NodeType::StringEnum       => { json.insert("type", "string enum").unwrap(); }
            NodeType::FloatEnum        => { json.insert("type", "float enum").unwrap(); }
        }
        match self.unit {
            NodeUnit::None         => { }
            NodeUnit::Db           => { json.insert("unit", "dB").unwrap(); }
            NodeUnit::Percent      => { json.insert("unit", "%").unwrap(); }
            NodeUnit::Milliseconds => { json.insert("unit", "ms").unwrap(); }
            NodeUnit::Hertz        => { json.insert("unit", "Hz").unwrap(); }
            NodeUnit::Meters       => { json.insert("unit", "meters").unwrap(); }
            NodeUnit::Seconds      => { json.insert("unit", "seconds").unwrap(); }
            NodeUnit::Octaves      => { json.insert("unit", "octaves").unwrap(); }
        }

        if self.read_only {
            json.insert("read_only", true).unwrap();
        }

        match self.node_type {
            NodeType::LinearFloat | 
            NodeType::LogarithmicFloat |
            NodeType::FaderLevel => {
                if let Some(min_float) = self.min_float { json.insert("minfloat", min_float).unwrap(); }
                if let Some(max_float) = self.max_float { json.insert("maxfloat", max_float).unwrap(); }
                if let Some(steps) = self.steps { json.insert("steps", steps).unwrap(); }
            }
            NodeType::Integer => {
                if let Some(min_int) = self.min_int { json.insert("minint", min_int).unwrap(); }
                if let Some(max_int) = self.max_int { json.insert("maxint", max_int).unwrap(); }
            }
            NodeType::String => {
                if let Some(max_string_len) = self.max_string_len { json.insert("maxstringlen", max_string_len).unwrap(); }
            }
            NodeType::StringEnum  => {
                if self.string_enum.is_some() {
                    json.insert("items", self.string_enum.as_ref().unwrap().iter().map(|item| {
                        let mut j = jzon::object!{ "item": item.item.clone() };
                        if !item.long_item.is_empty() {
                            j.insert("longitem", item.long_item.clone()).unwrap();
                        }
                        j
                    }).collect::<Vec<_>>()).unwrap();
                }
            }
            NodeType::FloatEnum => {
                if self.float_enum.is_some() {
                    json.insert("items", self.float_enum.as_ref().unwrap().iter().map(|item| {
                        let mut j = jzon::object!{ "item": item.item };
                        if !item.long_item.is_empty() {
                            j.insert("longitem", item.long_item.clone()).unwrap();
                        }
                        j
                    }).collect::<Vec<_>>()).unwrap();
                }
            }
            _ => {}
        }
        json
    }
}
