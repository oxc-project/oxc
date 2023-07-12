use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackageJson<'a> {
    pub main: Option<&'a str>,
}

impl<'a> TryFrom<&'a str> for PackageJson<'a> {
    type Error = serde_json::Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        serde_json::from_str(s)
    }
}
