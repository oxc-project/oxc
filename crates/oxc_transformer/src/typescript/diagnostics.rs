use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

pub fn import_equals_require_unsupported(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("`import lib = require(...);` is only supported when compiling modules to CommonJS.\nPlease consider using `import lib from '...';` alongside Typescript's --allowSyntheticDefaultImports option, or add @babel/plugin-transform-modules-commonjs to your Babel config.")
.with_labels([span0.into()])
}

pub fn export_assignment_unsupported(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("`export = <value>;` is only supported when compiling modules to CommonJS.\nPlease consider using `export default <value>;`, or add @babel/plugin-transform-modules-commonjs to your Babel config.")
.with_labels([span0.into()])
}
