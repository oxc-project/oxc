use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum ImportInterop {
    #[default]
    Babel,
    Node,
    None,
}

impl FromStr for ImportInterop {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "babel" => Ok(Self::Babel),
            "node" => Ok(Self::Node),
            "none" => Ok(Self::None),
            _ => Err(()),
        }
    }
}
