use oxc_ast::{
    AstKind,
    ast::{
        AssignmentOperator, ClassElement, Expression, LogicalOperator, MethodDefinitionKind,
        Statement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn missing_super_all(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to call 'super()'")
        .with_help("Add a 'super()' call to the constructor")
        .with_label(span)
}

fn missing_super_some(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lacked a call of 'super()' in some code paths")
        .with_help("Ensure 'super()' is called in all code paths")
        .with_label(span)
}

fn duplicate_super(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected duplicate 'super()'")
        .with_help("Remove the duplicate 'super()' call")
        .with_label(span)
}

fn bad_super(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected 'super()' because 'super' is not a constructor")
        .with_help("Remove the 'super()' call or check the class declaration")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires `super()` calls in constructors of derived classes and disallows `super()` calls
    /// in constructors of non-derived classes.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, calling `super()` in the constructor of a derived class (a class that extends
    /// another class) is required. Failing to do so will result in a ReferenceError at runtime.
    /// Conversely, calling `super()` in a non-derived class is a syntax error.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // Missing super() call
    /// class A extends B {
    ///     constructor() { }
    /// }
    ///
    /// // super() in non-derived class
    /// class A {
    ///     constructor() {
    ///         super();
    ///     }
    /// }
    ///
    /// // super() only in some code paths
    /// class C extends D {
    ///     constructor() {
    ///         if (condition) {
    ///             super();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // Proper super() call in derived class
    /// class A extends B {
    ///     constructor() {
    ///         super();
    ///     }
    /// }
    ///
    /// // No super() in non-derived class
    /// class A {
    ///     constructor() { }
    /// }
    ///
    /// // super() in all code paths
    /// class C extends D {
    ///     constructor() {
    ///         if (condition) {
    ///             super();
    ///         } else {
    ///             super();
    ///         }
    ///     }
    /// }
    /// ```
    ConstructorSuper,
    eslint,
    correctness
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SuperClassType {
    None,    // No extends clause
    Null,    // extends null
    Invalid, // extends <literal or invalid expression>
    Valid,   // extends <potentially valid class expression>
}

#[derive(Debug, Clone)]
struct SuperCallInfo {
    span: Span,
    node_id: NodeId,
}

impl Rule for ConstructorSuper {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Match on Class declarations/expressions
        let AstKind::Class(class) = node.kind() else { return };

        // Classify the superclass
        let super_class_type = Self::classify_super_class(class.super_class.as_ref());

        // Find the constructor in the class body
        let Some(constructor) = class.body.body.iter().find_map(|elem| {
            if let ClassElement::MethodDefinition(method) = elem {
                if matches!(method.kind, MethodDefinitionKind::Constructor) {
                    return Some(method);
                }
            }
            None
        }) else {
            // No constructor defined - this is OK
            return;
        };

        // Get the constructor body
        let Some(body) = &constructor.value.body else { return };

        // Find all super() calls in the constructor
        let super_calls = Self::find_super_calls(body, ctx, node.id());

        // Filter out super() calls in nested scopes (functions, classes)
        let direct_super_calls: Vec<_> = super_calls
            .iter()
            .filter(|call| !Self::is_in_nested_scope(call.node_id, ctx, node.id()))
            .collect();

        // Apply validation based on superclass type
        match super_class_type {
            SuperClassType::None | SuperClassType::Invalid => {
                // Should NOT have super() calls
                for call in direct_super_calls {
                    ctx.diagnostic(bad_super(call.span));
                }
            }
            SuperClassType::Null => {
                // extends null: must have super() call OR return statement
                // But super() call is invalid (null is not a constructor)
                if direct_super_calls.is_empty() {
                    // No super() - check if there's a return statement
                    if !Self::has_return_statement(&body.statements) {
                        ctx.diagnostic(missing_super_all(constructor.span));
                    }
                } else {
                    // Has super() call - this is invalid
                    for call in direct_super_calls {
                        ctx.diagnostic(bad_super(call.span));
                    }
                }
            }
            SuperClassType::Valid => {
                // MUST have super() call(s)
                if direct_super_calls.is_empty() {
                    ctx.diagnostic(missing_super_all(constructor.span));
                    return;
                }

                // Check if super() is guaranteed to execute in all paths
                let has_guaranteed = Self::has_guaranteed_super_call(&body.statements, ctx);

                if !has_guaranteed {
                    ctx.diagnostic(missing_super_some(constructor.span));
                }

                // Check for duplicate super() calls
                if direct_super_calls.len() > 1 {
                    let calls_in_control_flow: Vec<bool> = direct_super_calls
                        .iter()
                        .map(|call| Self::is_inside_control_flow(call.node_id, ctx, node.id()))
                        .collect();

                    let all_in_control_flow = calls_in_control_flow.iter().all(|&x| x);
                    let none_in_control_flow = calls_in_control_flow.iter().all(|&x| !x);

                    // Duplicate if:
                    // - All calls are sequential (none in control flow) - definitely duplicate
                    // - All calls are in control flow but has_guaranteed is false - not mutually exclusive
                    if none_in_control_flow || (all_in_control_flow && !has_guaranteed) {
                        ctx.diagnostic(duplicate_super(direct_super_calls[1].span));
                    }
                    // OK if:
                    // - All calls in control flow AND has_guaranteed - mutually exclusive branches
                    // - Mixed (some in, some out) - fallback pattern like loop with super() after
                }
            }
        }
    }
}

impl ConstructorSuper {
    /// Classify the superclass expression to determine if super() is needed/valid
    fn classify_super_class(super_class: Option<&Expression>) -> SuperClassType {
        match super_class {
            None => SuperClassType::None,
            Some(Expression::NullLiteral(_)) => SuperClassType::Null,
            Some(expr) => {
                if Self::is_invalid_super_class(expr) {
                    SuperClassType::Invalid
                } else {
                    SuperClassType::Valid
                }
            }
        }
    }

    /// Check if an expression is definitely an invalid superclass
    fn is_invalid_super_class(expr: &Expression) -> bool {
        match expr {
            // Literal values are invalid
            Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::BigIntLiteral(_) => true,

            // Parenthesized: unwrap and check inner expression
            Expression::ParenthesizedExpression(paren) => {
                Self::is_invalid_super_class(&paren.expression)
            }

            // Assignment expressions
            Expression::AssignmentExpression(assign) => {
                match assign.operator {
                    // Direct assignment to literal is invalid: extends (B = 5)
                    AssignmentOperator::Assign => Self::is_invalid_super_class(&assign.right),

                    // Arithmetic/bitwise assignments are invalid
                    AssignmentOperator::Addition
                    | AssignmentOperator::Subtraction
                    | AssignmentOperator::Multiplication
                    | AssignmentOperator::Division
                    | AssignmentOperator::Remainder
                    | AssignmentOperator::ShiftLeft
                    | AssignmentOperator::ShiftRight
                    | AssignmentOperator::ShiftRightZeroFill
                    | AssignmentOperator::BitwiseOR
                    | AssignmentOperator::BitwiseXOR
                    | AssignmentOperator::BitwiseAnd
                    | AssignmentOperator::Exponential => true,

                    // Logical assignments: extends (B &&= C)
                    // &&= is invalid if right side is invalid
                    // ||= and ??= are valid (could short-circuit to left)
                    AssignmentOperator::LogicalAnd => Self::is_invalid_super_class(&assign.right),
                    AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish => false,
                }
            }

            // Logical expressions
            Expression::LogicalExpression(logical) => {
                match logical.operator {
                    // extends (A && B)
                    // Result is A if A is falsy, otherwise B
                    // Invalid if B is invalid (could be the result if A is truthy)
                    // Exception: if A is a falsy literal, result is always A
                    LogicalOperator::And => {
                        // If right is invalid, the whole expression could be invalid
                        // unless we can prove left is falsy
                        Self::is_invalid_super_class(&logical.right)
                    }
                    // extends (B || 5) or (B ?? 5) - could be valid if left is valid
                    LogicalOperator::Or | LogicalOperator::Coalesce => false,
                }
            }

            // Binary expressions with operators
            Expression::BinaryExpression(_) => {
                // Binary operations could produce invalid values
                true
            }

            // Conditional: extends (a ? B : C) - valid if either branch could be valid
            Expression::ConditionalExpression(cond) => {
                Self::is_invalid_super_class(&cond.consequent)
                    && Self::is_invalid_super_class(&cond.alternate)
            }

            // Sequence: extends (B, C) - result is last expression
            Expression::SequenceExpression(seq) => {
                seq.expressions.last().map_or(true, |e| Self::is_invalid_super_class(e))
            }

            // Everything else could potentially be a valid class
            _ => false,
        }
    }

    /// Find all super() calls in the constructor body
    fn find_super_calls(
        _body: &oxc_ast::ast::FunctionBody,
        ctx: &LintContext,
        class_node_id: NodeId,
    ) -> Vec<SuperCallInfo> {
        ctx.nodes()
            .iter()
            .filter_map(|node| {
                if let AstKind::CallExpression(call_expr) = node.kind() {
                    if matches!(&call_expr.callee, Expression::Super(_)) {
                        // Check if this call is within our class
                        let in_our_class = ctx.nodes().ancestors(node.id()).any(|ancestor| {
                            if ancestor.id() == class_node_id {
                                return true;
                            }
                            // Stop if we hit another class
                            matches!(ancestor.kind(), AstKind::Class(_))
                                && ancestor.id() != class_node_id
                        });

                        if in_our_class {
                            return Some(SuperCallInfo {
                                span: call_expr.span,
                                node_id: node.id(),
                            });
                        }
                    }
                }
                None
            })
            .collect()
    }

    /// Check if a node is inside a nested function or class
    fn is_in_nested_scope(node_id: NodeId, ctx: &LintContext, class_node_id: NodeId) -> bool {
        for ancestor in ctx.nodes().ancestors(node_id) {
            if ancestor.id() == class_node_id {
                return false;
            }

            match ancestor.kind() {
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    // Check if this function is the constructor itself
                    let parent = ctx.nodes().parent_node(ancestor.id());
                    if let AstKind::MethodDefinition(method) = parent.kind() {
                        if matches!(method.kind, MethodDefinitionKind::Constructor) {
                            continue;
                        }
                    }
                    return true;
                }
                AstKind::Class(_) if ancestor.id() != class_node_id => return true,
                _ => {}
            }
        }
        false
    }

    /// Check if a node is inside a control flow statement or conditional expression
    fn is_inside_control_flow(node_id: NodeId, ctx: &LintContext, class_node_id: NodeId) -> bool {
        for ancestor in ctx.nodes().ancestors(node_id) {
            if ancestor.id() == class_node_id {
                return false;
            }

            match ancestor.kind() {
                AstKind::IfStatement(_)
                | AstKind::SwitchStatement(_)
                | AstKind::TryStatement(_)
                | AstKind::ConditionalExpression(_) => {
                    return true;
                }
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    // Check if this is the constructor
                    let parent = ctx.nodes().parent_node(ancestor.id());
                    if let AstKind::MethodDefinition(method) = parent.kind() {
                        if matches!(method.kind, MethodDefinitionKind::Constructor) {
                            continue;
                        }
                    }
                    return false;
                }
                _ => {}
            }
        }
        false
    }

    /// Check if statements contain a return statement
    fn has_return_statement(statements: &[Statement]) -> bool {
        statements.iter().any(|stmt| Self::statement_contains_return(stmt))
    }

    /// Recursively check if a statement contains a return
    fn statement_contains_return(stmt: &Statement) -> bool {
        match stmt {
            Statement::ReturnStatement(_) => true,
            Statement::BlockStatement(block) => Self::has_return_statement(&block.body),
            Statement::IfStatement(if_stmt) => {
                Self::statement_contains_return(&if_stmt.consequent)
                    || if_stmt
                        .alternate
                        .as_ref()
                        .map_or(false, |alt| Self::statement_contains_return(alt))
            }
            Statement::SwitchStatement(switch) => {
                switch.cases.iter().any(|case| Self::has_return_statement(&case.consequent))
            }
            Statement::TryStatement(try_stmt) => {
                Self::has_return_statement(&try_stmt.block.body)
                    || try_stmt
                        .handler
                        .as_ref()
                        .map_or(false, |handler| Self::has_return_statement(&handler.body.body))
                    || try_stmt
                        .finalizer
                        .as_ref()
                        .map_or(false, |finalizer| Self::has_return_statement(&finalizer.body))
            }
            Statement::WhileStatement(s) => Self::statement_contains_return(&s.body),
            Statement::DoWhileStatement(s) => Self::statement_contains_return(&s.body),
            Statement::ForStatement(s) => Self::statement_contains_return(&s.body),
            Statement::ForInStatement(s) => Self::statement_contains_return(&s.body),
            Statement::ForOfStatement(s) => Self::statement_contains_return(&s.body),
            _ => false,
        }
    }

    /// Check if super() is guaranteed to execute in all code paths
    fn has_guaranteed_super_call(statements: &[Statement], ctx: &LintContext) -> bool {
        // Check if any statement guarantees super() execution
        // This handles sequential statements - if any guarantees super(), we're good
        for stmt in statements {
            if Self::contains_guaranteed_super(stmt, ctx) {
                return true;
            }
            // If we hit a control flow statement that doesn't guarantee super(),
            // check if it's acceptable (exits acceptably or always exits)
            if Self::is_control_flow_statement(stmt) {
                if !Self::statement_always_exits(stmt) && !Self::exits_acceptably(stmt) {
                    // Control flow that doesn't guarantee super(), doesn't always exit,
                    // and doesn't exit acceptably -> can't trust super() after it
                    return false;
                }
            }
            // If we hit a statement that always returns/throws, stop checking
            // (statements after it are unreachable)
            if Self::statement_always_exits(stmt) {
                break;
            }
        }
        false
    }

    /// Check if a statement is a control flow statement that prevents reaching code after it
    fn is_control_flow_statement(stmt: &Statement) -> bool {
        matches!(
            stmt,
            Statement::IfStatement(_) | Statement::SwitchStatement(_) | Statement::TryStatement(_)
        )
    }

    /// Check if a statement always exits (return/throw) without falling through
    fn statement_always_exits(stmt: &Statement) -> bool {
        match stmt {
            Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => true,
            Statement::BlockStatement(block) => {
                block.body.last().map_or(false, |s| Self::statement_always_exits(s))
            }
            Statement::IfStatement(if_stmt) => {
                // Both branches must always exit
                let then_exits = Self::statement_always_exits(&if_stmt.consequent);
                let else_exits = if_stmt
                    .alternate
                    .as_ref()
                    .map_or(false, |alt| Self::statement_always_exits(alt));
                then_exits && else_exits
            }
            _ => false,
        }
    }

    /// Check if statements end with break, return, or throw
    fn ends_with_break_or_exit(statements: &[Statement]) -> bool {
        statements.last().map_or(false, |stmt| {
            matches!(
                stmt,
                Statement::BreakStatement(_)
                    | Statement::ReturnStatement(_)
                    | Statement::ThrowStatement(_)
            ) || matches!(stmt, Statement::BlockStatement(block) if Self::ends_with_break_or_exit(&block.body))
        })
    }

    /// Check if a statement exits via throw or return-with-value (acceptable for constructor)
    fn exits_acceptably(stmt: &Statement) -> bool {
        match stmt {
            Statement::ThrowStatement(_) => true,
            Statement::ReturnStatement(ret) => {
                // Return with a value is acceptable
                ret.argument.is_some()
            }
            Statement::BlockStatement(block) => {
                block.body.last().map_or(false, |s| Self::exits_acceptably(s))
            }
            Statement::IfStatement(if_stmt) => {
                // If the then branch exits acceptably, that's OK
                Self::exits_acceptably(&if_stmt.consequent)
            }
            _ => false,
        }
    }

    /// Check if an expression guarantees super() execution
    fn expression_guarantees_super(expr: &Expression) -> bool {
        match expr {
            // Direct super() call
            Expression::CallExpression(call) => matches!(&call.callee, Expression::Super(_)),

            // Conditional: both branches must have super()
            Expression::ConditionalExpression(cond) => {
                Self::expression_guarantees_super(&cond.consequent)
                    && Self::expression_guarantees_super(&cond.alternate)
            }

            // Parenthesized: check inner expression
            Expression::ParenthesizedExpression(paren) => {
                Self::expression_guarantees_super(&paren.expression)
            }

            _ => false,
        }
    }

    /// Recursively check if a statement guarantees super() execution
    fn contains_guaranteed_super(stmt: &Statement, ctx: &LintContext) -> bool {
        match stmt {
            // Direct super() call as an expression statement
            Statement::ExpressionStatement(expr_stmt) => {
                Self::expression_guarantees_super(&expr_stmt.expression)
            }

            // If-else: BOTH branches must have guaranteed super()
            Statement::IfStatement(if_stmt) => {
                let then_has_super = Self::contains_guaranteed_super(&if_stmt.consequent, ctx);
                let else_has_super = if_stmt
                    .alternate
                    .as_ref()
                    .map_or(false, |alt| Self::contains_guaranteed_super(alt, ctx));

                then_has_super && else_has_super
            }

            // Switch: ALL cases including default must have super()
            // Also check for fallthrough - if a case has super() but no break, it's a problem
            Statement::SwitchStatement(switch) => {
                // Must have a default case
                let has_default = switch.cases.iter().any(|case| case.test.is_none());
                if !has_default {
                    return false;
                }

                // Check each case: must have super() AND must not fall through after super()
                for (i, case) in switch.cases.iter().enumerate() {
                    let has_super = Self::has_guaranteed_super_call(&case.consequent, ctx);
                    let has_break = Self::ends_with_break_or_exit(&case.consequent);

                    if has_super {
                        // If this case has super() but no break/return/throw,
                        // and it's not the last case, it could fall through
                        if !has_break && i < switch.cases.len() - 1 {
                            // Fallthrough after super() - not valid
                            return false;
                        }
                    } else {
                        // Case doesn't have super() - not guaranteed in all paths
                        return false;
                    }
                }

                true
            }

            // Try-finally: super() in finally block is guaranteed
            Statement::TryStatement(try_stmt) => try_stmt
                .finalizer
                .as_ref()
                .map_or(false, |finalizer| Self::has_guaranteed_super_call(&finalizer.body, ctx)),

            // Block statement: recursively check
            Statement::BlockStatement(block) => Self::has_guaranteed_super_call(&block.body, ctx),

            // Loops, returns, throws, etc. do not guarantee execution
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A { }",
        "class A { constructor() { } }",
        "class A extends null { }",
        "class A extends B { }",
        "class A extends B { constructor() { super(); } }",
        "class A extends B { constructor() { if (true) { super(); } else { super(); } } }",
        "class A extends (class B {}) { constructor() { super(); } }",
        "class A extends (B = C) { constructor() { super(); } }",
        "class A extends (B &&= C) { constructor() { super(); } }",
        "class A extends (B ||= C) { constructor() { super(); } }",
        "class A extends (B ??= C) { constructor() { super(); } }",
        "class A extends (B ||= 5) { constructor() { super(); } }",
        "class A extends (B ??= 5) { constructor() { super(); } }",
        "class A extends (B || C) { constructor() { super(); } }",
        "class A extends (5 && B) { constructor() { super(); } }",
        "class A extends (false && B) { constructor() { super(); } }",
        "class A extends (B || 5) { constructor() { super(); } }",
        "class A extends (B ?? 5) { constructor() { super(); } }",
        "class A extends (a ? B : C) { constructor() { super(); } }",
        "class A extends (B, C) { constructor() { super(); } }",
        "class A { constructor() { class B extends C { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C { constructor() { } } } }",
        "class A extends B { constructor() { a ? super() : super(); } }",
        "class A extends B { constructor() { if (a) super(); else super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); break; default: super(); } } }",
        "class A extends B { constructor() { try {} finally { super(); } } }",
        "class A extends B { constructor() { if (a) throw Error(); super(); } }",
        "class A extends B { constructor() { if (true) return a; super(); } }",
        "class A extends null { constructor() { return a; } }",
        "class A { constructor() { return a; } }",
        "class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }",
        "class A extends B { constructor(a) { super(); for (b in a) ( foo(b) ); } }",
        "class Foo extends Object { constructor(method) { super(); this.method = method || function() {}; } }",
        "class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0; i < 0; i++);
			    }
			}
			",
        "class A extends Object {
			    constructor() {
			        super();
			        for (; i < 0; i++);
			    }
			}
			",
        "class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0;; i++) {
			            if (foo) break;
			        }
			    }
			}
			",
        "class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0; i < 0;);
			    }
			}
			",
        "class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0;;) {
			            if (foo) break;
			        }
			    }
			}
			",
        "
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
			        ",
        "class A extends obj?.prop { constructor() { super(); } }",
        "
			            class A extends Base {
			                constructor(list) {
			                    for (const a of list) {
			                        if (a.foo) {
			                            super(a);
			                            return;
			                        }
			                    }
			                    super();
			                }
			            }
			        ",
    ];

    let fail = vec![
        "class A extends null { constructor() { super(); } }",
        "class A extends null { constructor() { } }",
        "class A extends 100 { constructor() { super(); } }",
        "class A extends 'test' { constructor() { super(); } }",
        "class A extends (B = 5) { constructor() { super(); } }",
        "class A extends (B && 5) { constructor() { super(); } }",
        "class A extends (B &&= 5) { constructor() { super(); } }",
        "class A extends (B += C) { constructor() { super(); } }",
        "class A extends (B -= C) { constructor() { super(); } }",
        "class A extends (B **= C) { constructor() { super(); } }",
        "class A extends (B |= C) { constructor() { super(); } }",
        "class A extends (B &= C) { constructor() { super(); } }",
        "class A extends B { constructor() { } }",
        "class A extends B { constructor() { for (var a of b) super.foo(); } }",
        "class A extends B { constructor() { for (var i = 1; i < 10; i++) super.foo(); } }",
        "class A extends B { constructor() { var c = class extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { var c = () => super(); } }",
        "class A extends B { constructor() { class C extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { var C = class extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C extends D { constructor() { } } } }",
        "class A extends B { constructor() { super(); var C = class extends D { constructor() { } } } }",
        "class A extends B { constructor() { if (a) super(); } }",
        "class A extends B { constructor() { if (a); else super(); } }",
        "class A extends B { constructor() { a && super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); } } }",
        "class A extends B { constructor() { switch (a) { case 0: break; default: super(); } } }",
        "class A extends B { constructor() { try { super(); } catch (err) {} } }",
        "class A extends B { constructor() { try { a; } catch (err) { super(); } } }",
        "class A extends B { constructor() { if (a) return; super(); } }",
        "class A extends B { constructor() { super(); super(); } }",
        "class A extends B { constructor() { super() || super(); } }",
        "class A extends B { constructor() { if (a) super(); super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); default: super(); } } }",
        "class A extends B { constructor(a) { while (a) super(); } }",
        "class A extends B { constructor() { return; super(); } }",
        "class Foo extends Bar {
			                constructor() {
			                    for (a in b) for (c in d);
			                }
			            }",
        "class C extends D {

			                constructor() {
			                    do {
			                        something();
			                    } while (foo);
			                }

			            }",
        "class C extends D {

			                constructor() {
			                    for (let i = 1;;i++) {
			                        if (bar) {
			                            break;
			                        }
			                    }
			                }

			            }",
        "class C extends D {

			                constructor() {
			                    do {
			                        super();
			                    } while (foo);
			                }

			            }",
        "class C extends D {

			                constructor() {
			                    while (foo) {
			                        if (bar) {
			                            super();
			                            break;
			                        }
			                    }
			                }

			            }",
    ];

    Tester::new(ConstructorSuper::NAME, ConstructorSuper::PLUGIN, pass, fail).test_and_snapshot();
}
