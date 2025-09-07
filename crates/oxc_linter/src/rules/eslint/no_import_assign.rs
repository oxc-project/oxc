use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, AssignmentTargetMaybeDefault, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, Reference, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule};

fn no_import_assign_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("do not assign to imported bindings")
        .with_help("imported bindings are readonly")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImportAssign;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow assigning to imported bindings
    ///
    /// ### Why is this bad?
    ///
    /// The updates of imported bindings by ES Modules cause runtime errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import mod, { named } from "./mod.mjs"
    /// import * as mod_ns from "./mod.mjs"
    ///
    /// mod = 1          // ERROR: 'mod' is readonly.
    /// named = 2        // ERROR: 'named' is readonly.
    /// mod_ns.named = 3 // ERROR: The members of 'mod_ns' are readonly.
    /// mod_ns = {}      // ERROR: 'mod_ns' is readonly.
    /// // Can't extend 'mod_ns'
    /// Object.assign(mod_ns, { foo: "foo" }) // ERROR: The members of 'mod_ns' are readonly.
    /// ```
    NoImportAssign,
    eslint,
    correctness
);

const OBJECT_MUTATION_METHODS: [&str; 5] =
    ["assign", "defineProperty", "defineProperties", "freeze", "setPrototypeOf"];

const REFLECT_MUTATION_METHODS: [&str; 4] =
    ["defineProperty", "deleteProperty", "set", "setPrototypeOf"];

impl Rule for NoImportAssign {
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let symbol_table = ctx.scoping();
        if symbol_table.symbol_flags(symbol_id).is_import() {
            let kind = ctx.nodes().kind(symbol_table.symbol_declaration(symbol_id));
            let is_namespace_specifier = matches!(kind, AstKind::ImportNamespaceSpecifier(_));
            for reference in symbol_table.get_resolved_references(symbol_id) {
                if is_namespace_specifier {
                    let parent_node = ctx.nodes().parent_node(reference.node_id());
                    if parent_node.kind().is_member_expression_kind() {
                        let expr = parent_node.kind();
                        let parent_parent_node = ctx.nodes().parent_node(parent_node.id());
                        let is_unary_expression_with_delete_operator = |kind| matches!(kind, AstKind::UnaryExpression(expr) if expr.operator == UnaryOperator::Delete);
                        let parent_parent_kind = parent_parent_node.kind();
                        if (matches!(parent_parent_kind, AstKind::IdentifierReference(_))
                            || is_unary_expression_with_delete_operator(parent_parent_kind)
                            || matches!(parent_parent_kind, AstKind::ChainExpression(_) if is_unary_expression_with_delete_operator(ctx.nodes().parent_kind(parent_parent_node.id()))))
                            && let Some((span, _)) = match expr {
                                AstKind::StaticMemberExpression(expr) => {
                                    Some(expr.static_property_info())
                                }
                                AstKind::ComputedMemberExpression(expr) => {
                                    expr.static_property_info()
                                }
                                _ => return,
                            }
                            && span != ctx.semantic().reference_span(reference)
                        {
                            return ctx.diagnostic(no_import_assign_diagnostic(expr.span()));
                        }
                        // Check for assignment to namespace property
                        match expr {
                            AstKind::StaticMemberExpression(member_expr) => {
                                let condition_met = is_assignment_condition_met(
                                    &parent_parent_kind,
                                    parent_node.span(),
                                    true, // is_static
                                );
                                check_namespace_member_assignment(
                                    &member_expr.object,
                                    parent_node,
                                    reference,
                                    ctx,
                                    condition_met,
                                );
                            }
                            AstKind::ComputedMemberExpression(member_expr) => {
                                let condition_met = is_assignment_condition_met(
                                    &parent_parent_kind,
                                    parent_node.span(),
                                    false, // is_static
                                );
                                check_namespace_member_assignment(
                                    &member_expr.object,
                                    parent_node,
                                    reference,
                                    ctx,
                                    condition_met,
                                );
                            }
                            _ => {}
                        }
                    }
                }

                if reference.is_write()
                    || (is_namespace_specifier
                        && is_argument_of_well_known_mutation_function(reference.node_id(), ctx))
                {
                    ctx.diagnostic(no_import_assign_diagnostic(
                        ctx.semantic().reference_span(reference),
                    ));
                }
            }
        }
    }
}

/// Check if a given node is at the first argument of a well-known mutation function.
/// - `Object.assign`
/// - `Object.defineProperty`
/// - `Object.defineProperties`
/// - `Object.freeze`
/// - `Object.setPrototypeOf`
/// - `Reflect.defineProperty`
/// - `Reflect.deleteProperty`
/// - `Reflect.set`
/// - `Reflect.setPrototypeOf`
fn is_argument_of_well_known_mutation_function(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let current_node = ctx.nodes().get_node(node_id);
    let call_expression_node = ctx.nodes().parent_kind(node_id);

    let AstKind::CallExpression(expr) = call_expression_node else {
        return false;
    };

    let Some(member_expr) = &expr.callee.get_member_expr() else {
        return false;
    };

    if let Expression::Identifier(ident) = member_expr.object() {
        let Some(property_name) = member_expr.static_property_name() else {
            return false;
        };

        if ((ident.name == "Object" && OBJECT_MUTATION_METHODS.contains(&property_name))
            || (ident.name == "Reflect" && REFLECT_MUTATION_METHODS.contains(&property_name)))
            && !ctx.scoping().has_binding(ident.reference_id())
        {
            return expr
                .arguments
                .first()
                .is_some_and(|argument| argument.span() == current_node.kind().span());
        }
    }

    false
}

/// Helper to check if a namespace member expression is being assigned to
fn check_namespace_member_assignment(
    member_expr: &Expression,
    parent_node: &AstNode,
    reference: &Reference,
    ctx: &LintContext,
    condition_met: bool,
) {
    if !condition_met {
        return;
    }

    let Expression::Identifier(obj_ident) = member_expr else { return };

    let ref_node = ctx.nodes().get_node(reference.node_id());
    if let AstKind::IdentifierReference(ref_ident) = ref_node.kind()
        && obj_ident.span == ref_ident.span
    {
        ctx.diagnostic(no_import_assign_diagnostic(parent_node.span()));
    }
}

/// Helper to determine if assignment condition is met for different parent kinds
fn is_assignment_condition_met(
    parent_parent_kind: &AstKind,
    parent_node_span: Span,
    is_static: bool,
) -> bool {
    match parent_parent_kind {
        AstKind::AssignmentExpression(assign) => assign.left.span() == parent_node_span,
        AstKind::UpdateExpression(update) => update.argument.span() == parent_node_span,
        AstKind::ForInStatement(for_in) => for_in.left.span() == parent_node_span,
        AstKind::ForOfStatement(for_of) => for_of.left.span() == parent_node_span,
        AstKind::ArrayAssignmentTarget(array_target) => {
            array_target.elements.iter().any(|el| match el.as_ref() {
                Some(AssignmentTargetMaybeDefault::StaticMemberExpression(expr)) if is_static => {
                    expr.span == parent_node_span
                }
                Some(AssignmentTargetMaybeDefault::ComputedMemberExpression(expr))
                    if !is_static =>
                {
                    expr.span == parent_node_span
                }
                _ => false,
            })
        }
        AstKind::AssignmentTargetPropertyProperty(prop_target) => match &prop_target.binding {
            AssignmentTargetMaybeDefault::StaticMemberExpression(expr) if is_static => {
                expr.span == parent_node_span
            }
            AssignmentTargetMaybeDefault::ComputedMemberExpression(expr) if !is_static => {
                expr.span == parent_node_span
            }
            _ => false,
        },
        AstKind::AssignmentTargetWithDefault(with_default) => match &with_default.binding {
            AssignmentTarget::StaticMemberExpression(expr) if is_static => {
                expr.span == parent_node_span
            }
            AssignmentTarget::ComputedMemberExpression(expr) if !is_static => {
                expr.span == parent_node_span
            }
            _ => false,
        },
        AstKind::AssignmentTargetRest(rest_target) => match &rest_target.target {
            AssignmentTarget::StaticMemberExpression(expr) if is_static => {
                expr.span == parent_node_span
            }
            AssignmentTarget::ComputedMemberExpression(expr) if !is_static => {
                expr.span == parent_node_span
            }
            _ => false,
        },
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("import mod from 'mod'; mod.prop = 0", None),
        ("import mod from 'mod'; mod.prop += 0", None),
        ("import mod from 'mod'; mod.prop++", None),
        ("import mod from 'mod'; delete mod.prop", None),
        ("import mod from 'mod'; for (mod.prop in foo);", None),
        ("import mod from 'mod'; for (mod.prop of foo);", None),
        ("import mod from 'mod'; [mod.prop] = foo;", None),
        ("import mod from 'mod'; [...mod.prop] = foo;", None),
        ("import mod from 'mod'; ({ bar: mod.prop } = foo);", None),
        ("import mod from 'mod'; ({ ...mod.prop } = foo);", None),
        ("import {named} from 'mod'; named.prop = 0", None),
        ("import {named} from 'mod'; named.prop += 0", None),
        ("import {named} from 'mod'; named.prop++", None),
        ("import {named} from 'mod'; delete named.prop", None),
        ("import {named} from 'mod'; for (named.prop in foo);", None),
        ("import {named} from 'mod'; for (named.prop of foo);", None),
        ("import {named} from 'mod'; [named.prop] = foo;", None),
        ("import {named} from 'mod'; [...named.prop] = foo;", None),
        ("import {named} from 'mod'; ({ bar: named.prop } = foo);", None),
        ("import {named} from 'mod'; ({ ...named.prop } = foo);", None),
        ("import * as mod from 'mod'; mod.named.prop = 0", None),
        ("import * as mod from 'mod'; mod.named.prop += 0", None),
        ("import * as mod from 'mod'; mod.named.prop++", None),
        ("import * as mod from 'mod'; delete mod.named.prop", None),
        ("import * as mod from 'mod'; for (mod.named.prop in foo);", None),
        ("import * as mod from 'mod'; for (mod.named.prop of foo);", None),
        ("import * as mod from 'mod'; [mod.named.prop] = foo;", None),
        ("import * as mod from 'mod'; [...mod.named.prop] = foo;", None),
        ("import * as mod from 'mod'; ({ bar: mod.named.prop } = foo);", None),
        ("import * as mod from 'mod'; ({ ...mod.named.prop } = foo);", None),
        ("import * as mod from 'mod'; obj[mod] = 0", None),
        ("import * as mod from 'mod'; obj[mod.named] = 0", None),
        ("import * as mod from 'mod'; for (var foo in mod.named);", None),
        ("import * as mod from 'mod'; for (var foo of mod.named);", None),
        ("import * as mod from 'mod'; [bar = mod.named] = foo;", None),
        ("import * as mod from 'mod'; ({ bar = mod.named } = foo);", None),
        ("import * as mod from 'mod'; ({ bar: baz = mod.named } = foo);", None),
        ("import * as mod from 'mod'; ({ [mod.named]: bar } = foo);", None),
        ("import * as mod from 'mod'; var obj = { ...mod.named };", None),
        ("import * as mod from 'mod'; var obj = { foo: mod.named };", None),
        ("import mod from 'mod'; { let mod = 0; mod = 1 }", None),
        ("import * as mod from 'mod'; { let mod = 0; mod = 1 }", None),
        ("import * as mod from 'mod'; { let mod = 0; mod.named = 1 }", None),
        ("import {} from 'mod'", None),
        ("import 'mod'", None),
        ("import mod from 'mod'; Object.assign(mod, obj);", None),
        ("import {named} from 'mod'; Object.assign(named, obj);", None),
        ("import * as mod from 'mod'; Object.assign(mod.prop, obj);", None),
        ("import * as mod from 'mod'; Object.assign(obj, mod, other);", None),
        ("import * as mod from 'mod'; Object[assign](mod, obj);", None),
        ("import * as mod from 'mod'; Object.getPrototypeOf(mod);", None),
        ("import * as mod from 'mod'; Reflect.set(obj, key, mod);", None),
        ("import * as mod from 'mod'; { var Object; Object.assign(mod, obj); }", None),
        ("import * as mod from 'mod'; var Object; Object.assign(mod, obj);", None),
        ("import * as mod from 'mod'; Object.seal(mod, obj)", None),
        ("import * as mod from 'mod'; Object.preventExtensions(mod)", None),
        ("import * as mod from 'mod'; Reflect.preventExtensions(mod)", None),
    ];

    let fail = vec![
        ("import mod1 from 'mod'; mod1 = 0", None),
        ("import mod2 from 'mod'; mod2 += 0", None),
        ("import mod3 from 'mod'; mod3++", None),
        ("import mod4 from 'mod'; for (mod4 in foo);", None),
        ("import mod5 from 'mod'; for (mod5 of foo);", None),
        ("import mod6 from 'mod'; [mod6] = foo", None),
        ("import mod7 from 'mod'; [mod7 = 0] = foo", None),
        ("import mod8 from 'mod'; [...mod8] = foo", None),
        ("import mod9 from 'mod'; ({ bar: mod9 } = foo)", None),
        ("import mod10 from 'mod'; ({ bar: mod10 = 0 } = foo)", None),
        ("import mod11 from 'mod'; ({ ...mod11 } = foo)", None),
        ("import {named1} from 'mod'; named1 = 0", None),
        ("import {named2} from 'mod'; named2 += 0", None),
        ("import {named3} from 'mod'; named3++", None),
        ("import {named4} from 'mod'; for (named4 in foo);", None),
        ("import {named5} from 'mod'; for (named5 of foo);", None),
        ("import {named6} from 'mod'; [named6] = foo", None),
        ("import {named7} from 'mod'; [named7 = 0] = foo", None),
        ("import {named8} from 'mod'; [...named8] = foo", None),
        ("import {named9} from 'mod'; ({ bar: named9 } = foo)", None),
        ("import {named10} from 'mod'; ({ bar: named10 = 0 } = foo)", None),
        ("import {named11} from 'mod'; ({ ...named11 } = foo)", None),
        ("import {named12 as foo} from 'mod'; foo = 0; named12 = 0", None),
        ("import * as mod1 from 'mod'; mod1 = 0", None),
        ("import * as mod2 from 'mod'; mod2 += 0", None),
        ("import * as mod3 from 'mod'; mod3++", None),
        ("import * as mod4 from 'mod'; for (mod4 in foo);", None),
        ("import * as mod5 from 'mod'; for (mod5 of foo);", None),
        ("import * as mod6 from 'mod'; [mod6] = foo", None),
        ("import * as mod7 from 'mod'; [mod7 = 0] = foo", None),
        ("import * as mod8 from 'mod'; [...mod8] = foo", None),
        ("import * as mod9 from 'mod'; ({ bar: mod9 } = foo)", None),
        ("import * as mod10 from 'mod'; ({ bar: mod10 = 0 } = foo)", None),
        ("import * as mod11 from 'mod'; ({ ...mod11 } = foo)", None),
        ("import * as mod1 from 'mod'; mod1.named = 0", None),
        ("import * as mod2 from 'mod'; mod2.named += 0", None),
        ("import * as mod3 from 'mod'; mod3.named++", None),
        ("import * as mod4 from 'mod'; for (mod4.named in foo);", None),
        ("import * as mod5 from 'mod'; for (mod5.named of foo);", None),
        ("import * as mod6 from 'mod'; [mod6.named] = foo", None),
        ("import * as mod7 from 'mod'; [mod7.named = 0] = foo", None),
        ("import * as mod8 from 'mod'; [...mod8.named] = foo", None),
        ("import * as mod9 from 'mod'; ({ bar: mod9.named } = foo)", None),
        ("import * as mod10 from 'mod'; ({ bar: mod10.named = 0 } = foo)", None),
        ("import * as mod11 from 'mod'; ({ ...mod11.named } = foo)", None),
        ("import * as mod12 from 'mod'; delete mod12.named", None),
        ("import * as mod from 'mod'; Object.assign(mod, obj)", None),
        ("import * as mod from 'mod'; Object.defineProperty(mod, key, d)", None),
        ("import * as mod from 'mod'; Object.defineProperties(mod, d)", None),
        ("import * as mod from 'mod'; Object.setPrototypeOf(mod, proto)", None),
        ("import * as mod from 'mod'; Object.freeze(mod)", None),
        ("import * as mod from 'mod'; Reflect.defineProperty(mod, key, d)", None),
        ("import * as mod from 'mod'; Reflect.deleteProperty(mod, key)", None),
        ("import * as mod from 'mod'; Reflect.set(mod, key, value)", None),
        ("import * as mod from 'mod'; Reflect.setPrototypeOf(mod, proto)", None),
        ("import mod, * as mod_ns from 'mod'; mod.prop = 0; mod_ns.prop = 0", None),
        ("import * as mod from 'mod'; Object?.defineProperty(mod, key, d)", None),
        ("import * as mod from 'mod'; (Object?.defineProperty)(mod, key, d)", None),
        ("import * as mod from 'mod'; delete mod?.prop", None),
    ];

    Tester::new(NoImportAssign::NAME, NoImportAssign::PLUGIN, pass, fail).test_and_snapshot();
}
