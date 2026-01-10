use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, KnownMemberExpressionProperty, PossibleJestNode,
        is_type_of_jest_fn_call, parse_general_jest_fn_call,
    },
};
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use std::{
    hash::{Hash, Hasher},
    sync::{Mutex, OnceLock},
};

/// A single setConfig entry (end position of the call, and whether it has a valid timeout)
#[derive(Debug, Clone, Copy)]
struct SetConfigEntry {
    end_position_of_call: u32,
    has_valid_test_timeout: bool,
}

/// Cached setConfig entries
type SetConfigEntries = Vec<SetConfigEntry>;
type SetConfigCache = OnceLock<Mutex<FxHashMap<String, SetConfigEntries>>>;
static SET_CONFIG_CACHE: SetConfigCache = OnceLock::new();
fn require_test_timeout_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test is missing a timeout.")
        .with_help("Add a timeout to prevent tests from hanging indefinitely.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireTestTimeout;
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that all Vitest test cases (`test`, `it`) have an explicit timeout defined.
    ///
    /// ### Why is this bad?
    ///
    /// Tests without timeouts can hang indefinitely, blocking CI/CD pipelines and wasting resources.
    /// Explicit timeouts ensure tests fail fast when they encounter issues, improving the development workflow.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('my test', () => {
    ///     expect(true).toBe(true);
    /// });
    ///
    /// it('another test', async () => {
    ///     await someAsyncOperation();
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('my test', () => {
    ///     expect(true).toBe(true);
    /// }, 5000);
    ///
    /// it('another test', async () => {
    ///     await someAsyncOperation();
    /// }, 10000);
    ///
    /// test('with options', () => {
    ///     expect(true).toBe(true);
    /// }, { timeout: 5000 });
    /// ```
    RequireTestTimeout,
    vitest,
    restriction
);

impl Rule for RequireTestTimeout {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) -> Option<()> {
    let node = possible_jest_node.node;
    let call_expr = node.kind().as_call_expression()?;

    if !is_type_of_jest_fn_call(
        call_expr,
        possible_jest_node,
        ctx,
        &[JestFnKind::General(JestGeneralFnKind::Test)],
    ) {
        return None;
    }

    // If this is a chain with .todo or .skip or the function name starts with 'x', skip
    if let Some(parsed) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) {
        let has_todo_or_skip = parsed
            .members
            .iter()
            .filter_map(KnownMemberExpressionProperty::name)
            .any(|n| n == "todo" || n == "skip");

        if has_todo_or_skip || parsed.name.starts_with('x') {
            return None;
        }
    }

    // If there's a prior `vi.setConfig({ testTimeout: <number> })` call that ends before
    // this test, exempt the test.
    let test_pos = call_expr.span.start;
    let source_text = ctx.semantic().source_text();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_text.hash(&mut hasher);

    let source_hash = hasher.finish();
    let file_key = format!("{}:{:x}", ctx.file_path().to_string_lossy(), source_hash);

    // Try to get cached entries for this file
    let mut set_config_entries = {
        let cache_mutex = SET_CONFIG_CACHE.get_or_init(|| Mutex::new(FxHashMap::default()));
        let cache = cache_mutex.lock().unwrap();
        cache.get(&file_key).cloned()
    };

    if set_config_entries.is_none() {
        let mut entries = Vec::new();

        // Scan all call expressions looking for `vi.setConfig({ testTimeout: ... })`.
        for node in ctx.semantic().nodes().iter() {
            let oxc_ast::AstKind::CallExpression(candidate_call) = node.kind() else {
                continue;
            };

            // Check for `vi.setConfig(...)` where the callee is a static member expression
            // like `vi.setConfig` and the base object is `vi`/`vitest`/`jest`.
            if let Expression::StaticMemberExpression(member) = &candidate_call.callee
                && member.static_property_info().1 == "setConfig"
            {
                let base = member.get_first_object();
                if (base.is_specific_id("vi")
                    || base.is_specific_id("vitest")
                    || base.is_specific_id("jest"))
                    && let Some(arg) =
                        candidate_call.arguments.first().and_then(|a| a.as_expression())
                    && let Expression::ObjectExpression(obj_expr) = arg.get_inner_expression()
                {
                    // Determine if this setConfig has an explicit valid numeric testTimeout
                    let mut has_valid_test_timeout = false;
                    for prop in &obj_expr.properties {
                        let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                        else {
                            continue;
                        };

                        let is_test_timeout = match &obj_prop.key {
                            oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => {
                                ident.name == "testTimeout"
                            }
                            oxc_ast::ast::PropertyKey::StringLiteral(lit) => {
                                lit.value == "testTimeout"
                            }
                            _ => false,
                        };

                        if !is_test_timeout {
                            continue;
                        }

                        if let Expression::NumericLiteral(lit) = &obj_prop.value
                            && lit.value >= 0.0
                            && lit.value.is_finite()
                        {
                            has_valid_test_timeout = true;
                            break;
                        }
                    }

                    entries.push(SetConfigEntry {
                        end_position_of_call: candidate_call.span.end,
                        has_valid_test_timeout,
                    });
                }
            }
        }

        // Store in cache
        let cache_mutex = SET_CONFIG_CACHE.get_or_init(|| Mutex::new(FxHashMap::default()));
        let mut cache = cache_mutex.lock().unwrap();
        cache.insert(file_key, entries.clone());
        set_config_entries = Some(entries);
    }

    let set_config_entries = set_config_entries.unwrap();

    // Look for the latest setConfig before the test start; if it has a valid timeout, exempt.
    if set_config_entries
        .iter()
        .filter(|e| e.end_position_of_call <= test_pos)
        .max_by_key(|e| e.end_position_of_call)
        .is_some_and(|e| e.has_valid_test_timeout)
    {
        return None;
    }

    match check_has_timeout(call_expr) {
        TimeoutCheck::Valid => {}
        TimeoutCheck::Invalid | TimeoutCheck::Missing => {
            ctx.diagnostic(require_test_timeout_diagnostic(call_expr.span));
        }
    }

    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeoutCheck {
    Valid,
    Invalid,
    Missing,
}

fn is_valid_numeric_literal(lit: &oxc_ast::ast::NumericLiteral) -> bool {
    lit.value >= 0.0 && lit.value.is_finite()
}

fn is_valid_unary_timeout(unary: &oxc_ast::ast::UnaryExpression) -> bool {
    // Only accept unary + operator with numeric literals, reject - (negative values)
    matches!(unary.operator, oxc_ast::ast::UnaryOperator::UnaryPlus)
        && matches!(unary.argument.get_inner_expression(), Expression::NumericLiteral(_))
}

/// Check whether an expression is a valid timeout value.
/// Returns `TimeoutCheck::Valid` if it is a valid timeout,
/// `TimeoutCheck::Invalid` if it is an explicit invalid timeout (e.g. negative, null, undefined),
/// and `TimeoutCheck::Missing` if it is not a timeout value at all.
fn check_timeout_value_expr(expr: &Expression, allow_identifier: bool) -> TimeoutCheck {
    match expr {
        Expression::NumericLiteral(lit) => {
            if is_valid_numeric_literal(lit) {
                TimeoutCheck::Valid
            } else {
                TimeoutCheck::Invalid
            }
        }
        Expression::Identifier(identifier) => {
            if allow_identifier {
                let name = identifier.name.as_str();
                if name != "undefined" && name != "null" && name != "NaN" && name != "Infinity" {
                    TimeoutCheck::Valid
                } else {
                    TimeoutCheck::Invalid
                }
            } else {
                TimeoutCheck::Invalid
            }
        }
        Expression::NullLiteral(_) => TimeoutCheck::Invalid,
        Expression::StaticMemberExpression(_) => TimeoutCheck::Valid,
        Expression::UnaryExpression(unary) => {
            if is_valid_unary_timeout(unary) {
                TimeoutCheck::Valid
            } else {
                TimeoutCheck::Invalid
            }
        }
        _ => TimeoutCheck::Missing,
    }
}

fn find_timeout_in_object(
    obj_expr: &oxc_ast::ast::ObjectExpression,
    allow_identifier: bool,
) -> TimeoutCheck {
    for prop in &obj_expr.properties {
        let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
            continue;
        };

        let is_timeout_key = match &obj_prop.key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => ident.name == "timeout",
            oxc_ast::ast::PropertyKey::StringLiteral(lit) => lit.value == "timeout",
            _ => false,
        };

        if !is_timeout_key {
            continue;
        }

        return check_timeout_value_expr(&obj_prop.value, allow_identifier);
    }

    TimeoutCheck::Missing
}

fn check_has_timeout(call_expr: &oxc_ast::ast::CallExpression) -> TimeoutCheck {
    let args = &call_expr.arguments;
    let mut found_numeric_valid = false;
    let mut found_object_valid = false;

    // Fast-path: inspect the common "third argument" style (test(name, fn, timeoutOrOpts))
    if args.len() >= 3
        && let Some(third_arg) = args.get(2).and_then(|arg| arg.as_expression())
    {
        let inner = third_arg.get_inner_expression();

        // Try to parse the third argument as a timeout value (identifiers are allowed here)
        match check_timeout_value_expr(inner, true) {
            TimeoutCheck::Valid => found_numeric_valid = true,
            TimeoutCheck::Invalid => return TimeoutCheck::Invalid,
            TimeoutCheck::Missing => {}
        }

        // If the third arg is an options object: check for a timeout property
        // Do not accept identifiers for object `timeout` properties — treat them as invalid
        // to match upstream behavior.
        if let Expression::ObjectExpression(obj_expr) = inner {
            match find_timeout_in_object(obj_expr, false) {
                TimeoutCheck::Valid => found_object_valid = true,
                TimeoutCheck::Invalid => return TimeoutCheck::Invalid,
                TimeoutCheck::Missing => {}
            }
        }
    }

    // General scan across all arguments: literal numeric timeouts or options objects
    for a in args {
        if let Some(expr) = a.as_expression() {
            match expr.get_inner_expression() {
                Expression::NumericLiteral(lit) => {
                    if is_valid_numeric_literal(lit) {
                        found_numeric_valid = true;
                    } else {
                        return TimeoutCheck::Invalid;
                    }
                }
                Expression::ObjectExpression(obj_expr) => {
                    // Do not accept identifiers for object `timeout` properties — treat them as invalid
                    // to match upstream behavior.
                    match find_timeout_in_object(obj_expr, false) {
                        TimeoutCheck::Valid => found_object_valid = true,
                        TimeoutCheck::Invalid => return TimeoutCheck::Invalid,
                        TimeoutCheck::Missing => {}
                    }
                }
                _ => {}
            }
        }
    }

    if found_numeric_valid || found_object_valid {
        TimeoutCheck::Valid
    } else {
        TimeoutCheck::Missing
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "test.todo(\"a\")",
        "xit(\"a\", () => {})",
        "test(\"a\", () => {}, 0)",
        "it(\"a\", () => {}, 500)",
        "it.skip(\"a\", () => {})",
        "test.skip(\"a\", () => {})",
        "test(\"a\", () => {}, 1000)",
        "it.only(\"a\", () => {}, 1234)",
        "test.only(\"a\", () => {}, 1234)",
        "it.concurrent(\"a\", () => {}, 400)",
        "test(\"a\", () => {}, { timeout: 0 })",
        "test.concurrent(\"a\", () => {}, 400)",
        "test(\"a\", () => {}, { timeout: 500 })",
        "test(\"a\", { timeout: 500 }, () => {})",
        "vi.setConfig({ testTimeout: 1000 }); test(\"a\", () => {})",
        "test(\"a\", { foo: 1 }, { timeout: 500 }, () => {})",
        "test(\"a\", { timeout: 500 }, 1000, () => {})",
        "test(\"a\", () => {}, 1000, { extra: true })",
    ];

    let fail = vec![
        "test(\"a\", () => {})",
        "it(\"a\", () => {})",
        "test.only(\"a\", () => {})",
        "test.concurrent(\"a\", () => {})",
        "it.concurrent(\"a\", () => {})",
        "vi.setConfig({}); test(\"a\", () => {})",
        "const t = 500; test(\"a\", { timeout: t }, () => {})",
        "test(\"a\", () => {}, { timeout: null })",
        "test(\"a\", () => {}, { timeout: undefined })",
        "test(\"a\", () => {}, -100)",
        "test(\"a\", () => {}, { timeout: -1 })",
        "vi.setConfig({ testTimeout: null }); test(\"a\", () => {})",
        "vi.setConfig({ testTimeout: undefined }); test(\"a\", () => {})",
        "test(\"a\", () => {}); vi.setConfig({ testTimeout: 1000 })",
        "const opts = { timeout: 1000 }; test(\"a\", { ...opts }, () => {})",
        "const opts = { timeout: 1000 }; test(\"a\", { ...opts }, { foo: 1 }, () => {})",
        "test(\"a\", () => {}, { timeout: -1 }, { timeout: 500 })",
        "test(\"a\", { timeout: 500 }, { timeout: -1 })",
        "test(\"a\", () => {}, { timeout: -1 }, 1000)",
        "test(\"a\", () => {}, 1000, { timeout: -1 })",
    ];

    Tester::new(RequireTestTimeout::NAME, RequireTestTimeout::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
