use std::collections::HashMap;
lazy_static::lazy_static! {
    pub static ref ID_TO_DATA: HashMap<i32, (String, i32, u8)> = HashMap::new();
}
