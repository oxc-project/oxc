use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty,
        VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_const_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}` is never reassigned."))
        .with_help("Use `const` instead.")
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct PreferConstConfig {
    // This can be set to `any` or `all`.
    //
    // For `any`, if any of the variables in a destructuring should be `const`, this rule will warn for those variables.
    //
    // For `all`, only warn if *all* variables in a destructuring should be `const`. Otherwise, it will ignore them.
    destructuring: Destructuring,
    // If `true`, the rule will not report variables that are read before their initial assignment.
    // This is mainly useful for preventing conflicts with the `typescript/no-use-before-define` rule.
    ignore_read_before_assign: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Destructuring {
    #[default]
    Any,
    All,
}

impl Default for PreferConstConfig {
    fn default() -> Self {
        Self { destructuring: Destructuring::Any, ignore_read_before_assign: false }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreferConst(PreferConstConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires `const` declarations for variables that are never
    /// reassigned after their initial declaration.
    ///
    /// ### Why is this bad?
    ///
    /// If a variable is never reassigned, using the `const` declaration is better.
    /// `const` declaration tells readers, "this variable is never reassigned," reducing cognitive load and improving maintainability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let a = 3;
    /// console.log(a);
    ///
    /// let b;
    /// b = 0;
    /// console.log(b);
    ///
    /// for (let i in [1,2,3]) {
    ///   console.log(i);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = 0;
    ///
    /// let a;
    /// a = 0;
    /// a = 1;
    ///
    /// let a;
    /// if (true) {
    ///   a = 0;
    /// }
    ///
    /// for (const i in [1,2,3]) {
    ///   console.log(i);
    /// }
    /// ```
    PreferConst,
    eslint,
    style,
    // TODO: Implement a fix for this rule, it should be possible and the
    // tests are available to use at the bottom of the file.
    pending,
    config = PreferConstConfig,
);

impl Rule for PreferConst {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(value.get(0).and_then(|v| PreferConstConfig::deserialize(v).ok()).unwrap_or_default())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(decl) = node.kind() else {
            return;
        };

        // Skip if not a `let` declaration (we do not check `var` or `const`)
        if decl.kind != VariableDeclarationKind::Let {
            return;
        }

        // Get parent to check if we're in a for-in or for-of loop initializer
        let parent = ctx.nodes().parent_node(node.id());
        let is_for_in_of_init =
            matches!(parent.kind(), AstKind::ForInStatement(_) | AstKind::ForOfStatement(_));

        // For regular for loops (for (let i = 0, j = 1; ...)), we need special handling
        // If ANY variable in the declaration is reassigned, we can't fix the whole declaration
        let is_for_statement_init = matches!(parent.kind(), AstKind::ForStatement(_));

        if is_for_statement_init && decl.declarations.len() > 1 {
            // Check if any declarator is reassigned
            let any_reassigned = decl.declarations.iter().any(|declarator| {
                let has_init = declarator.init.is_some();
                declarator.id.get_binding_identifiers().iter().any(|ident| {
                    let symbol_id = ident.symbol_id();
                    !self.should_be_const(symbol_id, has_init, is_for_in_of_init, ctx)
                })
            });

            // If any variable is reassigned, we can't convert any of them
            if any_reassigned {
                return;
            }
        }

        for declarator in &decl.declarations {
            let binding_identifiers = declarator.id.get_binding_identifiers();
            let has_init = declarator.init.is_some();
            let is_destructuring = declarator.id.kind.is_destructuring_pattern();

            // For destructuring patterns with "all" mode, check if ALL variables should be const
            if matches!(self.0.destructuring, Destructuring::All) && is_destructuring {
                let all_const = binding_identifiers.iter().all(|ident| {
                    let symbol_id = ident.symbol_id();
                    self.should_be_const(symbol_id, has_init, is_for_in_of_init, ctx)
                });

                if all_const {
                    // Flag all variables in the destructuring pattern
                    for ident in binding_identifiers {
                        ctx.diagnostic(prefer_const_diagnostic(ident.name.as_str(), ident.span));
                    }
                }
                // If not all_const, don't flag any of them in "all" mode
            } else {
                // "any" mode (default): check each binding identifier independently
                for ident in binding_identifiers {
                    let symbol_id = ident.symbol_id();
                    if self.should_be_const(symbol_id, has_init, is_for_in_of_init, ctx) {
                        ctx.diagnostic(prefer_const_diagnostic(ident.name.as_str(), ident.span));
                    }
                }
            }
        }
    }
}

impl PreferConst {
    /// Check if an assignment target contains any member expressions (recursively)
    fn has_member_expression_target(target: &AssignmentTargetMaybeDefault) -> bool {
        match target {
            AssignmentTargetMaybeDefault::ComputedMemberExpression(_)
            | AssignmentTargetMaybeDefault::StaticMemberExpression(_)
            | AssignmentTargetMaybeDefault::PrivateFieldExpression(_) => true,
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(t) => {
                Self::has_member_expression_in_assignment_target(&t.binding)
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(array) => array
                .elements
                .iter()
                .any(|elem| elem.as_ref().is_some_and(Self::has_member_expression_target)),
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                obj.properties.iter().any(|prop| match prop {
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                        Self::has_member_expression_target(&p.binding)
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => false,
                })
            }
            _ => false,
        }
    }

    /// Check if an assignment target contains any member expressions (for non-default targets)
    fn has_member_expression_in_assignment_target(target: &AssignmentTarget) -> bool {
        match target {
            AssignmentTarget::ComputedMemberExpression(_)
            | AssignmentTarget::StaticMemberExpression(_)
            | AssignmentTarget::PrivateFieldExpression(_) => true,
            AssignmentTarget::ArrayAssignmentTarget(array) => array
                .elements
                .iter()
                .any(|elem| elem.as_ref().is_some_and(Self::has_member_expression_target)),
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                obj.properties.iter().any(|prop| match prop {
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                        Self::has_member_expression_target(&p.binding)
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => false,
                })
            }
            _ => false,
        }
    }

    /// Check if a single identifier from a destructuring assignment can be declared as const.
    /// Returns false if the identifier is a parameter (can't be changed to const)
    /// or if the identifier is reassigned elsewhere.
    fn can_identifier_be_const(
        &self,
        symbol_id: SymbolId,
        symbol_table: &oxc_semantic::Scoping,
        ctx: &LintContext<'_>,
    ) -> bool {
        // Check if this symbol is a parameter
        // Parameters can't be changed to const
        let decl_node = symbol_table.symbol_declaration(symbol_id);
        if matches!(ctx.nodes().kind(decl_node), oxc_ast::AstKind::FormalParameter(_)) {
            return false;
        }

        // In "all" mode, check if this variable has more than one write
        // (one is the destructuring assignment itself)
        if matches!(self.0.destructuring, Destructuring::All) {
            let references: Vec<_> = symbol_table.get_resolved_references(symbol_id).collect();
            let write_count = references.iter().filter(|r| r.is_write()).count();
            if write_count > 1 {
                return false;
            }
        }

        true
    }

    fn can_all_destructuring_identifiers_be_const(
        &self,
        array_target: &oxc_ast::ast::ArrayAssignmentTarget,
        symbol_table: &oxc_semantic::Scoping,
        ctx: &LintContext<'_>,
    ) -> bool {
        use oxc_ast::ast::AssignmentTargetMaybeDefault;

        for target in array_target.elements.iter().flatten() {
            if let AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) = target {
                // Get the symbol for this identifier
                if let Some(reference_id) = ident.reference_id.get()
                    && let Some(symbol_id) =
                        ctx.semantic().scoping().get_reference(reference_id).symbol_id()
                    && !self.can_identifier_be_const(symbol_id, symbol_table, ctx)
                {
                    return false;
                }
            }
        }
        true
    }

    /// Check if all identifiers in an object destructuring assignment can be const
    /// Returns false if any identifier is a parameter (can't be changed to const)
    /// or if any identifier is reassigned elsewhere
    fn can_all_destructuring_identifiers_be_const_obj(
        &self,
        obj_target: &oxc_ast::ast::ObjectAssignmentTarget,
        symbol_table: &oxc_semantic::Scoping,
        ctx: &LintContext<'_>,
    ) -> bool {
        use oxc_ast::ast::AssignmentTargetProperty;

        for prop in &obj_target.properties {
            match prop {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                    // Get the symbol for this identifier
                    if let Some(reference_id) = ident.binding.reference_id.get()
                        && let Some(symbol_id) =
                            ctx.semantic().scoping().get_reference(reference_id).symbol_id()
                        && !self.can_identifier_be_const(symbol_id, symbol_table, ctx)
                    {
                        return false;
                    }
                }
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(_) => {
                    // For complex properties, we can't easily check, so allow it
                    // This is handled by the member expression check above
                }
            }
        }
        true
    }

    /// Check if a variable should be declared as const
    fn should_be_const(
        &self,
        symbol_id: SymbolId,
        has_init: bool,
        is_for_in_of: bool,
        ctx: &LintContext<'_>,
    ) -> bool {
        let symbol_table = ctx.scoping();

        // Get all references to this symbol
        let references: Vec<_> = symbol_table.get_resolved_references(symbol_id).collect();

        // Count write references (assignments)
        let write_count = references.iter().filter(|r| r.is_write()).count();

        // If configured to ignore reads before the initial assignment and this variable has an
        // initializer, then suppress the suggestion when there exists a read that appears before
        // the declaration. This matches ESLint's `ignoreReadBeforeAssign` behavior for cases like
        // `class C { static { a; } } let a = 1;` where the read happens prior to the init write.
        if self.0.ignore_read_before_assign && has_init {
            let decl_node_id = symbol_table.symbol_declaration(symbol_id);
            let decl_start = ctx.nodes().get_node(decl_node_id).span().start;

            let has_read_before_decl = references.iter().any(|r| {
                if !r.is_read() {
                    return false;
                }
                let read_span = ctx.nodes().get_node(r.node_id()).kind().span();
                read_span.start < decl_start
            });

            if has_read_before_decl {
                return false;
            }
        }

        // For for-in and for-of loops, the variable gets a new binding on each iteration
        // If it's never reassigned in the loop body, it should be const
        if is_for_in_of && write_count == 0 {
            return true;
        }

        // If has initializer and no writes, it should be const
        if has_init && write_count == 0 {
            return true;
        }

        // Handle ignoreReadBeforeAssign option
        if self.0.ignore_read_before_assign && !has_init && write_count == 1 {
            let write_only_refs: Vec<_> =
                references.iter().filter(|r| r.is_write() && !r.is_read()).collect();

            if write_only_refs.len() == 1 {
                let write_ref = write_only_refs[0];
                let write_node_id = write_ref.node_id();

                // Check if there are any reads before the write
                let has_read_before_write = references.iter().any(|r| {
                    if !r.is_read() || r.node_id() == write_node_id {
                        return false;
                    }
                    // Simple span comparison - if read comes before write in source
                    let read_span = ctx.nodes().get_node(r.node_id()).kind().span();
                    let write_span = ctx.nodes().get_node(write_node_id).kind().span();
                    read_span.start < write_span.start
                });

                if has_read_before_write {
                    // With ignoreReadBeforeAssign, don't flag this
                    return false;
                }
            }
        }

        if has_init {
            return false;
        }

        // For variables without initializers, check if there's exactly one write-only reference
        // (not read+write like `a = a + 1`)
        // The write must be in the same scope and not inside any control flow or loops
        let write_only_refs: Vec<_> =
            references.iter().filter(|r| r.is_write() && !r.is_read()).collect();

        if write_only_refs.len() != 1 {
            return false;
        }

        let write_ref = write_only_refs[0];
        let symbol_scope = symbol_table.symbol_scope_id(symbol_id);
        let write_node_id = write_ref.node_id();
        let write_scope = ctx.nodes().get_node(write_node_id).scope_id();

        // If the write is not in the same scope, it can't be const
        if write_scope != symbol_scope {
            return false;
        }

        // Check if the write is inside any control flow, loops, or certain destructuring assignments
        // If a destructuring assignment:
        // 1. Is inside a block (not at function/program level), OR
        // 2. Contains member expressions (property access)
        // Then we can't use const because you can't initialize const without a value
        //
        // EXCEPTION: Variables declared in for-in/of loop bodies get fresh bindings per iteration
        let mut is_invalid_destructuring = false;
        let mut is_in_loop_body = false;

        for ancestor in ctx.nodes().ancestors(write_node_id).skip(1) {
            match ancestor.kind() {
                // Stop at the scope boundary
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::Program(_)
                | AstKind::StaticBlock(_) => break,

                // Check for for-in/of loops FIRST before other checks
                // Variables declared in the loop body get a fresh binding each iteration
                AstKind::ForInStatement(_) | AstKind::ForOfStatement(_) => {
                    // Check if the variable's scope is a child of this loop's scope
                    let loop_scope = ancestor.scope_id();
                    let mut current_scope = Some(symbol_scope);

                    // Walk up the scope tree from the variable's scope
                    while let Some(scope) = current_scope {
                        if scope == loop_scope {
                            // We found the loop scope - variable is NOT in loop body
                            break;
                        }
                        let parent_scope = symbol_table.scope_parent_id(scope);
                        if parent_scope == Some(loop_scope) {
                            // The variable's scope's parent is the loop scope
                            // This means the variable is declared in the loop body
                            is_in_loop_body = true;
                            break;
                        }
                        current_scope = parent_scope;
                    }

                    if !is_in_loop_body {
                        // Variable is declared outside the loop - can't be const
                        return false;
                    }
                    // Variable is in loop body with a single write - it should be const
                    // Each iteration gets a fresh binding, so the single write per iteration is OK
                    // Skip all other ancestor checks
                    return true;
                }

                // If there's a BlockStatement before the scope boundary, and we're in a
                // destructuring assignment, we can't use const
                AstKind::BlockStatement(_) => {
                    // Check if there's a destructuring assignment between us and this block
                    for inner_ancestor in ctx.nodes().ancestors(write_node_id).skip(1) {
                        match inner_ancestor.kind() {
                            AstKind::BlockStatement(_) => break,
                            AstKind::AssignmentExpression(assign) => {
                                if matches!(
                                    assign.left,
                                    AssignmentTarget::ArrayAssignmentTarget(_)
                                        | AssignmentTarget::ObjectAssignmentTarget(_)
                                ) {
                                    is_invalid_destructuring = true;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    // Don't break here - continue checking for for-in/of loops
                    // which override this check
                }

                // Check if this is a destructuring assignment with member expression targets
                AstKind::AssignmentExpression(assign) => {
                    match &assign.left {
                        AssignmentTarget::ArrayAssignmentTarget(array_target) => {
                            // Check if the array contains any member expressions (recursively)
                            if array_target.elements.iter().any(|elem| {
                                elem.as_ref().is_some_and(Self::has_member_expression_target)
                            }) {
                                is_invalid_destructuring = true;
                                break;
                            }

                            // Check if all identifiers in the destructuring can be const
                            // If any identifier has multiple writes or can't be const,
                            // we shouldn't suggest const for others in the same pattern
                            if !self.can_all_destructuring_identifiers_be_const(
                                array_target,
                                symbol_table,
                                ctx,
                            ) {
                                is_invalid_destructuring = true;
                                break;
                            }
                        }
                        AssignmentTarget::ObjectAssignmentTarget(obj_target) => {
                            // Check if the object contains any member expressions (recursively)
                            if obj_target.properties.iter().any(|prop| match prop {
                                AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
                                    Self::has_member_expression_target(&p.binding)
                                }
                                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => {
                                    false
                                }
                            }) {
                                is_invalid_destructuring = true;
                                break;
                            }

                            // Check if all identifiers in the destructuring can be const
                            if !self.can_all_destructuring_identifiers_be_const_obj(
                                obj_target,
                                symbol_table,
                                ctx,
                            ) {
                                is_invalid_destructuring = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                // If the assignment is inside an array expression, we can't convert to const
                // because the assignment is not in the same scope.
                AstKind::ArrayExpression(_) => {
                    return false;
                }

                // Special handling for ForStatement: check if assignment is in the condition
                // e.g., `for (let x; (x = foo()); )` - the assignment is repeated on each iteration
                AstKind::ForStatement(for_stmt) => {
                    // Check if the write is in the condition part (test) of the for loop
                    // If so, it's a repeated assignment and can't be const
                    if let Some(test) = &for_stmt.test {
                        let test_span = test.span();
                        let write_span = ctx.nodes().get_node(write_node_id).kind().span();

                        // If the write is within the test expression span, it's in the condition
                        if write_span.start >= test_span.start && write_span.end <= test_span.end {
                            // Assignment is in the for loop condition, which is evaluated repeatedly
                            return false;
                        }
                    }

                    // Otherwise, check if variable is declared inside the for loop scope
                    let control_flow_scope = ancestor.scope_id();
                    let mut current = symbol_table.scope_parent_id(symbol_scope);
                    let mut is_inside = false;

                    while let Some(scope) = current {
                        if scope == control_flow_scope {
                            is_inside = true;
                            break;
                        }
                        current = symbol_table.scope_parent_id(scope);
                    }

                    if !is_inside {
                        return false;
                    }
                }

                // These indicate conditional or repeated execution
                // If the variable is declared INSIDE the control flow structure,
                // and there's only one write also inside, it can be const
                AstKind::IfStatement(_)
                | AstKind::SwitchStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::ConditionalExpression(_)
                | AstKind::LogicalExpression(_)
                | AstKind::SequenceExpression(_)
                | AstKind::TryStatement(_) => {
                    // Check if the variable's scope is a descendant of this control flow's scope
                    // If yes, the variable is declared inside the control flow
                    let control_flow_scope = ancestor.scope_id();

                    // Walk up from symbol_scope - if we find control_flow_scope as a parent,
                    // then symbol_scope is inside the control flow
                    let mut current = symbol_table.scope_parent_id(symbol_scope);
                    let mut is_inside = false;

                    while let Some(scope) = current {
                        if scope == control_flow_scope {
                            is_inside = true;
                            break;
                        }
                        current = symbol_table.scope_parent_id(scope);
                    }

                    if !is_inside {
                        // Variable is declared outside the control flow but written inside
                        // This means the write is conditional
                        return false;
                    }
                    // Variable is declared inside the control flow, continue checking
                }
                _ => {}
            }
        }

        if is_invalid_destructuring {
            return false;
        }

        true
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = 0;", None),
        ("let x;", None),
        ("let x; { x = 0; } foo(x);", None),
        ("/* let foo = 'bar'; */ const baz = undefined;", None),
        ("/*let*/ foo = 'bar';", None),
        ("let x = 0; x = 1;", None),
        ("using resource = fn();", None), // { "sourceType": "module", "ecmaVersion": 2026 },
        ("await using resource = fn();", None), // { "sourceType": "module", "ecmaVersion": 2026 },
        ("const x = 0;", None),
        ("for (let i = 0, end = 10; i < end; ++i) {}", None),
        ("for (let i in [1,2,3]) { i = 0; }", None),
        ("for (let x of [1,2,3]) { x = 0; }", None),
        ("(function() { var x = 0; })();", None),
        ("(function() { let x; })();", None),
        ("(function() { let x; { x = 0; } foo(x); })();", None),
        ("(function() { let x = 0; x = 1; })();", None),
        ("(function() { const x = 0; })();", None),
        ("(function() { for (let i = 0, end = 10; i < end; ++i) {} })();", None),
        ("(function() { for (let i in [1,2,3]) { i = 0; } })();", None),
        ("(function() { for (let x of [1,2,3]) { x = 0; } })();", None),
        ("(function(x = 0) { })();", None),
        ("let a; while (a = foo());", None),
        ("let a; do {} while (a = foo());", None),
        ("let a; for (; a = foo(); );", None),
        ("let a; for (;; ++a);", None),
        ("let a; for (const {b = ++a} in foo());", None),
        ("let a; for (const {b = ++a} of foo());", None),
        ("let a; for (const x of [1,2,3]) { if (a) {} a = foo(); }", None),
        ("let a; for (const x of [1,2,3]) { a = a || foo(); bar(a); }", None),
        ("let a; for (const x of [1,2,3]) { foo(++a); }", None),
        ("let a; function foo() { if (a) {} a = bar(); }", None),
        ("let a; function foo() { a = a || bar(); baz(a); }", None),
        ("let a; function foo() { bar(++a); }", None),
        (
            "let id;
			function foo() {
			    if (typeof id !== 'undefined') {
			        return;
			    }
			    id = setInterval(() => {}, 250);
			}
			foo();
			",
            None,
        ),
        // NOTE: Oxc does not support the `/*exported*/` directive
        // ("/*exported a*/ let a; function init() { a = foo(); }", None),
        // ("/*exported a*/ let a = 1", None),
        ("let a; if (true) a = 0; foo(a);", None),
        (
            "(function (a) {
                let b;
                ({ a, b } = obj);
            })();",
            None,
        ),
        (
            "(function (a) {
                let b;
                ([ a, b ] = obj);
            })();",
            None,
        ),
        ("var a; { var b; ({ a, b } = obj); }", None),
        ("let a; { let b; ({ a, b } = obj); }", None),
        ("var a; { var b; ([ a, b ] = obj); }", None),
        ("let a; { let b; ([ a, b ] = obj); }", None),
        ("let x; { x = 0; foo(x); }", None),
        ("(function() { let x; { x = 0; foo(x); } })();", None),
        ("let x; for (const a of [1,2,3]) { x = foo(); bar(x); }", None),
        ("(function() { let x; for (const a of [1,2,3]) { x = foo(); bar(x); } })();", None),
        ("let x; for (x of array) { x; }", None),
        ("let {a, b} = obj; b = 0;", Some(serde_json::json!([{ "destructuring": "all" }]))),
        ("let a, b; ({a, b} = obj); b++;", Some(serde_json::json!([{ "destructuring": "all" }]))),
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType, ...predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType,, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType=5, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [[typeNode.returnType=5], predicate] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [[typeNode.returnType, predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [typeNode.returnType, [predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, [typeNode.returnType, predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:typeNode.returnType, predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:typeNode.returnType, ...predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        ("let a; const b = {}; ({ a, c: b.c } = func());", None), // { "ecmaVersion": 2018 },
        (
            "let x; function foo() { bar(x); } x = 0;",
            Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        ),
        ("const x = [1,2]; let y; [,y] = x; y = 0;", None),
        ("const x = [1,2,3]; let y, z; [y,,z] = x; y = 0; z = 0;", None),
        ("class C { static { let a = 1; a = 2; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a = 1; a = 2; } }", None), // { "ecmaVersion": 2022 },
        ("let a; class C { static { a = 1; } }", None),     // { "ecmaVersion": 2022 },
        ("class C { static { let a; if (foo) { a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; if (foo) a = 1; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; if (foo) { ({ a, b } = foo); } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; if (foo) ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
        (
            "class C { static { a; } } let a = 1; ",
            Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { () => a; let a = 1; } };",
            Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 }
        (
            // const must be initialized with a value, so this cannot become a const
            "function example() {
               let value; // declared, not initialized
               return someCheck() && (value = getValue());
             }
            ",
            None,
        ),
        (
            "let t;
             const r = await Promise.race([ a(), (t = b()) ]);
             t.cancel();
            ",
            None,
        ),
        (
            "for (let match; (match = REGEX.exec(line)); ) {
               handle(match);
             }",
            None,
        ),
        // Works with TypeScript type annotations too.
        (
            "for (let match: string[]; (match = REGEX.exec(line)); ) {
               handle(match);
             }",
            None,
        ),
    ];

    let fail = vec![
        ("let x = 1; foo(x);", None),
        ("for (let i in [1,2,3]) { foo(i); }", None),
        ("for (let x of [1,2,3]) { foo(x); }", None),
        ("let [x = -1, y] = [1,2]; y = 0;", None),
        // Ensure a "reassignment" in a comment does not count
        ("let foo = 'bar'; /* foo = 'baz'; */", None),
        ("let foo = 'bar'; // foo = 'baz';", None),
        ("let {a: x = -1, b: y} = {a:1,b:2}; y = 0;", None),
        ("(function() { let x = 1; foo(x); })();", None),
        ("(function() { for (let i in [1,2,3]) { foo(i); } })();", None),
        ("(function() { for (let x of [1,2,3]) { foo(x); } })();", None),
        ("(function() { let [x = -1, y] = [1,2]; y = 0; })();", None),
        ("let f = (function() { let g = x; })(); f = 1;", None),
        ("(function() { let {a: x = -1, b: y} = {a:1,b:2}; y = 0; })();", None),
        ("let x = 0; { let x = 1; foo(x); } x = 0;", None),
        ("for (let i = 0; i < 10; ++i) { let x = 1; foo(x); }", None),
        ("for (let i in [1,2,3]) { let x = 1; foo(x); }", None),
        (
            "var foo = function() {
        	    for (const b of c) {
        	       let a;
        	       a = 1;
        	   }
        	};",
            None,
        ),
        (
            "var foo = function() {
        	    for (const b of c) {
        	       let a;
        	       ({a} = 1);
        	   }
        	};",
            None,
        ),
        ("let x; x = 0;", None),
        ("switch (a) { case 0: let x; x = 0; }", None),
        ("(function() { let x; x = 1; })();", None),
        (
            "let {a = 0, b} = obj; b = 0; foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        (
            "let {a: {b, c}} = {a: {b: 1, c: 2}}; b = 3;",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        (
            "let {a: {b, c}} = {a: {b: 1, c: 2}}",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let a, b; ({a = 0, b} = obj); b = 0; foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        ("let {a = 0, b} = obj; foo(a, b);", Some(serde_json::json!([{ "destructuring": "all" }]))),
        ("let [a] = [1]", Some(serde_json::json!([]))),
        ("let {a} = obj", Some(serde_json::json!([]))),
        (
            "let a, b; ({a = 0, b} = obj); foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let {a = 0, b} = obj, c = a; b = a;",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        (
            "let {a = 0, b} = obj, c = a; b = a;",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ), // { "ecmaVersion": 2018 },
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ), // { "parser": require(fixtureParser("babel-eslint5/destructuring-object-spread"), ), },
        ("let x; function foo() { bar(x); } x = 0;", None),
        ("let x = 1", None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("{ let x = 1 }", None),
        ("let [a] = [1]", Some(serde_json::json!([]))),
        ("let {a} = obj", Some(serde_json::json!([]))),
        (
            "let a, b; ({a = 0, b} = obj); foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let {a = 0, b} = obj, c = a; b = a;",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ),
        (
            "let {a = 0, b} = obj, c = a; b = a;",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ), // { "ecmaVersion": 2018 },
        (
            "let { name, ...otherStuff } = obj; otherStuff = {};",
            Some(serde_json::json!([{ "destructuring": "any" }])),
        ), // { "parser": require(fixtureParser("babel-eslint5/destructuring-object-spread"), ), },
        ("let x; function foo() { bar(x); } x = 0;", None),
        ("let x = 1", None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        // Works with typescript type annotations
        ("let x: string; x = 'hello'; console.log(x);", None),
        ("{ let x = 1 }", None),
        ("let { foo, bar } = baz;", None),
        ("const x = [1,2]; let [,y] = x;", None),
        ("const x = [1,2,3]; let [y,,z] = x;", None),
        ("let predicate; [, {foo:returnType, predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:returnType, predicate}, ...bar ] = foo();", None), // { "ecmaVersion": 2018 },
        ("let predicate; [, {foo:returnType, ...predicate} ] = foo();", None), // { "ecmaVersion": 2018 },
        ("let x = 'x', y = 'y';", None),
        ("let x = 'x', y = 'y'; x = 1", None),
        ("let x = 1, y = 'y'; let z = 1;", None),
        ("let { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;", None),
        ("let x = 'x', y = 'y'; function someFunc() { let a = 1, b = 2; foo(a, b) }", None),
        ("let someFunc = () => { let a = 1, b = 2; foo(a, b) }", None),
        ("let {a, b} = c, d;", None),
        ("let {a, b, c} = {}, e, f;", None),
        (
            "function a() {
			let foo = 0,
			  bar = 1;
			foo = 1;
			}
			function b() {
			let foo = 0,
			  bar = 2;
			foo = 2;
			}",
            None,
        ),
        ("let foo = undefined;", None),
        ("let a = 1; class C { static { a; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { a; } } let a = 1;", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a = 1; } }", None),    // { "ecmaVersion": 2022 },
        ("class C { static { if (foo) { let a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a = 1; if (foo) { a; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { if (foo) { let a; a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a = 1; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let { a, b } = foo; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; let b; ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a = 0; console.log(a); } }", None), // { "ecmaVersion": 2022 },
        (
            "
        	            let { itemId, list } = {},
        	            obj = [],
        	            total = 0;
        	            total = 9;
        	            console.log(itemId, list, obj, total);
        	            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
        	            let { itemId, list } = {},
        	            obj = [];
        	            console.log(itemId, list, obj);
        	            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
        	            let [ itemId, list ] = [],
        	            total = 0;
        	            total = 9;
        	            console.log(itemId, list, total);
        	            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "
        	            let [ itemId, list ] = [],
        	            obj = [];
        	            console.log(itemId, list, obj);
        	            ",
            Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
        ), // { "ecmaVersion": 2022 }
        (
            "for (let seen = new Set(); Math.random() < 0.5;) {
               seen.add('foo');
             }",
            None,
        ),
    ];

    // TODO: Implement the fixer and enable these tests.
    // let fix = vec![
    //     ("let x = 1; foo(x);", "const x = 1; foo(x);", None),
    //     ("for (let i in [1,2,3]) { foo(i); }", "for (const i in [1,2,3]) { foo(i); }", None),
    //     ("for (let x of [1,2,3]) { foo(x); }", "for (const x of [1,2,3]) { foo(x); }", None),
    //     (
    //         "(function() { let x = 1; foo(x); })();",
    //         "(function() { const x = 1; foo(x); })();",
    //         None,
    //     ),
    //     (
    //         "(function() { for (let i in [1,2,3]) { foo(i); } })();",
    //         "(function() { for (const i in [1,2,3]) { foo(i); } })();",
    //         None,
    //     ),
    //     (
    //         "(function() { for (let x of [1,2,3]) { foo(x); } })();",
    //         "(function() { for (const x of [1,2,3]) { foo(x); } })();",
    //         None,
    //     ),
    //     (
    //         "let f = (function() { let g = x; })(); f = 1;",
    //         "let f = (function() { const g = x; })(); f = 1;",
    //         None,
    //     ),
    //     (
    //         "let x = 0; { let x = 1; foo(x); } x = 0;",
    //         "let x = 0; { const x = 1; foo(x); } x = 0;",
    //         None,
    //     ),
    //     (
    //         "for (let i = 0; i < 10; ++i) { let x = 1; foo(x); }",
    //         "for (let i = 0; i < 10; ++i) { const x = 1; foo(x); }",
    //         None,
    //     ),
    //     (
    //         "for (let i in [1,2,3]) { let x = 1; foo(x); }",
    //         "for (const i in [1,2,3]) { const x = 1; foo(x); }",
    //         None,
    //     ),
    //     (
    //         "let {a: {b, c}} = {a: {b: 1, c: 2}}",
    //         "const {a: {b, c}} = {a: {b: 1, c: 2}}",
    //         Some(serde_json::json!([{ "destructuring": "all" }])),
    //     ),
    //     (
    //         "let {a = 0, b} = obj; foo(a, b);",
    //         "const {a = 0, b} = obj; foo(a, b);",
    //         Some(serde_json::json!([{ "destructuring": "all" }])),
    //     ),
    //     ("let [a] = [1]", "const [a] = [1]", Some(serde_json::json!([]))),
    //     ("let {a} = obj", "const {a} = obj", Some(serde_json::json!([]))),
    //     (
    //         "/*eslint custom/use-x:error*/ let x = 1",
    //         "/*eslint custom/use-x:error*/ const x = 1",
    //         None,
    //     ),
    //     (
    //         "/*eslint custom/use-x:error*/ { let x = 1 }",
    //         "/*eslint custom/use-x:error*/ { const x = 1 }",
    //         None,
    //     ),
    //     ("let { foo, bar } = baz;", "const { foo, bar } = baz;", None),
    //     ("const x = [1,2]; let [,y] = x;", "const x = [1,2]; const [,y] = x;", None),
    //     ("const x = [1,2,3]; let [y,,z] = x;", "const x = [1,2,3]; const [y,,z] = x;", None),
    //     ("let x = 'x', y = 'y';", "const x = 'x', y = 'y';", None),
    //     ("let x = 1, y = 'y'; let z = 1;", "const x = 1, y = 'y'; const z = 1;", None),
    //     (
    //         "let { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;",
    //         "const { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;",
    //         None,
    //     ),
    //     (
    //         "let x = 'x', y = 'y'; function someFunc() { let a = 1, b = 2; foo(a, b) }",
    //         "const x = 'x', y = 'y'; function someFunc() { const a = 1, b = 2; foo(a, b) }",
    //         None,
    //     ),
    //     (
    //         "let someFunc = () => { let a = 1, b = 2; foo(a, b) }",
    //         "const someFunc = () => { let a = 1, b = 2; foo(a, b) }",
    //         None,
    //     ),
    //     (
    //         "/*eslint no-undef-init:error*/ let foo = undefined;",
    //         "/*eslint no-undef-init:error*/ const foo = undefined;",
    //         None,
    //     ),
    //     ("let a = 1; class C { static { a; } }", "const a = 1; class C { static { a; } }", None),
    //     ("class C { static { a; } } let a = 1;", "class C { static { a; } } const a = 1;", None),
    //     ("class C { static { let a = 1; } }", "class C { static { const a = 1; } }", None),
    //     (
    //         "class C { static { if (foo) { let a = 1; } } }",
    //         "class C { static { if (foo) { const a = 1; } } }",
    //         None,
    //     ),
    //     (
    //         "class C { static { let a = 1; if (foo) { a; } } }",
    //         "class C { static { const a = 1; if (foo) { a; } } }",
    //         None,
    //     ),
    //     (
    //         "class C { static { let { a, b } = foo; } }",
    //         "class C { static { const { a, b } = foo; } }",
    //         None,
    //     ),
    // ];

    Tester::new(PreferConst::NAME, PreferConst::PLUGIN, pass, fail)
        // .expect_fix(fix)
        .test_and_snapshot();
}
