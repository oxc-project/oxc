use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{
        AssignmentOperator, AssignmentTarget, AssignmentTargetProperty, BindingIdentifier,
        BindingPattern, ClassElement, Expression, IdentifierReference, MethodDefinitionKind,
        PropertyKey, TSAccessibility, TSType, TSTypeName, TSTypeQueryExprName,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, Semantic};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    class::{ElementId, ElementKind},
    symbol::SymbolId,
};
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule};

fn no_unused_private_class_members_diagnostic(
    name: impl std::fmt::Display,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is defined but never used."))
        .with_help("Remove the declaration or use it in the code.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnusedPrivateClassMembers;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unused private class members.
    ///
    /// ### Why is this bad?
    ///
    /// Private class members that are declared and not used anywhere in the code are most likely an error due to incomplete refactoring. Such class members take up space in the code and can lead to confusion by readers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class A {
    ///		#unusedMember = 5;
    ///	}
    ///
    ///	class B {
    ///			#usedOnlyInWrite = 5;
    ///			method() {
    ///					this.#usedOnlyInWrite = 42;
    ///			}
    ///	}
    ///
    ///	class C {
    ///			#usedOnlyToUpdateItself = 5;
    ///			method() {
    ///					this.#usedOnlyToUpdateItself++;
    ///			}
    ///	}
    ///
    ///	class D {
    ///			#unusedMethod() {}
    ///	}
    ///
    ///	class E {
    ///			get #unusedAccessor() {}
    ///			set #unusedAccessor(value) {}
    ///	}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class A {
    ///		#usedMember = 42;
    ///		method() {
    ///				return this.#usedMember;
    ///		}
    ///	}
    ///
    ///	class B {
    ///			#usedMethod() {
    ///					return 42;
    ///			}
    ///			anotherMethod() {
    ///					return this.#usedMethod();
    ///			}
    ///	}
    ///
    ///	class C {
    ///			get #usedAccessor() {}
    ///			set #usedAccessor(value) {}
    ///
    ///			method() {
    ///					this.#usedAccessor = 42;
    ///			}
    ///	}
    /// ```
    NoUnusedPrivateClassMembers,
    eslint,
    correctness,
    recommended,
    pending,
    version = "0.1.1",
    short_description = "Disallow unused private class members.",
);

impl Rule for NoUnusedPrivateClassMembers {
    fn run_once<'a>(&self, ctx: &LintContext<'a>) {
        let is_typescript = ctx.source_type().is_typescript();
        let mut members = TsPrivateMembers::default();

        ctx.classes().iter_enumerated().for_each(|(class_id, class_node_id)| {
            let AstKind::Class(class) = ctx.nodes().kind(*class_node_id) else { return };
            let mut push = |name: &'a str, span: Span, is_property: bool, is_static: bool| {
                members.push(TsPrivateMember {
                    class_node_id: *class_node_id,
                    name,
                    span,
                    is_property,
                    is_static,
                    used: false,
                });
            };
            // The class table holds one element per named property, accessor and non-constructor
            // method, in source order; `element_index` tracks the current element's id.
            let mut element_index = 0;
            for element in &class.body.body {
                let (key, accessibility, kind, is_static) = match element {
                    ClassElement::PropertyDefinition(def) => {
                        (&def.key, def.accessibility, ElementKind::Property, def.r#static)
                    }
                    ClassElement::AccessorProperty(def) => {
                        (&def.key, def.accessibility, ElementKind::Accessor, def.r#static)
                    }
                    ClassElement::MethodDefinition(def) => {
                        if def.kind == MethodDefinitionKind::Constructor {
                            if is_typescript {
                                for param in &def.value.params.items {
                                    if param.accessibility == Some(TSAccessibility::Private)
                                        && let BindingPattern::BindingIdentifier(id) =
                                            &param.pattern
                                    {
                                        push(id.name.as_str(), param.span, true, false);
                                    }
                                }
                            }
                            continue;
                        }
                        if def.value.is_typescript_syntax() {
                            continue;
                        }
                        (&def.key, def.accessibility, ElementKind::Method, def.r#static)
                    }
                    _ => continue,
                };
                if key.name().is_none() {
                    continue;
                }
                let element_id = ElementId::from_usize(element_index);
                element_index += 1;

                if key.is_private_identifier() {
                    if kind == ElementKind::Accessor {
                        continue;
                    }
                    let element = &ctx.classes().elements[class_id][element_id];
                    if !ctx.classes().iter_private_identifiers(class_id).any(|ident| {
                        // If the element is a property, it must be read.
                        (!element.kind.is_property() || is_read(ident.id, ctx.semantic()))
                            && ident.element_ids.contains(&element_id)
                    }) {
                        ctx.diagnostic(no_unused_private_class_members_diagnostic(
                            element.name.display(),
                            element.span,
                        ));
                    }
                } else if accessibility == Some(TSAccessibility::Private)
                    && !element_is_computed(element)
                    && let Some(name) = static_key_name(key)
                {
                    push(name, key.span(), kind == ElementKind::Property, is_static);
                }
            }
        });

        // Nodes are numbered in pre-order, so a class's descendants directly follow it. Only the
        // outermost class with TypeScript `private` members needs scanning, as it covers nested ones.
        let mut class_node_ids: Vec<NodeId> =
            members.list.iter().map(|m| m.class_node_id).collect();
        class_node_ids.dedup();
        let mut scanned_end = 0;
        let mut scan = |node: &AstNode<'a>| match node.kind() {
            AstKind::StaticMemberExpression(member) => {
                mark_used(
                    ctx,
                    &mut members,
                    node,
                    &member.object,
                    member.property.name.as_str(),
                    true,
                );
            }
            AstKind::ComputedMemberExpression(member) => {
                if let Expression::StringLiteral(lit) = member.expression.without_parentheses()
                    && let Some(name) = lit.value.as_str()
                {
                    mark_used(ctx, &mut members, node, &member.object, name, true);
                }
            }
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    mark_pattern_used(ctx, &mut members, node, &decl.id, init);
                }
            }
            AstKind::FormalParameter(param) => {
                if let Some(init) = &param.initializer {
                    mark_pattern_used(ctx, &mut members, node, &param.pattern, init);
                }
            }
            AstKind::AssignmentPattern(pattern) => {
                mark_pattern_used(ctx, &mut members, node, &pattern.left, &pattern.right);
            }
            AstKind::AssignmentExpression(assign) => {
                if let AssignmentTarget::ObjectAssignmentTarget(target) = &assign.left {
                    for prop in &target.properties {
                        let name = match prop {
                            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(p) => {
                                Some(p.binding.name.as_str())
                            }
                            AssignmentTargetProperty::AssignmentTargetPropertyProperty(p)
                                if !p.computed =>
                            {
                                static_key_name(&p.name)
                            }
                            AssignmentTargetProperty::AssignmentTargetPropertyProperty(_) => None,
                        };
                        if let Some(name) = name {
                            mark_used(ctx, &mut members, node, &assign.right, name, false);
                        }
                    }
                }
            }
            _ => {}
        };

        for class_node_id in class_node_ids {
            let class_span = ctx.nodes().get_node(class_node_id).span();
            if class_span.start < scanned_end {
                continue;
            }
            scanned_end = class_span.end;
            for node in ctx.nodes().iter().skip(class_node_id.index() + 1) {
                if node.span().start >= class_span.end {
                    break;
                }
                scan(node);
            }
        }

        for member in members.list.iter().filter(|m| !m.used) {
            ctx.diagnostic(no_unused_private_class_members_diagnostic(member.name, member.span));
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.semantic().classes().len() > 0
    }
}

struct TsPrivateMember<'a> {
    class_node_id: NodeId,
    name: &'a str,
    span: Span,
    is_property: bool,
    is_static: bool,
    used: bool,
}

/// Collected members plus an index by name, so accesses to unrelated names are rejected cheaply.
#[derive(Default)]
struct TsPrivateMembers<'a> {
    list: Vec<TsPrivateMember<'a>>,
    by_name: FxHashMap<&'a str, Vec<usize>>,
}

impl<'a> TsPrivateMembers<'a> {
    /// Adds a member unless the same class already has one with that name and staticness,
    /// as with a getter/setter pair.
    fn push(&mut self, member: TsPrivateMember<'a>) {
        let indices = self.by_name.entry(member.name).or_default();
        if indices.iter().any(|&i| {
            let m = &self.list[i];
            m.class_node_id == member.class_node_id && m.is_static == member.is_static
        }) {
            return;
        }
        indices.push(self.list.len());
        self.list.push(member);
    }
}

fn mark_pattern_used<'a>(
    ctx: &LintContext<'a>,
    members: &mut TsPrivateMembers<'a>,
    node: &AstNode<'a>,
    pattern: &BindingPattern<'a>,
    source: &Expression<'a>,
) {
    let BindingPattern::ObjectPattern(pattern) = pattern else { return };
    for prop in &pattern.properties {
        if !prop.computed
            && let Some(name) = static_key_name(&prop.key)
        {
            mark_used(ctx, members, node, source, name, false);
        }
    }
}

fn element_is_computed(element: &ClassElement) -> bool {
    match element {
        ClassElement::PropertyDefinition(def) => def.computed,
        ClassElement::AccessorProperty(def) => def.computed,
        ClassElement::MethodDefinition(def) => def.computed,
        _ => false,
    }
}

fn static_key_name<'a>(key: &PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(lit) => lit.value.as_str(),
        _ => None,
    }
}

fn mark_used<'a>(
    ctx: &LintContext<'a>,
    members: &mut TsPrivateMembers<'a>,
    node: &AstNode<'a>,
    object: &Expression<'a>,
    name: &'a str,
    check_read: bool,
) {
    let Some(candidates) = members.by_name.get(name) else { return };
    // Nested classes may declare the same name, so try every enclosing class that declares it
    // and pick the one the receiver actually resolves to.
    let mut target = None;
    for class_node_id in ctx.nodes().ancestor_ids(node.id()) {
        if !candidates.iter().any(|&i| members.list[i].class_node_id == class_node_id) {
            continue;
        }
        let Some(is_static) = receiver_is_static(ctx, object, node.id(), class_node_id) else {
            continue;
        };
        if let Some(&index) = candidates.iter().find(|&&i| {
            let m = &members.list[i];
            m.class_node_id == class_node_id && m.is_static == is_static
        }) {
            target = Some(index);
            break;
        }
    }
    let Some(index) = target else { return };
    let member = &mut members.list[index];
    if member.used {
        return;
    }
    if check_read
        && member.is_property
        && !is_read_from(
            std::iter::once(node).chain(ctx.nodes().ancestors(node.id())),
            ctx.semantic(),
        )
    {
        return;
    }
    member.used = true;
}

/// Resolves the lexical `this` at `node_id` to the class that owns it and whether it is the class
/// constructor (static context). Arrow functions keep the outer `this`; any other function that is
/// not a class method has its own `this`, so `None` is returned.
fn resolve_this(ctx: &LintContext<'_>, node_id: NodeId) -> Option<(NodeId, bool)> {
    let mut is_static = None;
    let mut previous_is_method_value = false;
    for ancestor in ctx.nodes().ancestors(node_id) {
        match ancestor.kind() {
            AstKind::Function(_) => {
                let parent = ctx.nodes().parent_kind(ancestor.id());
                if !matches!(parent, AstKind::MethodDefinition(_)) {
                    return None;
                }
                previous_is_method_value = true;
            }
            AstKind::StaticBlock(_) => is_static = Some(true),
            AstKind::PropertyDefinition(def) => is_static = Some(def.r#static),
            AstKind::AccessorProperty(def) => is_static = Some(def.r#static),
            AstKind::MethodDefinition(def) if previous_is_method_value => {
                is_static = Some(def.r#static);
            }
            AstKind::Class(_) => return is_static.map(|is_static| (ancestor.id(), is_static)),
            _ => {}
        }
    }
    None
}

/// Returns `Some(is_static)` if `expr` refers to the class constructor (static) or an instance of
/// the class, or `None` if it can't be resolved to the class.
fn receiver_is_static<'a>(
    ctx: &LintContext<'a>,
    expr: &Expression<'a>,
    node_id: NodeId,
    class_node_id: NodeId,
) -> Option<bool> {
    let class_symbol = match ctx.nodes().kind(class_node_id) {
        AstKind::Class(class) => class.id.as_ref().map(BindingIdentifier::symbol_id),
        _ => None,
    };
    match expr.get_inner_expression() {
        Expression::ThisExpression(_) => resolve_this_of_class(ctx, node_id, class_node_id),
        expr @ Expression::NewExpression(_) => {
            is_new_of_class(ctx, expr, class_symbol).then_some(false)
        }
        Expression::Identifier(ident) => {
            let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id()?;
            if Some(symbol_id) == class_symbol {
                return Some(true);
            }
            let decl_id = ctx.scoping().symbol_declaration(symbol_id);
            let annotation = match ctx.nodes().kind(decl_id) {
                AstKind::VariableDeclarator(d) => {
                    match d.init.as_ref().map(Expression::get_inner_expression) {
                        Some(Expression::ThisExpression(_)) => {
                            return resolve_this_of_class(ctx, decl_id, class_node_id);
                        }
                        Some(init) if is_new_of_class(ctx, init, class_symbol) => {
                            return Some(false);
                        }
                        _ => {}
                    }
                    d.type_annotation.as_ref()
                }
                AstKind::FormalParameter(p) => p.type_annotation.as_ref(),
                _ => None,
            };
            type_refers_to_class(ctx, &annotation?.type_annotation, class_symbol)
        }
        _ => None,
    }
}

/// Whether `expr` is `new C(...)` where `C` resolves to the class symbol.
fn is_new_of_class(
    ctx: &LintContext<'_>,
    expr: &Expression<'_>,
    class_symbol: Option<SymbolId>,
) -> bool {
    let Expression::NewExpression(new_expr) = expr else { return false };
    let Expression::Identifier(id) = new_expr.callee.get_inner_expression() else { return false };
    class_symbol.is_some()
        && ctx.scoping().get_reference(id.reference_id()).symbol_id() == class_symbol
}

fn resolve_this_of_class(
    ctx: &LintContext<'_>,
    node_id: NodeId,
    class_node_id: NodeId,
) -> Option<bool> {
    resolve_this(ctx, node_id)
        .and_then(|(owner, is_static)| (owner == class_node_id).then_some(is_static))
}

/// `Some(false)` for `Foo`, `Some(true)` for `typeof Foo`.
fn type_refers_to_class(
    ctx: &LintContext<'_>,
    ty: &TSType,
    class_symbol: Option<SymbolId>,
) -> Option<bool> {
    let class_symbol = class_symbol?;
    let resolves = |id: &IdentifierReference| {
        ctx.scoping().get_reference(id.reference_id()).symbol_id() == Some(class_symbol)
    };
    match ty {
        TSType::TSTypeReference(r) => {
            matches!(&r.type_name, TSTypeName::IdentifierReference(id) if resolves(id))
                .then_some(false)
        }
        TSType::TSTypeQuery(q) => {
            matches!(&q.expr_name, TSTypeQueryExprName::IdentifierReference(id) if resolves(id))
                .then_some(true)
        }
        _ => None,
    }
}

fn is_read(current_node_id: NodeId, semantic: &Semantic<'_>) -> bool {
    is_read_from(semantic.nodes().ancestors(current_node_id), semantic)
}

fn is_read_from<'a: 'b, 'b>(
    nodes: impl Iterator<Item = &'b AstNode<'a>>,
    semantic: &Semantic<'a>,
) -> bool {
    for (curr, parent) in nodes
        .filter(|parent| {
            !matches!(
                parent.kind(),
                AstKind::ParenthesizedExpression(_)
                    | AstKind::TSAsExpression(_)
                    | AstKind::TSSatisfiesExpression(_)
                    | AstKind::TSInstantiationExpression(_)
                    | AstKind::TSNonNullExpression(_)
                    | AstKind::TSTypeAssertion(_)
            )
        })
        .tuple_windows::<(&AstNode<'_>, &AstNode<'_>)>()
    {
        match (curr.kind(), parent.kind()) {
            // Skip member expressions in identifier context
            (member_expr, AstKind::IdentifierReference(_))
                if member_expr.is_member_expression_kind() => {}
            // Skip identifier references in assignment targets
            (
                AstKind::IdentifierReference(_),
                AstKind::ArrayAssignmentTarget(_)
                | AstKind::ObjectAssignmentTarget(_)
                | AstKind::IdentifierReference(_),
            ) => {}
            // All these are read contexts for private fields
            (k, _) if k.is_member_expression_kind() && is_value_context(parent, curr, semantic) => {
                return true;
            }
            // AssignmentExpression: right-hand side is a read, compound assignment result in value context is a read
            (k, AstKind::AssignmentExpression(assign_expr)) if k.is_member_expression_kind() => {
                // Right-hand side of assignment is a read
                if assign_expr.right.span() == curr.span() {
                    return true;
                }
                // Compound assignment result used in a value context is a read
                if assign_expr.operator != AssignmentOperator::Assign
                    && is_compound_assignment_read(parent.id(), semantic)
                {
                    return true;
                }
                // Not a read otherwise
                return false;
            }
            // ForIn/ForOf: only right-hand side is a read
            (k, AstKind::ForInStatement(for_in)) if k.is_member_expression_kind() => {
                if for_in.right.span() == curr.span() {
                    return true;
                }
            }
            (k, AstKind::ForOfStatement(for_of)) if k.is_member_expression_kind() => {
                if for_of.right.span() == curr.span() {
                    return true;
                }
            }
            // AssignmentTargetPropertyProperty: only computed property name is a read
            (k, AstKind::AssignmentTargetPropertyProperty(prop))
                if k.is_member_expression_kind() =>
            {
                if prop.computed && prop.name.span() == curr.span() {
                    return true;
                }
            }
            (k, AstKind::ConditionalExpression(conditional_expr))
                if k.is_member_expression_kind() =>
            {
                if conditional_expr.test.span() == curr.span() {
                    return true;
                }
                if is_value_context(parent, curr, semantic) {
                    return true;
                }
            }
            (k, AstKind::LogicalExpression(logical_expr)) if k.is_member_expression_kind() => {
                // Reading the left side of a logical expression can affect control flow
                // (e.g. `this.#flag && sideEffect()`), even when used as a statement.
                if logical_expr.left.span().contains_inclusive(curr.span()) {
                    return true;
                }
            }
            _ => {
                return false;
            }
        }
    }
    true
}

/// Check if the given AST kind represents a context where a value is being read/used
/// `parent` is the parent node, `child` is the child node we're checking
fn is_value_context(parent: &AstNode, child: &AstNode, semantic: &Semantic<'_>) -> bool {
    match parent.kind() {
        AstKind::ReturnStatement(_)
        | AstKind::CallExpression(_)
        | AstKind::NewExpression(_)
        | AstKind::BinaryExpression(_)
        | AstKind::VariableDeclarator(_)
        | AstKind::PropertyDefinition(_)
        | AstKind::ArrayExpression(_)
        | AstKind::ObjectProperty(_)
        | AstKind::JSXExpressionContainer(_)
        | AstKind::ChainExpression(_)
        | AstKind::StaticMemberExpression(_)
        | AstKind::ComputedMemberExpression(_)
        | AstKind::TemplateLiteral(_)
        | AstKind::UnaryExpression(_)
        | AstKind::IfStatement(_)
        | AstKind::SpreadElement(_)
        | AstKind::AssignmentPattern(_)
        | AstKind::SwitchCase(_)
        | AstKind::SwitchStatement(_)
        | AstKind::ThrowStatement(_)
        | AstKind::WhileStatement(_)
        | AstKind::DoWhileStatement(_)
        | AstKind::AwaitExpression(_)
        | AstKind::SequenceExpression(_) => true,
        AstKind::FormalParameter(p) => {
            p.initializer.as_ref().is_some_and(|init| init.span().contains_inclusive(child.span()))
        }
        AstKind::AssignmentExpression(assign_expr) => {
            // The right-hand side of an assignment is always in a value context (being read for assignment)
            assign_expr.right.span().contains_inclusive(child.span())
        }
        AstKind::ArrowFunctionExpression(arrow) => arrow.is_expression(),
        AstKind::ParenthesizedExpression(_)
        | AstKind::TSAsExpression(_)
        | AstKind::TSSatisfiesExpression(_)
        | AstKind::TSInstantiationExpression(_)
        | AstKind::TSNonNullExpression(_)
        | AstKind::TSTypeAssertion(_)
        | AstKind::UpdateExpression(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::LogicalExpression(_) => {
            let grandparent = semantic.nodes().parent_node(parent.id());
            is_value_context(grandparent, parent, semantic)
        }

        _ => false,
    }
}

/// Check if a compound assignment result is being used in a value context
fn is_compound_assignment_read(parent_id: NodeId, semantic: &Semantic) -> bool {
    let assignment_expr = semantic.nodes().get_node(parent_id);
    semantic.nodes().ancestors(parent_id).next().is_some_and(|assignment_parent| {
        is_value_context(assignment_parent, assignment_expr, semantic)
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        r"
            class Foo { #privateMember = {}; a() { return { ...this.#privateMember }; } }
        ",
        r"
            class Test {
                #prop = undefined

                getProp() {
                    return this.#prop ??= 0
                }
            }
        ",
        r"
            class Test {
                #prop = undefined

                getProp() {
                    return this.#prop ||= 0
                }
            }
        ",
        r"
            class Test {
                #prop = undefined

                getProp() {
                    return this.#prop += 0
                }
            }
        ",
        r"class Foo {}",
        r"class Foo {
                publicMember = 42;
            }",
        r"class Foo {
                #usedMember = 42;
                method() {
                    return this.#usedMember;
                }
            }",
        r"class Foo {
                #usedMember = 42;
                anotherMember = this.#usedMember;
            }",
        r"class Foo {
                #usedMember = 42;
                foo() {
                    anotherMember = this.#usedMember;
                }
            }",
        r"class C {
                #usedMember;

                foo() {
                    bar(this.#usedMember += 1);
                }
            }",
        r"class Foo {
                #usedMember = 42;
                method() {
                    return someGlobalMethod(this.#usedMember);
                }
            }",
        r"class C {
                #usedInOuterClass;

                foo() {
                    return class {};
                }

                bar() {
                    return this.#usedInOuterClass;
                }
            }",
        r"class Foo {
                #usedInForInLoop;
                method() {
                    for (const bar in this.#usedInForInLoop) {

                    }
                }
            }",
        r"class Foo {
                #usedInForOfLoop;
                method() {
                    for (const bar of this.#usedInForOfLoop) {

                    }
                }
            }",
        r"class Foo {
                #usedInAssignmentPattern;
                method() {
                    [bar = 1] = this.#usedInAssignmentPattern;
                }
            }",
        r"class Foo {
                #usedInArrayPattern;
                method() {
                    [bar] = this.#usedInArrayPattern;
                }
            }",
        r"class Foo {
                #usedInAssignmentPattern;
                method() {
                    [bar] = this.#usedInAssignmentPattern;
                }
            }",
        r"class C {
                #usedInObjectAssignment;

                method() {
                    ({ [this.#usedInObjectAssignment]: a } = foo);
                }
            }",
        r"class C {
            set #accessorWithSetterFirst(value) {
                doSomething(value);
            }
            get #accessorWithSetterFirst() {
                return something();
            }
            method() {
                this.#accessorWithSetterFirst += 1;
            }
        }",
        r"class Foo {
            set #accessorUsedInMemberAccess(value) {}

            method(a) {
                [this.#accessorUsedInMemberAccess] = a;
            }
        }",
        r"class C {
            get #accessorWithGetterFirst() {
                return something();
            }
            set #accessorWithGetterFirst(value) {
                doSomething(value);
            }
            method() {
                this.#accessorWithGetterFirst += 1;
            }
        }",
        r"class C {
            #usedInInnerClass;

            method(a) {
                return class {
                    foo = a.#usedInInnerClass;
                }
            }
        }",
        r"class Foo {
                #usedMethod() {
                    return 42;
                }
                anotherMethod() {
                    return this.#usedMethod();
                }
            }",
        r"class C {
            set #x(value) {
                doSomething(value);
            }

            foo() {
                this.#x = 1;
            }
        }",
        r"type Callback<T> = () => Promise<T> | T;

         export class Issue_11039<T> {
            load: () => Promise<T>;

            constructor(callback: Callback<T>) {
                this.load = () => this.#load(callback);
            }

            async #load(callback: Callback<T>) {
                callback;
            }
         }",
        r"class ChildProcess extends EventEmitter { #stdioObject; #createStdioObject() {} get stdio() { return (this.#stdioObject ??= this.#createStdioObject()); } }",
        "export class Foo { readonly #select = 123; override render() { return html`foo=${this.#select}`; } }",
        "export class Foo { #listened = false; bar() { if (!this.#listened) return; this.#listened = false; } } ",
        "export class RichText { #verticalScrollContainer; init() { const verticalScrollContainer = this.#verticalScrollContainer || (this.#verticalScrollContainer = this.verticalScrollContainerGetter?.() || null); } }",
        "class Foo { #a = false; on(data) { return this.#a ? [data] : data; } set setA(value) { this.#a = value; } }",
        "class Foo { #a = false; on(data) { return this.#a ? [data] : data; } }",
        "class WeakReference { #i = 0; inc() { return ++this.#i; }; dec() { return --this.#i; } }",
        "class Foo { #d; constructor(d) { this.#d = d || kDefaultD; } get getD(): string { return this.#d!; } }",
        "class F { #o; initialize(output) { this.#o = output; } text(e) { return this.#o!.text(e); } }",
        "class Foo { #a; constructor(a) { this.#a = a; }; b(b?: string): this { this.#a!.setB(b); return this; } resetA() { this.#a = undefined; } }",
        // Test for static block - issue #13179
        r"let getPrivate; class C { #private; constructor(v) { this.#private = v; } static { getPrivate = klass => klass.#private; } }",
        r"let getPrivate; class C { #private; constructor(v) { this.#private = v; } static { getPrivate = klass => { return klass.#private; } } }",
        r"class C { #field = 1; static { const obj = new C(); console.log(obj.#field); } }",
        r"class C { #method() { return 42; } static { const obj = new C(); obj.#method(); } }",
        r"class C { #field = 1; static { const getField = obj => { return obj.#field; }; } }",
        r"export class Database<const S extends idb.DBSchema> { readonly #db: Promise<idb.IDBPDatabase<S>>; constructor(name: string, version: number, hooks: idb.OpenDBCallbacks<S>) { this.#db = idb.openDB<S>(name, version, hooks); }  async read() { let db = await this.#db; } }",
        r"export class A { #x; constructor(x: number) { this.#x = x; } get(y = this.#x) { return y; } }",
        r"class B { #value = 42; method(param = this.#value) { return param * 2; } }",
        r"class C { #arr = [1, 2, 3]; process(items = this.#arr) { return items.map(x => x * 2); } }",
        r"export class BugClass { readonly #BUG: readonly [] = []; method() { return Math.random() > 0.5 ? this.#BUG : []; } }",
        r"class Foo { #x; #y; method(a, b, c) { return a ? (b ? this.#x : c) : this.#y; } }",
        r"class Foo { #x; method() { return () => a ? this.#x : b; } }",
        r"class Foo { #x; method() { return a && (b ? this.#x : c); } }",
        r"class Foo { #x; method() { fn(a ? this.#x : b); } }",
        r"class Foo { #x; method() { return `${a ? this.#x : b}`; } }",
        r"class Foo { #x; method() { return [a ? this.#x : b]; } }",
        r"class Foo { #x; method() { return { key: a ? this.#x : b }; } }",
        r"class Foo { #x; method(val) { switch(val) { case (a ? this.#x : b): break; } } }",
        r"class Foo { #x; method() { throw a ? this.#x : new Error(); } }",
        r"class Foo { #x; method() { while (a ? this.#x : b) {} } }",
        r"class Bug { #flag = false; foo() { this.#flag && console.log('spam'); } }",
        r"class Foo { #a; #b; #c; method() { return this.#a ? this.#b : this.#c; } }",
        // Issue #15548: Private member used in logical expression on RHS of assignment
        r"class ExampleFoo { #foo = 0; foo(foo) { foo = foo ?? this.#foo; return foo; } }",
        // Issue #15548: Private member used in update expression on RHS of assignment
        r"class ExampleBar { #bar = 0; bar(bar) { bar = ++this.#bar; return bar; } }",
        r"class Foo { #awaitedMember; async method() { await this.#awaitedMember; } }",
        r"class Test { #url: string; constructor(url: string) { this.#url = url; } open() { return new WebSocket(this.#url); } }",
        r"export class Foo { #fetch: typeof fetch; constructor() { this.#fetch = fetch; } async bar() { return (0, this.#fetch)('https://example.com'); } }",
        r"export class StateMachine { #state = 'idle'; step() { switch (this.#state) { case 'idle': { this.#state = 'running'; break; } case 'running': { this.#state = 'done'; break; } } } }",
        r"class Stopper { #promise; async stop() { this.#promise ??= this.makePromise(); await this.#promise; } makePromise() { return Promise.resolve(); } }",
        "class C {
            #usedInInnerClass;

            method(a) {
                return class {
                    foo = a.#usedInInnerClass;
                }
            }
        }",
        "class C { #x; m() { return class { #y; n(o) { return o.#x + this.#y; } }; } }",
        "class C { #m() {} run(o) { return class { go() { o.#m(); } }; } }",
    ];

    let mut fail = vec![
        r"class Foo {
                #unusedMember = 5;
            }",
        r"class First {}
            class Second {
                #unusedMemberInSecondClass = 5;
            }",
        r"class First {
                #unusedMemberInFirstClass = 5;
            }
            class Second {}",
        r"class First {
                #firstUnusedMemberInSameClass = 5;
                #secondUnusedMemberInSameClass = 5;
            }",
        r"class Foo {
                #usedOnlyInWrite = 5;
                method() {
                    this.#usedOnlyInWrite = 42;
                }
            }",
        r"class Foo {
                #usedOnlyInWriteStatement = 5;
                method() {
                    this.#usedOnlyInWriteStatement += 42;
                }
            }",
        r"class C {
                #usedOnlyInIncrement;

                foo() {
                    this.#usedOnlyInIncrement++;
                }
            }",
        r"class C {
                #unusedInOuterClass;

                foo() {
                    return class {
                        #unusedInOuterClass;

                        bar() {
                            return this.#unusedInOuterClass;
                        }
                    };
                }
            }",
        r"class C {
                #unusedOnlyInSecondNestedClass;

                foo() {
                    return class {
                        #unusedOnlyInSecondNestedClass;

                        bar() {
                            return this.#unusedOnlyInSecondNestedClass;
                        }
                    };
                }

                baz() {
                    return this.#unusedOnlyInSecondNestedClass;
                }

                bar() {
                    return class {
                        #unusedOnlyInSecondNestedClass;
                    }
                }
            }",
        r"class Foo {
                #unusedMethod() {}
            }",
        r"class Foo {
                #unusedMethod() {}
                #usedMethod() {
                    return 42;
                }
                publicMethod() {
                    return this.#usedMethod();
                }
            }",
        r"class Foo {
                set #unusedSetter(value) {}
            }",
        r"class Foo {
                #unusedForInLoop;
                method() {
                    for (this.#unusedForInLoop in bar) {

                    }
                }
            }",
        r"class Foo {
                #unusedForOfLoop;
                method() {
                    for (this.#unusedForOfLoop of bar) {

                    }
                }
            }",
        r"class Foo {
                #unusedInDestructuring;
                method() {
                    ({ x: this.#unusedInDestructuring } = bar);
                }
            }",
        r"class Foo {
                #unusedInRestPattern;
                method() {
                    [...this.#unusedInRestPattern] = bar;
                }
            }",
        r"class Foo {
                #unusedInAssignmentPattern;
                method() {
                    [this.#unusedInAssignmentPattern = 1] = bar;
                }
            }",
        r"class Foo {
                #unusedInAssignmentPattern;
                method() {
                    [this.#unusedInAssignmentPattern] = bar;
                }
            }",
        r"class C {
                #usedOnlyInTheSecondInnerClass;

                method(a) {
                    return class {
                        #usedOnlyInTheSecondInnerClass;

                        method2(b) {
                            foo = b.#usedOnlyInTheSecondInnerClass;
                        }

                        method3(b) {
                            foo = b.#usedOnlyInTheSecondInnerClass;
                        }
                    }
                }
            }",
        r"class StatementLogicalAssignment { #prop; method() { this.#prop ??= 1; } }",
        r"class Foo { #unused; method() { Math.random() > 0.5 ? this.#unused : []; } }",
        r"class Foo { #x; #y; method(a, b, c) { a ? (b ? this.#x : c) : this.#y; } }",
        r"class Foo { #x; method() { a && (b ? this.#x : c); } }",
        r"class Foo { #a; #b; #c; method() { this.#a ? this.#b : this.#c; } }",
        "class C { #x; m() { return class { #x; n(o) { return o.#x; } }; } }",
    ];

    let pass_typescript = vec![
        "
            class C {
              private value = 1;
              read() {
                const other = new C();
                return other.value;
              }
            }
                ",
        "
            class C {
              private value = 1;
              read() {
                return new C().value;
              }
            }
                ",
        "
            class Outer {
              private value = 1;
              make(other: Outer) {
                return class Inner {
                  private value = 2;
                  read() { return other.value + this.value; }
                };
              }
            }
                ",
        "
            class Foo {
              private value = 1;
              method() {
                const self = this;
                return () => this.value + self.value;
              }
            }
                ",
        "class Foo {}",
        "
            class Foo {
              publicMember = 42;
            }
                ",
        "
            class Foo {
              public publicMember = 42;
            }
                ",
        "
            class Foo {
              protected publicMember = 42;
            }
                ",
        "
            class C {
              #usedInInnerClass;

              method(a) {
                return class {
                  foo = a.#usedInInnerClass;
                };
              }
            }
                ",
        "
            class C {
              private accessor accessorMember = 42;

              method() {
                return this.accessorMember;
              }
            }
                  ",
        "
            class C {
              private static staticMember = 42;

              static method() {
                return this.staticMember;
              }
            }
                ",
        "
            class C {
              private static staticMember = 42;

              method() {
                return C.staticMember;
              }
            }
                  ",
        "
            class Test1 {
              constructor(private parameterProperty: number) {}
              method() {
                return this.parameterProperty;
              }
            }
                ",
        "
            class Test1 {
              constructor(private readonly parameterProperty: number) {}
              method() {
                return this.parameterProperty;
              }
            }
                ",
        "
            class Test1 {
              constructor(private readonly parameterProperty: number = 1) {}
              method() {
                return this.parameterProperty;
              }
            }
                ",
        "
            class Foo {
              private prop: number;

              method(thing: Foo) {
                return thing.prop;
              }
            }
                ",
        "
            class Foo {
              private static staticProp: number;

              method(thing: typeof Foo) {
                return thing.staticProp;
              }
            }
                ",
        "
            class Foo {
              private prop: number;

              method() {
                const self = this;
                return self.prop;
              }
            }
                ",
        "
            class Foo {
              #privateMember = 42;
              method() {
                return this.#privateMember;
              }
            }
                ",
        "
            class Foo {
              private privateMember = 42;
              method() {
                return this.privateMember;
              }
            }
                ",
        "
            class Foo {
              #privateMember = 42;
              anotherMember = this.#privateMember;
            }
                ",
        "
            class Foo {
              private privateMember = 42;
              anotherMember = this.privateMember;
            }
                ",
        "
            class Foo {
              #privateMember = 42;
              foo() {
                anotherMember = this.#privateMember;
              }
            }
                ",
        "
            class Foo {
              private privateMember = 42;
              foo() {
                anotherMember = this.privateMember;
              }
            }
                ",
        "
            class C {
              #privateMember;

              foo() {
                bar((this.#privateMember += 1));
              }
            }
                ",
        "
            class C {
              private privateMember;

              foo() {
                bar((this.privateMember += 1));
              }
            }
                ",
        "
            class Foo {
              #privateMember = 42;
              method() {
                return someGlobalMethod(this.#privateMember);
              }
            }
                ",
        "
            class Foo {
              private privateMember = 42;
              method() {
                return someGlobalMethod(this.privateMember);
              }
            }
                ",
        "
            class C {
              #privateMember;

              foo() {
                return class {};
              }

              bar() {
                return this.#privateMember;
              }
            }
                ",
        "
            class C {
              private privateMember;

              foo() {
                return class {};
              }

              bar() {
                return this.privateMember;
              }
            }
                ",
        "
            class Foo {
              #privateMember;
              method() {
                for (const bar in this.#privateMember) {
                }
              }
            }
                ",
        "
            class Foo {
              private privateMember;
              method() {
                for (const bar in this.privateMember) {
                }
              }
            }
                ",
        "
            class Foo {
              #privateMember;
              method() {
                for (const bar of this.#privateMember) {
                }
              }
            }
                ",
        "
            class Foo {
              private privateMember;
              method() {
                for (const bar of this.privateMember) {
                }
              }
            }
                ",
        "
            class Foo {
              #privateMember;
              method() {
                [bar = 1] = this.#privateMember;
              }
            }
                ",
        "
            class Foo {
              private privateMember;
              method() {
                [bar = 1] = this.privateMember;
              }
            }
                ",
        "
            class Foo {
              #privateMember;
              method() {
                [bar] = this.#privateMember;
              }
            }
                ",
        "
            class Foo {
              private privateMember;
              method() {
                [bar] = this.privateMember;
              }
            }
                ",
        "
            class C {
              #privateMember;

              method() {
                ({ [this.#privateMember]: a } = foo);
              }
            }
                ",
        "
            class C {
              private privateMember;

              method() {
                ({ [this.privateMember]: a } = foo);
              }
            }
                ",
        "
            class C {
              set #privateMember(value) {
                doSomething(value);
              }
              get #privateMember() {
                return something();
              }
              method() {
                this.#privateMember += 1;
              }
            }
                ",
        "
            class C {
              private set privateMember(value) {
                doSomething(value);
              }
              private get privateMember() {
                return something();
              }
              method() {
                this.privateMember += 1;
              }
            }
                ",
        "
            class Foo {
              set #privateMember(value) {}

              method(a) {
                [this.#privateMember] = a;
              }
            }
                ",
        "
            class Foo {
              private set privateMember(value) {}

              method(a) {
                [this.privateMember] = a;
              }
            }
                ",
        "
            class C {
              get #privateMember() {
                return something();
              }
              set #privateMember(value) {
                doSomething(value);
              }
              method() {
                this.#privateMember += 1;
              }
            }
                ",
        "
            class C {
              private get privateMember() {
                return something();
              }
              private set privateMember(value) {
                doSomething(value);
              }
              method() {
                this.privateMember += 1;
              }
            }
                ",
        "
            class Foo {
              private privateMember;
              private privateMember2;

              method() {
                const { privateMember, privateMember2 } = this;
                console.log(privateMember, privateMember2);
              }
            }
                ",
        "
            class Foo {
              private static staticMember = 1;
              static method() {
                const { staticMember } = this;
                console.log(staticMember);
              }
            }
                ",
        "
            class Foo {
              private privateMember = 1;
              method() {
                const { privateMember } = this;
              }
            }
                ",
        "
            class Foo {
              private privateMember = 1;
              method() {
                const { privateMember: privateMember2 } = this;
              }
            }
                ",
        "
            class Foo {
              private privateMember = 1;

              method() {
                let privateMember;
                ({ privateMember } = this);
              }
            }
                ",
        "
            class Foo {
              private privateMember = 1;

              method() {
                const foo = ({ privateMember } = this) => {};
              }
            }
                ",
        "
            class Foo {
              private privateMember;

              method() {
                const { privateMember: used } = this;
              }
            }
                ",
        "
            class Foo {
              #privateMember() {
                return 42;
              }
              anotherMethod() {
                return this.#privateMember();
              }
            }
                ",
        "
            class Foo {
              private privateMember() {
                return 42;
              }
              anotherMethod() {
                return this.privateMember();
              }
            }
                ",
        "
            class C {
              set #privateMember(value) {
                doSomething(value);
              }

              foo() {
                this.#privateMember = 1;
              }
            }
                ",
        "
            class C {
              private set privateMember(value) {
                doSomething(value);
              }

              foo() {
                this.privateMember = 1;
              }
            }
                ",
    ];

    let fail_typescript = vec![
        "
            class C {
              #unusedInOuterClass;

              foo() {
                return class D {
                  #unusedInOuterClass;

                  bar() {
                    return this.#unusedInOuterClass;
                  }
                };
              }
            }
                  ",
        "
            class C {
              #unusedOnlyInSecondNestedClass;

              foo() {
                return class {
                  #unusedOnlyInSecondNestedClass;

                  bar() {
                    return this.#unusedOnlyInSecondNestedClass;
                  }
                };
              }

              baz() {
                return this.#unusedOnlyInSecondNestedClass;
              }

              bar() {
                return class {
                  #unusedOnlyInSecondNestedClass;
                };
              }
            }
                  ",
        "
            class C {
              #usedOnlyInTheSecondInnerClass;

              method(a) {
                return class {
                  #usedOnlyInTheSecondInnerClass;

                  method2(b) {
                    foo = b.#usedOnlyInTheSecondInnerClass;
                  }

                  method3(b) {
                    foo = b.#usedOnlyInTheSecondInnerClass;
                  }
                };
              }
            }
                  ",
        "
            class C {
              private static value = 1;
              private value = 2;
              read() { return this.value; }
            }
                  ",
        "
            class Outer {
              private value = 1;
              make() {
                return class {
                  value = 2;
                  read() { return this.value; }
                };
              }
            }
                  ",
        "
            class Foo {
              private static value = 1;
              method(Other) {
                const Foo = Other;
                return Foo.value;
              }
            }
                  ",
        "
            class C {
              private accessor accessorMember = 42;
            }
                  ",
        "
            class C {
              private static staticMember = 42;
            }
                  ",
        "
            class Test1 {
              constructor(private parameterProperty: number) {}
            }
                  ",
        "
            class Test1 {
              constructor(private readonly parameterProperty: number) {}
            }
                  ",
        "
            class Test1 {
              constructor(private readonly parameterProperty: number = 1) {}
            }
                  ",
        "
            class C {
              private usedOutsideClass;
            }

            const instance = new C();
            console.log(instance.usedOutsideClass);
                  ",
        "
            class C {
              private usedOutsideClass;
            }

            const instance = new C();
            console.log(instance['usedOutsideClass']);
                  ",
        "
            class Foo {
              private prop: number;

              method() {
                const self1 = this;
                const self2 = self1;
                return self2.prop;
              }
            }
                  ",
        "
            class Foo {
              #privateMember = 5;
            }
                  ",
        "
            class Foo {
              private privateMember = 5;
            }
                  ",
        "
            class First {}
            class Second {
              #privateMember = 5;
            }
                  ",
        "
            class First {}
            class Second {
              private privateMember = 5;
            }
                  ",
        "
            class First {
              #privateMember = 5;
            }
            class Second {}
                  ",
        "
            class First {
              private privateMember = 5;
            }
            class Second {}
                  ",
        "
            class First {
              #privateMember = 5;
              #privateMember2 = 5;
            }
                  ",
        "
            class First {
              private privateMember = 5;
              private privateMember2 = 5;
            }
                  ",
        "
            class Foo {
              #privateMember = 5;
              method() {
                this.#privateMember = 42;
              }
            }
                  ",
        "
            class Foo {
              private privateMember = 5;
              method() {
                this.privateMember = 42;
              }
            }
                  ",
        "
            class Foo {
              #privateMember = 5;
              method() {
                this.#privateMember += 42;
              }
            }
                  ",
        "
            class Foo {
              private privateMember = 5;
              method() {
                this.privateMember += 42;
              }
            }
                  ",
        "
            class C {
              #privateMember;

              foo() {
                this.#privateMember++;
              }
            }
                  ",
        "
            class C {
              private privateMember;

              foo() {
                this.privateMember++;
              }
            }
                  ",
        "
            class Foo {
              #privateMember() {}
            }
                  ",
        "
            class Foo {
              private privateMember() {}
            }
                  ",
        "
            class Foo {
              #privateMember() {}
              #privateMemberUsed() {
                return 42;
              }
              publicMethod() {
                return this.#privateMemberUsed();
              }
            }
                  ",
        "
            class Foo {
              private privateMember() {}
              private privateMemberUsed() {
                return 42;
              }
              publicMethod() {
                return this.privateMemberUsed();
              }
            }
                  ",
        "
            class Foo {
              set #privateMember(value) {}
            }
                  ",
        "
            class Foo {
              private set privateMember(value) {}
            }
                  ",
        "
            class Foo {
              #privateMember;
              method() {
                for (this.#privateMember in bar) {
                }
              }
            }
                  ",
        "
            class Foo {
              private privateMember;
              method() {
                for (this.privateMember in bar) {
                }
              }
            }
                  ",
        "
            class Foo {
              #privateMember;
              method() {
                for (this.#privateMember of bar) {
                }
              }
            }
                  ",
        "
            class Foo {
              private privateMember;
              method() {
                for (this.privateMember of bar) {
                }
              }
            }
                  ",
        "
            class Foo {
              #privateMember;
              method() {
                ({ x: this.#privateMember } = bar);
              }
            }
                  ",
        "
            class Foo {
              private privateMember;
              method() {
                ({ x: this.privateMember } = bar);
              }
            }
                  ",
        "
            class Foo {
              #privateMember;
              method() {
                [...this.#privateMember] = bar;
              }
            }
                  ",
        "
            class Foo {
              private privateMember;
              method() {
                [...this.privateMember] = bar;
              }
            }
                  ",
        "
            class Foo {
              #privateMember;
              method() {
                [this.#privateMember = 1] = bar;
              }
            }
                  ",
        "
            class Foo {
              private privateMember;
              method() {
                [this.privateMember = 1] = bar;
              }
            }
                  ",
        "
            class Foo {
              #privateMember;
              method() {
                [this.#privateMember] = bar;
              }
            }
                  ",
        "
            class Foo {
              private privateMember;
              method() {
                [this.privateMember] = bar;
              }
            }
                  ",
        "
            class Foo {
              private privateMember;

              method() {
                const { unused: privateMember } = this;
              }
            }
                  ",
        "
            const foo = 'bar';
            class Foo {
              private foo = 1;

              method() {
                const { [foo]: test } = this;
              }
            }
                  ",
        "
            const foo = 'bar';
            class Foo {
              private foo = 1;
              private bar = 2;

              method() {
                const { [foo]: test } = this;
              }
            }
                  ",
    ];

    pass.extend(pass_typescript);
    fail.extend(fail_typescript);

    Tester::new(NoUnusedPrivateClassMembers::NAME, NoUnusedPrivateClassMembers::PLUGIN, pass, fail)
        .test_and_snapshot();
}
