//! postcss-simple-vars: `$var: value;` declarations and `$var` references
//! in [`crate::CssVariant::Css`] mode
//! (auto-enabled for Css variant via `ParserOptions::allow_postcss_simple_vars`).
//!
//! postcss-simple-vars is a textual substitution,
//! so the AST is intentionally minimal (no namespace, no `!default`/`!global`, no list semantics).
//! The formatter only normalizes spacing around `$name`, `:`, and the value stream,
//! anything more would diverge from the plugin's runtime behavior.

use oxc_css_parser::ast::{PostcssSimpleVar, PostcssSimpleVarDeclaration};

use oxc_formatter_core::{
    Buffer,
    builders::{space, text},
    write,
};

use crate::{
    format::to_span,
    print::{
        CssFormatter,
        value::{self, ValueContext},
    },
};

/// `$var: value;`
pub(super) fn write_postcss_simple_var_declaration<'a>(
    decl: &PostcssSimpleVarDeclaration<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    write_postcss_simple_var(&decl.name, f);
    write!(f, ":");
    write!(f, space());

    // The value stream (including any trailing `ImportantAnnotation` pushed by the parser)
    // prints like any declaration value:
    // gap-driven rules (word-glued numbers like `sandstone.10`),
    // and Prettier's multi-value list break (`$fontFamily:\n  "Lato",\n  ...`) apply here too.
    value::write_declaration_value(&decl.value, ValueContext::default(), f);
}

/// `$var` reference in value position.
pub(super) fn write_postcss_simple_var<'a>(
    variable: &PostcssSimpleVar<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let span = to_span(&variable.span);
    write!(f, text(source.text_for(&span)));
}
