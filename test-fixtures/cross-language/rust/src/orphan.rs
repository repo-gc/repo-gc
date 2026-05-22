// Dead-weight: this file is never imported by any other file
pub fn orphan_function() -> String {
    "I am orphaned".to_string()
}

pub struct OrphanStruct {
    pub name: String,
    pub value: i32,
}

impl OrphanStruct {
    pub fn new(name: &str, value: i32) -> Self {
        Self { name: name.to_string(), value }
    }

    pub fn process(&self) -> i32 {
        self.value * 2
    }
}
