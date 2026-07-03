use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

/// Build a type-error diagnostic labelled at `span`.
///
/// This is a convenience template for reporting problems from the checker. As real checks
/// land, prefer dedicated constructors that carry precise, TypeScript-style error codes
/// (for example `TS2322: Type 'string' is not assignable to type 'number'.`), following
/// the pattern used by `oxc_isolated_declarations`.
///
/// ```ignore
/// self.diagnostics.push(type_error("Type 'string' is not assignable to type 'number'.", span));
/// ```
#[cold]
pub fn type_error<M: Into<Cow<'static, str>>>(message: M, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(message).with_label(span)
}
