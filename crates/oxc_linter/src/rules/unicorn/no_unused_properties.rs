use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, AssignmentTargetProperty, BindingPattern, ComputedMemberExpression,
        Expression, FormalParameter, JSXMemberExpression, ObjectAssignmentTarget, ObjectExpression,
        ObjectPattern, ObjectPropertyKind, PropertyKey, StaticMemberExpression, TSSignature,
        TSType, TSTypeAnnotation, TSTypeLiteral, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::BoundNames;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, Reference, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

fn no_unused_properties_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Property `{name}` is defined but never used."))
        .with_help("Remove the property or use it in the code.")
        .with_label(span)
}

fn property_display<'a>(ctx: &LintContext<'_>, property: &TrackedProperty<'a>) -> Cow<'a, str> {
    property
        .name
        .clone()
        .unwrap_or_else(|| Cow::Owned(ctx.source_range(property.key_span).to_string()))
}

#[derive(Debug, Default, Clone)]
pub struct NoUnusedProperties;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows unused properties in object literals and in inline TypeScript object types.
    ///
    /// This covers two shapes of "object as a bag of values":
    ///
    /// - Enum-like object literals assigned to a variable, where some entries are never read.
    /// - Inline object types (`{ ... }`) attached to initialized variables or to function/method
    ///   parameters, where some fields are never read through the binding.
    ///
    /// ### Why is this bad?
    ///
    /// Unused properties, much like unused variables, are often a result of incomplete
    /// refactoring and may confuse readers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const myEnum = {
    ///     used: 1,
    ///     unused: 2, // <- Property `unused` is defined but never used
    /// };
    /// console.log(myEnum.used);
    /// ```
    ///
    /// ```ts
    /// function foo(args: { used: number; unused: number }) {
    ///     return args.used; // <- Property `unused` is defined but never used
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const myEnum = {
    ///     used: 1,
    /// };
    /// console.log(myEnum.used);
    /// ```
    ///
    /// ```ts
    /// function foo(args: { used: number }) {
    ///     return args.used;
    /// }
    /// ```
    NoUnusedProperties,
    unicorn,
    nursery,
    version = "next",
    short_description = "Disallow unused object properties.",
);

/// An object-like value whose members can be tracked statically:
/// - either an object literal
/// - or an inline TypeScript object type literal
enum Container<'a> {
    Object(&'a ObjectExpression<'a>),
    Type(&'a TSTypeLiteral<'a>),
}

/// A property of a [`Container`] whose usage gets tracked
struct TrackedProperty<'a> {
    name: Option<Cow<'a, str>>,
    /// Span of the raw key, used to render [`Self::name`] lazily when unnamed
    key_span: Span,
    span: Span,
    nested: Option<Container<'a>>,
}

impl<'a> TrackedProperty<'a> {
    fn new(key: &'a PropertyKey<'a>, span: Span, nested: Option<Container<'a>>) -> Self {
        Self { name: property_key_name(key), key_span: key.span(), span, nested }
    }
}

/// Outcome of checking one reference against one property name
enum Step {
    Survive(NodeId),
    Drop,
}

impl Rule for NoUnusedProperties {
    fn run_once(&self, ctx: &LintContext) {
        for symbol_id in ctx.scoping().symbol_ids() {
            Self::check_symbol(symbol_id, ctx);
        }
    }
}

impl NoUnusedProperties {
    fn check_symbol(symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));

        let Some((container, references)) = Self::dispatch_declaration(ctx, symbol_id, declaration)
        else {
            return;
        };

        Self::check_container(ctx, &container, &references);
    }

    fn dispatch_declaration<'a>(
        ctx: &LintContext<'_>,
        symbol_id: SymbolId,
        declaration: &'a AstNode<'a>,
    ) -> Option<(Container<'a>, Vec<NodeId>)> {
        let scoping = ctx.scoping();

        let (pattern, container) = match declaration.kind() {
            AstKind::VariableDeclarator(declarator) => {
                if Self::is_exported_declarator(ctx, declaration.id()) {
                    return None;
                }

                (
                    &declarator.id,
                    if declarator.init.is_some() {
                        Self::declarator_container(declarator).or_else(|| {
                            declarator
                                .type_annotation
                                .as_deref()
                                .and_then(type_annotation_container)
                        })
                    } else {
                        None
                    },
                )
            }

            AstKind::FormalParameter(parameter) => {
                (&parameter.pattern, Self::parameter_container(parameter))
            }

            _ => return None,
        };

        let Some(container) = container else {
            return None;
        };

        let BindingPattern::ObjectPattern(object_pattern) = pattern else {
            if !scoping.get_resolved_references(symbol_id).any(Reference::is_read) {
                return None;
            }

            let references =
                scoping.get_resolved_references(symbol_id).map(Reference::node_id).collect();
            return Some((container, references));
        };

        Self::check_destructured_pattern(ctx, symbol_id, &container, object_pattern);
        None
    }

    fn is_exported_declarator(ctx: &LintContext<'_>, declarator_id: NodeId) -> bool {
        let mut ancestors = ctx.nodes().ancestors(declarator_id);
        let _variable_declaration = ancestors.next();
        matches!(ancestors.next().map(AstNode::kind), Some(AstKind::ExportDeclaration(_)))
    }

    fn declarator_container<'a>(declarator: &'a VariableDeclarator<'a>) -> Option<Container<'a>> {
        if let Some(init) = &declarator.init
            && let Expression::ObjectExpression(object) = unwrap_ts_expression(init)
        {
            return Some(Container::Object(object));
        }

        if declarator.init.is_some()
            && matches!(&declarator.id, BindingPattern::BindingIdentifier(_))
            && let Some(annotation) = &declarator.type_annotation
            && let TSType::TSTypeLiteral(literal) = &annotation.type_annotation
        {
            return Some(Container::Type(literal));
        }

        None
    }

    fn check_destructured_pattern<'a>(
        ctx: &LintContext<'_>,
        symbol_id: SymbolId,
        container: &Container<'a>,
        object_pattern: &ObjectPattern<'a>,
    ) {
        let mut anchor_span = None;
        object_pattern.bound_names(&mut |ident| {
            if anchor_span.is_none() {
                anchor_span = Some(ident.span);
            }
        });
        if anchor_span != Some(ctx.scoping().symbol_span(symbol_id)) {
            return;
        }

        Self::report_unbound_members(ctx, container, object_pattern);
    }

    fn report_unbound_members<'a>(
        ctx: &LintContext<'_>,
        container: &Container<'a>,
        pattern: &ObjectPattern<'a>,
    ) {
        if pattern.rest.is_some() || pattern.properties.iter().any(|property| property.computed) {
            return;
        }

        for property in Self::tracked_properties(container) {
            let Some(name) = property.name.as_deref() else {
                continue;
            };
            if name == "__proto__" {
                continue;
            }

            let bound = pattern.properties.iter().find(|bound| {
                property_key_name(&bound.key).is_some_and(|key| key.as_ref() == name)
            });
            let Some(bound) = bound else {
                let display = property_display(ctx, &property);
                ctx.diagnostic(no_unused_properties_diagnostic(property.span, &display));
                continue;
            };

            let value = match &bound.value {
                BindingPattern::AssignmentPattern(assignment) => &assignment.left,
                value => value,
            };
            if let (Some(nested), BindingPattern::ObjectPattern(inner)) =
                (property.nested.as_ref(), value)
            {
                Self::report_unbound_members(ctx, nested, inner);
            }
        }
    }

    fn parameter_container<'a>(parameter: &'a FormalParameter<'a>) -> Option<Container<'a>> {
        type_annotation_container(parameter.type_annotation.as_deref()?)
    }

    fn check_container(ctx: &LintContext<'_>, container: &Container<'_>, references: &[NodeId]) {
        for property in Self::tracked_properties(container) {
            if property.name.as_deref() == Some("__proto__") {
                continue;
            }

            let mut survived = Vec::with_capacity(references.len());

            for &reference in references {
                match Self::narrow_reference(ctx, reference, property.name.as_deref()) {
                    Step::Survive(node_id) => survived.push(node_id),
                    Step::Drop => {}
                }
            }

            if survived.is_empty() {
                let display = property_display(ctx, &property);
                ctx.diagnostic(no_unused_properties_diagnostic(property.span, &display));
            } else if let Some(nested) = &property.nested {
                Self::check_container(ctx, nested, &survived);
            }
        }
    }

    fn tracked_properties<'a>(container: &Container<'a>) -> Vec<TrackedProperty<'a>> {
        match container {
            Container::Object(object) => object
                .properties
                .iter()
                .filter_map(|property| Self::tracked_object_property(property))
                .collect(),

            Container::Type(type_literal) => type_literal
                .members
                .iter()
                .filter_map(|member| Self::tracked_type_signature(member))
                .collect(),
        }
    }

    fn tracked_object_property<'a>(
        property: &'a ObjectPropertyKind<'a>,
    ) -> Option<TrackedProperty<'a>> {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };

        let nested = match unwrap_ts_expression(&property.value) {
            Expression::ObjectExpression(nested) => Some(Container::Object(nested)),
            _ => None,
        };
        Some(TrackedProperty::new(&property.key, property.span, nested))
    }

    fn tracked_type_signature<'a>(member: &'a TSSignature<'a>) -> Option<TrackedProperty<'a>> {
        let TSSignature::TSPropertySignature(signature) = member else {
            return None;
        };

        let nested = signature.type_annotation.as_deref().and_then(type_annotation_container);
        Some(TrackedProperty::new(&signature.key, signature.span, nested))
    }

    fn narrow_reference(ctx: &LintContext<'_>, node_id: NodeId, key: Option<&str>) -> Step {
        let node_span = ctx.nodes().get_node(node_id).span();

        let mut parent = ctx.nodes().parent_node(node_id);

        loop {
            let inner_span = match parent.kind() {
                AstKind::TSAsExpression(expression) => &expression.expression,
                AstKind::TSSatisfiesExpression(expression) => &expression.expression,
                AstKind::TSNonNullExpression(expression) => &expression.expression,
                AstKind::TSInstantiationExpression(expression) => &expression.expression,
                _ => break,
            };
            if inner_span.span() != node_span {
                break;
            }

            parent = ctx.nodes().parent_node(parent.id());
        }

        Self::narrow_parent(ctx, node_id, node_span, parent, key)
    }

    fn narrow_parent(
        ctx: &LintContext<'_>,
        node_id: NodeId,
        node_span: Span,
        parent: &AstNode<'_>,
        key: Option<&str>,
    ) -> Step {
        match parent.kind() {
            AstKind::StaticMemberExpression(member) if member.object.span() == node_span => {
                Self::narrow_static_member(ctx, parent.id(), member, key)
            }

            AstKind::ComputedMemberExpression(member) if member.object.span() == node_span => {
                Self::narrow_computed_member(ctx, parent.id(), member, key)
            }

            AstKind::JSXMemberExpression(member) if member.object.span() == node_span => {
                Self::narrow_jsx_member(parent.id(), member, key)
            }

            AstKind::VariableDeclarator(declarator)
                if declarator.init.as_ref().is_some_and(|init| init.span() == node_span) =>
            {
                Self::narrow_declarator_init(parent.id(), node_id, &declarator.id, key)
            }

            AstKind::AssignmentExpression(assignment) if assignment.right.span() == node_span => {
                Self::narrow_assignment_target(parent.id(), node_id, &assignment.left, key)
            }

            _ => Step::Survive(node_id),
        }
    }

    fn narrow_static_member(
        ctx: &LintContext<'_>,
        member_id: NodeId,
        member: &StaticMemberExpression<'_>,
        key: Option<&str>,
    ) -> Step {
        let name = Cow::Borrowed(member.property.name.as_str());
        Self::member_step(ctx, member_id, Some(&name), false, key)
    }

    fn narrow_computed_member(
        ctx: &LintContext<'_>,
        member_id: NodeId,
        member: &ComputedMemberExpression<'_>,
        key: Option<&str>,
    ) -> Step {
        let name = computed_member_name(&member.expression);
        Self::member_step(ctx, member_id, name.as_ref(), true, key)
    }

    fn narrow_jsx_member(
        member_id: NodeId,
        member: &JSXMemberExpression<'_>,
        key: Option<&str>,
    ) -> Step {
        if key == Some(member.property.name.as_str()) {
            Step::Survive(member_id)
        } else {
            Step::Drop
        }
    }

    fn narrow_declarator_init(
        member_id: NodeId,
        node_id: NodeId,
        pattern: &BindingPattern<'_>,
        key: Option<&str>,
    ) -> Step {
        let BindingPattern::ObjectPattern(object_pattern) = pattern else {
            return Step::Survive(node_id);
        };

        if binding_pattern_keeps_key(object_pattern, key) {
            Step::Survive(member_id)
        } else {
            Step::Drop
        }
    }

    fn narrow_assignment_target(
        member_id: NodeId,
        node_id: NodeId,
        target: &AssignmentTarget<'_>,
        key: Option<&str>,
    ) -> Step {
        let AssignmentTarget::ObjectAssignmentTarget(object_target) = target else {
            return Step::Survive(node_id);
        };

        if assignment_target_keeps_key(object_target, key) {
            Step::Survive(member_id)
        } else {
            Step::Drop
        }
    }

    fn member_step(
        ctx: &LintContext<'_>,
        member_id: NodeId,
        property_name: Option<&Cow<'_, str>>,
        computed: bool,
        key: Option<&str>,
    ) -> Step {
        let member_span = ctx.nodes().get_node(member_id).span();
        let grandparent = ctx.nodes().parent_node(member_id);

        let survives_everything = match grandparent.kind() {
            AstKind::CallExpression(call) => call.callee.span() == member_span,
            AstKind::AssignmentExpression(_) => true,
            _ => false,
        } || (computed && property_name.is_none());
        if survives_everything {
            return Step::Survive(member_id);
        }

        match property_name.zip(key) {
            Some((name, key)) if name == key => Step::Survive(member_id),
            _ => Step::Drop,
        }
    }
}

fn type_annotation_container<'a>(annotation: &'a TSTypeAnnotation<'a>) -> Option<Container<'a>> {
    match &annotation.type_annotation {
        TSType::TSTypeLiteral(literal) => Some(Container::Type(literal)),
        _ => None,
    }
}

fn property_key_name<'a>(key: &PropertyKey<'a>) -> Option<Cow<'a, str>> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(Cow::Borrowed(ident.name.as_str())),
        PropertyKey::StringLiteral(literal) => Some(Cow::Borrowed(literal.value.as_str())),
        PropertyKey::NumericLiteral(literal) => Some(Cow::Owned(literal.value.to_string())),
        PropertyKey::BooleanLiteral(literal) => {
            Some(Cow::Borrowed(if literal.value { "true" } else { "false" }))
        }
        PropertyKey::NullLiteral(_) => Some(Cow::Borrowed("null")),
        PropertyKey::BigIntLiteral(literal) => Some(Cow::Borrowed(literal.value.as_str())),
        PropertyKey::TemplateLiteral(literal) => literal.single_quasi().map(Into::into),
        PropertyKey::Identifier(identifier) => Some(Cow::Borrowed(identifier.name.as_str())),
        _ => None,
    }
}

fn computed_member_name<'a>(expression: &'a Expression<'a>) -> Option<Cow<'a, str>> {
    match expression {
        Expression::StringLiteral(literal) => Some(Cow::Borrowed(literal.value.as_str())),
        Expression::NumericLiteral(literal) => Some(Cow::Owned(literal.value.to_string())),
        _ => None,
    }
}

fn binding_pattern_keeps_key(pattern: &oxc_ast::ast::ObjectPattern<'_>, key: Option<&str>) -> bool {
    pattern.rest.is_some()
        || key.is_some_and(|key| {
            pattern.properties.iter().any(|property| property.key.name().as_deref() == Some(key))
        })
}

fn assignment_target_keeps_key(target: &ObjectAssignmentTarget<'_>, key: Option<&str>) -> bool {
    target.rest.is_some()
        || key.is_some_and(|key| {
            target.properties.iter().any(|property| match property {
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                    property.name.name().as_deref() == Some(key)
                }
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                    property.binding.name == key
                }
            })
        })
}

fn unwrap_ts_expression<'a, 'b>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression {
        Expression::TSAsExpression(inner) => unwrap_ts_expression(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => unwrap_ts_expression(&inner.expression),
        Expression::TSNonNullExpression(inner) => unwrap_ts_expression(&inner.expression),
        Expression::TSInstantiationExpression(inner) => unwrap_ts_expression(&inner.expression),
        _ => expression,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Every key is proven through static member access
        "const foo = {a: 1, b: 2};
        console.log(foo.a, foo.b);",
        // Quoted keys resolve like identifiers, even across function boundaries
        r#"const foo = {'a': 1, "b": 2};
        function main() {
            console.log(foo.a, foo.b);
        }"#,
        // Literal computed accesses keep their keys alive
        r#"const foo = {a: 1, b: 2};
        console.log(foo['a'], foo["b"]);"#,
        // Computed literal keys pair up with literal computed accesses
        r#"const foo = {["a"]: 1, ['b']: 2};
        console.log(foo['a'], foo["b"]);"#,
        // Quote direction does not matter
        r#"const foo = {['a']: 1, ["b"]: 2};
        console.log(foo['a'], foo["b"]);"#,
        // Computed keys backed by anything but literals are skipped entirely
        "const a = Symbol('a');
        const b = 'b';
        const c = {};
        const foo = {
            [a]: 1,
            [b]: 2,
            [c]: 3
        };
        console.log(foo[a]);",
        // Computed accesses through local aliases still resolve
        "const a = 'a';
        const foo = {
            [a]: 1,
        };
        const a_ = a;
        console.log(foo[a_]);",
        // Unresolvable computed accesses keep everything alive
        "const a = 'a';
        const foo = {
            [a]: 1,
        };
        console.log(foo[x]);",
        // Whole-object reads prove nothing specific, so nothing reports
        "const a = Symbol('a');
        const foo = {[a]: 1};
        console.log(foo);",
        "const b = 'b';
        const foo = {[b]: 2};
        console.log(foo);",
        "const c = {};
        const foo = {[c]: 3};
        console.log(foo);",
        // Destructuring consumes exactly the keys it binds
        "const foo = {a: 1, b: 2};
        const {a, b} = foo;",
        // Same for destructuring assignment targets
        "const foo = {a: 1, b: 2};
        ({a, b} = foo);",
        // Accesses to unknown properties keep everything alive
        "const foo = {a: 1, b: 2};
        console.log(foo[x]);",
        // Same, observed from another function scope
        "const foo = {a: 1, b: 2};
        function main() {
            console.log(foo[x]);
        }",
        // Unproven segments below a proven path leave the subtree alive
        "const foo = {a: { b: 2 }};
        console.log(foo.a[x]);",
        // Reading a property keeps its whole subtree alive
        "const foo = {a: { b: 2 }};
        console.log(foo.a);",
        // Whole-object references exempt the bag
        "const foo = {a: 1, b: 2};
        console.log(foo);",
        "const foo = {a: 1, b: 2};
        function main() {
            console.log(foo);
        }",
        // Objects containing `this`-aware members are exempt entirely
        "const foo = {
            a: 1,
            f() {
                return this.a;
            }
        };",
        "const foo = {
            a: 1,
            f() {
                return this;
            }
        };",
        // Member writes switch the object to keep-alive
        "const foo = {
            a: 1
        };
        foo.f = function () { return this.a };",
        // Regardless of what the assigned function returns
        "const foo = {
            a: 1
        };
        foo.f = function () { return this };",
        // Writes into nested subtrees keep those subtrees alive
        "const foo = {
            a: {
                b: 1
            }
        };
        foo.a.f = function () { return this };",
        // Object.assign targets behave like member writes
        "const foo = {
            a: {
                b: 1
            }
        };
        Object.assign(foo.a, {
            f() {
                return this;
            }
        });",
        // __proto__ values are never inspected
        "const foo = {
            a: 1,
            __proto__: {
                c: 3
            }
        };
        console.log(foo.a);",
        // Quoted __proto__ keys are exempt alike
        "const bar = {
            b: 2
        };
        const foo = {
            a: 1,
            ['__proto__']: bar
        };
        console.log(foo.a);",
        // Well-known inspection calls like hasOwnProperty force keep-alive
        "const foo = {
            a: 1
        };
        foo.hasOwnProperty(x);",
        // Fully-proven chains narrow precisely
        "const foo = {
            a: {
                b: {
                    c: 1
                }
            }
        };
        console.log(foo.a.b.c);",
        // Bindings that are themselves unused belong to `no-unused-vars`
        "const foo = {a: 1, b: 2};",
        // Properties added by later assignments are not tracked
        "const foo = {};
        foo.a = 1;
        foo.b = 2;
        console.log(foo.a);",
        // Same, with `var`
        "var foo = {};
        foo.a = 1;
        foo.b = 2;
        console.log(foo.a);",
        // Reassigned bindings fall back to keep-alive
        "var foo = {a: 1, b: 2};
        foo = { a: 3, b: 4 };
        console.log(foo.a);",
        // Non-object initializers have nothing to track
        "const foo = function () {};",
        // Arrays are out of scope
        "const foo = [];",
        // Declarators without initializers have nothing to track
        "let foo;",
        "var foo;",
        // Function declarations carry no property bags
        "function foo() {}
        foo();",
        // Exported bindings are exempt
        "const foo = {};
        export default foo;",
        // Same, via named export
        "var foo = {
            a: {
                b: {
                    c: {
                        d: 1
                    }
                }
            }
        };
        export {foo};",
        // Directly exported literals skip tracking altogether
        "export const foo = {
            a: 1,
            b: 2
        };
        console.log(foo.a);",
        // CommonJS exports keep bindings alive
        "var foo = {
            a: 1
        };
        module.exports = foo;",
        // Same, as a namespace member
        "var foo = {
            a: 1
        };
        exports.foo = foo;",
        // Rest elements consume every remaining key
        "const foo = {a: 1, b: 2};
        const {a, ...rest} = foo;",
        // Same, whatever the source binding is called
        "const foo1 = {a: 1, b: 2};
        const {a, ...rest} = foo1;",
        // Spreads force keep-alive
        "const foo = {
            ...bar,
        };
        console.log(foo.a);",
        // Spread in the initializer defeats annotation-based tracking
        "type Configuration = {
            debounce: {
                wait: number;
            };
        };
        const configurationInput = {};
        const {
            debounce: userDebounce,
        }: Configuration = {
            debounce: {
                wait: 1000,
            },
            ...configurationInput,
        };
        console.log(userDebounce);",
        // Inline parameter types narrow normally when fully read
        "function foo(args: {x: number; y: number}) {
            return args.x + args.y;
        }",
        // Passing the whole parameter around exempts its type
        "function foo(args: {x: number; y: number}) {
            console.log(args);
        }",
        // Union-typed computed access keeps candidates alive
        "function foo(args: {x: number; y: number}, key: 'x' | 'y') {
            return args[key];
        }",
        // Member writes count as usage
        "function foo(args: {x: number; y: number}) {
            args.x = 1;
        }",
        // Calls count as reads
        "function foo(args: {x: () => void; y: number}) {
            args.x();
        }",
        // Named type aliases are not followed
        "type Arguments = {
            x: number;
            y: number;
        };
        function foo(args: Arguments) {
            return args.x;
        }",
        // Interfaces are not followed either
        "interface Arguments {
            x: number;
            y: number;
        }
        function foo(args: Arguments) {
            return args.x;
        }",
        // Ambient declarations are out of scope
        "declare const args: {x: number; y: number};
        console.log(args.x);",
        // Annotations alone can seed containers
        "let args: {x: number; y: number};
        console.log(args.x);",
        // Method signatures track like data members
        "function foo(args: {x: number; y(): void}) {
            return args.x;
        }",
        // Index signatures force keep-alive
        "function foo(args: {x: number; [key: string]: unknown}) {
            return args.x;
        }",
        // Call signatures force keep-alive
        "function foo(args: {x: number; (value: string): void}) {
            return args.x;
        }",
        // Rest elements consume every remaining annotation key
        "function foo({x, ...rest}: {x: number; y: number}) {
            console.log(x, rest);
        }",
        // Computed pattern keys cannot be matched against annotations
        "function foo({[key]: value}: {x: number; y: number}) {
            console.log(value);
        }",
        // Defaulted parameters may swap the whole bag; kept conservatively
        "function foo({x} = getDefault()) {
            console.log(x);
        }",
        // Whole-object aliasing ends per-property tracking, like any other escape;
        // the aliased binding's own fate belongs to `no-unused-vars`
        "const foo = {a: 1, u: 2};
        const bar = foo;
        console.log(bar.a);",
        // Self-references inside the initializer are classified like any other reference
        // and keep every property alive
        "const foo = {u: 1, copy: () => foo};
        console.log(foo.copy);",
    ];

    let fail = vec![
        // Static member access proves only `a`; `u` is never read
        "const foo = {a: 1, u: 2};
        console.log(foo.a);",
        // Quoted keys resolve exactly like identifier keys
        r#"const foo = {"a": 1, "u": 2};
        console.log(foo.a);"#,
        // Computed access with a literal key narrows as well
        "const foo = {a: 1, u: 2};
        console.log(foo['a']);",
        // Narrowing applies across function boundaries
        "const foo = {a: 1, u: 2};
        function main() {
            console.log(foo.a);
        }",
        // Destructuring declaration consumes only the keys it binds
        "const foo = {a: 1, u: 2};
        const {a} = foo;",
        // Same for destructuring assignment targets
        "const foo = {a: 1, u: 2};
        ({a} = foo);",
        // Destructuring straight from an initializer literal tracks that literal
        "const {a} = {u: 2};
        console.log(a);",
        // A nested object value does not rescue its never-read parent property
        "const foo = {
            a: 1,
            u: {
                b: 2,
                c: 3
            }
        };
        console.log(foo.a);",
        // Within a used subtree, nested members that are never reached still report
        "const foo = {
            a: 1,
            b: {
                c: 2,
                u: 3
            }
        };
        console.log(foo.a, foo.b.c);",
        // Same, observed from another function scope
        "const foo = {
            a: 1,
            b: {
                c: 2,
                u: 3
            }
        };
        function main() {
            console.log(foo.a, foo.b.c);
        }",
        // An assignment onto `foo.a` keeps `a` alive but says nothing about siblings
        "const foo = {
            a: {
                b: 1
            },
            u: 2
        };
        foo.a.f = function () { return this };",
        // Computed keys that cannot be resolved statically cannot be proven read
        "const foo = {
            a: 1,
            [u]: 2
        };
        console.log(foo.a);",
        // `__proto__` is exempt; ordinary unused siblings still report
        "const foo = {
            __proto__: {a: 1},
            b: 2,
            u: 3
        };
        console.log(foo.b);",
        // Self-references inside the initializer prove nothing about key usage
        "const foo = {
            [foo.bar]: 1
        };",
        // JSX member access narrows; unused siblings report
        "const styles = {
            wrapper: styled('div', {}),
            unused: styled('div', {}),
        };
        function Component() {
            return <styles.wrapper />;
        }",
        // Inline parameter-bag types narrow down to the accessed members
        "function foo(args: {x: number; y: number}) {
            return args.x * 2;
        }",
        // Typed declarators behave like parameter bags
        "const args: {x: number; y: number} = getArgs();
        console.log(args.x);",
        // Nested inline types narrow recursively
        "function foo(args: {options: {enabled: boolean; unused: boolean}; label: string}) {
            return args.options.enabled && args.label.length > 0;
        }",
        // String-literal keys in type literals resolve like identifiers
        "function foo(args: {'x': number; 'y': number}) {
            return args['x'];
        }",
        // When both exist, the initializer object wins over the annotation;
        // its own unused entries report
        "type Arguments = {
            x: number;
            unused: number;
        };
        const args: Arguments = {x: 1, unused: 2};
        console.log(args.x);",
        // Non-null assertions are unwrapped while narrowing
        "function foo(args: {x: {a: number; b: number}; y: number}) {
            return args.x!.a;
        }",
        // Type casts are unwrapped while narrowing
        "function foo(args: {x: {a: number; b: number}; y: number}) {
            return (args.x as {a: number; b: number}).a;
        }",
        // `as const` wrappers keep narrowing intact
        "const args = {x: 1, y: 2} as const;
        console.log(args.x);",
        // `satisfies` wrappers keep narrowing intact
        "const args = {x: 1, y: 2} satisfies {x: number; y: number};
        console.log(args.x);",
        // Function-local object bags behave identically
        "function foo() {
            const bar = {
                b: 2,
                u: 3
            };
            console.log(bar.b);
        }",
        // NOTE: Stricter than upstream:
        // destructured typed bindings keep tracking;
        // unbound annotation keys cannot be reached through any expression
        "function foo({x}: {x: number; y: number}) {
            return x;
        }",
        "const {x}: {x: number; y: number} = args;
        console.log(x);",
        // Nested patterns narrow recursively through the annotation
        "function foo({options: {enabled}}: {options: {enabled: boolean; unused: boolean}}) {
            return enabled;
        }",
        // NOTE: Divergence from upstream:
        // optional chaining narrows like a normal member access instead of keeping every property alive
        "function foo(args: {x: number; y: number}) {
            return args?.y;
        }",
        "const foo = {a: 1, u: 2};
        console.log(foo?.a);",
    ];

    Tester::new(NoUnusedProperties::NAME, NoUnusedProperties::PLUGIN, pass, fail)
        .test_and_snapshot();
}
