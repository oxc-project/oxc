use oxc_diagnostics::OxcDiagnostic;

/// ECMAScript target version for compiler options.
///
/// Unlike `oxc_syntax::ESTarget` (used by the transformer pipeline, which
/// does not support ES5), this enum covers the full range of tsc targets
/// and is used for compiler-option validation (e.g. deprecated target warnings).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptTarget {
    ES3,
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ES2024,
    ES2025,
    ES2026,
    ESNext,
}

impl ScriptTarget {
    /// Parse a target string (case-insensitive) into a `ScriptTarget`.
    pub fn from_str_option(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "es3" => Some(Self::ES3),
            "es5" => Some(Self::ES5),
            "es6" | "es2015" => Some(Self::ES2015),
            "es2016" => Some(Self::ES2016),
            "es2017" => Some(Self::ES2017),
            "es2018" => Some(Self::ES2018),
            "es2019" => Some(Self::ES2019),
            "es2020" => Some(Self::ES2020),
            "es2021" => Some(Self::ES2021),
            "es2022" => Some(Self::ES2022),
            "es2023" => Some(Self::ES2023),
            "es2024" => Some(Self::ES2024),
            "es2025" => Some(Self::ES2025),
            "es2026" => Some(Self::ES2026),
            "esnext" => Some(Self::ESNext),
            _ => None,
        }
    }

    /// The canonical display name used in diagnostics (e.g. `"ES5"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ES3 => "ES3",
            Self::ES5 => "ES5",
            Self::ES2015 => "ES2015",
            Self::ES2016 => "ES2016",
            Self::ES2017 => "ES2017",
            Self::ES2018 => "ES2018",
            Self::ES2019 => "ES2019",
            Self::ES2020 => "ES2020",
            Self::ES2021 => "ES2021",
            Self::ES2022 => "ES2022",
            Self::ES2023 => "ES2023",
            Self::ES2024 => "ES2024",
            Self::ES2025 => "ES2025",
            Self::ES2026 => "ES2026",
            Self::ESNext => "ESNext",
        }
    }
}

/// Project-level compiler options that affect validation and pipeline behavior.
///
/// This is distinct from `CheckerOptions` (which controls type-checking
/// behavior like strict mode). `CompilerOptions` holds settings such as
/// `target` and (in the future) `module`, `moduleResolution`, etc. that
/// determine which lib files are loaded, how modules are resolved, and
/// whether deprecated-option diagnostics are emitted.
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub target: Option<ScriptTarget>,
}

/// Validate compiler options, returning any diagnostics.
///
/// This checks for deprecated or removed option values and emits the
/// appropriate tsc-compatible error codes (e.g. TS5107 for deprecated
/// enum-valued options like `target=ES5`).
pub fn validate_compiler_options(options: &CompilerOptions) -> Vec<OxcDiagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(target) = options.target {
        // TS5107: deprecated enum-valued compiler options.
        // tsc deprecation schedule: ES5 deprecated in 5.5, removed in 7.0,
        // silenced by ignoreDeprecations: "6.0".
        if target == ScriptTarget::ES5 {
            diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Option 'target={}' is deprecated and will stop functioning in TypeScript 7.0. \
                     Specify compilerOption '\"ignoreDeprecations\": \"6.0\"' to silence this error.",
                    target.as_str()
                ))
                .with_error_code("ts", "5107"),
            );
        }
    }

    diagnostics
}
