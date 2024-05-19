use std::collections::HashMap;

use schematic::{Config, ConfigEnum};
use serde::Deserialize;

#[derive(Config, Deserialize, Debug, Clone)]
pub struct ESLintConfig2 {
    #[setting(skip_serializing)]
    pub globals: HashMap<String, GlobalValue2>,
}

#[derive(ConfigEnum, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GlobalValue2 {
    Readonly,
    Writeable,
    Off,
}
