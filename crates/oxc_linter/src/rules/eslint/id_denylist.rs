use rustc_hash::FxHashSet;
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{ArrayValidation, InstanceType, Schema, SchemaObject},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{
        BindingIdentifier, ExportSpecifier, IdentifierName, IdentifierReference, LabelIdentifier,
        ModuleExportName, PrivateIdentifier,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{Rule, TupleRuleConfig},
    rules::eslint::id_match::{
        is_dynamic_import_attribute_object_property, is_known_external_global,
        transparent_reference_parent,
    },
};

fn id_denylist_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier '{name}' is restricted.")).with_label(span)
}

fn id_denylist_private_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier '#{name}' is restricted.")).with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct IdDenylist(Box<FxHashSet<String>>);

impl JsonSchema for IdDenylist {
    fn schema_name() -> String {
        "IdDenylist".to_string()
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                additional_items: Some(Box::new(r#gen.subschema_for::<String>())),
                unique_items: Some(true),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow specified identifiers
    ///
    /// ### Why is this bad?
    ///
    /// Generic names can lead to hard-to-decipher code. This rule allows you to specify a deny list of disallowed identifier names to avoid this practice.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint id-denylist: ["error", "data", "callback"] */
    ///
    /// const data = { ...values };
    /// function callback() {
    ///     // ...
    /// }
    /// element.callback = function() {
    ///     // ...
    /// };
    /// const itemSet = {
    ///     data: [...values]
    /// };
    /// class Foo {
    ///     data = [];
    /// }
    /// class Bar {
    ///     #data = [];
    /// }
    /// class Baz {
    ///     callback() {}
    /// }
    /// class Qux {
    ///     #callback() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /*eslint id-denylist: ["error", "data", "callback"] */
    ///
    /// const encodingOptions = {...values};
    /// function processFileResult() {
    ///     // ...
    /// }
    /// element.successHandler = function() {
    ///     // ...
    /// };
    /// const itemSet = {
    ///     entities: [...values]
    /// };
    /// callback(); // all function calls are ignored
    /// foo.callback(); // all function calls are ignored
    /// foo.data; // all property names that are not assignments are ignored
    /// class Foo {
    ///     items = [];
    /// }
    /// class Bar {
    ///     #items = [];
    /// }
    /// class Baz {
    ///     method() {}
    /// }
    /// class Qux {
    ///     #method() {}
    /// }
    /// ```
    IdDenylist,
    eslint,
    style,
    config = IdDenylist,
    version = "next",
    short_description = "Disallow specified identifiers.",
);

impl Rule for IdDenylist {
    fn from_configuration(value: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BindingIdentifier(ident) => {
                self.check_binding_identifier(ident, node, ctx);
            }
            AstKind::IdentifierReference(ident) => {
                self.check_identifier_reference(ident, node, ctx);
            }
            AstKind::IdentifierName(ident) => self.check_identifier_name(ident, node, ctx),
            AstKind::PrivateIdentifier(ident) => self.check_private_identifier(ident, node, ctx),
            AstKind::LabelIdentifier(ident) => self.check_label_identifier(ident, ctx),
            _ => {}
        }
    }

    fn should_run(&self, _ctx: &ContextHost) -> bool {
        !self.0.is_empty()
    }
}

impl IdDenylist {
    fn check_binding_identifier(
        &self,
        ident: &BindingIdentifier,
        node: &AstNode,
        ctx: &LintContext,
    ) {
        if self.is_restricted(ident.name.as_str()) {
            ctx.diagnostic(id_denylist_diagnostic(node.span(), ident.name.as_str()));
        }
    }

    fn check_identifier_reference<'a>(
        &self,
        ident: &IdentifierReference<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let name = ident.name.as_str();
        if !self.is_restricted(name) {
            return;
        }

        let (parent, _) = transparent_reference_parent(node, ctx);
        match parent.kind() {
            AstKind::CallExpression(_) | AstKind::NewExpression(_) => return,
            AstKind::ObjectProperty(property) if property.shorthand => return,
            AstKind::AssignmentTargetPropertyProperty(property)
                if property.name.span().contains_inclusive(ident.span) && !property.computed =>
            {
                return;
            }
            _ => {}
        }

        if is_known_external_global(ident, ctx) {
            return;
        }

        ctx.diagnostic(id_denylist_diagnostic(node.span(), name));
    }

    fn check_identifier_name(&self, ident: &IdentifierName, node: &AstNode, ctx: &LintContext) {
        let name = ident.name.as_str();
        if !self.is_restricted(name) {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());
        match parent.kind() {
            AstKind::ImportAttribute(_) | AstKind::WithClause(_) | AstKind::ImportSpecifier(_) => {
                return;
            }
            AstKind::ExportSpecifier(specifier) => {
                if export_specifier_identifier_should_skip(specifier, node, parent, ctx) {
                    return;
                }
            }
            AstKind::ObjectProperty(property)
                if property.key.span().contains_inclusive(ident.span) && !property.computed =>
            {
                if is_dynamic_import_attribute_object_property(property, ctx) {
                    return;
                }
            }
            AstKind::BindingProperty(property)
                if property.key.span().contains_inclusive(ident.span) && !property.computed =>
            {
                return;
            }
            AstKind::AssignmentTargetPropertyProperty(property)
                if property.name.span().contains_inclusive(ident.span) && !property.computed =>
            {
                return;
            }
            AstKind::StaticMemberExpression(member)
                if member.property.span == ident.span
                    && !member_is_assignment_target(parent, ctx) =>
            {
                return;
            }
            _ => {}
        }

        ctx.diagnostic(id_denylist_diagnostic(node.span(), name));
    }

    fn check_private_identifier(
        &self,
        ident: &PrivateIdentifier,
        node: &AstNode,
        ctx: &LintContext,
    ) {
        let name = ident.name.as_str();
        if !self.is_restricted(name) {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());
        if matches!(parent.kind(), AstKind::PrivateFieldExpression(_))
            && !member_is_assignment_target(parent, ctx)
        {
            return;
        }

        ctx.diagnostic(id_denylist_private_diagnostic(node.span(), name));
    }

    fn check_label_identifier(&self, ident: &LabelIdentifier, ctx: &LintContext) {
        let name = ident.name.as_str();
        if self.is_restricted(name) {
            ctx.diagnostic(id_denylist_diagnostic(ident.span, name));
        }
    }

    fn is_restricted(&self, name: &str) -> bool {
        self.0.contains(name)
    }
}

fn export_specifier_identifier_should_skip(
    specifier: &ExportSpecifier,
    current_node: &AstNode,
    specifier_node: &AstNode,
    ctx: &LintContext,
) -> bool {
    if specifier.local.span() == specifier.exported.span() {
        return matches!(
            &specifier.exported,
            ModuleExportName::IdentifierName(exported)
                if exported.node_id.get() == current_node.id()
        );
    }

    matches!(
        ctx.nodes().parent_kind(specifier_node.id()),
        AstKind::ExportNamedDeclaration(declaration)
            if declaration.source.is_some() && specifier.local.span() == current_node.span()
    )
}

fn member_is_assignment_target(member_node: &AstNode, ctx: &LintContext) -> bool {
    match ctx.nodes().parent_kind(member_node.id()) {
        AstKind::AssignmentExpression(assignment) => assignment.left.span() == member_node.span(),
        AstKind::ArrayAssignmentTarget(_) | AstKind::AssignmentTargetRest(_) => true,
        AstKind::AssignmentTargetPropertyProperty(property) => {
            property.binding.span() == member_node.span()
        }
        AstKind::AssignmentTargetWithDefault(target) => target.binding.span() == member_node.span(),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::{TestCase, Tester};

    // spellchecker:off
    let pass = vec![
        (r#"foo = "bar""#, Some(serde_json::json!(["bar"]))),
        (r#"bar = "bar""#, Some(serde_json::json!(["foo"]))),
        (r#"foo = "bar""#, Some(serde_json::json!(["f", "fo", "fooo", "bar"]))),
        ("function foo(){}", Some(serde_json::json!(["bar"]))),
        ("foo()", Some(serde_json::json!(["f", "fo", "fooo", "bar"]))),
        ("import { foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("foo.bar()", Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "baz"]))),
        (
            "var foo = bar.baz;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz"])),
        ),
        (
            "var foo = bar.baz.bing;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "foo.bar.baz = bing.bong.bash;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar) {}",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "var obj = { key: foo.bar };",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        ("const {foo: bar} = baz", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6 },
        ("const {foo: {bar: baz}} = qux", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: baz }) {}", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: {baz: qux} }) {}", Some(serde_json::json!(["bar", "baz"]))), // { "ecmaVersion": 6 },
        ("function foo({baz} = obj.qux) {}", Some(serde_json::json!(["qux"]))), // { "ecmaVersion": 6 },
        ("function foo({ foo: {baz} = obj.qux }) {}", Some(serde_json::json!(["qux"]))), // { "ecmaVersion": 6 },
        ("({a: bar = obj.baz});", Some(serde_json::json!(["baz"]))), // { "ecmaVersion": 6 },
        ("({foo: {a: bar = obj.baz}} = qux);", Some(serde_json::json!(["baz"]))), // { "ecmaVersion": 6 },
        (
            "var arr = [foo.bar];",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "[foo.bar]",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "[foo.bar.nesting]",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar === bar.baz) { [foo.bar] }",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "var myArray = new Array(); var myDate = new Date();",
            Some(serde_json::json!(["array", "date", "mydate", "myarray", "new", "var"])),
        ),
        ("foo()", Some(serde_json::json!(["foo"]))),
        ("foo.bar()", Some(serde_json::json!(["bar"]))),
        ("foo.bar", Some(serde_json::json!(["bar"]))),
        ("({foo: obj.bar.bar.bar.baz} = {});", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("({[obj.bar]: a = baz} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("Number.parseInt()", Some(serde_json::json!(["Number"]))),
        ("x = Number.NaN;", Some(serde_json::json!(["Number"]))),
        ("var foo = undefined;", Some(serde_json::json!(["undefined"]))),
        ("if (foo === undefined);", Some(serde_json::json!(["undefined"]))),
        ("obj[undefined] = 5;", Some(serde_json::json!(["undefined"]))),
        ("class C { camelCase; #camelCase; #camelCase2() {} }", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 2022 },
        (
            "class C { snake_case; #snake_case; #snake_case2() {} }",
            Some(serde_json::json!(["foo"])),
        ), // { "ecmaVersion": 2022 },
        ("import foo from 'foo.json' with { type: 'json' }", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025, "sourceType": "module" },
        ("export * from 'foo.json' with { type: 'json' }", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025, "sourceType": "module" },
        (
            "export { default } from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!(["type"])),
        ), // { "ecmaVersion": 2025, "sourceType": "module" },
        (
            "import('foo.json', { with: { type: 'json' } })",
            Some(serde_json::json!(["with", "type"])),
        ), // { "ecmaVersion": 2025 },
        ("import('foo.json', { 'with': { type: 'json' } })", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025 },
        ("import('foo.json', { with: { type } })", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025 }
    ];

    let mut pass = pass.into_iter().map(TestCase::from).collect::<Vec<_>>();
    pass.extend([
        TestCase::from((
            "foo = { [myGlobal]: 1 };",
            Some(serde_json::json!(["myGlobal"])),
            Some(serde_json::json!({ "globals": { "myGlobal": "readonly" } })),
        )),
        TestCase::from((
            "({ myGlobal } = foo);",
            Some(serde_json::json!(["myGlobal"])),
            Some(serde_json::json!({ "globals": { "myGlobal": "writable" } })),
        )),
        TestCase::from((
            "myGlobal = 5;",
            Some(serde_json::json!(["myGlobal"])),
            Some(serde_json::json!({ "globals": { "myGlobal": "readonly" } })),
        )),
        TestCase::from((
            "var foo = { bar: window.baz };",
            Some(serde_json::json!(["window"])),
            Some(serde_json::json!({ "globals": { "window": "readonly" } })),
        )),
        TestCase::from((
            "var foo = [Map];",
            Some(serde_json::json!(["Map"])),
            Some(serde_json::json!({ "globals": { "Map": "readonly" } })),
        )),
    ]);

    let fail = vec![
        (r#"foo = "bar""#, Some(serde_json::json!(["foo"]))),
        (r#"bar = "bar""#, Some(serde_json::json!(["bar"]))),
        (r#"foo = "bar""#, Some(serde_json::json!(["f", "fo", "foo", "bar"]))),
        ("function foo(){}", Some(serde_json::json!(["f", "fo", "foo", "bar"]))),
        ("import foo from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import * as foo from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export * as foo from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 2020, "sourceType": "module" },
        ("import { foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as bar } from 'mod'", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as bar } from 'mod'", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo, foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import { foo as bar, foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("import foo, { foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as bar };", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo };", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as bar };", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as foo };", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo; export { foo as bar };", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar } from 'mod'", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar } from 'mod'", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo, foo as bar } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export { foo as bar, foo } from 'mod'", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("foo.bar()", Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "baz"]))),
        ("foo[bar] = baz;", Some(serde_json::json!(["bar"]))),
        ("baz = foo[bar];", Some(serde_json::json!(["bar"]))),
        (
            "var foo = bar.baz;",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz"])),
        ),
        (
            "var foo = bar.baz;",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "bar", "bazz"])),
        ),
        (
            "if (foo.bar) {}",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        ("var obj = { key: foo.bar };", Some(serde_json::json!(["obj"]))),
        ("var obj = { key: foo.bar };", Some(serde_json::json!(["key"]))),
        ("var obj = { key: foo.bar };", Some(serde_json::json!(["foo"]))),
        ("var arr = [foo.bar];", Some(serde_json::json!(["arr"]))),
        ("var arr = [foo.bar];", Some(serde_json::json!(["foo"]))),
        (
            "[foo.bar]",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar === bar.baz) { [bing.baz] }",
            Some(serde_json::json!(["f", "fo", "foo", "b", "ba", "barr", "bazz", "bingg"])),
        ),
        (
            "if (foo.bar === bar.baz) { [foo.bar] }",
            Some(serde_json::json!(["f", "fo", "fooo", "b", "ba", "bar", "bazz", "bingg"])),
        ),
        (
            "var myArray = new Array(); var myDate = new Date();",
            Some(serde_json::json!(["array", "date", "myDate", "myarray", "new", "var"])),
        ),
        (
            "var myArray = new Array(); var myDate = new Date();",
            Some(serde_json::json!(["array", "date", "mydate", "myArray", "new", "var"])),
        ),
        ("foo.bar = 1", Some(serde_json::json!(["bar"]))),
        ("foo.bar.baz = 1", Some(serde_json::json!(["bar", "baz"]))),
        ("const {foo} = baz", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6 },
        ("const {foo: bar} = baz", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("const {[foo]: bar} = baz", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("const {foo: {bar: baz}} = qux", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("const {foo: {[bar]: baz}} = qux", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("const {[foo]: {[bar]: baz}} = qux", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: baz }) {}", Some(serde_json::json!(["bar", "baz"]))), // { "ecmaVersion": 6 },
        ("function foo({ bar: {baz: qux} }) {}", Some(serde_json::json!(["bar", "baz", "qux"]))), // { "ecmaVersion": 6 },
        ("({foo: obj.bar} = baz);", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("({foo: obj.bar.bar.bar.baz} = {});", Some(serde_json::json!(["foo", "bar", "baz"]))), // { "ecmaVersion": 6 },
        ("({[foo]: obj.bar} = baz);", Some(serde_json::json!(["foo", "bar"]))), // { "ecmaVersion": 6 },
        ("({foo: { a: obj.bar }} = baz);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({a: obj.bar = baz} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        (
            "({a: obj.bar.bar.baz = obj.qux} = obj.qux);",
            Some(serde_json::json!(["a", "bar", "baz", "qux"])),
        ), // { "ecmaVersion": 6 },
        (
            "({a: obj[bar] = obj.qux} = obj.qux);",
            Some(serde_json::json!(["a", "bar", "baz", "qux"])),
        ), // { "ecmaVersion": 6 },
        ("({a: [obj.bar] = baz} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({foo: { a: obj.bar = baz}} = qux);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({foo: { [a]: obj.bar }} = baz);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 6 },
        ("({...obj.bar} = baz);", Some(serde_json::json!(["bar"]))), // { "ecmaVersion": 9 },
        ("([obj.bar] = baz);", Some(serde_json::json!(["bar"]))),    // { "ecmaVersion": 6 },
        ("const [bar] = baz;", Some(serde_json::json!(["bar"]))),    // { "ecmaVersion": 6 },
        ("foo.undefined = 1;", Some(serde_json::json!(["undefined"]))),
        ("var foo = { undefined: 1 };", Some(serde_json::json!(["undefined"]))),
        ("var foo = { undefined: undefined };", Some(serde_json::json!(["undefined"]))),
        ("var foo = { Number() {} };", Some(serde_json::json!(["Number"]))), // { "ecmaVersion": 6 },
        ("class Foo { Number() {} }", Some(serde_json::json!(["Number"]))), // { "ecmaVersion": 6 },
        ("myGlobal: while(foo) { break myGlobal; } ", Some(serde_json::json!(["myGlobal"]))), // { "globals": { "myGlobal": "readonly" }, },
        ("const foo = 1; bar = foo;", Some(serde_json::json!(["foo"]))), // { "ecmaVersion": 6 },
        ("let foo; foo = bar;", Some(serde_json::json!(["foo"]))),       // { "ecmaVersion": 6 },
        ("bar = foo; var foo;", Some(serde_json::json!(["foo"]))),
        ("function foo() {} var bar = foo;", Some(serde_json::json!(["foo"]))),
        ("class Foo {} var bar = Foo;", Some(serde_json::json!(["Foo"]))), // { "ecmaVersion": 6 },
        ("let undefined; undefined = 1;", Some(serde_json::json!(["undefined"]))), // { "ecmaVersion": 6 },
        ("foo = undefined; var undefined;", Some(serde_json::json!(["undefined"]))),
        ("function undefined(){} x = undefined;", Some(serde_json::json!(["undefined"]))),
        ("class Number {} x = Number.NaN;", Some(serde_json::json!(["Number"]))), // { "ecmaVersion": 6 },
        ("if (foo) { let undefined; bar = undefined; }", Some(serde_json::json!(["undefined"]))), // { "ecmaVersion": 6 },
        ("function foo(Number) { var x = Number.NaN; }", Some(serde_json::json!(["Number"]))),
        ("function foo() { var myGlobal; x = myGlobal; }", Some(serde_json::json!(["myGlobal"]))), // { "globals": { "myGlobal": "readonly" }, },
        (
            "function foo(bar) { return Number.parseInt(bar); } const Number = 1;",
            Some(serde_json::json!(["Number"])),
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "import Number from 'myNumber'; const foo = Number.parseInt(bar);",
            Some(serde_json::json!(["Number"])),
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo = function undefined() {};", Some(serde_json::json!(["undefined"]))),
        ("var foo = { undefined }", Some(serde_json::json!(["undefined"]))), // { "ecmaVersion": 6 },
        (
            "class C { camelCase; #camelCase; #camelCase2() {} }",
            Some(serde_json::json!(["camelCase"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { snake_case; #snake_case() {}; #snake_case2() {} }",
            Some(serde_json::json!(["snake_case"])),
        ), // { "ecmaVersion": 2022 },
        ("import('foo.json', { with: { [type]: 'json' } })", Some(serde_json::json!(["type"]))), // { "ecmaVersion": 2025 },
        ("import('foo.json', { with: { type: json } })", Some(serde_json::json!(["json"]))), // { "ecmaVersion": 2025 }
    ];

    let mut fail = fail.into_iter().map(TestCase::from).collect::<Vec<_>>();
    fail.extend([
        TestCase::from((
            "window.myGlobal = 5; foo = myGlobal;",
            Some(serde_json::json!(["myGlobal"])),
            Some(serde_json::json!({
                "globals": { "myGlobal": "readonly", "window": "readonly" }
            })),
        )),
        TestCase::from((
            "var foo = undefined;",
            Some(serde_json::json!(["undefined"])),
            Some(serde_json::json!({ "globals": { "undefined": "off" } })),
        )),
        TestCase::from((
            "Number.parseInt()",
            Some(serde_json::json!(["Number"])),
            Some(serde_json::json!({ "globals": { "Number": "off" } })),
        )),
        TestCase::from((
            "var foo = [Map];",
            Some(serde_json::json!(["Map"])),
            Some(serde_json::json!({ "globals": { "Map": "off" } })),
        )),
    ]);
    // spellchecker:on

    Tester::new(IdDenylist::NAME, IdDenylist::PLUGIN, pass, fail).test_and_snapshot();
}
