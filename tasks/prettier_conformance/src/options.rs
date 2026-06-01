use std::path::{Path, PathBuf};

#[derive(Default, Clone)]
pub struct TestRunnerOptions {
    pub language: TestLanguage,
    pub debug: bool,
    pub filter: Option<String>,
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
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
            Self::Js => ["js", "jsx"].iter().map(|dir| base.join(dir)).collect::<Vec<_>>(),
            // There is no `tsx` directory, just check it works with TS
            // `SourceType`.`variant` is handled by spec file extension
            Self::Ts => ["typescript", "jsx"].iter().map(|dir| base.join(dir)).collect::<Vec<_>>(),
            // Phase 1 of the JSON formatter targets the `json` parser only.
            // Out-of-scope siblings:
            // - `jsonc/*` / `json5-as-json-with-trailing-commas/` — jsonc/json5 parsers
            // - `json-superset/` — inline `snippets` shape, not parseable by spec.rs
            // - `range/` — range-formatting tests, not a whole-file format
            // `with-comment/` is included because each of its `format.test.js`
            // entries lists a parser explicitly; spec.rs filters out non-`json` ones.
            Self::Json => {
                vec![base.join("json").join("json"), base.join("json").join("with-comment")]
            }
        }
    }
}
