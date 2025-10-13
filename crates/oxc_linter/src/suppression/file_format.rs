use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionFile {
    pub version: String,
    pub suppressions: HashMap<String, HashMap<String, SuppressionEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionEntry {
    pub count: u32,
}

impl Default for SuppressionFile {
    fn default() -> Self {
        Self { version: "0.1.0".to_string(), suppressions: HashMap::new() }
    }
}

impl SuppressionFile {
    pub fn new() -> Self {
        Self::default()
    }
}
