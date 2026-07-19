use oxc_ast::ast::*;

use crate::ast_nodes::{AstNode, AstNodes};

/// This is a specialized function that checks if the current [call expression]
/// resembles a call expression usually used by a testing frameworks.
///
/// If the [call expression] matches the criteria, a different formatting is applied.
///
/// To evaluate the eligibility of a  [call expression] to be a test framework like,
/// we need to check its [callee] and its [arguments].
///
/// 1. The [callee] must contain a name or a chain of names that belongs to the
///    test frameworks, for example: `test()`, `test.only()`, etc.
/// 2. The [arguments] should be at the least 2
/// 3. The first argument has to be a string literal
/// 4. The third argument, if present, has to be a number literal
/// 5. The second argument has to be an [arrow function expression] or [function expression]
/// 6. Both function must have zero or one parameters
///
/// [call expression]: CallExpression
/// [callee]: Expression
/// [arguments]: CallExpression::arguments
/// [arrow function expression]: ArrowFunctionExpression
/// [function expression]: Function
pub fn is_test_call_expression(call: &AstNode<CallExpression<'_>>) -> bool {
    if call.optional {
        return false;
    }

    let callee = &call.callee;
    let arguments = &call.arguments;

    let mut args = arguments.iter();

    match (args.next(), args.next(), args.next()) {
        (Some(argument), None, None) if arguments.len() == 1 => {
            if is_angular_test_wrapper(call) && {
                if let AstNodes::CallExpression(call) = call.parent() {
                    is_test_call_expression(call)
                } else {
                    false
                }
            } {
                return argument.as_expression().is_some_and(Expression::is_function);
            }

            if is_unit_test_set_up_callee(callee) {
                return argument.as_expression().is_some_and(is_angular_test_wrapper_expression);
            }

            false
        }

        // it("description", ..)
        (Some(first), Some(second), third)
            if arguments.len() <= 3
                && first.as_expression().is_some_and(|e| {
                    matches!(e.tag(), ExpressionTag::StringLiteral | ExpressionTag::TemplateLiteral)
                })
                && contains_a_test_pattern(callee) =>
        {
            // it('name', callback, duration)
            if !third.is_none_or(|t| t.as_expression().is_some_and(Expression::is_numeric_literal))
            {
                return false;
            }

            if second.as_expression().is_some_and(is_angular_test_wrapper_expression) {
                return true;
            }

            let (parameter_count, has_block_body) =
                match second.as_expression().map(Expression::kind) {
                    Some(ExpressionKind::FunctionExpression(function)) => {
                        (function.params.parameters_count(), true)
                    }
                    Some(ExpressionKind::ArrowFunctionExpression(arrow)) => {
                        (arrow.params.parameters_count(), !arrow.expression)
                    }
                    _ => return false,
                };

            arguments.len() == 2 || (parameter_count <= 1 && has_block_body)
        }
        _ => false,
    }
}

/// Note: `inject` is used in AngularJS 1.x, `async` and `fakeAsync` in
/// Angular 2+, although `async` is deprecated and replaced by `waitForAsync`
/// since Angular 12.
///
/// example: <https://docs.angularjs.org/guide/unit-testing#using-beforeall->
///
/// @param {CallExpression} node
/// @returns {boolean}
///
fn is_angular_test_wrapper_expression(expression: &Expression) -> bool {
    expression.as_call_expression().is_some_and(|call| is_angular_test_wrapper(call))
}

pub fn is_angular_test_wrapper(call: &CallExpression) -> bool {
    call.callee.as_identifier().is_some_and(|ident| {
        matches!(ident.name.as_str(), "async" | "inject" | "fakeAsync" | "waitForAsync")
    })
}

/// Tests if the callee is a `beforeEach`, `beforeAll`, `afterEach` or `afterAll` identifier
/// that is commonly used in test frameworks.
fn is_unit_test_set_up_callee(callee: &Expression) -> bool {
    callee.as_identifier().is_some_and(|ident| {
        matches!(ident.name.as_str(), "beforeEach" | "beforeAll" | "afterEach" | "afterAll")
    })
}

/// Iterator that returns the callee names in "top down order".
///
/// # Examples
///
/// Some:
///
/// ```javascript
/// it.only() -> [`only`, `it`];
/// describe.skip() -> [`skip`, `describe`];
/// ```
///
/// None:
///
/// ```javascript
/// it().only();
/// describe().skip();
/// ```
///
/// Same as <https://github.com/biomejs/biome/blob/4a5ef84930344ae54f3877da36888a954711f4a6/crates/biome_js_syntax/src/expr_ext.rs#L1402-L1438>.
pub fn callee_name_iterator<'b>(expr: &'b Expression<'_>) -> Option<impl Iterator<Item = &'b str>> {
    let mut names = [Option::None; 5];
    let mut current = Some(expr);

    for index in 0..5 {
        match current.map(Expression::kind) {
            Some(ExpressionKind::Identifier(ident)) => {
                names[index] = Some(ident.name.as_str());
                return Some(names.into_iter().rev().flatten());
            }
            Some(ExpressionKind::StaticMemberExpression(member)) => {
                current = Some(&member.object);
                names[index] = Some(member.property.name.as_str());
            }
            _ => break,
        }
    }

    None
}

/// This function checks if a call expression matches a test framework pattern:
///
/// ```text
/// ├─ it[.only|skip|skipIf|runIf|concurrent|sequential|todo|fails]
/// ├─ describe[.only|skip|skipIf|runIf|concurrent|sequential|shuffle|todo]
/// ├─ test
/// │  ├─ [.only|skip|skipIf|runIf|concurrent|sequential|todo|fails|extend|step|fixme]
/// │  └─ .describe
/// │     ├─ [.only|skip|fixme]
/// │     ├─ .parallel[.only]
/// │     └─ .serial[.only]
/// ├─ bench[.only|skip|todo]
/// ├─ skip|xit|xdescribe|xtest|fit|fdescribe|ftest
/// └─ Deno.test
/// ```
///
/// Implementation (trie-tree) is inspired by the article:
/// <https://craftinginterpreters.com/scanning-on-demand.html#tries-and-state-machines>
pub fn contains_a_test_pattern(expr: &Expression<'_>) -> bool {
    let Some(mut names) = callee_name_iterator(expr) else { return false };

    match names.next() {
        Some("it") => match names.next() {
            None => true,
            Some(
                "only" | "skip" | "skipIf" | "runIf" | "concurrent" | "sequential" | "todo"
                | "fails",
            ) => names.next().is_none(),
            _ => false,
        },
        Some("describe") => match names.next() {
            None => true,
            Some(
                "only" | "skip" | "skipIf" | "runIf" | "concurrent" | "sequential" | "shuffle"
                | "todo",
            ) => names.next().is_none(),
            _ => false,
        },
        Some("Deno") => matches!(names.next(), Some("test")) && names.next().is_none(),
        Some("test") => match names.next() {
            None => true,
            Some(
                "only" | "skip" | "skipIf" | "runIf" | "concurrent" | "sequential" | "todo"
                | "fails" | "extend" | "step" | "fixme",
            ) => names.next().is_none(),
            Some("describe") => match names.next() {
                None => true,
                Some("only" | "skip" | "fixme") => names.next().is_none(),
                Some("parallel" | "serial") => match names.next() {
                    None => true,
                    Some("only") => names.next().is_none(),
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        },
        Some("bench") => match names.next() {
            None => true,
            Some("only" | "skip" | "todo") => names.next().is_none(),
            _ => false,
        },
        Some("skip" | "xit" | "xdescribe" | "xtest" | "fit" | "fdescribe" | "ftest") => true,
        _ => false,
    }
}

pub fn is_test_each_pattern(expr: &Expression<'_>) -> bool {
    let Some(mut names) = callee_name_iterator(expr) else {
        return false;
    };

    let first = names.next();
    let second = names.next();
    let third = names.next();
    let fourth = names.next();
    let fifth = names.next();

    match first {
        Some("describe" | "xdescribe" | "fdescribe") => match second {
            Some("each") => third.is_none(),
            Some("skip" | "only") => match third {
                Some("each") => fourth.is_none(),
                _ => false,
            },
            _ => false,
        },
        Some("test" | "xtest" | "ftest" | "it" | "xit" | "fit") => match second {
            Some("each") => third.is_none(),
            Some("skip" | "only" | "failing") => match third {
                Some("each") => fourth.is_none(),
                _ => false,
            },
            Some("concurrent") => match third {
                Some("each") => fourth.is_none(),
                Some("only" | "skip") => match fourth {
                    Some("each") => fifth.is_none(),
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}
