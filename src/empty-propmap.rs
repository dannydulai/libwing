use std::collections::HashMap;
use crate::node::WingNodeDef;

lazy_static::lazy_static! {
    pub static ref NAME_TO_DEF: HashMap<String, WingNodeDef> = HashMap::new();
}
