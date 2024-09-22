#[derive(Debug, Default, Clone)]
pub enum ModulesFormat {
    #[default]
    None,
    Commonjs,
}

impl From<&str> for ModulesFormat {
    fn from(s: &str) -> Self {
        match s {
            "commonjs" => Self::Commonjs,
            _ => Self::None,
        }
    }
}
