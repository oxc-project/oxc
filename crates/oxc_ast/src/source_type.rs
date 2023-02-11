use std::path::Path;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceType {
    /// JavaScript or TypeScript, default JavaScript
    language: Language,

    /// Script or Module, default Module
    module_kind: ModuleKind,

    /// support JSX for JavaScript and TypeScript? default without JSX
    variant: LanguageVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    JavaScript,
    TypeScript { is_definition_file: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleKind {
    Script,
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageVariant {
    Standard,
    Jsx,
}

#[derive(Debug, Error)]
pub enum SourceTypeError {
    #[error("Unkown Extension: {0}")]
    UnkownExtension(String),
}

impl Default for SourceType {
    fn default() -> Self {
        Self {
            language: Language::JavaScript,
            module_kind: ModuleKind::Script,
            variant: LanguageVariant::Standard,
        }
    }
}

pub const VALID_EXTENSIONS: [&str; 8] = ["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"];

impl SourceType {
    #[must_use]
    pub fn builder() -> SourceTypeBuilder {
        SourceTypeBuilder::default()
    }

    pub fn set_script(&mut self) {
        self.module_kind = ModuleKind::Script;
    }

    pub fn set_module(&mut self) {
        self.module_kind = ModuleKind::Module;
    }

    pub fn set_typescript(&mut self) {
        self.language = Language::TypeScript { is_definition_file: false };
    }

    pub fn set_typescript_definition(&mut self) {
        self.language = Language::TypeScript { is_definition_file: true };
    }

    pub fn set_jsx(&mut self) {
        self.variant = LanguageVariant::Jsx;
    }

    #[must_use]
    pub fn is_script(self) -> bool {
        self.module_kind == ModuleKind::Script
    }

    #[must_use]
    pub fn is_module(self) -> bool {
        self.module_kind == ModuleKind::Module
    }

    #[must_use]
    pub const fn module_kind(self) -> ModuleKind {
        self.module_kind
    }

    #[must_use]
    pub const fn is_javascript(self) -> bool {
        matches!(self.language, Language::JavaScript)
    }

    #[must_use]
    pub const fn is_typescript(self) -> bool {
        matches!(self.language, Language::TypeScript { .. })
    }

    #[must_use]
    pub const fn is_typescript_definition(self) -> bool {
        matches!(self.language, Language::TypeScript { is_definition_file: true })
    }

    #[must_use]
    pub fn is_jsx(self) -> bool {
        self.variant == LanguageVariant::Jsx
    }

    /// Converts file path to `SourceType`
    /// returns `SourceTypeError::UnkownExtension` if:
    ///   * there is no file name
    ///   * the file extension is not one of "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"
    /// # Errors
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, SourceTypeError> {
        let file_name =
            path.as_ref().file_name().and_then(std::ffi::OsStr::to_str).ok_or_else(|| {
                SourceTypeError::UnkownExtension("Please provide a valid file name.".to_string())
            })?;

        let extension = path
            .as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .filter(|s| VALID_EXTENSIONS.contains(s))
            .ok_or_else(|| {
                let path = path.as_ref().to_string_lossy();
                SourceTypeError::UnkownExtension(
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

        Ok(Self { language, module_kind: ModuleKind::Module, variant })
    }
}

#[derive(Default)]
pub struct SourceTypeBuilder {
    source_type: SourceType,
}

impl SourceTypeBuilder {
    #[must_use]
    pub const fn build(self) -> SourceType {
        self.source_type
    }

    /// turn on `ModuleKind::Module`
    #[must_use]
    pub const fn module(mut self) -> Self {
        self.source_type.module_kind = ModuleKind::Module;
        self
    }

    /// turn on `ModuleKind::Script`
    #[must_use]
    pub const fn script(mut self) -> Self {
        self.source_type.module_kind = ModuleKind::Script;
        self
    }

    /// turn on `LanguageVariant::JSX`
    #[must_use]
    pub const fn jsx(mut self) -> Self {
        self.source_type.variant = LanguageVariant::Jsx;
        self
    }

    /// turn on `Language::TypeScript { is_definition_file: false }`
    #[must_use]
    pub const fn typescript(mut self) -> Self {
        self.source_type.language = Language::TypeScript { is_definition_file: false };
        self
    }

    #[must_use]
    /// turn on `Language::TypeScript { is_definition_file: true }`
    pub const fn typescript_definition(mut self) -> Self {
        self.source_type.language = Language::TypeScript { is_definition_file: true };
        self
    }
}
