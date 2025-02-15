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
    pub raw: Vec<u8>,
}

impl WingNodeDef {
    pub fn from_bytes(raw: &[u8]) -> Self {
        let mut i = 0;

        let parent_id = i32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]);
        i += 4;
        let id = i32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]);
        i += 4;
        let index = u16::from_be_bytes([raw[i], raw[i+1]]);
        i += 2;
        let name_len = raw[i];
        i += 1;
        let name = String::from_utf8(raw[i..i+name_len as usize].to_vec()).unwrap();
        i += name_len as usize;
        let long_name_len = raw[i];
        i += 1;
        let long_name = String::from_utf8(raw[i..i+long_name_len as usize].to_vec()).unwrap();
        i += long_name_len as usize;
        let flags = u16::from_be_bytes([raw[i], raw[i+1]]);
        i += 2;

        let node_type = match (flags >> 4) & 0x0F {
            0 => NodeType::Node,
            1 => NodeType::LinearFloat,
            2 => NodeType::LogarithmicFloat,
            3 => NodeType::FaderLevel,
            4 => NodeType::Integer,
            5 => NodeType::StringEnum,
            6 => NodeType::FloatEnum,
            7 => NodeType::String,
            _ => NodeType::Node,
        };

        let unit = match flags & 0x0F {
            0 => NodeUnit::None,
            1 => NodeUnit::Db,
            2 => NodeUnit::Percent,
            3 => NodeUnit::Milliseconds,
            4 => NodeUnit::Hertz,
            5 => NodeUnit::Meters,
            6 => NodeUnit::Seconds,
            7 => NodeUnit::Octaves,
            _ => NodeUnit::None,
        };

        let read_only = ((flags >> 9) & 0x01) != 0;

        let mut min_float      = Option::None;
        let mut max_float      = Option::None;
        let mut steps          = Option::None;
        let mut min_int        = Option::None;
        let mut max_int        = Option::None;
        let mut max_string_len = Option::None;
        let mut string_enum    = Option::None;
        let mut float_enum     = Option::None;

        match node_type {
            NodeType::Node | NodeType::FaderLevel => { }
            NodeType::String => {
                max_string_len = Some(u16::from_be_bytes([raw[i], raw[i+1]]));
                //i += 2;
            }
            NodeType::LinearFloat | 
                NodeType::LogarithmicFloat => {
                    min_float = Some(f32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]));
                    i += 4;
                    max_float = Some(f32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]));
                    i += 4;
                    steps = Some(i32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]));
                    //i += 4;
                }
            NodeType::Integer => {
                min_int = Some(i32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]));
                i += 4;
                max_int = Some(i32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]));
                //i += 4;
            }
            NodeType::StringEnum => {
                let num = u16::from_be_bytes([raw[i], raw[i+1]]);
                i += 2;
                for _ in 0..num {
                    let item_len = raw[i] as usize;
                    i += 1;
                    let item = String::from_utf8(raw[i..i+item_len].to_vec()).unwrap();
                    i += item_len;
                    let long_item_len = raw[i] as usize;
                    i += 1;
                    let long_item = String::from_utf8(raw[i..i+long_item_len].to_vec()).unwrap();
                    i += long_item_len;
                    if string_enum.is_none() {
                        string_enum = Some(Vec::new());
                    }
                    string_enum.as_mut().unwrap().push(StringEnumItem {
                        item,
                        long_item,
                    });
                }
            }
            NodeType::FloatEnum => {
                let num = u16::from_be_bytes([raw[i], raw[i+1]]);
                i += 2;
                for _ in 0..num {
                    let item = f32::from_be_bytes([raw[i], raw[i+1], raw[i+2], raw[i+3]]);
                    i += 4;
                    let long_item_len = raw[i] as usize;
                    i += 1;
                    let long_item = String::from_utf8(raw[i..i+long_item_len].to_vec()).unwrap();
                    i += long_item_len;
                    if float_enum.is_none() {
                        float_enum = Some(Vec::new());
                    }
                    float_enum.as_mut().unwrap().push(FloatEnumItem {
                        item,
                        long_item,
                    });
                }
            }
        }

        WingNodeDef {
            id,
            parent_id,
            index,
            name,
            long_name,
            node_type,
            unit,
            read_only,
            min_float,
            max_float,
            steps,
            min_int,
            max_int,
            max_string_len,
            string_enum,
            float_enum,
            raw: raw.to_vec(),
        }
    }
}

impl Clone for WingNodeDef {
    fn clone(&self) -> Self {
        let mut string_enum = None;
        if self.string_enum.is_some() {
            string_enum = Some(self.string_enum.as_ref().unwrap().iter().map(|item| {
                StringEnumItem {
                    item: item.item.clone(),
                    long_item: item.long_item.clone(),
                }
            }).collect::<Vec<_>>());
        }

        let mut float_enum = None;
        if self.float_enum.is_some() {
            float_enum = Some(self.float_enum.as_ref().unwrap().iter().map(|item| {
                FloatEnumItem {
                    item: item.item,
                    long_item: item.long_item.clone(),
                }
            }).collect::<Vec<_>>());
        }

        Self {
            id: self.id,
            parent_id: self.parent_id,
            index: self.index,
            name: self.name.clone(),
            long_name: self.long_name.clone(),
            node_type: self.node_type,
            unit: self.unit,
            read_only: self.read_only,
            min_float: self.min_float,
            max_float: self.max_float,
            steps: self.steps,
            min_int: self.min_int,
            max_int: self.max_int,
            max_string_len: self.max_string_len,
            string_enum,
            float_enum,
            raw: self.raw.clone(),
        }
    }
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
        let mut r = String::with_capacity(1000);
        // if let Some(data) = WingConsole::id_to_data(self.id) {
        // }
        //
        //
        // if let Some(fullname) = fullname {
        //     r.push_str(fullname);
        // } else {
        //     let pname = WingConsole::id_to_name(self.parent_id);
        //     if let Some(pname) = pname {
        //         r.push_str(pname);
        //     } else {
        //         r.push_str(&format!("<Unknown:{}>", self.parent_id));
        //     }
        //     r.push_str(&format!("/<Unknown:{}>", self.id));
        // }
        //
        r.push_str(&format!(  "Id:        {}", self.id));
        r.push_str(&format!("\nRead-only: {}", if self.read_only { "yes" } else { "no" }));
        if self.index != 0 {
        r.push_str(&format!("\nIndex:     {}", self.index));
        }
        if !self.name.is_empty() {
        r.push_str(&format!("\nName:      {}", self.name));
        }
        if !self.long_name.is_empty() {
        r.push_str(&format!("\nLong Name: {}", self.long_name));
        }

        r.push_str(&format!("\nType:      {}",
            match self.node_type {
                NodeType::Node             => "node",
                NodeType::LinearFloat      => "linear float",
                NodeType::LogarithmicFloat => "log float",
                NodeType::Integer          => "integer",
                NodeType::String           => "string",
                NodeType::FaderLevel       => "fader level (float)",
                NodeType::StringEnum       => "string enum",
                NodeType::FloatEnum        => "float enum",
            }));
        if self.unit != NodeUnit::None {
            r.push_str(&format!("\nUnit:      {}",
                match self.unit {
                    NodeUnit::Db           => "dB",
                    NodeUnit::Percent      => "%",
                    NodeUnit::Milliseconds => "ms",
                    NodeUnit::Hertz        => "Hz",
                    NodeUnit::Meters       => "meters",
                    NodeUnit::Seconds      => "seconds",
                    NodeUnit::Octaves      => "octaves",
                    _ => "UNKNOWN"
                }));
        }

        match self.node_type {
            NodeType::LinearFloat | 
            NodeType::LogarithmicFloat |
            NodeType::FaderLevel => {
                if let Some(min_float) = self.min_float { r.push_str(&format!("\nMinimum:   {}", min_float)); }
                if let Some(max_float) = self.max_float { r.push_str(&format!("\nMaximum:   {}", max_float)); }
                if let Some(steps)     = self.steps     { r.push_str(&format!("\nSteps:     {}", steps)); }
            }
            NodeType::Integer => {
                if let Some(min_int) = self.min_int { r.push_str(&format!("\nMinimum:   {}", min_int)); }
                if let Some(max_int) = self.max_int { r.push_str(&format!("\nMaximum:   {}", max_int)); }
            }
            NodeType::String => {
                if let Some(max_string_len) = self.max_string_len { r.push_str(&format!("\nMaxLength: {}", max_string_len)); }
            }
            NodeType::StringEnum  => {
                if self.string_enum.is_some() {
                    r.push_str("\nItems:");
                    let mut first = true;
                    for item in self.string_enum.as_ref().unwrap() {
                        if first {
                            r.push_str(&format!("     {}", item.item));
                            first = false;
                        } else {
                            r.push_str(&format!("           {}", item.item));
                        }

                        if !item.long_item.is_empty() {
                            r.push_str(&format!(" ({})", item.long_item));
                        }
                        r.push('\n');
                    }
                }
            }
            NodeType::FloatEnum => {
                if self.float_enum.is_some() {
                    r.push_str("\nItems:");
                    let mut first = true;
                    for item in self.float_enum.as_ref().unwrap() {
                        if first {
                            r.push_str(&format!("     {}", item.item));
                            first = false;
                        } else {
                            r.push_str(&format!("           {}", item.item));
                        }
                        if !item.long_item.is_empty() {
                            r.push_str(&format!(" ({})", item.long_item));
                        }
                        r.push('\n');
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

        // if let Some(fullname) = WingConsole::id_to_name(self.id) {
        //     json.insert("fullname", fullname).unwrap();
        // }

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
