use std::path::Path;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Source Type for JavaScript vs TypeScript / Script vs Module / JSX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub struct SourceType {
    /// JavaScript or TypeScript, default JavaScript
    language: Language,

    /// Script or Module, default Module
    module_kind: ModuleKind,

    /// Support JSX for JavaScript and TypeScript? default without JSX
    variant: LanguageVariant,

    /// Mark strict mode as always strict
    /// See <https://github.com/tc39/test262/blob/main/INTERPRETING.md#strict-mode>
    always_strict: bool,
}

/// JavaScript or TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub enum Language {
    JavaScript,
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    TypeScript {
        is_definition_file: bool,
    },
}

/// Script or Module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub enum ModuleKind {
    Script,
    Module,
}

/// JSX for JavaScript and TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub enum LanguageVariant {
    Standard,
    Jsx,
}

#[derive(Debug)]
pub struct UnknownExtension(pub String);

impl Default for SourceType {
    fn default() -> Self {
        Self {
            language: Language::JavaScript,
            module_kind: ModuleKind::Script,
            variant: LanguageVariant::Standard,
            always_strict: false,
        }
    }
}

/// Valid file extensions
pub const VALID_EXTENSIONS: [&str; 8] = ["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"];

impl SourceType {
    pub fn is_script(self) -> bool {
        self.module_kind == ModuleKind::Script
    }

    pub fn is_module(self) -> bool {
        self.module_kind == ModuleKind::Module
    }

    pub fn module_kind(self) -> ModuleKind {
        self.module_kind
    }

    pub fn is_javascript(self) -> bool {
        matches!(self.language, Language::JavaScript)
    }

    pub fn is_typescript(self) -> bool {
        matches!(self.language, Language::TypeScript { .. })
    }

    pub fn is_typescript_definition(self) -> bool {
        matches!(self.language, Language::TypeScript { is_definition_file: true })
    }

    pub fn is_jsx(self) -> bool {
        self.variant == LanguageVariant::Jsx
    }

    pub fn always_strict(self) -> bool {
        self.always_strict
    }

    #[must_use]
    pub fn with_script(mut self, yes: bool) -> Self {
        if yes {
            self.module_kind = ModuleKind::Script;
        }
        self
    }

    #[must_use]
    pub fn with_module(mut self, yes: bool) -> Self {
        if yes {
            self.module_kind = ModuleKind::Module;
        } else {
            self.module_kind = ModuleKind::Script;
        }
        self
    }

    #[must_use]
    pub fn with_typescript(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::TypeScript { is_definition_file: false };
        }
        self
    }

    #[must_use]
    pub fn with_typescript_definition(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::TypeScript { is_definition_file: true };
        }
        self
    }

    #[must_use]
    pub fn with_jsx(mut self, yes: bool) -> Self {
        if yes {
            self.variant = LanguageVariant::Jsx;
        }
        self
    }

    #[must_use]
    pub fn with_always_strict(mut self, yes: bool) -> Self {
        self.always_strict = yes;
        self
    }

    /// Converts file path to `SourceType`
    /// returns `SourceTypeError::UnknownExtension` if:
    ///   * there is no file name
    ///   * the file extension is not one of "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"
    /// # Errors
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, UnknownExtension> {
        let file_name = path
            .as_ref()
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| UnknownExtension("Please provide a valid file name.".to_string()))?;

        let extension = path
            .as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .filter(|s| VALID_EXTENSIONS.contains(s))
            .ok_or_else(|| {
                let path = path.as_ref().to_string_lossy();
                UnknownExtension(
                    format!("Please provide a valid file extension for {path}: .js, .mjs, .jsx or .cjs for JavaScript, or .ts, .mts, .cts or .tsx for TypeScript"),
                )
            })?;

        let is_definition_file = file_name.ends_with(".d.ts")
            || file_name.ends_with(".d.mts")
            || file_name.ends_with(".d.cts");

        let language = match extension {
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => Language::TypeScript { is_definition_file },
            _ => unreachable!(),
        };

        let variant = match extension {
            "js" | "mjs" | "cjs" | "jsx" | "tsx" => LanguageVariant::Jsx,
            _ => LanguageVariant::Standard,
        };

        Ok(Self { language, module_kind: ModuleKind::Module, variant, always_strict: false })
    }
}
