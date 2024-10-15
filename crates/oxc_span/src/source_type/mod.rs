// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

mod error;

use std::{hash::Hash, path::Path};

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast_macros::ast;
#[cfg(feature = "serialize")]
use {serde::Serialize, tsify::Tsify};

use crate::{cmp::ContentEq, hash::ContentHash};
pub use error::UnknownExtension;

/// Source Type for JavaScript vs TypeScript / Script vs Module / JSX
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SourceType {
    /// JavaScript or TypeScript, default JavaScript
    pub(super) language: Language,

    /// Script or Module, default Module
    pub(super) module_kind: ModuleKind,

    /// Support JSX for JavaScript and TypeScript? default without JSX
    pub(super) variant: LanguageVariant,
}

/// JavaScript or TypeScript
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Indicates a JavaScript or JSX file
    JavaScript = 0,
    /// Indicates a TypeScript file
    TypeScript = 1,
    /// Indicates a TypeScript definition file (`*.d.ts`)
    #[serde(rename = "typescriptDefinition")]
    TypeScriptDefinition = 2,
}

/// Script or Module
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum ModuleKind {
    /// Regular JS script or CommonJS file
    Script = 0,
    /// ES6 Module
    Module = 1,
    /// Consider the file a "module" if ESM syntax is present, or else consider it a "script".
    ///
    /// ESM syntax includes `import` statement, `export` statement and `import.meta`.
    ///
    /// Note1: This value is only valid as a parser input, and does not appear on a valid AST's `Program::source_type`.
    /// Note2: Dynamic import expression is not ESM syntax.
    ///
    /// See <https://babel.dev/docs/options#misc-options>
    Unambiguous = 2,
}

/// JSX for JavaScript and TypeScript
#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[serde(rename_all = "camelCase")]
pub enum LanguageVariant {
    /// Standard JavaScript or TypeScript without any language extensions. Stage
    /// 3 proposals do not count as language extensions.
    Standard = 0,
    /// For sources using JSX or TSX
    Jsx = 1,
}

impl Default for SourceType {
    #[inline]
    fn default() -> Self {
        Self::mjs()
    }
}

impl<'a> CloneIn<'a> for SourceType {
    type Cloned = Self;

    #[inline]
    fn clone_in(&self, _: &'a Allocator) -> Self {
        *self
    }
}

impl ContentEq for SourceType {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentHash for SourceType {
    #[inline]
    fn content_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

/// Valid file extensions
pub const VALID_EXTENSIONS: [&str; 8] = ["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"];

impl SourceType {
    /// Creates a [`SourceType`] representing a regular [`JavaScript`] file.
    ///
    /// This file could be a vanilla script (no module system of any kind) or a
    /// CommonJS file.
    ///
    /// The resulting source type is not a [`module`], nor does it support [`JSX`].
    /// Use [`SourceType::jsx`] for [`JSX`] sources.
    ///
    /// ## Example
    /// ```
    /// # use oxc_span::SourceType;
    ///
    /// let js = SourceType::cjs();
    /// assert!(js.is_javascript());
    /// assert!(js.is_script()); // not a module
    /// assert!(!js.is_jsx());
    /// ```
    ///
    /// [`JavaScript`]: Language::JavaScript
    /// [`module`]: ModuleKind::Module
    /// [`JSX`]: LanguageVariant::Jsx
    pub const fn cjs() -> Self {
        Self {
            language: Language::JavaScript,
            module_kind: ModuleKind::Script,
            variant: LanguageVariant::Standard,
        }
    }

    /// Creates a [`SourceType`] representing a [`JavaScript`] file using ES6
    /// modules. This is akin to a file with an `.mjs` extension.
    ///
    /// ## Example
    /// ```
    /// # use oxc_span::SourceType;
    ///
    /// let mjs = SourceType::mjs();
    /// ```
    /// [`JavaScript`]: Language::JavaScript
    pub const fn mjs() -> Self {
        Self {
            language: Language::JavaScript,
            module_kind: ModuleKind::Module,
            variant: LanguageVariant::Standard,
        }
    }

    /// A [`SourceType`] that will be treated as a module if it contains ESM syntax.
    ///
    /// After a file is parsed with an `unambiguous` source type, it will have a final
    /// [`ModuleKind`] of either [`Module`] or [`Script`].
    ///
    /// [`Module`]: ModuleKind::Module
    /// [`Script`]: ModuleKind::Script
    pub const fn unambiguous() -> Self {
        Self {
            language: Language::JavaScript,
            module_kind: ModuleKind::Unambiguous,
            variant: LanguageVariant::Standard,
        }
    }

    /// Creates a [`SourceType`] representing a [`JavaScript`]` file with JSX.
    ///
    /// ## Example
    /// ```
    /// # use oxc_span::SourceType;
    ///
    /// let jsx = SourceType::jsx();
    /// assert!(jsx.is_javascript());
    /// assert!(jsx.is_jsx());
    /// ```
    ///
    /// [`JavaScript`]: Language::JavaScript
    pub const fn jsx() -> Self {
        Self::mjs().with_jsx(true)
    }

    /// Creates a [`SourceType`] representing a [`TypeScript`] file.
    ///
    /// Unlike [`SourceType::cjs`], this method creates [`modules`]. Use
    /// [`SourceType::tsx`] for TypeScript files with [`JSX`] support.
    ///
    /// ## Example
    /// ```
    /// # use oxc_span::SourceType;
    ///
    /// let ts = SourceType::ts();
    /// assert!(ts.is_typescript());
    /// assert!(!ts.is_typescript_definition());
    /// assert!(ts.is_module());
    /// assert!(!ts.is_jsx());
    /// ```
    ///
    /// [`TypeScript`]: Language::TypeScript
    /// [`modules`]: ModuleKind::Module
    /// [`JSX`]: LanguageVariant::Jsx
    pub const fn ts() -> Self {
        Self {
            language: Language::TypeScript,
            module_kind: ModuleKind::Module,
            variant: LanguageVariant::Standard,
        }
    }

    /// Creates a [`SourceType`] representing a [`TypeScript`] file with [`JSX`].
    ///
    /// ## Example
    /// ```
    /// # use oxc_span::SourceType;
    ///
    /// let tsx = SourceType::tsx();
    /// assert!(tsx.is_typescript());
    /// assert!(!tsx.is_typescript_definition());
    /// assert!(tsx.is_module());
    /// assert!(tsx.is_jsx());
    /// ```
    ///
    /// [`TypeScript`]: Language::TypeScript
    /// [`JSX`]: LanguageVariant::Jsx
    pub const fn tsx() -> Self {
        Self::ts().with_jsx(true)
    }

    /// Creates a [`SourceType`] representing a [`TypeScript definition`] file.
    ///
    /// ## Example
    /// ```
    /// # use oxc_span::SourceType;
    ///
    /// let dts = SourceType::d_ts();
    /// assert!(dts.is_typescript());
    /// assert!(dts.is_typescript_definition());
    /// assert!(dts.is_module());
    /// assert!(!dts.is_jsx());
    /// ```
    pub const fn d_ts() -> Self {
        Self {
            language: Language::TypeScriptDefinition,
            module_kind: ModuleKind::Module,
            variant: LanguageVariant::Standard,
        }
    }

    /// Mark this source type as a [script].
    ///
    /// [script]: ModuleKind::Script
    pub fn is_script(self) -> bool {
        self.module_kind == ModuleKind::Script
    }

    /// Mark this source type as a [module].
    ///
    /// [module]: ModuleKind::Module
    pub fn is_module(self) -> bool {
        self.module_kind == ModuleKind::Module
    }

    /// `true` if this [`SourceType`] is [unambiguous].
    ///
    /// [unambiguous]: ModuleKind::Unambiguous
    pub fn is_unambiguous(self) -> bool {
        self.module_kind == ModuleKind::Unambiguous
    }

    /// What module system is this source type using?
    pub fn module_kind(self) -> ModuleKind {
        self.module_kind
    }

    /// Returns `true` if this is a JavaScript file with or without syntax
    /// extensions (like JSX).
    pub fn is_javascript(self) -> bool {
        self.language == Language::JavaScript
    }

    /// Returns `true` if this is a TypeScript file or TypeScript definition file.
    ///
    /// I.e., `true` for `.ts`, `.cts`, `.mts`, `.tsx`, and `.d.ts` files.
    pub fn is_typescript(self) -> bool {
        matches!(self.language, Language::TypeScript | Language::TypeScriptDefinition)
    }

    /// Returns `true` if this is a TypeScript definition file (e.g. `.d.ts`).
    pub fn is_typescript_definition(self) -> bool {
        self.language == Language::TypeScriptDefinition
    }

    /// Returns `true` if this source type is using JSX.
    ///
    /// Note that TSX is considered JSX in this context.
    pub fn is_jsx(self) -> bool {
        self.variant == LanguageVariant::Jsx
    }

    /// Does this source type implicitly use strict mode semantics?
    ///
    /// Does not consider `"use strict";` directives.
    pub fn is_strict(self) -> bool {
        self.is_module()
    }

    /// Mark this [`SourceType`] as a [script] if `yes` is `true`. No change
    /// will occur if `yes` is `false`.
    ///
    /// [script]: ModuleKind::Script
    #[must_use]
    pub const fn with_script(mut self, yes: bool) -> Self {
        if yes {
            self.module_kind = ModuleKind::Script;
        }
        self
    }

    /// Mark this [`SourceType`] as a [module] if `yes` is `true`. No change
    /// will occur if `yes` is `false`.
    ///
    /// [module]: ModuleKind::Module
    #[must_use]
    pub const fn with_module(mut self, yes: bool) -> Self {
        if yes {
            self.module_kind = ModuleKind::Module;
        } else {
            self.module_kind = ModuleKind::Script;
        }
        self
    }

    /// Mark this [`SourceType`] as [unambiguous] if `yes` is `true`. No change
    /// will occur if `yes` is `false`.
    ///
    /// [unambiguous]: ModuleKind::Unambiguous
    #[must_use]
    pub const fn with_unambiguous(mut self, yes: bool) -> Self {
        if yes {
            self.module_kind = ModuleKind::Unambiguous;
        }
        self
    }

    /// Mark this [`SourceType`] as using [JavaScript] if `yes` is `true`. No change
    /// will occur if `yes` is `false`.
    ///
    /// [JavaScript]: Language::JavaScript
    #[must_use]
    pub const fn with_javascript(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::JavaScript;
        }
        self
    }

    /// Mark this [`SourceType`] as using [TypeScript] if `yes` is `true`. No change
    /// will occur if `yes` is `false`.
    ///
    /// [TypeScript]: Language::TypeScript
    #[must_use]
    pub const fn with_typescript(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::TypeScript;
        }
        self
    }

    /// Mark this [`SourceType`] as a [TypeScript definition] file if `yes` is `true`.
    #[must_use]
    pub const fn with_typescript_definition(mut self, yes: bool) -> Self {
        if yes {
            self.language = Language::TypeScriptDefinition;
        }
        self
    }

    /// Mark this [`SourceType`] as using [JSX] if `yes` is `true`. No change
    /// will occur if `yes` is `false`.
    ///
    /// When using [TypeScript], this source type now represents a TSX file.
    ///
    /// [JSX]: LanguageVariant::Jsx
    /// [TypeScript]: Language::TypeScript
    #[must_use]
    pub const fn with_jsx(mut self, yes: bool) -> Self {
        if yes {
            self.variant = LanguageVariant::Jsx;
        }
        self
    }

    /// Disable language extensions (e.g. [JSX]) if `yes` is `true`. No change
    /// will occur if `yes` is `false`.
    ///
    /// [JSX]: LanguageVariant::Jsx
    #[must_use]
    pub const fn with_standard(mut self, yes: bool) -> Self {
        if yes {
            self.variant = LanguageVariant::Standard;
        }
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
    /// Note that this behavior deviates from [`SourceType::cjs`], which produces
    /// [`scripts`].
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
    /// [`scripts`]: ModuleKind::Script
    /// [`modules`]: ModuleKind::Module
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, UnknownExtension> {
        let file_name = path
            .as_ref()
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| UnknownExtension::new("Please provide a valid file name."))?;

        let extension = path
            .as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .filter(|s| VALID_EXTENSIONS.contains(s))
            .ok_or_else(|| {
                let path = path.as_ref().to_string_lossy();
                UnknownExtension::new(
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
                return Err(UnknownExtension(format!("Unknown extension: {}", extension).into()));
            }
        };

        let variant = match extension {
            "js" | "mjs" | "cjs" | "jsx" | "tsx" => LanguageVariant::Jsx,
            _ => LanguageVariant::Standard,
        };

        Ok(Self { language, module_kind, variant })
    }
}

#[cfg(test)]
mod tests {
    use super::SourceType;

    #[test]
    #[allow(clippy::similar_names)]
    fn test_ts_from_path() {
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

        assert_eq!(SourceType::ts(), ts);

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
    fn test_d_ts_from_path() {
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

        assert_eq!(SourceType::d_ts(), dts);

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
    fn test_js_from_path() {
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

        assert_eq!(SourceType::jsx(), js);
        assert_eq!(SourceType::jsx().with_module(true), jsx);

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
