use oxc_ast::{
    AstKind,
    ast::{BindingPattern, ClassType, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use oxc_syntax::symbol::SymbolId;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn swapped_reference_diagnostic(span: Span, preferred: &str, instead_of: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `{preferred}` instead of `{instead_of}` in static methods."))
        .with_label(span)
}

fn unavailable_name_diagnostic(span: Span, keyword: &str) -> OxcDiagnostic {
    let kind = if keyword == "this" { "class" } else { "superclass" };
    OxcDiagnostic::warn(format!("Use the {kind} name instead of `{keyword}` in static methods."))
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ClassReferenceInStaticMethodsOptions {
    /// Prefer `this` over the current class name in static methods.
    prefer_this: bool,
    /// Prefer `super` over the superclass name in static methods.
    prefer_super: bool,
}

impl Default for ClassReferenceInStaticMethodsOptions {
    fn default() -> Self {
        Self { prefer_this: true, prefer_super: true }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClassReferenceInStaticMethods {
    options: ClassReferenceInStaticMethodsOptions,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces consistent references to the current class and its superclass within static methods:
    /// either always use dynamic dispatch with `this`/`super`, or always reference the class names directly.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing `this`/`super` with direct class-name references makes it unclear whether a static method is
    /// intended to follow subclass dispatch (dynamic binding) or stay tied to a specific class. Choosing one
    /// style makes the intent obvious.
    ///
    /// By default, this rule prefers `this` over the current class name and `super` over the superclass name.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class Foo extends Bar {
    ///     static baz() {
    ///         Foo.qux();
    ///         Bar.qux();
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class Foo extends Bar {
    ///     static baz() {
    ///         this.qux();
    ///         super.qux();
    ///     }
    /// }
    /// ```
    ClassReferenceInStaticMethods,
    unicorn,
    nursery,
    suggestion,
    config = ClassReferenceInStaticMethodsOptions,
    version = "next",
    short_description = "Enforce consistent class references in static methods.",
);

impl Rule for ClassReferenceInStaticMethods {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<ClassReferenceInStaticMethodsOptions>::from_value(value)
            .map(|config| Self { options: config.into_inner() })
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ThisExpression(_) => {
                if self.options.prefer_this {
                    return;
                }
                check_this(node.id(), ctx);
            }
            AstKind::Super(_) => {
                if self.options.prefer_super {
                    return;
                }
                check_super(node.id(), ctx);
            }
            AstKind::IdentifierReference(identifier) => {
                check_identifier(&self.options, identifier.name.as_str(), node.id(), ctx);
            }
            _ => {}
        }
    }
}

/// The name and binding that can be used to refer to a class or its superclass.
///
/// `symbol_id` is `None` when the name is not bound in the program (e.g. a bare
/// `class A extends B` where `B` is an external free variable). Two unresolved references
/// of the same name are considered identical.
struct ClassReferenceName<'a> {
    name: &'a str,
    symbol_id: Option<SymbolId>,
}

fn check_this(node_id: NodeId, ctx: &LintContext<'_>) {
    if is_assignment_target(ctx, node_id)
        || is_direct_callee(ctx, node_id)
        || is_private_member_access(ctx, node_id)
        || is_private_brand_check(ctx, node_id)
    {
        return;
    }

    let Some(static_method_id) = get_static_method(ctx, node_id) else {
        return;
    };
    let Some((class_id, class)) = enclosing_class(ctx, static_method_id) else {
        return;
    };

    let reference_name = class_reference_node(ctx, class_id, class).filter(|reference| {
        // If the class name is shadowed at the reference location, it can't be used.
        ctx.scoping().find_binding(ctx.nodes().get_node(node_id).scope_id(), reference.name.into())
            == reference.symbol_id
    });

    let Some(reference_name) = reference_name else {
        ctx.diagnostic(unavailable_name_diagnostic(span_of(ctx, node_id), "this"));
        return;
    };

    let span = replacement_span(ctx, node_id);
    let name = reference_name.name;
    ctx.diagnostic_with_suggestion(
        swapped_reference_diagnostic(span_of(ctx, node_id), name, "this"),
        |fixer| replace_with(fixer, span, name),
    );
}

fn check_super(node_id: NodeId, ctx: &LintContext<'_>) {
    if is_assignment_target(ctx, node_id) {
        return;
    }

    let Some(static_method_id) = get_static_method(ctx, node_id) else {
        return;
    };
    let Some((_, class)) = enclosing_class(ctx, static_method_id) else {
        return;
    };

    let reference_name = superclass_reference_node(ctx, class).filter(|reference| {
        ctx.scoping().find_binding(ctx.nodes().get_node(node_id).scope_id(), reference.name.into())
            == reference.symbol_id
    });

    let Some(reference_name) = reference_name else {
        ctx.diagnostic(unavailable_name_diagnostic(span_of(ctx, node_id), "super"));
        return;
    };

    let span = replacement_span(ctx, node_id);
    let name = reference_name.name;
    ctx.diagnostic_with_suggestion(
        swapped_reference_diagnostic(span_of(ctx, node_id), name, "super"),
        |fixer| replace_with(fixer, span, name),
    );
}

fn check_identifier(
    options: &ClassReferenceInStaticMethodsOptions,
    name: &str,
    node_id: NodeId,
    ctx: &LintContext<'_>,
) {
    if is_type_position_identifier(ctx, node_id)
        || is_assignment_target(ctx, node_id)
        || is_direct_callee(ctx, node_id)
        || is_private_member_access(ctx, node_id)
        || is_private_brand_check(ctx, node_id)
    {
        return;
    }

    let Some(static_method_id) = get_static_method(ctx, node_id) else {
        return;
    };
    let Some((class_id, class)) = enclosing_class(ctx, static_method_id) else {
        return;
    };

    if options.prefer_this
        && let Some(reference_name) = class_reference_node(ctx, class_id, class)
            .filter(|r| r.name == name && resolves_to(ctx, node_id, r))
    {
        let span = replacement_span(ctx, node_id);
        let report_name = reference_name.name;
        ctx.diagnostic_with_suggestion(
            swapped_reference_diagnostic(span_of(ctx, node_id), "this", report_name),
            |fixer| replace_with(fixer, span, "this"),
        );
        return;
    }

    if !options.prefer_super {
        return;
    }

    // Only simple, unparenthesized member accesses like `Super.foo()` can become `super.foo()`.
    if !is_simple_member_access(ctx, node_id) || is_parenthesized(ctx, node_id) {
        return;
    }

    if let Some(reference_name) = superclass_reference_node(ctx, class)
        .filter(|r| r.name == name && resolves_to(ctx, node_id, r))
    {
        let span = replacement_span(ctx, node_id);
        let report_name = reference_name.name;
        ctx.diagnostic_with_suggestion(
            swapped_reference_diagnostic(span_of(ctx, node_id), "super", report_name),
            |fixer| replace_with(fixer, span, "super"),
        );
    }
}

fn replace_with(fixer: RuleFixer<'_, '_>, span: Span, replacement: &str) -> RuleFix {
    fixer
        .replace(span, replacement.to_string())
        .with_message(format!("Replace `{}` with `{replacement}`.", fixer.source_range(span)))
}

fn span_of(ctx: &LintContext<'_>, node_id: NodeId) -> Span {
    ctx.nodes().kind(node_id).span()
}

/// Returns the enclosing static method (`MethodDefinition`) for a node inside its body.
///
/// Arrow functions are transparent (they inherit `this`); any other nested function breaks out of
/// the method; class fields, accessors and static blocks are not methods.
fn get_static_method(ctx: &LintContext<'_>, node_id: NodeId) -> Option<NodeId> {
    let mut child_span = span_of(ctx, node_id);
    for parent_id in ctx.nodes().ancestor_ids(node_id) {
        let parent_span = span_of(ctx, parent_id);
        match ctx.nodes().kind(parent_id) {
            AstKind::Function(_) => {
                // The function must be a method's value, not e.g. a computed key function.
                match ctx.nodes().kind(ctx.nodes().parent_id(parent_id)) {
                    AstKind::MethodDefinition(method) if method.value.span() == parent_span => {}
                    _ => return None,
                }
            }
            AstKind::MethodDefinition(method) => {
                return (method.r#static && method.value.span() == child_span).then_some(parent_id);
            }
            // Type annotations live inside the method subtree but contain no runtime references.
            AstKind::PropertyDefinition(_)
            | AstKind::AccessorProperty(_)
            | AstKind::StaticBlock(_)
            | AstKind::ClassBody(_)
            | AstKind::TSTypeAnnotation(_)
            | AstKind::TSTypeQuery(_) => return None,
            _ => {}
        }
        child_span = parent_span;
    }
    None
}

/// The class enclosing a static method definition.
fn enclosing_class<'a>(
    ctx: &LintContext<'a>,
    static_method_id: NodeId,
) -> Option<(NodeId, &'a oxc_ast::ast::Class<'a>)> {
    for ancestor_id in ctx.nodes().ancestor_ids(static_method_id) {
        if let AstKind::Class(class) = ctx.nodes().kind(ancestor_id) {
            return Some((ancestor_id, class));
        }
    }
    None
}

/// The identifier usable to refer to the given class: its binding identifier, or, for an anonymous
/// class expression assigned to a variable, the variable's identifier (`const A = class {}`).
fn class_reference_node<'a>(
    ctx: &LintContext<'a>,
    class_id: NodeId,
    class: &'a oxc_ast::ast::Class<'a>,
) -> Option<ClassReferenceName<'a>> {
    if let Some(id) = &class.id {
        return Some(ClassReferenceName {
            name: id.name.as_str(),
            symbol_id: Some(id.symbol_id()),
        });
    }

    if class.r#type != ClassType::ClassExpression {
        return None;
    }
    // Anonymous class expression: use the variable name it is assigned to, if any.
    match ctx.nodes().kind(ctx.nodes().parent_id(class_id)) {
        AstKind::VariableDeclarator(declarator)
            if declarator.init.as_ref().map(GetSpan::span) == Some(class.span()) =>
        {
            if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                return Some(ClassReferenceName {
                    name: id.name.as_str(),
                    symbol_id: Some(id.symbol_id()),
                });
            }
            None
        }
        _ => None,
    }
}

fn superclass_reference_node<'a>(
    ctx: &LintContext<'a>,
    class: &'a oxc_ast::ast::Class<'a>,
) -> Option<ClassReferenceName<'a>> {
    let heritage = class.heritage.as_ref()?;
    let Expression::Identifier(super_class) = &heritage.expression else {
        return None;
    };
    let symbol_id = ctx.scoping().get_reference(super_class.reference_id()).symbol_id();
    Some(ClassReferenceName { name: super_class.name.as_str(), symbol_id })
}

/// Whether an identifier reference resolves to the expected symbol (shadowing-aware).
fn resolves_to(ctx: &LintContext<'_>, node_id: NodeId, reference: &ClassReferenceName<'_>) -> bool {
    let AstKind::IdentifierReference(identifier) = ctx.nodes().kind(node_id) else {
        return false;
    };
    ctx.scoping().get_reference(identifier.reference_id()).symbol_id() == reference.symbol_id
}

/// `this`/an identifier wrapped by a `ChainExpression` must be replaced together with the wrapper.
fn replacement_span(ctx: &LintContext<'_>, node_id: NodeId) -> Span {
    let parent_id = ctx.nodes().parent_id(node_id);
    if let AstKind::ChainExpression(chain) = ctx.nodes().kind(parent_id)
        && chain.expression.span() == span_of(ctx, node_id)
    {
        return chain.span();
    }
    span_of(ctx, node_id)
}

fn is_parenthesized(ctx: &LintContext<'_>, node_id: NodeId) -> bool {
    matches!(ctx.nodes().kind(ctx.nodes().parent_id(node_id)), AstKind::ParenthesizedExpression(_))
}

/// Whether the identifier at `node_id` sits in a TypeScript type position (`A` in `: A`,
/// `typeof A`, qualified names), where it names a type, not a value.
fn is_type_position_identifier(ctx: &LintContext<'_>, mut node_id: NodeId) -> bool {
    loop {
        let parent_id = ctx.nodes().parent_id(node_id);
        match ctx.nodes().kind(parent_id) {
            AstKind::TSQualifiedName(_) => node_id = parent_id,
            AstKind::TSTypeReference(_) | AstKind::TSTypeQuery(_) => return true,
            _ => return false,
        }
    }
}

/// Climbs through TypeScript expression wrappers (`as`, `!`, `satisfies`, ...) and parenthesized
/// expressions, returning the innermost non-wrapper node.
fn climb_ts_wrappers(ctx: &LintContext<'_>, mut node_id: NodeId) -> NodeId {
    loop {
        let node_span = span_of(ctx, node_id);
        let parent_id = ctx.nodes().parent_id(node_id);
        let wrapped = match ctx.nodes().kind(parent_id) {
            AstKind::TSAsExpression(expr) => expr.expression.span(),
            AstKind::TSInstantiationExpression(expr) => expr.expression.span(),
            AstKind::TSNonNullExpression(expr) => expr.expression.span(),
            AstKind::TSSatisfiesExpression(expr) => expr.expression.span(),
            AstKind::TSTypeAssertion(expr) => expr.expression.span(),
            AstKind::ParenthesizedExpression(expr) => expr.expression.span(),
            _ => return node_id,
        };
        if wrapped != node_span {
            return node_id;
        }
        node_id = parent_id;
    }
}

fn is_direct_callee(ctx: &LintContext<'_>, node_id: NodeId) -> bool {
    let inner = climb_ts_wrappers(ctx, node_id);
    let inner_span = span_of(ctx, inner);
    match ctx.nodes().kind(ctx.nodes().parent_id(inner)) {
        AstKind::CallExpression(call) => call.callee.span() == inner_span,
        AstKind::NewExpression(new) => new.callee.span() == inner_span,
        AstKind::TaggedTemplateExpression(tag) => tag.tag.span() == inner_span,
        _ => false,
    }
}

fn is_private_member_access(ctx: &LintContext<'_>, node_id: NodeId) -> bool {
    let inner = climb_ts_wrappers(ctx, node_id);
    let inner_span = span_of(ctx, inner);
    matches!(
        ctx.nodes().kind(ctx.nodes().parent_id(inner)),
        AstKind::PrivateFieldExpression(field) if field.object.span() == inner_span
    )
}

/// `#foo in X`
fn is_private_brand_check(ctx: &LintContext<'_>, node_id: NodeId) -> bool {
    let inner = climb_ts_wrappers(ctx, node_id);
    let inner_span = span_of(ctx, inner);
    matches!(
        ctx.nodes().kind(ctx.nodes().parent_id(inner)),
        AstKind::PrivateInExpression(private_in) if private_in.right.span() == inner_span
    )
}

/// Simple, non-optional member access used as an object: `Super.foo()` / `Super["foo"]()`.
fn is_simple_member_access(ctx: &LintContext<'_>, node_id: NodeId) -> bool {
    let parent_id = ctx.nodes().parent_id(node_id);
    let node_span = span_of(ctx, node_id);
    let is_object = match ctx.nodes().kind(parent_id) {
        AstKind::StaticMemberExpression(member) => {
            !member.optional && member.object.span() == node_span
        }
        AstKind::ComputedMemberExpression(member) => {
            !member.optional && member.object.span() == node_span
        }
        _ => false,
    };
    is_object && !is_assignment_target(ctx, parent_id)
}

/// Whether the node is part of an assignment target: the root of `x = ...`, `x++`,
/// `delete x.y`, `for (x in/of ...)`, including destructuring targets like
/// `({foo: A.bar} = object)` or `[...A.baz] = values`.
fn is_assignment_target(ctx: &LintContext<'_>, start: NodeId) -> bool {
    let mut current = start;
    loop {
        let parent_id = ctx.nodes().parent_id(current);
        let span = span_of(ctx, current);
        match ctx.nodes().kind(parent_id) {
            // Assignment target roots.
            AstKind::AssignmentExpression(expr) => return expr.left.span() == span,
            AstKind::UpdateExpression(expr) => return expr.argument.span() == span,
            AstKind::UnaryExpression(expr) => {
                return expr.operator == UnaryOperator::Delete && expr.argument.span() == span;
            }
            AstKind::ForInStatement(expr) => return expr.left.span() == span,
            AstKind::ForOfStatement(expr) => return expr.left.span() == span,
            // Wrappers that keep the node inside the assignment target.
            AstKind::StaticMemberExpression(expr) if expr.object.span() == span => {
                current = parent_id;
            }
            AstKind::ComputedMemberExpression(expr) if expr.object.span() == span => {
                current = parent_id;
            }
            AstKind::PrivateFieldExpression(expr) if expr.object.span() == span => {
                current = parent_id;
            }
            AstKind::ChainExpression(expr) if expr.expression.span() == span => current = parent_id,
            AstKind::ParenthesizedExpression(expr) if expr.expression.span() == span => {
                current = parent_id;
            }
            AstKind::TSAsExpression(expr) if expr.expression.span() == span => current = parent_id,
            AstKind::TSInstantiationExpression(expr) if expr.expression.span() == span => {
                current = parent_id;
            }
            AstKind::TSNonNullExpression(expr) if expr.expression.span() == span => {
                current = parent_id;
            }
            AstKind::TSSatisfiesExpression(expr) if expr.expression.span() == span => {
                current = parent_id;
            }
            AstKind::TSTypeAssertion(expr) if expr.expression.span() == span => {
                current = parent_id;
            }
            // Destructuring-target containers.
            AstKind::ObjectAssignmentTarget(_) | AstKind::ArrayAssignmentTarget(_) => {
                current = parent_id;
            }
            AstKind::AssignmentTargetWithDefault(expr) if expr.binding.span() == span => {
                current = parent_id;
            }
            AstKind::AssignmentTargetRest(expr) if expr.target.span() == span => {
                current = parent_id;
            }
            AstKind::AssignmentTargetPropertyProperty(expr) if expr.binding.span() == span => {
                current = parent_id;
            }
            _ => return false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("class A {static foo() {return this.foo();}}", None),
        ("class A extends B {static foo() {return super.foo();}}", None),
        ("class A {foo() {return A.foo();}}", None),
        ("class A extends B {foo() {return B.foo();}}", None),
        ("class A {static foo = A.foo;}", None),
        ("class A {static {A.foo();}}", None),
        ("class A {static foo() {return function () {return A.foo();};}}", None),
        ("class A extends B {static foo() {return function () {return B.foo();};}}", None),
        ("class A {static foo() {const A = other; return A.foo();}}", None),
        ("class A extends B {static foo() {const B = other; return B.foo();}}", None),
        ("class A extends mixin(B) {static foo() {return B.foo();}}", None),
        ("class A {static foo() {A();}}", None),
        ("class A {static foo() {new A();}}", None),
        ("class A {static foo() {A`foo`;}}", None),
        ("class A {static foo() {A.foo = 1;}}", None),
        ("class A {static foo() {A.foo.bar = 1;}}", None),
        ("class A {static foo() {({foo: A.foo} = object);}}", None),
        ("class A {static foo() {({foo: A.foo = fallback} = object);}}", None),
        ("class A {static foo() {({...A.foo} = object);}}", None),
        ("class A {static foo() {([A.foo] = values);}}", None),
        ("class A {static foo() {A.foo++;}}", None),
        ("class A {static foo() {A.foo.bar++;}}", None),
        ("class A {static foo() {delete A.foo;}}", None),
        ("class A {static foo() {delete A?.foo;}}", None),
        ("class A {static foo() {delete A.foo.bar;}}", None),
        ("class A {static foo() {for (A.foo in object) {}}}", None),
        ("class A {static foo() {for (A.foo of values) {}}}", None),
        ("class A extends B {static foo() {B();}}", None),
        ("class A extends B {static foo() {B.foo = 1;}}", None),
        ("class A extends B {static foo() {B.foo.bar = 1;}}", None),
        ("class A extends B {static foo() {({foo: B.foo} = object);}}", None),
        ("class A extends B {static foo() {({foo: B.foo.bar} = object);}}", None),
        ("class A extends B {static foo() {({...B.foo} = object);}}", None),
        ("class A extends B {static foo() {([B.foo] = values);}}", None),
        ("class A extends B {static foo() {B.foo++;}}", None),
        ("class A extends B {static foo() {B.foo.bar++;}}", None),
        ("class A extends B {static foo() {delete B.foo;}}", None),
        ("class A extends B {static foo() {delete B.foo.bar;}}", None),
        ("class A extends B {static foo() {for (B.foo in object) {}}}", None),
        ("class A extends B {static foo() {for (B.foo of values) {}}}", None),
        ("class A extends B {static foo() {new B();}}", None),
        ("class A extends B {static foo() {B`foo`;}}", None),
        ("class A extends B {static foo() {return (B).foo();}}", None),
        (r#"class A extends B {static foo() {return (B)["foo"]();}}"#, None),
        ("class A extends B {static foo() {return {B};}}", None),
        ("class A {static #foo; static foo() {return A.#foo;}}", None),
        ("class A {static #foo; static foo() {return #foo in A;}}", None),
        ("class A extends B {static #foo; static foo() {return B.#foo;}}", None),
        ("class A extends B {}", None),
        (
            "class A {static foo() {return A.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {return B.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {foo() {return this.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {foo() {return super.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo = this.foo;}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {this();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {new this();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {this`foo`;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {this.foo = 1;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {this.foo.bar = 1;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {({foo: this.foo} = object);}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {([this.foo] = values);}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {({...this.foo} = object);}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {delete this?.foo;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static #foo; static foo() {return this.#foo;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static #foo; static foo() {return #foo in this;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {for (this.foo in object) {}}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {for (this.foo of values) {}}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static {super.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {super.foo = 1;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {super.foo.bar = 1;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {({foo: super.foo} = object);}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {([super.foo] = values);}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {({...super.foo} = object);}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {for (super.foo in object) {}}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {for (super.foo of values) {}}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {return function () {return this.foo();};}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {return function () {return this.foo();};}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {return A.foo() + super.foo();}}",
            Some(serde_json::json!([{"preferThis": false}])),
        ),
        (
            "class A extends B {static foo() {return this.foo() + B.foo();}}",
            Some(serde_json::json!([{"preferSuper": false}])),
        ),
        ("class A {static foo(): unknown {return this.foo();}}", None),
        ("class A extends B {static foo(): unknown {return super.foo();}}", None),
        ("class A {static foo(): A {return this.foo();}}", None),
        ("class A {static foo() {const value: A = this.foo(); return value;}}", None),
        ("class A {static foo() {return identity<A>(this.foo());}}", None),
        ("class A {static foo() {const value: typeof A = this.foo(); return value;}}", None),
        ("class A {static foo() {(A as typeof A)();}}", None),
        ("class A {static foo() {new (A as typeof A)();}}", None),
        ("class A {static foo() {(A as typeof A)`foo`;}}", None),
        ("class A {static foo() {A!();}}", None),
        ("class A {static foo() {(A satisfies typeof A)();}}", None),
        ("class A {static foo() {(A as typeof A).foo = 1;}}", None),
        ("class A {static foo() {A!.foo = 1;}}", None),
        ("class A {static foo() {(A satisfies typeof A).foo = 1;}}", None),
        ("class A {static #foo; static foo() {return (A as typeof A).#foo;}}", None),
        ("class A {static #foo; static foo() {return A!.#foo;}}", None),
        ("class A {static #foo; static foo() {return #foo in (A as typeof A);}}", None),
        ("class A {static #foo; static foo() {return #foo in A!;}}", None),
        ("class A extends B {static foo() {(B as typeof B).foo = 1;}}", None),
        (
            "class A {static foo() {(this as typeof A).foo = 1;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {this!.foo = 1;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static #foo; static foo() {return (this as typeof A).#foo;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static #foo; static foo() {return this!.#foo;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static #foo; static foo() {return #foo in (this as typeof A);}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static #foo; static foo() {return #foo in this!;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {(this as typeof A)();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {new (this as typeof A)();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {(this as typeof A)`foo`;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {this!();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
    ];

    let fail = vec![
        ("class A {static foo() {return A.foo();}}", None),
        ("class A {static foo() {return A;}}", None),
        ("class A extends B {static foo() {return B.foo();}}", None),
        (r#"class A extends B {static foo() {return B["foo"]();}}"#, None),
        ("class A extends B {static foo() {return () => B.foo();}}", None),
        (
            "const A = class {
                static foo() {
                    return A.foo();
                }
            };",
            None,
        ),
        (
            "const A = class B {
                static foo() {
                    return B.foo();
                }
            };",
            None,
        ),
        (
            "class A {static foo() {return this.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {return this;}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A {static foo() {const A = other; return this.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {const B = other; return super.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {return super.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {return () => super.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "const A = class {static foo() {return this.foo();}};",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "export default class {static foo() {return this.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends mixin(B) {static foo() {return super.foo();}}",
            Some(serde_json::json!([{"preferThis": false, "preferSuper": false}])),
        ),
        (
            "class A extends B {static foo() {return this.foo() + B.foo();}}",
            Some(serde_json::json!([{"preferThis": false}])),
        ),
        (
            "class A extends B {static foo() {return A.foo() + super.foo();}}",
            Some(serde_json::json!([{"preferSuper": false}])),
        ),
        ("class A {static foo(): unknown {return A.foo();}}", None),
        ("class A {static foo(): A {return A.foo();}}", None),
        ("class A extends B {static foo(): unknown {return B.foo();}}", None),
    ];

    let fix: Vec<(&str, &str, Option<serde_json::Value>)> = vec![
        (
            "class A {static foo() {return A.foo();}}",
            "class A {static foo() {return this.foo();}}",
            None,
        ),
        (
            "class A extends B {static foo() {return B.foo();}}",
            "class A extends B {static foo() {return super.foo();}}",
            None,
        ),
        (
            "const A = class {static foo() {return A.foo();}};",
            "const A = class {static foo() {return this.foo();}};",
            None,
        ),
        (
            "class A {static foo() {return this;}}",
            "class A {static foo() {return A;}}",
            Some(serde_json::json!([{"preferThis": false}])),
        ),
    ];

    Tester::new(
        ClassReferenceInStaticMethods::NAME,
        ClassReferenceInStaticMethods::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}

// The `<typeof A>A` type assertion is TS-only syntax that cannot be parsed as TSX,
// so it runs in its own Tester below, parsed as TypeScript.
#[test]
fn test_tsx() {
    use crate::tester::Tester;

    let pass = vec![
        ("class A {static foo() {(<typeof A>A)();}}", None),
        ("class A {static foo() {(<typeof A>A).foo = 1;}}", None),
    ];

    Tester::new(
        ClassReferenceInStaticMethods::NAME,
        ClassReferenceInStaticMethods::PLUGIN,
        pass,
        vec![],
    )
    .intentionally_allow_no_fix_tests()
    .change_rule_path_extension("ts")
    .test();
}
