use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// Decorator transform options.
pub struct DecoratorOptions {
    /// Enables experimental support for decorators, which is a version of decorators that predates the TC39 standardization process.
    ///
    /// Decorators are a language feature which hasn’t yet been fully ratified into the JavaScript specification.
    /// This means that the implementation version in TypeScript may differ from the implementation in JavaScript when it is decided by TC39.
    ///
    /// <https://www.typescriptlang.org/tsconfig#experimentalDecorators>
    #[serde(skip)]
    pub legacy: bool,

    /// Lower standard decorators and auto-accessors. Defaults to `true`.
    ///
    /// Set to `false` with an `esnext` target to preserve this syntax for a later transform
    /// while stripping TypeScript types. `legacy: true` takes precedence over this option.
    ///
    /// Target-selected transforms still run. Preserving this syntax requires targets that
    /// keep class fields native; targets that lower class fields report an error for classes
    /// with decorators or auto-accessors.
    #[serde(default = "default_as_true")]
    pub standard: bool,

    /// Enables emitting decorator metadata.
    ///
    /// This option is the same as [emitDecoratorMetadata](https://www.typescriptlang.org/tsconfig/#emitDecoratorMetadata)
    /// in TypeScript, and it only works when `legacy` is true.
    pub emit_decorator_metadata: bool,

    /// Whether the source is compiled under `--strictNullChecks`. Defaults to `true`.
    ///
    /// When `false`, `null` and `undefined` are elided from union `design:type` annotations
    /// (`T | null` emits the constructor of `T` instead of `Object`).
    ///
    /// <https://www.typescriptlang.org/tsconfig#strictNullChecks>
    #[serde(default = "default_as_true")]
    pub strict_null_checks: bool,
}

impl Default for DecoratorOptions {
    fn default() -> Self {
        Self {
            legacy: false,
            standard: default_as_true(),
            emit_decorator_metadata: false,
            strict_null_checks: default_as_true(),
        }
    }
}

fn default_as_true() -> bool {
    true
}

/// Decorator semantics selected by the public transform options.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DecoratorMode {
    Preserve,
    Legacy,
    Standard,
}

impl DecoratorMode {
    /// Whether the legacy (`experimentalDecorators`) semantics are selected.
    pub(crate) fn is_legacy(self) -> bool {
        matches!(self, Self::Legacy)
    }
}

impl DecoratorOptions {
    pub(crate) fn mode(self) -> DecoratorMode {
        if self.legacy {
            DecoratorMode::Legacy
        } else if self.standard {
            DecoratorMode::Standard
        } else {
            DecoratorMode::Preserve
        }
    }
}
