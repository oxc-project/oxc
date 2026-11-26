use oxc_semantic::Scoping;
use serde::Deserialize;

use crate::EnvOptions;

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

    /// Lower standard decorators and auto-accessors.
    ///
    /// When omitted, lowering follows the target: a concrete target lowers them, and `esnext`
    /// or no target preserves them. Set to `false` to preserve this syntax for a later transform
    /// at any target while stripping TypeScript types. Set to `true` to lower it regardless of
    /// target. Other target-selected transforms still run on unrelated syntax.
    /// `legacy: true` takes precedence over this option.
    #[serde(default)]
    pub lower: Option<bool>,

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
            lower: None,
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
    /// The decorator semantics [`Transformer::build_with_scoping`] applies to the program
    /// `scoping` describes.
    ///
    /// [`Transformer::build_with_scoping`]: crate::Transformer::build_with_scoping
    pub(crate) fn mode(self, env: &EnvOptions, scoping: &Scoping) -> DecoratorMode {
        if self.legacy {
            DecoratorMode::Legacy
        } else if self.lower.unwrap_or(env.standard_decorators)
            // Standard decorator lowering makes class-properties visit every class, statement
            // and expression. A program without decorators or `accessor` has nothing for it to
            // lower.
            && scoping.has_decorators_or_auto_accessors()
        {
            DecoratorMode::Standard
        } else {
            DecoratorMode::Preserve
        }
    }

    /// Whether [`Transformer::build_with_scoping`] would transform decorators or `accessor`
    /// properties in the program `scoping` describes, with these options and `env`.
    ///
    /// `scoping` must be built by [`SemanticBuilder`](oxc_semantic::SemanticBuilder) for the
    /// program, as [`Transformer::build_with_scoping`] requires.
    ///
    /// Together with [`EnvOptions::lowers_syntax`], this tells whether a JavaScript program
    /// needs the transformer for syntax lowering at all; see there for what neither covers.
    ///
    /// With `legacy: true`, this is `true` for any program with decorators or `accessor`
    /// properties: legacy decorators lower `accessor` properties too, at any target.
    ///
    /// [`Transformer::build_with_scoping`]: crate::Transformer::build_with_scoping
    pub fn transforms_decorators(self, env: &EnvOptions, scoping: &Scoping) -> bool {
        // Legacy decorators run whatever `has_decorators_or_auto_accessors` says, but they have
        // nothing to do in a program without decorators or `accessor`.
        self.mode(env, scoping) != DecoratorMode::Preserve
            && scoping.has_decorators_or_auto_accessors()
    }
}
