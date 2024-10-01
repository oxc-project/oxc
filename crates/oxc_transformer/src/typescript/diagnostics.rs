use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

pub fn import_equals_require_unsupported(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`import lib = require(...);` is only supported when compiling modules to CommonJS.\nPlease consider using `import lib from '...';` alongside Typescript's --allowSyntheticDefaultImports option, or add @babel/plugin-transform-modules-commonjs to your Babel config.")
        .with_label(span)
}

pub fn export_assignment_unsupported(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`export = <value>;` is only supported when compiling modules to CommonJS.\nPlease consider using `export default <value>;`, or add @babel/plugin-transform-modules-commonjs to your Babel config.")
        .with_label(span)
}

pub fn ambient_module_nested(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Ambient modules cannot be nested in other modules or namespaces.")
        .with_label(span)
}

pub fn namespace_exporting_non_const(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Namespaces exporting non-const are not supported by Babel. Change to const or see: https://babeljs.io/docs/en/babel-plugin-transform-typescript")
        .with_label(span)
}

pub fn namespace_not_supported(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Namespace not marked type-only declare. Non-declarative namespaces are only supported experimentally in Babel. To enable and review caveats see: https://babeljs.io/docs/en/babel-plugin-transform-typescript")
        .with_label(span)
}
