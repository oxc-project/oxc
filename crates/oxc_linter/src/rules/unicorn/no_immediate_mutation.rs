use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrayExpressionElement, AssignmentExpression, AssignmentTarget, CallExpression,
        Expression, NewExpression, ObjectPropertyKind, Statement, VariableDeclaration,
        VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn array_mutation_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not call `{method}()` immediately after initializing an array."
    ))
    .with_help(format!("Move the elements from `{method}()` into the array initializer."))
    .with_label(span)
}

fn object_assign_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not call `Object.assign()` immediately after initializing an object.")
        .with_help("Move the properties from `Object.assign()` into the object initializer.")
        .with_label(span)
}

fn object_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Do not assign a property immediately after initializing an object literal.",
    )
    .with_help("Move the property into the object initializer.")
    .with_label(span)
}

fn set_add_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not call `.add()` immediately after initializing a Set.")
        .with_help("Add the element to the Set initializer array.")
        .with_label(span)
}

fn map_set_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not call `.set()` immediately after initializing a Map.")
        .with_help("Add the entry to the Map initializer array.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImmediateMutation;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows mutating a variable immediately after initialization.
    ///
    /// ### Why is this bad?
    ///
    /// When you initialize a variable and immediately mutate it, it's cleaner to include
    /// the mutation in the initialization. This makes the code more readable and reduces
    /// the number of statements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const array = [1, 2];
    /// array.push(3);
    ///
    /// const object = {foo: 1};
    /// object.bar = 2;
    ///
    /// const set = new Set([1, 2]);
    /// set.add(3);
    ///
    /// const map = new Map([["foo", 1]]);
    /// map.set("bar", 2);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const array = [1, 2, 3];
    ///
    /// const object = {foo: 1, bar: 2};
    ///
    /// const set = new Set([1, 2, 3]);
    ///
    /// const map = new Map([["foo", 1], ["bar", 2]]);
    /// ```
    NoImmediateMutation,
    unicorn,
    pedantic,
    pending
);

/// The type of initialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitType {
    Array,
    Object,
    Set,
    Map,
}

impl Rule for NoImmediateMutation {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // We look for ExpressionStatements that might be mutations
        let AstKind::ExpressionStatement(expr_stmt) = node.kind() else {
            return;
        };

        // Get parent to find sibling statements
        let parent = ctx.nodes().parent_node(node.id());

        let statements: &[Statement<'a>] = match parent.kind() {
            AstKind::BlockStatement(block) => &block.body,
            AstKind::Program(program) => &program.body,
            AstKind::FunctionBody(body) => &body.statements,
            AstKind::StaticBlock(block) => &block.body,
            AstKind::SwitchCase(case) => &case.consequent,
            _ => return,
        };

        // Find the index of current statement
        let Some(current_idx) = statements.iter().position(|stmt| stmt.span() == expr_stmt.span)
        else {
            return;
        };

        // Need at least one statement before this one
        if current_idx == 0 {
            return;
        }

        let prev_stmt = &statements[current_idx - 1];

        // Check what kind of mutation we're looking at and match with previous statement
        check_mutation(&expr_stmt.expression, prev_stmt, ctx);
    }
}

/// Check if the expression is a mutation that matches the previous statement
fn check_mutation<'a>(expr: &Expression<'a>, prev_stmt: &'a Statement<'a>, ctx: &LintContext<'a>) {
    let expr = expr.get_inner_expression();

    match expr {
        Expression::CallExpression(call) => {
            check_call_mutation(call, prev_stmt, ctx);
        }
        Expression::AssignmentExpression(assign) => {
            check_property_assignment(assign, prev_stmt, ctx);
        }
        _ => {}
    }
}

/// Check call expressions like array.push(), set.add(), map.set(), Object.assign()
fn check_call_mutation<'a>(
    call: &CallExpression<'a>,
    prev_stmt: &'a Statement<'a>,
    ctx: &LintContext<'a>,
) {
    // Skip optional chaining
    if call.optional {
        return;
    }

    // Check for Object.assign(obj, ...)
    if is_object_assign_call(call, ctx) {
        if let Some(first_arg) = call.arguments.first()
            && let Some(arg_expr) = first_arg.as_expression()
            && let Expression::Identifier(id) = arg_expr.get_inner_expression()
            && let Some((var_name, init_type)) = get_prev_declaration(prev_stmt, ctx)
            && var_name == id.name.as_str()
            && init_type == InitType::Object
            && is_valid_object_assign_args(call, &id.name)
        {
            ctx.diagnostic(object_assign_diagnostic(call.span));
        }
        return;
    }

    // Check for method calls like array.push(), set.add(), map.set()
    let Some(member) = call.callee.get_member_expr() else {
        return;
    };

    // Skip optional chaining on member access
    if member.optional() {
        return;
    }

    let Some(method_name) = member.static_property_name() else {
        return;
    };

    let obj = member.object().get_inner_expression();
    let Expression::Identifier(id) = obj else {
        return;
    };

    let Some((var_name, init_type)) = get_prev_declaration(prev_stmt, ctx) else {
        return;
    };

    if var_name != id.name.as_str() {
        return;
    }

    // Check if the mutation arguments reference the variable itself (self-reference)
    if args_reference_variable(call, &id.name) {
        return;
    }

    match (init_type, method_name) {
        (InitType::Array, "push" | "unshift") => {
            // push/unshift must have at least one argument
            if call.arguments.is_empty() {
                return;
            }
            ctx.diagnostic(array_mutation_diagnostic(call.span, method_name));
        }
        (InitType::Set, "add") => {
            // add() must have exactly one argument (not spread, not empty)
            if call.arguments.len() != 1 {
                return;
            }
            if call.arguments.first().is_some_and(Argument::is_spread) {
                return;
            }
            ctx.diagnostic(set_add_diagnostic(call.span));
        }
        (InitType::Map, "set") => {
            // set() must have exactly two arguments (not spread)
            if call.arguments.len() != 2 {
                return;
            }
            if call.arguments.iter().any(Argument::is_spread) {
                return;
            }
            ctx.diagnostic(map_set_diagnostic(call.span));
        }
        _ => {}
    }
}

/// Check property assignments like obj.foo = bar
fn check_property_assignment<'a>(
    assign: &AssignmentExpression<'a>,
    prev_stmt: &'a Statement<'a>,
    ctx: &LintContext<'a>,
) {
    // Only check simple assignment (not +=, -=, etc.)
    if !assign.operator.is_assign() {
        return;
    }

    // Get the member expression from the assignment target
    let member = match &assign.left {
        AssignmentTarget::StaticMemberExpression(m) => Some(m.object.get_inner_expression()),
        AssignmentTarget::ComputedMemberExpression(m) => {
            // Check if the computed property references the object itself
            let obj = m.object.get_inner_expression();
            if let Expression::Identifier(id) = obj
                && expression_references_variable(&m.expression, &id.name)
            {
                return;
            }
            Some(obj)
        }
        _ => None,
    };

    let Some(obj) = member else {
        return;
    };

    let Expression::Identifier(id) = obj else {
        return;
    };

    let Some((var_name, init_type)) = get_prev_declaration(prev_stmt, ctx) else {
        return;
    };

    if var_name != id.name.as_str() {
        return;
    }

    // Only report for object literals
    if init_type != InitType::Object {
        return;
    }

    // Check if right-hand side references the variable (for chained assignments)
    if expression_references_variable(&assign.right, &id.name) {
        return;
    }

    ctx.diagnostic(object_property_diagnostic(assign.span));
}

/// Get the variable name and init type from the previous statement
fn get_prev_declaration<'a>(
    prev_stmt: &'a Statement<'a>,
    ctx: &LintContext<'a>,
) -> Option<(&'a str, InitType)> {
    match prev_stmt {
        Statement::VariableDeclaration(decl) => get_declaration_info(decl, ctx),
        Statement::ExpressionStatement(expr_stmt) => {
            // Check for assignment expression: foo = [1, 2]
            if let Expression::AssignmentExpression(assign) = &expr_stmt.expression {
                get_assignment_info(assign, ctx)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Get the variable name and init type from a variable declaration
fn get_declaration_info<'a>(
    decl: &'a VariableDeclaration<'a>,
    ctx: &LintContext<'a>,
) -> Option<(&'a str, InitType)> {
    // Only check the last declarator (to match ESLint behavior)
    let declarator = decl.declarations.last()?;

    // Only simple identifier bindings (no destructuring)
    let var_name = declarator.id.get_identifier_name()?;

    let init_type = get_init_type(declarator, ctx)?;

    Some((var_name.as_str(), init_type))
}

/// Get the init type from a variable declarator
fn get_init_type<'a>(
    declarator: &VariableDeclarator<'a>,
    ctx: &LintContext<'a>,
) -> Option<InitType> {
    let init = declarator.init.as_ref()?;
    get_expression_init_type(init.get_inner_expression(), ctx)
}

/// Get the init type from an expression
fn get_expression_init_type<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> Option<InitType> {
    match expr {
        Expression::ArrayExpression(_) => Some(InitType::Array),
        Expression::ObjectExpression(_) => Some(InitType::Object),
        Expression::NewExpression(new_expr) => get_new_expression_type(new_expr, ctx),
        _ => None,
    }
}

/// Get the init type from a new expression (new Set(), new Map())
fn get_new_expression_type<'a>(
    new_expr: &NewExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<InitType> {
    let callee = new_expr.callee.get_inner_expression();
    let Expression::Identifier(id) = callee else {
        return None;
    };

    // Only match global Set/Map constructors
    if !id.is_global_reference(ctx.scoping()) {
        return None;
    }

    match id.name.as_str() {
        "Set" | "WeakSet" => Some(InitType::Set),
        "Map" | "WeakMap" => Some(InitType::Map),
        _ => None,
    }
}

/// Get variable name and init type from an assignment expression
fn get_assignment_info<'a>(
    assign: &'a AssignmentExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<(&'a str, InitType)> {
    // Must be simple assignment (=), not compound assignments like +=, ??=, etc.
    if !assign.operator.is_assign() {
        return None;
    }

    // Only simple identifier targets
    let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left else {
        return None;
    };

    // Check for chained assignment (a = b = [...])
    if let Expression::AssignmentExpression(_) = assign.right.get_inner_expression() {
        return None;
    }

    let init_type = get_expression_init_type(assign.right.get_inner_expression(), ctx)?;

    Some((id.name.as_str(), init_type))
}

/// Check if this is an Object.assign() call with global Object
fn is_object_assign_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    let Some(member) = call.callee.get_member_expr() else {
        return false;
    };

    if member.optional() {
        return false;
    }

    let Some(prop_name) = member.static_property_name() else {
        return false;
    };

    if prop_name != "assign" {
        return false;
    }

    let obj = member.object().get_inner_expression();
    if let Expression::Identifier(id) = obj {
        id.name == "Object" && id.is_global_reference(ctx.scoping())
    } else {
        false
    }
}

/// Check if Object.assign() arguments are valid (at least 2 args, no self-reference)
fn is_valid_object_assign_args(call: &CallExpression<'_>, var_name: &str) -> bool {
    // Must have at least 2 arguments (target and at least one source)
    if call.arguments.len() < 2 {
        return false;
    }

    // First argument must not be spread
    if call.arguments.first().is_some_and(Argument::is_spread) {
        return false;
    }

    // If the first source argument (second overall) is a spread, skip
    // Object.assign(obj, ...spread) is not easily fixable
    if call.arguments.get(1).is_some_and(Argument::is_spread) {
        return false;
    }

    // Check if any argument references the variable
    for arg in call.arguments.iter().skip(1) {
        if let Some(expr) = arg.as_expression()
            && expression_references_variable(expr, var_name)
        {
            return false;
        }
    }

    true
}

/// Check if any argument in a call references the variable
fn args_reference_variable(call: &CallExpression<'_>, var_name: &str) -> bool {
    for arg in &call.arguments {
        if let Some(expr) = arg.as_expression()
            && expression_references_variable(expr, var_name)
        {
            return true;
        }
    }
    false
}

/// Check if an expression references a variable (shallow check for common patterns)
fn expression_references_variable(expr: &Expression<'_>, var_name: &str) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(id) => id.name == var_name,
        Expression::StaticMemberExpression(m) => {
            expression_references_variable(&m.object, var_name)
        }
        Expression::ComputedMemberExpression(m) => {
            expression_references_variable(&m.object, var_name)
                || expression_references_variable(&m.expression, var_name)
        }
        Expression::CallExpression(c) => {
            if expression_references_variable(&c.callee, var_name) {
                return true;
            }
            c.arguments.iter().any(|arg| {
                arg.as_expression().is_some_and(|e| expression_references_variable(e, var_name))
            })
        }
        Expression::ArrayExpression(arr) => arr.elements.iter().any(|el| match el {
            ArrayExpressionElement::SpreadElement(s) => {
                expression_references_variable(&s.argument, var_name)
            }
            ArrayExpressionElement::Elision(_) => false,
            _ => el.as_expression().is_some_and(|e| expression_references_variable(e, var_name)),
        }),
        Expression::ObjectExpression(obj) => obj.properties.iter().any(|prop| match prop {
            ObjectPropertyKind::ObjectProperty(p) => {
                expression_references_variable(&p.value, var_name)
                    || (p.computed
                        && p.key
                            .as_expression()
                            .is_some_and(|e| expression_references_variable(e, var_name)))
            }
            ObjectPropertyKind::SpreadProperty(s) => {
                expression_references_variable(&s.argument, var_name)
            }
        }),
        Expression::ArrowFunctionExpression(arrow) => {
            // Check if any parameter shadows the variable name
            let is_shadowed = arrow.params.items.iter().any(|param| {
                param.pattern.get_identifier_name().is_some_and(|name| name == var_name)
            });
            if is_shadowed {
                return false;
            }
            // Check the function body for references
            arrow.body.statements.iter().any(|stmt| {
                if let Statement::ExpressionStatement(expr_stmt) = stmt {
                    expression_references_variable(&expr_stmt.expression, var_name)
                } else if let Statement::ReturnStatement(ret) = stmt {
                    ret.argument
                        .as_ref()
                        .is_some_and(|e| expression_references_variable(e, var_name))
                } else {
                    false
                }
            })
        }
        Expression::AssignmentExpression(assign) => {
            // Check both sides of the assignment
            let left_refs = match &assign.left {
                AssignmentTarget::StaticMemberExpression(m) => {
                    expression_references_variable(&m.object, var_name)
                }
                AssignmentTarget::ComputedMemberExpression(m) => {
                    expression_references_variable(&m.object, var_name)
                        || expression_references_variable(&m.expression, var_name)
                }
                AssignmentTarget::AssignmentTargetIdentifier(id) => id.name == var_name,
                _ => false,
            };
            left_refs || expression_references_variable(&assign.right, var_name)
        }
        Expression::FunctionExpression(func) => {
            // Check if any parameter shadows the variable name
            let is_shadowed = func.params.items.iter().any(|param| {
                param.pattern.get_identifier_name().is_some_and(|name| name == var_name)
            });
            if is_shadowed {
                return false;
            }
            // Check the function body for references
            if let Some(body) = &func.body {
                body.statements.iter().any(|stmt| {
                    if let Statement::ExpressionStatement(expr_stmt) = stmt {
                        expression_references_variable(&expr_stmt.expression, var_name)
                    } else if let Statement::ReturnStatement(ret) = stmt {
                        ret.argument
                            .as_ref()
                            .is_some_and(|e| expression_references_variable(e, var_name))
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        }
        // Conditional expression (ternary): cond ? array[0] : x
        Expression::ConditionalExpression(cond) => {
            expression_references_variable(&cond.test, var_name)
                || expression_references_variable(&cond.consequent, var_name)
                || expression_references_variable(&cond.alternate, var_name)
        }
        // Logical expressions: a && b, a || b, a ?? b
        Expression::LogicalExpression(logic) => {
            expression_references_variable(&logic.left, var_name)
                || expression_references_variable(&logic.right, var_name)
        }
        // Binary expressions: a + b, a - b, etc.
        Expression::BinaryExpression(binary) => {
            expression_references_variable(&binary.left, var_name)
                || expression_references_variable(&binary.right, var_name)
        }
        // Unary expressions: !a, -a, typeof a, etc.
        Expression::UnaryExpression(unary) => {
            expression_references_variable(&unary.argument, var_name)
        }
        // Sequence expressions: (a, b, c)
        Expression::SequenceExpression(seq) => {
            seq.expressions.iter().any(|e| expression_references_variable(e, var_name))
        }
        // Template literals: `${array.length}`
        Expression::TemplateLiteral(template) => {
            template.expressions.iter().any(|e| expression_references_variable(e, var_name))
        }
        // Tagged template: tag`${array.length}`
        Expression::TaggedTemplateExpression(tagged) => {
            expression_references_variable(&tagged.tag, var_name)
                || tagged
                    .quasi
                    .expressions
                    .iter()
                    .any(|e| expression_references_variable(e, var_name))
        }
        // New expression: new Foo(array)
        Expression::NewExpression(new_expr) => {
            expression_references_variable(&new_expr.callee, var_name)
                || new_expr.arguments.iter().any(|arg| {
                    arg.as_expression().is_some_and(|e| expression_references_variable(e, var_name))
                })
        }
        // Await expression: await array
        Expression::AwaitExpression(await_expr) => {
            expression_references_variable(&await_expr.argument, var_name)
        }
        // Yield expression: yield array
        Expression::YieldExpression(yield_expr) => yield_expr
            .argument
            .as_ref()
            .is_some_and(|e| expression_references_variable(e, var_name)),
        // Update expression: array++, ++array
        Expression::UpdateExpression(update) => {
            if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) =
                &update.argument
            {
                id.name == var_name
            } else {
                false
            }
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const array = [1, 2];
            array.notPush(3, 4);",
        "const array = [1, 2];
            ; // Not next to each other
            array.push(3, 4);",
        "const array = [1, 2],
                otherVariable = 1;
            array.push(3, 4);",
        "const array = [1, 2];
            array.push();",
        "const {array} = [1, 2];
            array.push(3, 4);",
        "const [array] = [1, 2];
            array.push(3, 4);",
        "const foo = [1, 2];
            bar.push(3, 4);",
        "const array = [1, 2];
            array.push(array[0]);",
        "const array = [1, 2];
            array.push(((foo) => foo(array.length))());",
        "let array;
            array.push(3, 4);",
        "const array = foo;
            array.push(3, 4);",
        "const array = [1, 2];
            array.push?.(3, 4);",
        "const array = [1, 2];
            array?.push(3, 4);",
        "let array;
            array ??= [1, 2];
            array.push(3, 4);",
        "let foo;
            foo = [1, 2];
            bar.push(3, 4);",
        "let foo, bar;
            foo = bar = [1, 2];
            bar.push(3, 4);",
        "const foo = new Foo();
            foo.bar = [1, 2];
            foo.bar.push(3, 4);",
        "const object = [];
            object.bar = 2;",
        "const [object] = {foo: 1};
            object.bar = 2;",
        "const {object} = {foo: 1};
            object.bar = 2;",
        "const object = {foo: 1};
            object.bar += 2;",
        "const object = {foo: 1};
            object.bar = object.baz = 2;",
        "const foo = {};
            bar.bar = 2;",
        "const object = {foo: 1};
            anotherObject.baz = object.bar = 2;",
        "const object = {foo: 1};
            object[object.foo] = 2;",
        "var object;
            object.bar = 2;",
        "const object = foo;
            object.bar = 2;",
        "let object;
            object ??= {foo: 1};
            object.bar = 2;",
        "let foo;
            foo = {foo: 1};
            bar.bar = 2;",
        "let foo, bar;
            foo = bar = {foo: 1};
            bar.bar = 2;",
        "const foo = new Foo();
            foo.bar = {foo: 1};
            foo.bar.bar = 2;",
        "const object = [];
            Object.assign(object, bar);",
        "const [object] = {foo: 1};
            Object.assign(object, bar);",
        "const {object} = {foo: 1};
            Object.assign(object, bar);",
        "const object = {foo: 1};
            Object.assign?.(object, bar);",
        "const object = {foo: 1};
            Object?.assign(object, bar);",
        "const object = {foo: 1};
            Object.assign();",
        "const object = {foo: 1};
            Object.assign(object);",
        "const object = {foo: 1};
            Object.assign(...object);",
        "const object = {foo: 1};
            Object.assign(object, ...spread);",
        "const object = {foo: 1};
            Object.assign(object, ...spread, bar);",
        "const object = {foo: 1};
            Object.assign(object, ...bar);",
        "const object = {foo: 1};
            NotObject.notAssign(object, bar);",
        "const foo = {foo: 1};
            Object.assign(bar, bar);",
        "let object;
            Object.assign(object, bar);",
        "const object = {foo: 1};
            Object.assign(object, object.foo);",
        "const object = {foo: 1};
            Object.assign(object, {baz(){return object}});",
        "let object;
            object ??= {foo: 1};
            Object.assign(object, bar);",
        "let foo;
            foo = {foo: 1};
            bar.assign(object, baz);",
        "let foo, bar;
            foo = bar = {foo: 1};
            Object.assign(bar, baz);",
        "const foo = new Foo();
            foo.bar = {foo: 1};
            Object.assign(foo.bar, baz);",
        "const set = new Set([1, 2]);
            set.notAdd(3);",
        "const set = new NotSet([1, 2]);
            set.notAdd(3);",
        "const set = new Set([1, 2]);
            ; // Not next to each other
            set.add(3);",
        "const set = new Set([1, 2]),
                otherVariable = 1;
            set.add(3);",
        "const set = new Set([1, 2]);
            set.add();",
        "const set = new Set([1, 2]);
            set.add(3, 4);",
        "const set = new Set([1, 2]);
            set.add(...bar);",
        "const {set} = new Set([1, 2]);
            set.add(3);",
        "const [set] = new Set([1, 2]);
            set.add(3);",
        "const foo = new Set([1, 2]);
            bar.add(3);",
        "const set = new Set([1, 2]);
            set.add(set.size);",
        "const set = new Set([1, 2]);
            set.add(((foo) => foo(set.size))());",
        "let set;
            set.add(3);",
        "const set = foo;
            set.add(3);",
        "const set = new Set([1, 2]);
            set.add?.(3);",
        "const set = new Set([1, 2]);
            set?.add(3);",
        "let set;
            set ??= new Set([1, 2]);
            set.add(3);",
        "let foo;
            foo ??= new Set([1, 2]);
            bar.add(3);",
        "let foo, bar;
            foo = bar = new Set([1, 2]);
            bar.add(3);",
        "const foo = new Foo();
            foo.bar = new Set([1, 2]);
            foo.bar.add(3);",
        r#"const map = new Map([["foo", 1]]);
            map.notSet("bar", 2);"#,
        r#"const map = new NotMap([["foo", 1]]);
            map.set("bar", 2);"#,
        // Shadowed Set/Map/Object should not trigger
        "const Set = CustomSet; const s = new Set(); s.add(1);",
        r#"const Map = CustomMap; const m = new Map(); m.set("a", 1);"#,
        "const Object = CustomObject; const o = {}; Object.assign(o, x);",
        r#"const map = new Map([["foo", 1]]);
            ; // Not next to each other
            map.set("bar", 2);"#,
        r#"const map = new Map([["foo", 1]]),
                otherVariable = 1;
            map.set("bar", 2);"#,
        r#"const map = new Map([["foo", 1]]);
            map.set();"#,
        r#"const map = new Map([["foo", 1]]);
            map.set("bar");"#,
        r#"const map = new Map([["foo", 1]]);
            map.set("bar", 2, extraArgument);"#,
        r#"const map = new Map([["foo", 1]]);
            map.set(..."bar", ..."2");"#,
        r#"const {map} = new Map([["foo", 1]]);
            map.set("bar", 2);"#,
        r#"const [map] = new Map([["foo", 1]]);
            map.set("bar", 2);"#,
        r#"const foo = new Map([["foo", 1]]);
            bar.set("bar", 2);"#,
        r#"const map = new Map([["foo", 1]]);
            map.set(map.size, 2);"#,
        r#"const map = new Map([["foo", 1]]);
            map.set("bar", map.size);"#,
        r#"const map = new Map([["foo", 1]]);
            map.set("bar", ((foo) => foo(map.size))());"#,
        r#"const map = new Map([["foo", 1]]);
            map.set(((foo) => foo(map.size))(), 2);"#,
        r#"let map;
            map.set("bar", 2);"#,
        r#"const map = foo;
            map.set("bar", 2);"#,
        r#"const map = new Map([["foo", 1]]);
            map.set?.("bar", 2);"#,
        r#"const map = new Map([["foo", 1]]);
            map?.set("bar", 2);"#,
        r#"let map;
            map ??= new Map([["foo", 1]]);
            map.set("bar", 2);"#,
        r#"let foo;
            foo = new Map([["foo", 1]]);
            bar.set("bar", 2);"#,
        r#"let foo, bar;
            foo = bar = new Map([["foo", 1]]);
            bar.set("bar", 2);"#,
        r#"const foo = new Foo();
            foo.bar = new Map([["foo", 1]]);
            foo.bar.set("bar", 2);"#,
    ];

    let fail = vec![
        "const array = [1, 2];
            array.push(3, 4);",
        "let array;
            array = [1, 2];
            array.push(3, 4);",
        "const array = [3, 4];
            array.unshift(1, 2);",
        "const array = [];
            array.push(3, 4,);",
        "const array = [];
            array.unshift(1, 2,);",
        "const array = [1, 2,];
            array.push(3, 4);",
        "const array = [3, 4,];
            array.unshift(1, 2);",
        "const otherVariable = 1,
                array = [1, 2,];
            array.push(3, 4);",
        "const array = [1, 2];
            array.push( (( 0, 3 )), (( 0, 4 )) );",
        "const array = [1, 2]; array.push(3, 4); foo()",
        "const array = [1, 2]; array.push(3, 4);",
        "const array = [1, 2];
            array.push(3, 4); // comment",
        "const array = [1, 2];
            array.push(3, 4);
            array.unshift(1, 2);",
        "const array = [1, 2];
            array.push(...bar);",
        "const array = [1, 2];
            array.unshift(...bar);",
        "const array = [1, 2];
            array.unshift(foo());",
        "const array = [1, 2];
            array.unshift(...foo());",
        "const array = [1, 2];
            array.unshift([foo()]);",
        "const array = [1, 2];
            array.push(
                3,
                4,
            );",
        "const array = [1, 2];
            array.push(((array) => foo(array.length))());",
        "let array= [1, 2];
            array.push(3, 4);",
        "var array = [1, 2];
            array.push(3, 4);",
        "const array = [1]
            array.push(2);
            [0].map()",
        "const array = [1]
            ;(( array.push(2) ))
            ;[0].map()",
        "const array = [1]
            array.push(2);
            notNeeded.map()",
        "const array = [1]
            array.push(2);
            array.push(3);
            [0].map()",
        "const array = [1]
            array.push(2);
            array.push(3);
            notNeeded.map()",
        "if(1) {
                const array = [1]
                array.push(2);
                [0].map()
            }",
        "let array
            array = [1, 2]
            array.push(3, 4)
            ;[0].map()",
        "const object = {foo: 1};
            object.bar = 2;",
        "let object;
            object = {foo: 1};
            object.bar = 2;",
        "const object = {foo: 1};
            object[bar] = 2;",
        "const object = {foo: 1};
            object[(( 0, bar ))] = (( baz ));",
        "const object = {};
            object.bar = 2;",
        "const object = {foo: 1,};
            object.bar = 2;",
        "const otherVariable = 1,
                object = {foo: 1};
            object.bar = 2;",
        "const object = {foo: 1}; object.bar = 2; foo()",
        "const object = {foo: 1}; object.bar = 2;",
        "const object = {foo: 1};
            object.bar = 2; // comment",
        "const object = {foo: 1};
            object.bar = 2;
            object.baz = 2;",
        "const object = {foo: 1};
            object.bar = anotherObject.baz = 2;",
        "const object = {foo: 1};
            object.bar = (object) => object.foo;",
        "const object = {foo: 1};
            object.object = 2;",
        "const object = {foo: 1}
            object.bar = 2
            ;[0].map()",
        "const object = {foo: 1}
            object.bar = 2
            ;notNeeded.map()",
        "let object
            object = {foo: 1}
            object.bar = 2
            ;[0].map()",
        "const object = {foo: 1};
            Object.assign(object, bar);",
        "let object;
            object = {foo: 1};
            Object.assign(object, bar);",
        "const object = {foo: 1};
            Object.assign(object, {bar: 2});",
        "const object = {foo: 1};
            Object.assign(object, {bar, baz,});",
        "const object = {foo: 1,};
            Object.assign(object, {bar, baz,});",
        "const object = {};
            Object.assign(object, {bar, baz,});",
        "const object = {};
            Object.assign(object, {});",
        "const object = {};
            Object.assign((( object )), (( 0, bar)));",
        "const object = {};
            Object.assign((( object )), (( {bar: 2} )));",
        "const otherVariable = 1,
                object = {foo: 1};
            Object.assign(object, bar);",
        "const object = {foo: 1}; object.bar = 2; foo()",
        "const object = {foo: 1}; object.bar = 2;",
        "const object = {foo: 1};
            Object.assign(object, bar) // comment",
        "const object = {foo: 1};
            Object.assign(object, bar)
            Object.assign(object, {baz})",
        "const object = {foo: 1};
            Object.assign(object, {baz(object){return object}})",
        "const object = {foo: 1};
            Object.assign(object, bar());",
        "let object = {foo: 1};
            Object.assign(object, bar);",
        "var object = {foo: 1};
            Object.assign(object, bar);",
        "const object = {foo: 1};
            Object.assign(object, bar, baz);",
        "const object = {foo: 1};
            Object.assign(object, {}, baz);",
        "const object = {foo: 1};
            Object.assign(object, bar, ...baz, {bar: 2});",
        "const object = {foo: 1}
            Object.assign(object, bar)
            ;[0].map()",
        "const object = {foo: 1}
            Object.assign(object, bar)
            ;notNeeded.map()",
        "let object
            object = {foo: 1}
            Object.assign(object, bar)
            ;[0].map()",
        "const set = new Set([1, 2]);
            set.add(3);",
        "let set;
            set = new Set([1, 2]);
            set.add(3);",
        "const weakSet = new WeakSet([a, b]);
            weakSet.add(c);",
        "const set = new Set([]);
            set.add(3);",
        "const set = new Set();
            set.add(3);",
        "const set = new Set;
            set.add(3);",
        "const set = (( new Set ));
            set.add(3);",
        "const set = new (( Set ));
            set.add(3);",
        "const otherVariable = 1,
                set = new Set;
            set.add(3);",
        "const set = new Set([1, 2]);
            set.add( ((0, 3)), );",
        "const set = new Set([1, 2]); set.add(3); foo()",
        "const set = new Set([1, 2]); set.add(3);",
        "const set = new Set([1, 2]);
            set.add(3); // comment",
        "const set = new Set([1, 2]);
            set.add(foo());",
        "const set = new Set([1, 2]);
            set
                .add(
                    3,
            );",
        "let set = new Set([1, 2]);
            set.add(3);",
        "var set = new Set([1, 2]);
            set.add(3);",
        "const set = new Set([1, 2])
            set.add(3);
            [0].map()",
        "const set = new Set([1, 2])
            set.add(3);
            notNeeded.map()",
        "const set = new Set
            set.add(3);
            [0].map()",
        "const set = new Set
            set.add(3);
            notNeeded.map()",
        "let set
            set = new Set([1, 2])
            set.add(3)
            ;[0].map()",
        r#"const map = new Map([["foo", 1]]);
            map.set("bar", 2);"#,
        r#"let map;
            map = new Map([["foo", 1]]);
            map.set("bar", 2);"#,
        "const weakMap = new WeakMap([[foo, 1]]);
            weakMap.set(bar, 2);",
        r#"const map = new Map([]);
            map.set("bar", 2);"#,
        r#"const map = new Map();
            map.set("bar", 2);"#,
        r#"const map = new Map;
            map.set("bar", 2);"#,
        r#"const map = (( new Map ));
            map.set("bar", 2);"#,
        r#"const map = new (( Map ));
            map.set("bar", 2);"#,
        r#"const otherVariable = 1,
                map = new Map;
            map.set("bar", 2);"#,
        r#"const map = new Map([["foo",1]]);
            map.set( ((0, "bar")), ((0, 2)), );"#,
        r#"const map = new Map([["foo", 1]]);    map.set("bar", 2);    foo()"#,
        r#"const map = new Map([["foo", 1]]);    map.set("bar", 2);"#,
        r#"const map = new Map([["foo", 1]]);
            map.set("bar", 2); // comment"#,
        r#"const map = new Map([["foo", 1]]);
            map.set("bar", foo());"#,
        r#"const map = new Map([["foo", 1]]);
            map.set(bar(), 2);"#,
        r#"const map = new Map([["foo", 1]]);
            map
                .set(
                    "bar",
                    2,
            );"#,
        r#"let map = new Map([["foo", 1]]);
            map.set("bar", 2);"#,
        r#"var map = new Map([["foo", 1]]);
            map.set("bar", 2);"#,
        r#"const map = new Map([["foo", 1]])
            map.set("bar", 2);
            [0].map()"#,
        r#"const map = new Map([["foo", 1]])
            map.set("bar", 2);
            notNeeded.map()"#,
        r#"const map = new Map
            map.set("bar", 2);
            [0].map()"#,
        r#"const map = new Map
            map.set("bar", 2);
            notNeeded.map()"#,
        r#"let map
            map = new Map([["foo", 1]])
            map.set("bar", 2)
            ;[0].map()"#,
        "const cellOutputMappers = new Map<OutputType, (output: any) => NotebookCellOutput>();
            cellOutputMappers.set('display_data', translateDisplayDataOutput);",
        "const cellOutputMappers = new Map<OutputType, (output: any) => NotebookCellOutput>([]);
            cellOutputMappers.set('display_data', translateDisplayDataOutput);",
        "const cellOutputMappers = new Map<OutputType, (output: any) => NotebookCellOutput>;
            cellOutputMappers.set('display_data', translateDisplayDataOutput);",
    ];

    Tester::new(NoImmediateMutation::NAME, NoImmediateMutation::PLUGIN, pass, fail)
        .test_and_snapshot();
}
