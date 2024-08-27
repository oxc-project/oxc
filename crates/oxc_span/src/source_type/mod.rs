use std::path::Path;

mod types;
use oxc_allocator::{Allocator, CloneIn};
pub use types::*;

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

impl<'a> CloneIn<'a> for SourceType {
    type Cloned = Self;
    #[inline]
    fn clone_in(&self, _: &'a Allocator) -> Self {
        *self
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
        self.language == Language::JavaScript
    }

    /// Returns `true` if this is a TypeScript file or TypeScript definition file.
    ///
    /// I.e., `true` for `.ts`, `.cts`, `.mts`, `.tsx`, and `.d.ts` files.
    pub fn is_typescript(self) -> bool {
        matches!(self.language, Language::TypeScript | Language::TypeScriptDefinition)
    }

    pub fn is_typescript_definition(self) -> bool {
        self.language == Language::TypeScriptDefinition
    }

    pub fn is_jsx(self) -> bool {
        self.variant == LanguageVariant::Jsx
    }

    pub fn always_strict(self) -> bool {
        self.always_strict
    }

    pub fn is_strict(self) -> bool {
        self.is_module() || self.always_strict
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
            self.language = Language::TypeScript;
        }
        self
    }

    #[must_use]
    pub fn with_typescript_definition(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::TypeScriptDefinition;
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

    /// Converts a file [`Path`] to [`SourceType`].
    ///
    /// ## Examples
    /// ```
    /// # use oxc_span::SourceType;
    ///
    /// // supports .ts, .mts, .cts, .tsx, .d.ts, etc.
    /// let ts = SourceType::from_path("foo.ts").unwrap();
    /// assert!(ts.is_typescript());
    /// assert!(!ts.is_typescript_definition());
    ///
    /// // supports .js, .mjs, .cjs, .jsx
    /// let jsx = SourceType::from_path("foo.jsx").unwrap();
    /// assert!(jsx.is_javascript());
    /// assert!(jsx.is_jsx());
    /// ```
    ///
    /// ## Behavior
    /// ### JSX
    /// All JavaScript-like files are treated as JSX, since some tools (like
    /// babel) also do not make a distinction between `.js` and `.jsx`. However,
    /// for TypeScript files, only `.tsx` files are treated as JSX.
    ///
    /// ### Modules vs. Scripts.
    /// Oxc has partial support for Node's
    /// [CommonJS](https://nodejs.org/api/modules.html#enabling) detection
    /// strategy. Any file with a `.c[tj]s` extension is treated as a [`script`].
    /// All other files are treated as [`modules`].
    ///
    /// # Errors
    /// Returns [`UnknownExtension`] if:
    ///   * there is no file name
    ///   * the file extension is not one of "js", "mjs", "cjs", "jsx", "ts",
    ///     "mts", "cts", "tsx". See [`VALID_EXTENSIONS`] for the list of valid
    ///     extensions.
    ///
    /// [`script`]: ModuleKind::Script
    /// [`modules`]: ModuleKind::Module
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
                    format!("Please provide a valid file extension for {path}: .js, .mjs, .jsx or .cjs for JavaScript, or .ts, .d.ts, .mts, .cts or .tsx for TypeScript"),
                )
            })?;

        let (language, module_kind) = match extension {
            "js" | "mjs" | "jsx" => (Language::JavaScript, ModuleKind::Module),
            "cjs" => (Language::JavaScript, ModuleKind::Script),
            "ts" if file_name.ends_with(".d.ts") => {
                (Language::TypeScriptDefinition, ModuleKind::Module)
            }
            "mts" if file_name.ends_with(".d.mts") => {
                (Language::TypeScriptDefinition, ModuleKind::Module)
            }
            "cts" if file_name.ends_with(".d.cts") => {
                (Language::TypeScriptDefinition, ModuleKind::Script)
            }
            "ts" | "mts" | "tsx" => (Language::TypeScript, ModuleKind::Module),
            "cts" => (Language::TypeScript, ModuleKind::Script),
            _ => {
                #[cfg(debug_assertions)]
                unreachable!();
                #[cfg(not(debug_assertions))]
                return Err(UnknownExtension(format!("Unknown extension: {}", extension)));
            }
        };

        let variant = match extension {
            "js" | "mjs" | "cjs" | "jsx" | "tsx" => LanguageVariant::Jsx,
            _ => LanguageVariant::Standard,
        };

        Ok(Self { language, module_kind, variant, always_strict: false })
    }
}

#[cfg(test)]
mod tests {
    use super::SourceType;

    #[test]
    #[allow(clippy::similar_names)]
    fn test_ts() {
        let ts = SourceType::from_path("foo.ts")
            .expect("foo.ts should be a valid TypeScript file path.");
        let mts = SourceType::from_path("foo.mts")
            .expect("foo.mts should be a valid TypeScript file path.");
        let cts = SourceType::from_path("foo.cts")
            .expect("foo.cts should be a valid TypeScript file path.");
        let tsx = SourceType::from_path("foo.tsx")
            .expect("foo.tsx should be a valid TypeScript file path.");

        for ty in &[ts, mts, cts, tsx] {
            assert!(ty.is_typescript());
            assert!(!ty.is_typescript_definition());
            assert!(!ty.is_javascript());
        }

        assert!(ts.is_module());
        assert!(mts.is_module());
        assert!(!cts.is_module());
        assert!(tsx.is_module());

        assert!(!ts.is_script());
        assert!(!mts.is_script());
        assert!(cts.is_script());
        assert!(!tsx.is_script());

        assert!(ts.is_strict());
        assert!(mts.is_strict());
        assert!(!cts.is_strict());
        assert!(tsx.is_strict());

        assert!(!ts.is_jsx());
        assert!(!mts.is_jsx());
        assert!(!cts.is_jsx());
        assert!(tsx.is_jsx());
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn test_d_ts() {
        let dts = SourceType::from_path("foo.d.ts")
            .expect("foo.d.ts should be a valid TypeScript definition file path.");
        let dmts = SourceType::from_path("foo.d.mts")
            .expect("foo.d.mts should be a valid TypeScript definition file path.");
        let dcts = SourceType::from_path("foo.d.cts")
            .expect("foo.d.cts should be a valid TypeScript definition file path.");

        for ty in &[dts, dmts, dcts] {
            assert!(ty.is_typescript());
            assert!(ty.is_typescript_definition());
            assert!(!ty.is_javascript());
        }

        assert!(dts.is_module());
        assert!(dmts.is_module());
        assert!(!dcts.is_module());

        assert!(!dts.is_script());
        assert!(!dmts.is_script());
        assert!(dcts.is_script());

        assert!(dts.is_strict());
        assert!(dmts.is_strict());
        assert!(!dcts.is_strict());

        assert!(!dts.is_jsx());
        assert!(!dmts.is_jsx());
        assert!(!dcts.is_jsx());
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn test_js() {
        let js = SourceType::from_path("foo.js")
            .expect("foo.js should be a valid JavaScript file path.");
        let mjs = SourceType::from_path("foo.mjs")
            .expect("foo.mjs should be a valid JavaScript file path.");
        let cjs = SourceType::from_path("foo.cjs")
            .expect("foo.cjs should be a valid JavaScript file path.");
        let jsx = SourceType::from_path("foo.jsx")
            .expect("foo.jsx should be a valid JavaScript file path.");

        for ty in &[js, mjs, cjs, jsx] {
            assert!(ty.is_javascript(), "{ty:?}");
            assert!(!ty.is_typescript(), "{ty:?}");
        }

        assert!(js.is_module());
        assert!(mjs.is_module());
        assert!(cjs.is_script());
        assert!(jsx.is_module());

        assert!(js.is_strict());
        assert!(mjs.is_strict());
        assert!(!cjs.is_strict());
        assert!(jsx.is_strict());

        assert!(js.is_jsx());
        assert!(mjs.is_jsx());
        assert!(cjs.is_jsx());
        assert!(jsx.is_jsx());
    }
}
