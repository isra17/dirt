use unicorn;
use std::collections::HashMap;

pub type MemFlags = unicorn::unicorn_const::Protection;

#[derive(Clone)]
pub struct MemMap {
    pub addr: u64,
    pub size: usize,
    pub name: String,
    pub flags: MemFlags,
}

pub struct ObjectInfo {
    pub mem_maps: HashMap<String, MemMap>,
}

impl ObjectInfo {
    pub fn new() -> ObjectInfo {
        return ObjectInfo { mem_maps: HashMap::new() };
    }
}
