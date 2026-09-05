//! postcss-simple-vars: `$var: value;` declarations and `$var` references in [`crate::CssVariant::Css`] mode
//!
//! postcss-simple-vars is a textual substitution,
//! so the AST is intentionally minimal (no namespace, no `!default`/`!global`, no list semantics).
//! The formatter normalizes spacing around `$name`, `:`, and typed values;
//! a raw value fallback stays verbatim because its bytes are what the plugin substitutes.

use oxc_css_parser::ast::{PostcssSimpleVar, PostcssSimpleVarDeclaration};
use oxc_formatter_core::{
    Buffer,
    builders::{space, text},
    write,
};
use oxc_span::Span;

use crate::{
    format::to_span,
    print::{
        CssFormatter, statement,
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

    let end = to_span(&decl.span).end;
    if decl.value_is_raw && !decl.value.is_empty() {
        // postcss-simple-vars substitutes the value textually,
        // so re-spacing a raw fallback would change the substituted token stream
        // (DIVERGENCES.md "postcss-simple-var-raw-verbatim").
        let value_start = to_span(decl.value[0].span()).start;
        value::write_verbatim_value(Span::new(value_start, end), f);
    } else {
        // The typed value stream (including any trailing `ImportantAnnotation` pushed by the parser)
        // prints like any declaration value:
        // gap-driven rules and Prettier's multi-value list break apply here too.
        value::write_declaration_value(&decl.value, ValueContext::default(), f);
    }
    statement::write_terminator_tail_comments(end, f);
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
