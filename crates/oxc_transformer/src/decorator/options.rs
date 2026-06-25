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
            emit_decorator_metadata: false,
            strict_null_checks: default_as_true(),
        }
    }
}

fn default_as_true() -> bool {
    true
}
