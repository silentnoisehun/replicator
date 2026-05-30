use std::collections::HashMap;
use crate::protocol::*;

pub struct Collective {
    pub records: HashMap<String, CloneRecord>,
    pub lineage: HashMap<String, Vec<String>>,
}

impl Collective {
    pub fn new() -> Self {
        let mut c = Self { records: HashMap::new(), lineage: HashMap::new() };
        c.genesis();
        c
    }

    fn genesis(&mut self) {
        // ORA & LIORA alapító rekordok (hamarosan implementáljuk)
    }
}
