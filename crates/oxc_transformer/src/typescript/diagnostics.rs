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

pub fn ambient_module_nested(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("Ambient modules cannot be nested in other modules or namespaces.")
        .with_label(span0)
}

pub fn type_assertion_reserved(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("This syntax is reserved in files with the .mts or .cts extension. Use an `as` expression instead.").with_label(span)
}

pub fn type_parameters_reserved(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warning("This syntax is reserved in files with the .mts or .cts extension. Add a trailing comma, as in `<T,>() => ...`.").with_label(span)
}
