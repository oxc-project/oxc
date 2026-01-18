use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTargetMaybeDefault, AssignmentTargetProperty, AssignmentTargetPropertyProperty,
        BindingPattern, BindingProperty,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn no_useless_rename_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Do not rename import, export, or destructured assignments to the same name",
    )
    .with_help("Use the variable's original name or rename it to a different name")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoUselessRename(Box<NoUselessRenameConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoUselessRenameConfig {
    /// When set to `true`, allows using the same name in destructurings.
    ignore_destructuring: bool,
    /// When set to `true`, allows renaming imports to the same name.
    ignore_import: bool,
    /// When set to `true`, allows renaming exports to the same name.
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
    /// ### Examples
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
    correctness,
    fix,
    config = NoUselessRenameConfig,
);

impl Rule for NoUselessRename {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
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

                    let renamed_key = match &property.value {
                        BindingPattern::AssignmentPattern(assignment_pattern) => {
                            match &assignment_pattern.left {
                                BindingPattern::BindingIdentifier(binding_ident) => {
                                    binding_ident.name
                                }
                                _ => continue,
                            }
                        }
                        BindingPattern::BindingIdentifier(binding_ident) => binding_ident.name,
                        _ => continue,
                    };

                    if key == renamed_key {
                        ctx.diagnostic_with_fix(
                            no_useless_rename_diagnostic(property.span),
                            |fixer| fix_object_pattern_property(fixer, property, ctx),
                        );
                    }
                }
            }

            AstKind::ObjectAssignmentTarget(object_assignment_target) => {
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
                        ctx.diagnostic_with_fix(
                            no_useless_rename_diagnostic(property.span),
                            |fixer| fix_object_assignment_target_property(fixer, property, ctx),
                        );
                    }
                }
            }
            AstKind::ImportSpecifier(import_specifier) => {
                if !self.ignore_import
                    && import_specifier.imported.span() != import_specifier.local.span
                    && import_specifier.local.name == import_specifier.imported.name()
                {
                    ctx.diagnostic_with_fix(
                        no_useless_rename_diagnostic(import_specifier.local.span),
                        |fixer| {
                            // Always replace the entire specifier with just the local identifier
                            // The local identifier is the name that will be used in the code
                            let local_text = import_specifier
                                .local
                                .span
                                .source_text(ctx.source_text())
                                .to_string();
                            fixer.replace(import_specifier.span, local_text)
                        },
                    );
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
                        ctx.diagnostic_with_fix(
                            no_useless_rename_diagnostic(specifier.local.span()),
                            |fixer| {
                                // Always replace the entire specifier with just the local part
                                // The local is what the variable is named inside the module
                                let local_text = specifier
                                    .local
                                    .span()
                                    .source_text(ctx.source_text())
                                    .to_string();
                                fixer.replace(specifier.span, local_text)
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// Fix for object pattern properties like `{foo: foo}` -> `{foo}`.
///
/// For properties with default values like `{foo: foo = 1}`, we produce `{foo = 1}`.
fn fix_object_pattern_property<'c, 'a: 'c>(
    fixer: RuleFixer<'c, 'a>,
    property: &BindingProperty<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let (ident_span, default_span): (Span, Option<Span>) = match &property.value {
        BindingPattern::AssignmentPattern(assignment_pattern) => {
            match &assignment_pattern.left {
                BindingPattern::BindingIdentifier(binding_ident) => {
                    // For `{foo: foo = 1}`, we need `{foo = 1}`
                    // ident_span is `foo`, default is from `=` onwards
                    let default_start = binding_ident.span.end;
                    let default_end = assignment_pattern.span.end;
                    (binding_ident.span, Some(Span::new(default_start, default_end)))
                }
                _ => return fixer.noop(),
            }
        }
        BindingPattern::BindingIdentifier(binding_ident) => (binding_ident.span, None),
        _ => return fixer.noop(),
    };

    let ident_name: &str = ident_span.source_text(ctx.source_text());

    let replacement = if let Some(default_span) = default_span {
        let default_text: &str = default_span.source_text(ctx.source_text());
        format!("{ident_name}{default_text}")
    } else {
        ident_name.to_string()
    };

    fixer.replace(property.span, replacement)
}

/// Fix for object assignment target properties like `({foo: foo} = obj)` -> `({foo} = obj)`.
fn fix_object_assignment_target_property<'c, 'a: 'c>(
    fixer: RuleFixer<'c, 'a>,
    property: &AssignmentTargetPropertyProperty<'a>,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let (ident_span, default_span): (Span, Option<Span>) = match &property.binding {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
            if with_default.binding.get_identifier_name().is_some() {
                // For assignment targets, we need to get the span from the binding itself
                let ident_span = with_default.binding.span();
                let default_start = ident_span.end;
                let default_end = with_default.span.end;
                (ident_span, Some(Span::new(default_start, default_end)))
            } else {
                return fixer.noop();
            }
        }
        _ => {
            if let Some(ident) = property.binding.identifier() {
                (ident.span, None)
            } else {
                return fixer.noop();
            }
        }
    };

    let ident_name: &str = ident_span.source_text(ctx.source_text());

    let replacement = if let Some(default_span) = default_span {
        let default_text: &str = default_span.source_text(ctx.source_text());
        format!("{ident_name}{default_text}")
    } else {
        ident_name.to_string()
    };

    fixer.replace(property.span, replacement)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let {foo} = obj;", None),
        ("let {foo: bar} = obj;", None),
        ("let {foo: bar, baz: qux} = obj;", None),
        ("let {foo: {bar: baz}} = obj;", None),
        ("let {foo, bar: {baz: qux}} = obj;", None),
        ("let {'foo': bar} = obj;", None),
        ("let {'foo': bar, 'baz': qux} = obj;", None),
        ("let {'foo': {'bar': baz}} = obj;", None),
        ("let {foo, 'bar': {'baz': qux}} = obj;", None),
        ("let {['foo']: bar} = obj;", None),
        ("let {['foo']: bar, ['baz']: qux} = obj;", None),
        ("let {['foo']: {['bar']: baz}} = obj;", None),
        ("let {foo, ['bar']: {['baz']: qux}} = obj;", None),
        ("let {[foo]: foo} = obj;", None),
        ("let {['foo']: foo} = obj;", None),
        ("let {[foo]: bar} = obj;", None),
        ("function func({foo}) {}", None),
        ("function func({foo: bar}) {}", None),
        ("function func({foo: bar, baz: qux}) {}", None),
        ("({foo}) => {}", None),
        ("({foo: bar}) => {}", None),
        ("({foo: bar, baz: qui}) => {}", None),
        ("import * as foo from 'foo';", None),
        ("import foo from 'foo';", None),
        ("import {foo} from 'foo';", None),
        ("import {foo as bar} from 'foo';", None),
        ("import {foo as bar, baz as qux} from 'foo';", None),
        ("import {'foo' as bar} from 'baz';", None), // { "ecmaVersion": 2022 },
        ("export {foo} from 'foo';", None),
        ("var foo = 0;export {foo as bar};", None),
        ("var foo = 0; var baz = 0; export {foo as bar, baz as qux};", None),
        ("export {foo as bar} from 'foo';", None),
        ("export {foo as bar, baz as qux} from 'foo';", None),
        ("var foo = 0; export {foo as 'bar'};", None), // { "ecmaVersion": 2022 },
        ("export {foo as 'bar'} from 'baz';", None),   // { "ecmaVersion": 2022 },
        ("export {'foo' as bar} from 'baz';", None),   // { "ecmaVersion": 2022 },
        ("export {'foo' as 'bar'} from 'baz';", None), // { "ecmaVersion": 2022 },
        ("export {'' as ' '} from 'baz';", None),      // { "ecmaVersion": 2022 },
        ("export {' ' as ''} from 'baz';", None),      // { "ecmaVersion": 2022 },
        ("export {'foo'} from 'bar';", None),          // { "ecmaVersion": 2022 },
        ("const {...stuff} = myObject;", None),        // { "ecmaVersion": 2018 },
        ("const {foo, ...stuff} = myObject;", None),   // { "ecmaVersion": 2018 },
        ("const {foo: bar, ...stuff} = myObject;", None), // { "ecmaVersion": 2018 },
        ("let {foo: foo} = obj;", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
        (
            "let {foo: foo, bar: baz} = obj;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "let {foo: foo, bar: bar} = obj;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("import {foo as foo} from 'foo';", Some(serde_json::json!([{ "ignoreImport": true }]))),
        (
            "import {foo as foo, bar as baz} from 'foo';",
            Some(serde_json::json!([{ "ignoreImport": true }])),
        ),
        (
            "import {foo as foo, bar as bar} from 'foo';",
            Some(serde_json::json!([{ "ignoreImport": true }])),
        ),
        ("var foo = 0;export {foo as foo};", Some(serde_json::json!([{ "ignoreExport": true }]))),
        (
            "var foo = 0;var bar = 0;export {foo as foo, bar as baz};",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        (
            "var foo = 0;var bar = 0;export {foo as foo, bar as bar};",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        ("export {foo as foo} from 'foo';", Some(serde_json::json!([{ "ignoreExport": true }]))),
        (
            "export {foo as foo, bar as baz} from 'foo';",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        (
            "export {foo as foo, bar as bar} from 'foo';",
            Some(serde_json::json!([{ "ignoreExport": true }])),
        ),
        ("const { ...foo } = bar;", None), // { "parser": require("../../fixtures/parsers/babel-eslint10/object-pattern-with-rest-element"), }
    ];

    let fail = vec![
        ("let {foo: foo} = obj;", None),
        ("({foo: (foo)} = obj);", None),
        (r"let {\u0061: a} = obj;", None),
        (r"let {a: \u0061} = obj;", None),
        (r"let {\u0061: \u0061} = obj;", None),
        ("let {a, foo: foo} = obj;", None),
        ("let {foo: foo, bar: baz} = obj;", None),
        ("let {foo: bar, baz: baz} = obj;", None),
        ("let {foo: foo, bar: bar} = obj;", None),
        ("let {foo: {bar: bar}} = obj;", None),
        ("let {foo: {bar: bar}, baz: baz} = obj;", None),
        ("let {'foo': foo} = obj;", None),
        ("let {'foo': foo, 'bar': baz} = obj;", None),
        ("let {'foo': bar, 'baz': baz} = obj;", None),
        ("let {'foo': foo, 'bar': bar} = obj;", None),
        ("let {'foo': {'bar': bar}} = obj;", None),
        ("let {'foo': {'bar': bar}, 'baz': baz} = obj;", None),
        ("let {foo: foo = 1, 'bar': bar = 1, baz: baz} = obj;", None),
        ("let {foo: {bar: bar = 1, 'baz': baz = 1}} = obj;", None),
        ("let {foo: {bar: bar = {}} = {}} = obj;", None),
        ("({foo: (foo) = a} = obj);", None),
        ("let {foo: foo = (a)} = obj;", None),
        ("let {foo: foo = (a, b)} = obj;", None),
        ("function func({foo: foo}) {}", None),
        ("function func({foo: foo, bar: baz}) {}", None),
        ("function func({foo: bar, baz: baz}) {}", None),
        ("function func({foo: foo, bar: bar}) {}", None),
        ("function func({foo: foo = 1, 'bar': bar = 1, baz: baz}) {}", None),
        ("function func({foo: {bar: bar = 1, 'baz': baz = 1}}) {}", None),
        ("function func({foo: {bar: bar = {}} = {}}) {}", None),
        ("({foo: foo}) => {}", None),
        ("({foo: foo, bar: baz}) => {}", None),
        ("({foo: bar, baz: baz}) => {}", None),
        ("({foo: foo, bar: bar}) => {}", None),
        ("({foo: foo = 1, 'bar': bar = 1, baz: baz}) => {}", None),
        ("({foo: {bar: bar = 1, 'baz': baz = 1}}) => {}", None),
        ("({foo: {bar: bar = {}} = {}}) => {}", None),
        ("const {foo: foo, ...stuff} = myObject;", None), // { "ecmaVersion": 2018 },
        ("const {foo: foo, bar: baz, ...stuff} = myObject;", None), // { "ecmaVersion": 2018 },
        ("const {foo: foo, bar: bar, ...stuff} = myObject;", None), // { "ecmaVersion": 2018 },
        ("import {foo as foo} from 'foo';", None),
        ("import {'foo' as foo} from 'foo';", None), // { "ecmaVersion": 2022 },
        (r"import {\u0061 as a} from 'foo';", None),
        (r"import {a as \u0061} from 'foo';", None),
        (r"import {\u0061 as \u0061} from 'foo';", None),
        ("import {foo as foo, bar as baz} from 'foo';", None),
        ("import {foo as bar, baz as baz} from 'foo';", None),
        ("import {foo as foo, bar as bar} from 'foo';", None),
        ("var foo = 0; export {foo as foo};", None),
        ("var foo = 0; export {foo as 'foo'};", None), // { "ecmaVersion": 2022 },
        ("export {foo as 'foo'} from 'bar';", None),   // { "ecmaVersion": 2022 },
        ("export {'foo' as foo} from 'bar';", None),   // { "ecmaVersion": 2022 },
        ("export {'foo' as 'foo'} from 'bar';", None), // { "ecmaVersion": 2022 },
        ("export {' 👍 ' as ' 👍 '} from 'bar';", None), // { "ecmaVersion": 2022 },
        ("export {'' as ''} from 'bar';", None),       // { "ecmaVersion": 2022 },
        (r"var a = 0; export {a as \u0061};", None),
        (r"var \u0061 = 0; export {\u0061 as a};", None),
        (r"var \u0061 = 0; export {\u0061 as \u0061};", None),
        ("var foo = 0; var bar = 0; export {foo as foo, bar as baz};", None),
        ("var foo = 0; var baz = 0; export {foo as bar, baz as baz};", None),
        ("var foo = 0; var bar = 0;export {foo as foo, bar as bar};", None),
        ("export {foo as foo} from 'foo';", None),
        (r"export {a as \u0061} from 'foo';", None),
        (r"export {\u0061 as a} from 'foo';", None),
        (r"export {\u0061 as \u0061} from 'foo';", None),
        ("export {foo as foo, bar as baz} from 'foo';", None),
        ("var foo = 0; var bar = 0; export {foo as bar, baz as baz} from 'foo';", None),
        ("export {foo as foo, bar as bar} from 'foo';", None),
        ("({/* comment */foo: foo} = {});", None),
        ("({/* comment */foo: foo = 1} = {});", None),
        ("({foo, /* comment */bar: bar} = {});", None),
        ("({foo/**/ : foo} = {});", None),
        ("({foo/**/ : foo = 1} = {});", None),
        ("({foo /**/: foo} = {});", None),
        ("({foo /**/: foo = 1} = {});", None),
        (
            "({foo://
            foo} = {});",
            None,
        ),
        ("({foo: /**/foo} = {});", None),
        ("({foo: (/**/foo)} = {});", None),
        ("({foo: (foo/**/)} = {});", None),
        (
            "({foo: (foo //
            )} = {});",
            None,
        ),
        ("({foo: /**/foo = 1} = {});", None),
        ("({foo: (/**/foo) = 1} = {});", None),
        ("({foo: (foo/**/) = 1} = {});", None),
        ("({foo: foo/* comment */} = {});", None),
        (
            "({foo: foo//comment
            ,bar} = {});",
            None,
        ),
        ("({foo: foo/* comment */ = 1} = {});", None),
        (
            "({foo: foo // comment
             = 1} = {});",
            None,
        ),
        ("({foo: foo = /* comment */ 1} = {});", None),
        (
            "({foo: foo = // comment
             1} = {});",
            None,
        ),
        ("({foo: foo = (1/* comment */)} = {});", None),
        ("import {/* comment */foo as foo} from 'foo';", None),
        ("import {foo,/* comment */bar as bar} from 'foo';", None),
        ("import {foo/**/ as foo} from 'foo';", None),
        ("import {foo /**/as foo} from 'foo';", None),
        (
            "import {foo //
            as foo} from 'foo';",
            None,
        ),
        ("import {foo as/**/foo} from 'foo';", None),
        ("import {foo as foo/* comment */} from 'foo';", None),
        ("import {foo as foo/* comment */,bar} from 'foo';", None),
        ("let foo; export {/* comment */foo as foo};", None),
        ("let foo, bar; export {foo,/* comment */bar as bar};", None),
        ("let foo; export {foo/**/as foo};", None),
        ("let foo; export {foo as/**/ foo};", None),
        ("let foo; export {foo as /**/foo};", None),
        (
            "let foo; export {foo as//comment
             foo};",
            None,
        ),
        ("let foo; export {foo as foo/* comment*/};", None),
        ("let foo, bar; export {foo as foo/* comment*/,bar};", None),
        (
            "let foo, bar; export {foo as foo//comment
            ,bar};",
            None,
        ),
    ];

    let fix = vec![
        ("let {foo: foo} = obj;", "let {foo} = obj;"),
        ("({foo: (foo)} = obj);", "({foo} = obj);"),
        (r"let {\u0061: a} = obj;", "let {a} = obj;"),
        (r"let {a: \u0061} = obj;", r"let {\u0061} = obj;"),
        (r"let {\u0061: \u0061} = obj;", r"let {\u0061} = obj;"),
        ("let {a, foo: foo} = obj;", "let {a, foo} = obj;"),
        ("let {foo: foo, bar: baz} = obj;", "let {foo, bar: baz} = obj;"),
        ("let {foo: bar, baz: baz} = obj;", "let {foo: bar, baz} = obj;"),
        ("let {foo: foo, bar: bar} = obj;", "let {foo, bar} = obj;"),
        ("let {foo: {bar: bar}} = obj;", "let {foo: {bar}} = obj;"),
        ("let {foo: {bar: bar}, baz: baz} = obj;", "let {foo: {bar}, baz} = obj;"),
        ("let {'foo': foo} = obj;", "let {foo} = obj;"),
        ("let {'foo': foo, 'bar': baz} = obj;", "let {foo, 'bar': baz} = obj;"),
        ("let {'foo': bar, 'baz': baz} = obj;", "let {'foo': bar, baz} = obj;"),
        ("let {'foo': foo, 'bar': bar} = obj;", "let {foo, bar} = obj;"),
        ("let {'foo': {'bar': bar}} = obj;", "let {'foo': {bar}} = obj;"),
        ("let {'foo': {'bar': bar}, 'baz': baz} = obj;", "let {'foo': {bar}, baz} = obj;"),
        (
            "let {foo: foo = 1, 'bar': bar = 1, baz: baz} = obj;",
            "let {foo = 1, bar = 1, baz} = obj;",
        ),
        (
            "let {foo: {bar: bar = 1, 'baz': baz = 1}} = obj;",
            "let {foo: {bar = 1, baz = 1}} = obj;",
        ),
        ("let {foo: {bar: bar = {}} = {}} = obj;", "let {foo: {bar = {}} = {}} = obj;"),
        ("let {foo: foo = (a)} = obj;", "let {foo = (a)} = obj;"),
        ("let {foo: foo = (a, b)} = obj;", "let {foo = (a, b)} = obj;"),
        ("function func({foo: foo}) {}", "function func({foo}) {}"),
        ("function func({foo: foo, bar: baz}) {}", "function func({foo, bar: baz}) {}"),
        ("function func({foo: bar, baz: baz}) {}", "function func({foo: bar, baz}) {}"),
        ("function func({foo: foo, bar: bar}) {}", "function func({foo, bar}) {}"),
        (
            "function func({foo: foo = 1, 'bar': bar = 1, baz: baz}) {}",
            "function func({foo = 1, bar = 1, baz}) {}",
        ),
        (
            "function func({foo: {bar: bar = 1, 'baz': baz = 1}}) {}",
            "function func({foo: {bar = 1, baz = 1}}) {}",
        ),
        (
            "function func({foo: {bar: bar = {}} = {}}) {}",
            "function func({foo: {bar = {}} = {}}) {}",
        ),
        ("({foo: foo}) => {}", "({foo}) => {}"),
        ("({foo: foo, bar: baz}) => {}", "({foo, bar: baz}) => {}"),
        ("({foo: bar, baz: baz}) => {}", "({foo: bar, baz}) => {}"),
        ("({foo: foo, bar: bar}) => {}", "({foo, bar}) => {}"),
        ("({foo: foo = 1, 'bar': bar = 1, baz: baz}) => {}", "({foo = 1, bar = 1, baz}) => {}"),
        ("({foo: {bar: bar = 1, 'baz': baz = 1}}) => {}", "({foo: {bar = 1, baz = 1}}) => {}"),
        ("({foo: {bar: bar = {}} = {}}) => {}", "({foo: {bar = {}} = {}}) => {}"),
        ("const {foo: foo, ...stuff} = myObject;", "const {foo, ...stuff} = myObject;"),
        (
            "const {foo: foo, bar: baz, ...stuff} = myObject;",
            "const {foo, bar: baz, ...stuff} = myObject;",
        ),
        (
            "const {foo: foo, bar: bar, ...stuff} = myObject;",
            "const {foo, bar, ...stuff} = myObject;",
        ),
        ("import {foo as foo} from 'foo';", "import {foo} from 'foo';"),
        ("import {'foo' as foo} from 'foo';", "import {foo} from 'foo';"),
        (r"import {\u0061 as a} from 'foo';", "import {a} from 'foo';"),
        (r"import {a as \u0061} from 'foo';", r"import {\u0061} from 'foo';"),
        (r"import {\u0061 as \u0061} from 'foo';", r"import {\u0061} from 'foo';"),
        ("import {foo as foo, bar as baz} from 'foo';", "import {foo, bar as baz} from 'foo';"),
        ("import {foo as bar, baz as baz} from 'foo';", "import {foo as bar, baz} from 'foo';"),
        ("import {foo as foo, bar as bar} from 'foo';", "import {foo, bar} from 'foo';"),
        ("var foo = 0; export {foo as foo};", "var foo = 0; export {foo};"),
        ("var foo = 0; export {foo as 'foo'};", "var foo = 0; export {foo};"),
        ("export {foo as 'foo'} from 'bar';", "export {foo} from 'bar';"),
        ("export {'foo' as foo} from 'bar';", "export {'foo'} from 'bar';"),
        ("export {'foo' as 'foo'} from 'bar';", "export {'foo'} from 'bar';"),
        ("export {' 👍 ' as ' 👍 '} from 'bar';", "export {' 👍 '} from 'bar';"),
        ("export {'' as ''} from 'bar';", "export {''} from 'bar';"),
        (r"var a = 0; export {a as \u0061};", "var a = 0; export {a};"),
        (r"var \u0061 = 0; export {\u0061 as a};", r"var \u0061 = 0; export {\u0061};"),
        (r"var \u0061 = 0; export {\u0061 as \u0061};", r"var \u0061 = 0; export {\u0061};"),
        (
            "var foo = 0; var bar = 0; export {foo as foo, bar as baz};",
            "var foo = 0; var bar = 0; export {foo, bar as baz};",
        ),
        (
            "var foo = 0; var baz = 0; export {foo as bar, baz as baz};",
            "var foo = 0; var baz = 0; export {foo as bar, baz};",
        ),
        (
            "var foo = 0; var bar = 0;export {foo as foo, bar as bar};",
            "var foo = 0; var bar = 0;export {foo, bar};",
        ),
        ("export {foo as foo} from 'foo';", "export {foo} from 'foo';"),
        (r"export {a as \u0061} from 'foo';", "export {a} from 'foo';"),
        (r"export {\u0061 as a} from 'foo';", r"export {\u0061} from 'foo';"),
        (r"export {\u0061 as \u0061} from 'foo';", r"export {\u0061} from 'foo';"),
        ("export {foo as foo, bar as baz} from 'foo';", "export {foo, bar as baz} from 'foo';"),
        (
            "var foo = 0; var bar = 0; export {foo as bar, baz as baz} from 'foo';",
            "var foo = 0; var bar = 0; export {foo as bar, baz} from 'foo';",
        ),
        ("export {foo as foo, bar as bar} from 'foo';", "export {foo, bar} from 'foo';"),
        ("({/* comment */foo: foo} = {});", "({/* comment */foo} = {});"),
        ("({/* comment */foo: foo = 1} = {});", "({/* comment */foo = 1} = {});"),
        ("({foo, /* comment */bar: bar} = {});", "({foo, /* comment */bar} = {});"),
        ("({foo: foo/* comment */} = {});", "({foo/* comment */} = {});"),
        (
            "({foo: foo//comment
            ,bar} = {});",
            "({foo//comment
            ,bar} = {});",
        ),
        ("({foo: foo/* comment */ = 1} = {});", "({foo/* comment */ = 1} = {});"),
        (
            "({foo: foo // comment
             = 1} = {});",
            "({foo // comment
             = 1} = {});",
        ),
        ("({foo: foo = /* comment */ 1} = {});", "({foo = /* comment */ 1} = {});"),
        (
            "({foo: foo = // comment
             1} = {});",
            "({foo = // comment
             1} = {});",
        ),
        ("({foo: foo = (1/* comment */)} = {});", "({foo = (1/* comment */)} = {});"),
        ("import {/* comment */foo as foo} from 'foo';", "import {/* comment */foo} from 'foo';"),
        (
            "import {foo,/* comment */bar as bar} from 'foo';",
            "import {foo,/* comment */bar} from 'foo';",
        ),
        ("import {foo as foo/* comment */} from 'foo';", "import {foo/* comment */} from 'foo';"),
        (
            "import {foo as foo/* comment */,bar} from 'foo';",
            "import {foo/* comment */,bar} from 'foo';",
        ),
        ("let foo; export {/* comment */foo as foo};", "let foo; export {/* comment */foo};"),
        (
            "let foo, bar; export {foo,/* comment */bar as bar};",
            "let foo, bar; export {foo,/* comment */bar};",
        ),
        ("let foo; export {foo as foo/* comment*/};", "let foo; export {foo/* comment*/};"),
        (
            "let foo, bar; export {foo as foo/* comment*/,bar};",
            "let foo, bar; export {foo/* comment*/,bar};",
        ),
        (
            "let foo, bar; export {foo as foo//comment
            ,bar};",
            "let foo, bar; export {foo//comment
            ,bar};",
        ),
    ];

    Tester::new(NoUselessRename::NAME, NoUselessRename::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
