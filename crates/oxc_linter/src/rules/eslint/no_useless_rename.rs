use oxc_ast::{
    ast::{AssignmentTarget, AssignmentTargetProperty, BindingPatternKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_useless_rename_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Do not rename import, export, or destructured assignments to the same name",
    )
    .with_help("Use the variable's original name or rename it to a different name")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessRename(Box<NoUselessRenameConfig>);

#[allow(clippy::struct_field_names)]
#[derive(Debug, Default, Clone)]
pub struct NoUselessRenameConfig {
    ignore_destructuring: bool,
    ignore_import: bool,
    ignore_export: bool,
}

impl std::ops::Deref for NoUselessRename {
    type Target = NoUselessRenameConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow renaming import, export, and destructured assignments to the same name.
    ///
    /// ### Why is this bad?
    ///
    /// It is unnecessary to rename a variable to the same name.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import { foo as foo } from 'foo';
    /// const { bar: bar } = obj;
    /// export { baz as baz };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import { foo } from 'foo';
    /// const { bar: renamed } = obj;
    /// export { baz };
    /// ```
    NoUselessRename,
    eslint,
    correctness
);

impl Rule for NoUselessRename {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self(Box::new(NoUselessRenameConfig {
            ignore_destructuring: obj
                .and_then(|v| v.get("ignoreDestructuring"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
            ignore_import: obj
                .and_then(|v| v.get("ignoreImport"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
            ignore_export: obj
                .and_then(|v| v.get("ignoreExport"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectPattern(object_pattern) => {
                if self.ignore_destructuring {
                    return;
                }

                for property in &object_pattern.properties {
                    if property.shorthand || property.computed {
                        continue;
                    }

                    let Some(key) = property.key.static_name() else {
                        continue;
                    };

                    let renamed_key = match &property.value.kind {
                        BindingPatternKind::AssignmentPattern(assignment_pattern) => {
                            match &assignment_pattern.left.kind {
                                BindingPatternKind::BindingIdentifier(binding_ident) => {
                                    binding_ident.name
                                }
                                _ => continue,
                            }
                        }
                        BindingPatternKind::BindingIdentifier(binding_ident) => binding_ident.name,
                        _ => continue,
                    };

                    if key == renamed_key {
                        ctx.diagnostic(no_useless_rename_diagnostic(property.span));
                    }
                }
            }
            AstKind::AssignmentTarget(AssignmentTarget::ObjectAssignmentTarget(
                object_assignment_target,
            )) => {
                if self.ignore_destructuring {
                    return;
                }
                for property in &object_assignment_target.properties {
                    let AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) =
                        property
                    else {
                        continue;
                    };
                    let Some(key) = property.name.static_name() else {
                        continue;
                    };
                    let Some(renamed_key) = property.binding.identifier().map(|ident| ident.name)
                    else {
                        continue;
                    };
                    if key == renamed_key {
                        ctx.diagnostic(no_useless_rename_diagnostic(property.span));
                    }
                }
            }
            AstKind::ImportSpecifier(import_specifier) => {
                if !self.ignore_import
                    && import_specifier.imported.span() != import_specifier.local.span
                    && import_specifier.local.name == import_specifier.imported.name()
                {
                    ctx.diagnostic(no_useless_rename_diagnostic(import_specifier.local.span));
                }
            }
            AstKind::ExportNamedDeclaration(export_named_decl) => {
                if self.ignore_export {
                    return;
                }
                for specifier in &export_named_decl.specifiers {
                    if specifier.local.span() != specifier.exported.span()
                        && specifier.local.name() == specifier.exported.name()
                    {
                        ctx.diagnostic(no_useless_rename_diagnostic(specifier.local.span()));
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"let {foo} = obj;", None),
        (r"let {foo: bar} = obj;", None),
        (r"let {foo: bar, baz: qux} = obj;", None),
        (r"let {foo: {bar: baz}} = obj;", None),
        (r"let {foo, bar: {baz: qux}} = obj;", None),
        (r"let {'foo': bar} = obj;", None),
        (r"let {'foo': bar, 'baz': qux} = obj;", None),
        (r"let {'foo': {'bar': baz}} = obj;", None),
        (r"let {foo, 'bar': {'baz': qux}} = obj;", None),
        (r"let {['foo']: bar} = obj;", None),
        (r"let {['foo']: bar, ['baz']: qux} = obj;", None),
        (r"let {['foo']: {['bar']: baz}} = obj;", None),
        (r"let {foo, ['bar']: {['baz']: qux}} = obj;", None),
        (r"let {[foo]: foo} = obj;", None),
        (r"let {['foo']: foo} = obj;", None),
        (r"let {[foo]: bar} = obj;", None),
        (r"function func({foo}) {}", None),
        (r"function func({foo: bar}) {}", None),
        (r"function func({foo: bar, baz: qux}) {}", None),
        (r"({foo}) => {}", None),
        (r"({foo: bar}) => {}", None),
        (r"({foo: bar, baz: qui}) => {}", None),
        (r"import * as foo from 'foo';", None),
        (r"import foo from 'foo';", None),
        (r"import {foo} from 'foo';", None),
        (r"import {foo as bar} from 'foo';", None),
        (r"import {foo as bar, baz as qux} from 'foo';", None),
        (r"import {'foo' as bar} from 'baz';", None),
        (r"export {foo} from 'foo';", None),
        (r"var foo = 0;export {foo as bar};", None),
        (r"var foo = 0; var baz = 0; export {foo as bar, baz as qux};", None),
        (r"export {foo as bar} from 'foo';", None),
        (r"export {foo as bar, baz as qux} from 'foo';", None),
        (r"var foo = 0; export {foo as 'bar'};", None),
        (r"export {foo as 'bar'} from 'baz';", None),
        (r"export {'foo' as bar} from 'baz';", None),
        (r"export {'foo' as 'bar'} from 'baz';", None),
        (r"export {'' as ' '} from 'baz';", None),
        (r"export {' ' as ''} from 'baz';", None),
        (r"export {'foo'} from 'bar';", None),
        (r"const {...stuff} = myObject;", None),
        (r"const {foo, ...stuff} = myObject;", None),
        (r"const {foo: bar, ...stuff} = myObject;", None),
        (r"let {foo: foo} = obj;", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
        (
            r"let {foo: foo, bar: baz} = obj;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            r"let {foo: foo, bar: bar} = obj;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (r"import {foo as foo} from 'foo';", Some(serde_json::json!([{ "ignoreImport": true }]))),
        (
            r"import {foo as foo, bar as baz} from 'foo';",
            Some(serde_json::json!([{ "ignoreImport": true }])),
        ),
        (
            r"import {foo as foo, bar as bar} from 'foo';",
            Some(serde_json::json!([{ "ignoreImport": true }])),
        ),
        (r"var foo = 0;export {foo as foo};", Some(serde_json::json!([{ "ignoreExport": true }]))),
        (
            r"var foo = 0;var bar = 0;export {foo as foo, bar as baz};",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        (
            r"var foo = 0;var bar = 0;export {foo as foo, bar as bar};",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        (r"export {foo as foo} from 'foo';", Some(serde_json::json!([{ "ignoreExport": true }]))),
        (
            r"export {foo as foo, bar as baz} from 'foo';",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        (
            r"export {foo as foo, bar as bar} from 'foo';",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        (r"const { ...foo } = bar;", None),
    ];

    let fail = vec![
        (r"let {foo: foo} = obj;", None),
        (r"({foo: (foo)} = obj);", None),
        (r"let {\u0061: a} = obj;", None),
        (r"let {a: \u0061} = obj;", None),
        (r"let {\u0061: \u0061} = obj;", None),
        (r"let {a, foo: foo} = obj;", None),
        (r"let {foo: foo, bar: baz} = obj;", None),
        (r"let {foo: bar, baz: baz} = obj;", None),
        (r"let {foo: foo, bar: bar} = obj;", None),
        (r"let {foo: {bar: bar}} = obj;", None),
        (r"let {foo: {bar: bar}, baz: baz} = obj;", None),
        (r"let {'foo': foo} = obj;", None),
        (r"let {'foo': foo, 'bar': baz} = obj;", None),
        (r"let {'foo': bar, 'baz': baz} = obj;", None),
        (r"let {'foo': foo, 'bar': bar} = obj;", None),
        (r"let {'foo': {'bar': bar}} = obj;", None),
        (r"let {'foo': {'bar': bar}, 'baz': baz} = obj;", None),
        (r"let {foo: foo = 1, 'bar': bar = 1, baz: baz} = obj;", None),
        (r"let {foo: {bar: bar = 1, 'baz': baz = 1}} = obj;", None),
        (r"let {foo: {bar: bar = {}} = {}} = obj;", None),
        (r"({foo: (foo) = a} = obj);", None),
        (r"let {foo: foo = (a)} = obj;", None),
        (r"let {foo: foo = (a, b)} = obj;", None),
        (r"function func({foo: foo}) {}", None),
        (r"function func({foo: foo, bar: baz}) {}", None),
        (r"function func({foo: bar, baz: baz}) {}", None),
        (r"function func({foo: foo, bar: bar}) {}", None),
        (r"function func({foo: foo = 1, 'bar': bar = 1, baz: baz}) {}", None),
        (r"function func({foo: {bar: bar = 1, 'baz': baz = 1}}) {}", None),
        (r"function func({foo: {bar: bar = {}} = {}}) {}", None),
        (r"({foo: foo}) => {}", None),
        (r"({foo: foo, bar: baz}) => {}", None),
        (r"({foo: bar, baz: baz}) => {}", None),
        (r"({foo: foo, bar: bar}) => {}", None),
        (r"({foo: foo = 1, 'bar': bar = 1, baz: baz}) => {}", None),
        (r"({foo: {bar: bar = 1, 'baz': baz = 1}}) => {}", None),
        (r"({foo: {bar: bar = {}} = {}}) => {}", None),
        (r"const {foo: foo, ...stuff} = myObject;", None),
        (r"const {foo: foo, bar: baz, ...stuff} = myObject;", None),
        (r"const {foo: foo, bar: bar, ...stuff} = myObject;", None),
        (r"import {foo as foo} from 'foo';", None),
        (r"import {'foo' as foo} from 'foo';", None),
        (r"import {\u0061 as a} from 'foo';", None),
        (r"import {a as \u0061} from 'foo';", None),
        (r"import {\u0061 as \u0061} from 'foo';", None),
        (r"import {foo as foo, bar as baz} from 'foo';", None),
        (r"import {foo as bar, baz as baz} from 'foo';", None),
        (r"import {foo as foo, bar as bar} from 'foo';", None),
        (r"var foo = 0; export {foo as foo};", None),
        (r"var foo = 0; export {foo as 'foo'};", None),
        (r"export {foo as 'foo'} from 'bar';", None),
        (r"export {'foo' as foo} from 'bar';", None),
        (r"export {'foo' as 'foo'} from 'bar';", None),
        (r"export {' 👍 ' as ' 👍 '} from 'bar';", None),
        (r"export {'' as ''} from 'bar';", None),
        (r"var a = 0; export {a as \u0061};", None),
        (r"var \u0061 = 0; export {\u0061 as a};", None),
        (r"var \u0061 = 0; export {\u0061 as \u0061};", None),
        (r"var foo = 0; var bar = 0; export {foo as foo, bar as baz};", None),
        (r"var foo = 0; var baz = 0; export {foo as bar, baz as baz};", None),
        (r"var foo = 0; var bar = 0;export {foo as foo, bar as bar};", None),
        (r"export {foo as foo} from 'foo';", None),
        (r"export {a as \u0061} from 'foo';", None),
        (r"export {\u0061 as a} from 'foo';", None),
        (r"export {\u0061 as \u0061} from 'foo';", None),
        (r"export {foo as foo, bar as baz} from 'foo';", None),
        (r"var foo = 0; var bar = 0; export {foo as bar, baz as baz} from 'foo';", None),
        (r"export {foo as foo, bar as bar} from 'foo';", None),
        (r"({/* comment */foo: foo} = {});", None),
        (r"({/* comment */foo: foo = 1} = {});", None),
        (r"({foo, /* comment */bar: bar} = {});", None),
        (r"({foo/**/ : foo} = {});", None),
        (r"({foo/**/ : foo = 1} = {});", None),
        (r"({foo /**/: foo} = {});", None),
        (r"({foo /**/: foo = 1} = {});", None),
        (
            r"({foo://
			foo} = {});",
            None,
        ),
        (r"({foo: /**/foo} = {});", None),
        (r"({foo: (/**/foo)} = {});", None),
        (r"({foo: (foo/**/)} = {});", None),
        (
            r"({foo: (foo //
			)} = {});",
            None,
        ),
        (r"({foo: /**/foo = 1} = {});", None),
        (r"({foo: (/**/foo) = 1} = {});", None),
        (r"({foo: (foo/**/) = 1} = {});", None),
        (r"({foo: foo/* comment */} = {});", None),
        (
            r"({foo: foo//comment
			,bar} = {});",
            None,
        ),
        (r"({foo: foo/* comment */ = 1} = {});", None),
        (
            r"({foo: foo // comment
			 = 1} = {});",
            None,
        ),
        (r"({foo: foo = /* comment */ 1} = {});", None),
        (
            r"({foo: foo = // comment
			 1} = {});",
            None,
        ),
        (r"({foo: foo = (1/* comment */)} = {});", None),
        (r"import {/* comment */foo as foo} from 'foo';", None),
        (r"import {foo,/* comment */bar as bar} from 'foo';", None),
        (r"import {foo/**/ as foo} from 'foo';", None),
        (r"import {foo /**/as foo} from 'foo';", None),
        (
            r"import {foo //
			as foo} from 'foo';",
            None,
        ),
        (r"import {foo as/**/foo} from 'foo';", None),
        (r"import {foo as foo/* comment */} from 'foo';", None),
        (r"import {foo as foo/* comment */,bar} from 'foo';", None),
        (r"let foo; export {/* comment */foo as foo};", None),
        (r"let foo, bar; export {foo,/* comment */bar as bar};", None),
        (r"let foo; export {foo/**/as foo};", None),
        (r"let foo; export {foo as/**/ foo};", None),
        (r"let foo; export {foo as /**/foo};", None),
        (
            r"let foo; export {foo as//comment
			 foo};",
            None,
        ),
        (r"let foo; export {foo as foo/* comment*/};", None),
        (r"let foo, bar; export {foo as foo/* comment*/,bar};", None),
        (
            r"let foo, bar; export {foo as foo//comment
			,bar};",
            None,
        ),
    ];

    Tester::new(NoUselessRename::NAME, NoUselessRename::PLUGIN, pass, fail).test_and_snapshot();
}
