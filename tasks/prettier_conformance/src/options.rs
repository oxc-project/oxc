use std::path::{Path, PathBuf};

#[derive(Default, Clone)]
pub struct TestRunnerOptions {
    pub language: TestLanguage,
    pub debug: bool,
    pub filter: Option<String>,
}

#[derive(Default, Clone, Copy)]
pub enum TestLanguage {
    #[default]
    Js,
    Ts,
    Json,
}

impl TestLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Js => "js",
            Self::Ts => "ts",
            Self::Json => "json",
        }
    }

    /// Prettier's test fixtures roots for different languages.
    pub fn fixtures_roots(self, base: &Path) -> Vec<PathBuf> {
        match self {
            Self::Js => ["js", "jsx"],
            // There is no `tsx` directory, just check it works with TS
            // `SourceType`.`variant` is handled by spec file extension
            Self::Ts => ["typescript", "jsx"],
            Self::Json => return vec![base.join("json").join("json")],
        }
        .iter()
        .map(|dir| base.join(dir))
        .collect::<Vec<_>>()
    }
}
