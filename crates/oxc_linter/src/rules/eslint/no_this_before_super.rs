use std::collections::HashSet;

use oxc_ast::{
    ast::{ArrowExpression, Expression, Function, MethodDefinitionKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{
    pg::neighbors_filtered_by_edge_weight, AssignmentValue, BasicBlockElement, CallType,
    CalleeWithArgumentsAssignmentValue, EdgeType, ObjectPropertyAccessAssignmentValue,
};
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-this-before-super): Expected to always call super() before this/super property access.")]
#[diagnostic(severity(warning), help("Call super() before this/super property access."))]
struct NoThisBeforeSuperDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoThisBeforeSuper;

declare_oxc_lint!(
    /// ### What it does
    /// Requires all getters to have a return statement
    ///
    /// ### Why is this bad?
    /// Getters should always return a value. If they don't, it's probably a mistake.
    ///
    /// ### Example
    /// ```javascript
    /// class Person{
    ///     get name(){
    ///         // no return
    ///     }
    /// }
    /// ```
    NoThisBeforeSuper,
    nursery
);

impl NoThisBeforeSuper {
    fn is_wanted_node(node: &AstNode, ctx: &LintContext<'_>) -> bool {
        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
            if let AstKind::MethodDefinition(mdef) = parent.kind() {
                if matches!(mdef.kind, MethodDefinitionKind::Constructor) {
                    let parent_2 = ctx.nodes().parent_node(parent.id());
                    if let Some(parent_2) = parent_2 {
                        let parent_3 = ctx.nodes().parent_node(parent_2.id());
                        if let Some(parent_3) = parent_3 {
                            if let AstKind::Class(c) = parent_3.kind() {
                                if let Some(super_class) = &c.super_class {
                                    return !matches!(super_class, Expression::NullLiteral(_));
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn run_diagnostic<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !Self::is_wanted_node(node, ctx) {
            return;
        }

        let cfg = ctx.semantic().cfg();

        let mut registers_currently_with_super_in_them = HashSet::new();
        let mut registers_currently_with_this_in_them = HashSet::new();

        neighbors_filtered_by_edge_weight(
            &cfg.graph,
            cfg.function_to_node_ix[&node.id()],
            &|edge| match edge {
                EdgeType::Normal => None,
                // We don't need to handle backedges because we would have already visited
                // them on the forward pass
                | EdgeType::Backedge
                // We don't need to visit NewFunction edges because it's not going to be evaluated
                // immediately, and we are only doing a pass over things that will be immediately evaluated
                | EdgeType::NewFunction
                // By returning Some(X),
                // we signal that we don't walk to this path any farther.
                //
                // The actual value that we return here doesn't matter because
                // we don't use the final value of cfg paths in this rule.
                => Some(CalledSuperBeforeThis::Initial),
            },
            // The state_going_into_this_rule represents whether or not we have seen an expression
            // call super().
            &mut |basic_block_id, state_going_into_this_rule| {
                let mut state = state_going_into_this_rule;
                // Scan through the values in this basic block.
                for entry in &cfg.basic_blocks[*basic_block_id] {
                    let mut should_clear = true;
                    match entry {
                        // If the element is an assignment.
                        //
                        // Everything you can write in javascript that would have
                        // the function continue are expressed as assignments in the cfg.
                        BasicBlockElement::Assignment(to_reg, val) => {
                            match val {
                                // If the assignment value is super, we are either just
                                // accessing this, or going to do something further with it,
                                // so let's take note of the register holding the super.
                                AssignmentValue::Super => {
                                    should_clear = false;
                                    registers_currently_with_super_in_them.insert(to_reg);
                                }
                                // Same as super, but for this
                                AssignmentValue::This => {
                                    should_clear = false;
                                    registers_currently_with_this_in_them.insert(to_reg);
                                }
                                AssignmentValue::CalleeWithArguments(b)
                                    if matches!(b.call_type, CallType::CallExpression) =>
                                {
                                    let CalleeWithArgumentsAssignmentValue { callee, .. } = &**b;
                                    // If we see a CallExpression with a callee that is a super,
                                    // we know that this path has now called super() and is free
                                    // to use super and this.
                                    //
                                    // We could also flag a diagnostic if the path calls this(),
                                    // however this isn't semantically meaningful or tested so
                                    // we do not.
                                    if registers_currently_with_super_in_them.contains(callee) {
                                        state = CalledSuperBeforeThis::SuperCalled;
                                    }
                                }
                                // If we see an object property access, check if we are accessing
                                // this or super, if so check if we have already called super().
                                // If not, flag the diagnostic.
                                AssignmentValue::ObjectPropertyAccess(b) => {
                                    let ObjectPropertyAccessAssignmentValue {
                                        id, access_on, ..
                                    } = &**b;
                                    if !matches!(state, CalledSuperBeforeThis::SuperCalled)
                                        && (registers_currently_with_super_in_them
                                            .contains(access_on)
                                            || registers_currently_with_this_in_them
                                                .contains(access_on))
                                    {
                                        ctx.diagnostic(NoThisBeforeSuperDiagnostic(
                                            ctx.nodes().get_node(*id).kind().span(),
                                        ));
                                    }
                                }
                                _ => {}
                            }

                            // Any assignments to registers other than super / this mean
                            // that this register is no longer significant to us.
                            if should_clear {
                                registers_currently_with_super_in_them.remove(to_reg);
                                registers_currently_with_this_in_them.remove(to_reg);
                            }
                        }
                        // No need to keep following this path if there is an unreachable as
                        // we simply can not do anything this rule checks for if there is an
                        // unreachable or throw.
                        BasicBlockElement::Unreachable | BasicBlockElement::Throw(_) => {
                            return (state, false);
                        }
                    }
                }

                // Return the current state going into the next basic block on this path,
                // returning true to indicate continue walking this path.
                (state, true)
            },
        );
    }
}

#[derive(Default, Copy, Clone, Debug)]
enum CalledSuperBeforeThis {
    #[default]
    Initial,
    SuperCalled,
}

impl Rule for NoThisBeforeSuper {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(Function { .. })
            | AstKind::ArrowExpression(ArrowExpression { .. }) => {
                Self::run_diagnostic(node, ctx);
            }

            _ => {}
        }
    }

    fn from_configuration(_value: serde_json::Value) -> Self {
        Self
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        /*
         * if the class has no extends or `extends null`, just ignore.
         * those classes cannot call `super()`.
         */
        ("class A { }", None),
        ("class A { constructor() { } }", None),
        ("class A { constructor() { this.b = 0; } }", None),
        ("class A { constructor() { this.b(); } }", None),
        ("class A extends null { }", None),
        ("class A extends null { constructor() { } }", None),

        // allows `this`/`super` after `super()`.
        ("class A extends B { }", None),
        ("class A extends B { constructor() { super(); } }", None),
        ("class A extends B { constructor() { super(); this.c = this.d; } }", None),
        ("class A extends B { constructor() { super(); this.c(); } }", None),
        ("class A extends B { constructor() { super(); super.c(); } }", None),
        ("class A extends B { constructor() { if (true) { super(); } else { super(); } this.c(); } }", None),
        ("class A extends B { constructor() { foo = super(); this.c(); } }", None),
        ("class A extends B { constructor() { foo += super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo |= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo &= super().a; this.c(); } }", None),

        // allows `this`/`super` in nested executable scopes, even if before `super()`.
        ("class A extends B { constructor() { class B extends C { constructor() { super(); this.d = 0; } } super(); } }", None),
        ("class A extends B { constructor() { var B = class extends C { constructor() { super(); this.d = 0; } }; super(); } }", None),
        ("class A extends B { constructor() { function c() { this.d(); } super(); } }", None),
        ("class A extends B { constructor() { var c = function c() { this.d(); }; super(); } }", None),
        ("class A extends B { constructor() { var c = () => this.d(); super(); } }", None),

        // ignores out of constructors.
        ("class A { b() { this.c = 0; } }", None),
        ("class A extends B { c() { this.d = 0; } }", None),
        ("function a() { this.b = 0; }", None),

        // multi code path.
        ("class A extends B { constructor() { if (a) { super(); this.a(); } else { super(); this.b(); } } }", None),
        ("class A extends B { constructor() { if (a) super(); else super(); this.a(); } }", None),
        ("class A extends B { constructor() { try { super(); } finally {} this.a(); } }", None),

        // https://github.com/eslint/eslint/issues/5261
        ("class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }", None),
        ("class A extends B { constructor(a) { for (const b of a) { foo(b); } super(); } }", None),

        // https://github.com/eslint/eslint/issues/5319
        ("class A extends B { constructor(a) { super(); this.a = a && function(){} && this.foo; } }", None),

        // https://github.com/eslint/eslint/issues/5394
        (
            r"class A extends Object {
                constructor() {
                    super();
                    for (let i = 0; i < 0; i++);
                    this;
                }
            }", None),

        // https://github.com/eslint/eslint/issues/5894
        ("class A { constructor() { return; this; } }", None),
        ("class A extends B { constructor() { return; this; } }", None),

        // https://github.com/eslint/eslint/issues/8848
        (r"
            class A extends B {
                constructor(props) {
                    super(props);

                    try {
                        let arr = [];
                        for (let a of arr) {
                        }
                    } catch (err) {
                    }
                }
            }
        ", None),

        // Class field initializers are always evaluated after `super()`.
        ("class C { field = this.toString(); }", None),
        ("class C extends B { field = this.foo(); }", None),
        ("class C extends B { field = this.foo(); constructor() { super(); } }", None),
        ("class C extends B { field = this.foo(); constructor() { } }", None) // < in this case, initializers are never evaluated.
    ];

    let fail = vec![
        // disallows all `this`/`super` if `super()` is missing.
        ("class A extends B { constructor() { this.c = 0; } }", None),
        ("class A extends B { constructor() { this.c(); } }", None),
        ("class A extends B { constructor() { super.c(); } }", None),
        // disallows `this`/`super` before `super()`.
        ("class A extends B { constructor() { this.c = 0; super(); } }", None),
        ("class A extends B { constructor() { this.c(); super(); } }", None),
        ("class A extends B { constructor() { super.c(); super(); } }", None),
        // disallows `this`/`super` in arguments of `super()`.
        ("class A extends B { constructor() { super(this.c); } }", None),
        ("class A extends B { constructor() { super(this.c()); } }", None),
        ("class A extends B { constructor() { super(super.c()); } }", None),
        // // even if is nested, reports correctly.
        ("class A extends B { constructor() { class C extends D { constructor() { super(); this.e(); } } this.f(); super(); } }", None),
        ("class A extends B { constructor() { class C extends D { constructor() { this.e(); super(); } } super(); this.f(); } }", None),
        // multi code path.
        ("class A extends B { constructor() { if (a) super(); this.a(); } }", None),
        ("class A extends B { constructor() { try { super(); } finally { this.a; } } }", None),
        ("class A extends B { constructor() { try { super(); } catch (err) { } this.a; } }", None),
        ("class A extends B { constructor() { foo &&= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo ||= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo ??= super().a; this.c(); } }", None),
    ];

    Tester::new(NoThisBeforeSuper::NAME, pass, fail).test_and_snapshot();
}
