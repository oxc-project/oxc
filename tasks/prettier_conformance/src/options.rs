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
    Jsonc,
    Json5,
}

impl TestLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Js => "js",
            Self::Ts => "ts",
            Self::Json => "json",
            Self::Jsonc => "jsonc",
            Self::Json5 => "json5",
        }
    }

    /// Prettier's test fixtures roots for different languages.
    pub fn fixtures_roots(self, base: &Path) -> Vec<PathBuf> {
        match self {
            Self::Js => ["js", "jsx"].iter().map(|dir| base.join(dir)).collect::<Vec<_>>(),
            // There is no `tsx` directory, just check it works with TS
            // `SourceType`.`variant` is handled by spec file extension
            Self::Ts => ["typescript", "jsx"].iter().map(|dir| base.join(dir)).collect::<Vec<_>>(),
            // For the JSON family (`Json`/`Jsonc`/`Json5`)
            // the `json/` and `with-comment/` dirs are shared:
            // each `format.test.js` call lists its own parser,
            // so `spec.rs` keeps only the calls matching the active language.
            //
            // Out-of-scope siblings (all JSON variants):
            // - `json-superset/`: inline `snippets`, not parseable by Rust(`spec.rs`)
            // - `range/`: range-formatting, not whole-file
            Self::Json => {
                vec![base.join("json").join("json"), base.join("json").join("with-comment")]
            }
            Self::Jsonc => {
                vec![base.join("json").join("jsonc"), base.join("json").join("with-comment")]
            }
            Self::Json5 => vec![
                base.join("json").join("json"),
                base.join("json").join("with-comment"),
                base.join("json").join("json5-as-json-with-trailing-commas"),
            ],
        }
    }
}
