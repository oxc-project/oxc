use std::{borrow::Cow, fmt};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

use crate::context::TransformCtx;

fn default_for_jsx_pragma() -> Cow<'static, str> {
    Cow::Borrowed("React.createElement")
}

fn default_for_jsx_pragma_frag() -> Cow<'static, str> {
    Cow::Borrowed("React.Fragment")
}

fn default_as_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeScriptOptions {
    /// Replace the function used when compiling JSX expressions.
    /// This is so that we know that the import is not a type import, and should not be removed.
    /// defaults to React
    #[serde(default = "default_for_jsx_pragma")]
    pub jsx_pragma: Cow<'static, str>,

    /// Replace the function used when compiling JSX fragment expressions.
    /// This is so that we know that the import is not a type import, and should not be removed.
    /// defaults to React.Fragment
    #[serde(default = "default_for_jsx_pragma_frag")]
    pub jsx_pragma_frag: Cow<'static, str>,

    /// When set to true, the transform will only remove type-only imports (introduced in TypeScript 3.8).
    /// This should only be used if you are using TypeScript >= 3.8.
    pub only_remove_type_imports: bool,

    // Enables compilation of TypeScript namespaces.
    #[serde(default = "default_as_true")]
    pub allow_namespaces: bool,

    // When enabled, type-only class fields are only removed if they are prefixed with the declare modifier:
    #[serde(default = "default_as_true")]
    pub allow_declare_fields: bool,

    /// Unused.
    pub optimize_const_enums: bool,

    // Preset options
    /// Modifies extensions in import and export declarations.
    ///
    /// This option, when used together with TypeScript's [`allowImportingTsExtension`](https://www.typescriptlang.org/tsconfig#allowImportingTsExtensions) option,
    /// allows to write complete relative specifiers in import declarations while using the same extension used by the source files.
    ///
    /// When set to `true`, same as [`RewriteExtensionsMode::Rewrite`]. Defaults to `false` (do nothing).
    #[serde(deserialize_with = "deserialize_rewrite_import_extensions")]
    pub rewrite_import_extensions: Option<RewriteExtensionsMode>,
}

impl TypeScriptOptions {
    /// Scan through all comments and find the following pragmas
    ///
    /// * @jsx React.createElement
    /// * @jsxFrag React.Fragment
    ///
    /// The comment does not need to be a jsdoc,
    /// otherwise `JSDoc` could be used instead.
    ///
    /// This behavior is aligned with babel.
    pub(crate) fn update_with_comments(&mut self, ctx: &TransformCtx) {
        for comment in ctx.trivias.comments() {
            let mut comment = comment.span.source_text(ctx.source_text).trim_start();
            // strip leading jsdoc comment `*` and then whitespaces
            while let Some(cur_comment) = comment.strip_prefix('*') {
                comment = cur_comment.trim_start();
            }
            // strip leading `@`
            let Some(comment) = comment.strip_prefix('@') else { continue };

            // read jsxFrag
            if let Some(pragma_frag) = comment.strip_prefix("jsxFrag").map(str::trim) {
                self.jsx_pragma_frag = Cow::from(pragma_frag.to_string());
                continue;
            }

            // Put this condition at the end to avoid breaking @jsxXX
            // read jsx
            if let Some(pragma) = comment.strip_prefix("jsx").map(str::trim) {
                self.jsx_pragma = Cow::from(pragma.to_string());
            }
        }
    }
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            jsx_pragma: default_for_jsx_pragma(),
            jsx_pragma_frag: default_for_jsx_pragma_frag(),
            only_remove_type_imports: false,
            allow_namespaces: default_as_true(),
            allow_declare_fields: default_as_true(),
            optimize_const_enums: false,
            rewrite_import_extensions: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum RewriteExtensionsMode {
    /// Rewrite `.ts`/`.mts`/`.cts` extensions in import/export declarations to `.js`/`.mjs`/`.cjs`.
    #[default]
    Rewrite,
    /// Remove `.ts`/`.mts`/`.cts`/`.tsx` extensions in import/export declarations.
    Remove,
}

impl RewriteExtensionsMode {
    pub fn is_remove(&self) -> bool {
        matches!(self, Self::Remove)
    }
}

pub fn deserialize_rewrite_import_extensions<'de, D>(
    deserializer: D,
) -> Result<Option<RewriteExtensionsMode>, D::Error>
where
    D: Deserializer<'de>,
{
    struct RewriteExtensionsModeVisitor;

    impl<'de> Visitor<'de> for RewriteExtensionsModeVisitor {
        type Value = Option<RewriteExtensionsMode>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("true, false, \"rewrite\", or \"remove\"")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value {
                Ok(Some(RewriteExtensionsMode::Rewrite))
            } else {
                Ok(None)
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                "rewrite" => Ok(Some(RewriteExtensionsMode::Rewrite)),
                "remove" => Ok(Some(RewriteExtensionsMode::Remove)),
                _ => Err(E::custom(format!(
                    "Expected RewriteExtensionsMode is either \"rewrite\" or \"remove\" but found: {value}"
                ))),
            }
        }
    }

    deserializer.deserialize_any(RewriteExtensionsModeVisitor)
}
