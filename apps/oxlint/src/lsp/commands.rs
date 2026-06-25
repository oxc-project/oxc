use serde::Deserialize;

pub const FIX_ALL_COMMAND_ID: &str = "oxc.fixAll";

#[derive(Deserialize)]
pub struct FixAllCommandArgs {
    pub uri: String,
}

impl TryFrom<Vec<serde_json::Value>> for FixAllCommandArgs {
    type Error = &'static str;

    fn try_from(value: Vec<serde_json::Value>) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err("Expected exactly one argument for FixAllCommandArgs");
        }

        let first_value = value.into_iter().next().ok_or("Missing argument")?;
        serde_json::from_value(first_value).map_err(|_| "Failed to parse FixAllCommandArgs")
    }
}
