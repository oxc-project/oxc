use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;
use schemars::JsonSchema;
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_instanceof_builtins_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `instanceof` with built-in constructors")
        .with_help(
            "Use `Array.isArray(…)`, `typeof … === 'string'`, \
             or another realm-safe alternative instead",
        )
        .with_label(span)
}

const PRIMITIVE_WRAPPERS: &[&str] = &["String", "Number", "Boolean", "BigInt", "Symbol"];

const STRICT_STRATEGY_CONSTRUCTORS: &[&str] = &[
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error
    "Error",
    "EvalError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "TypeError",
    "URIError",
    "InternalError",
    "AggregateError",
    // Collection types
    "Map",
    "Set",
    "WeakMap",
    "WeakRef",
    "WeakSet",
    // Arrays and Typed Arrays
    "ArrayBuffer",
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
    // Data types
    "Object",
    // Regular Expressions
    "RegExp",
    // Async and functions
    "Promise",
    "Proxy",
    // Other
    "DataView",
    "Date",
    "SharedArrayBuffer",
    "FinalizationRegistry",
];

#[derive(Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoInstanceofBuiltinsConfig {
    /// Additional constructor names to check beyond the default set.
    /// Use this to extend the rule with additional constructors.
    include: Vec<String>,
    /// Constructor names to exclude from checking.
    exclude: Vec<String>,
    /// When `true`, checks `instanceof Error` and suggests using `Error.isError()` instead.
    /// Requires [the `Error.isError()` function](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/isError)
    /// to be available.
    use_error_is_error: bool,
    /// Controls which built-in constructors are checked.
    /// - `"loose"` (default): Only checks Array, Function, Error (if `useErrorIsError` is true), and primitive wrappers
    /// - `"strict"`: Additionally checks Error types, collections, typed arrays, and other built-in constructors
    strategy: Strategy,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum Strategy {
    Strict,
    #[default]
    Loose,
}

#[derive(Debug, Default, Clone)]
pub struct NoInstanceofBuiltins(Box<NoInstanceofBuiltinsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `instanceof` with ECMAScript built-in constructors because:
    ///
    /// * it breaks across execution contexts (`iframe`, Web Worker, Node VM, etc.);
    /// * it is often misleading (e.g. `instanceof Array` fails for a subclass);
    /// * there is always a clearer and safer alternative (`Array.isArray`, `typeof`, `Buffer.isBuffer`, …).
    ///
    /// ### Why is this bad?
    ///
    /// `instanceof` breaks across execution contexts (`iframe`, Web Worker, Node `vm`),
    /// and may give misleading results for subclasses or exotic objects.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (arr instanceof Array) { … }
    /// if (el instanceof HTMLElement) { … }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (Array.isArray(arr)) { … }
    /// if (el?.nodeType === 1) { … }
    /// ```
    NoInstanceofBuiltins,
    unicorn,
    suspicious,
    pending,
    config = NoInstanceofBuiltinsConfig,
);

impl Rule for NoInstanceofBuiltins {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin) = node.kind() else { return };
        if bin.operator != BinaryOperator::Instanceof {
            return;
        }

        let ctor_expr = bin.right.get_inner_expression();

        let Expression::Identifier(ident) = ctor_expr else { return };
        let ctor_name = ident.name.as_str();

        if self.0.exclude.iter().any(|s| s == ctor_name) {
            return;
        }

        if ctor_name == "Array" || (ctor_name == "Error" && self.0.use_error_is_error) {
            ctx.diagnostic(no_instanceof_builtins_diagnostic(bin.span));
            return;
        }
        if ctor_name == "Function" {
            ctx.diagnostic(no_instanceof_builtins_diagnostic(bin.span));
            return;
        }

        if PRIMITIVE_WRAPPERS.contains(&ctor_name) {
            ctx.diagnostic(no_instanceof_builtins_diagnostic(bin.span));
            return;
        }

        if self.0.include.iter().any(|s| s == ctor_name)
            || (self.0.strategy == Strategy::Strict
                && STRICT_STRATEGY_CONSTRUCTORS.contains(&ctor_name))
        {
            ctx.diagnostic(no_instanceof_builtins_diagnostic(bin.span));
        }
    }

    fn from_configuration(value: Value) -> Self {
        let mut include = Vec::<String>::new();
        let mut exclude = Vec::<String>::new();
        let mut use_error_is_error = false;
        let mut strategy = Strategy::Loose;

        if let Value::Array(arr) = value
            && let Some(Value::Object(map)) = arr.first()
        {
            if let Some(Value::Array(inc)) = map.get("include") {
                for v in inc {
                    if let Value::String(s) = v {
                        include.push(s.clone());
                    }
                }
            }
            if let Some(Value::Array(exc)) = map.get("exclude") {
                for v in exc {
                    if let Value::String(s) = v {
                        exclude.push(s.clone());
                    }
                }
            }
            if let Some(Value::Bool(b)) = map.get("useErrorIsError") {
                use_error_is_error = *b;
            }
            if let Some(Value::String(b)) = map.get("strategy") {
                match b.as_str() {
                    "strict" => strategy = Strategy::Strict,
                    "loose" => strategy = Strategy::Loose,
                    _ => {}
                }
            }
        }

        Self(Box::new(NoInstanceofBuiltinsConfig {
            include,
            exclude,
            use_error_is_error,
            strategy,
        }))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("fooExclude instanceof Function", Some(serde_json::json!([{"exclude": ["Function"]}]))),
        ("fooExclude instanceof Array", Some(serde_json::json!([{"exclude": ["Array"]}]))),
        ("fooExclude instanceof String", Some(serde_json::json!([{"exclude": ["String"]}]))),
        ("Array.isArray(arr)", None),
        ("arr instanceof array", None),
        ("a instanceof 'array'", None),
        ("a instanceof ArrayA", None),
        ("a.x[2] instanceof foo()", None),
        ("Array.isArray([1,2,3]) === true", None),
        (r#""arr instanceof Array""#, None),
        ("foo instanceof WebWorker", None),
        // Error types
        ("foo instanceof Error", None),
        ("foo instanceof EvalError", None),
        ("foo instanceof RangeError", None),
        ("foo instanceof ReferenceError", None),
        ("foo instanceof SyntaxError", None),
        ("foo instanceof TypeError", None),
        ("foo instanceof URIError", None),
        ("foo instanceof InternalError", None),
        ("foo instanceof AggregateError", None),
        // Collection types
        ("foo instanceof Map", None),
        ("foo instanceof Set", None),
        ("foo instanceof WeakMap", None),
        ("foo instanceof WeakRef", None),
        ("foo instanceof WeakSet", None),
        // Arrays and Typed Arrays
        ("foo instanceof ArrayBuffer", None),
        ("foo instanceof Int8Array", None),
        ("foo instanceof Uint8Array", None),
        ("foo instanceof Uint8ClampedArray", None),
        ("foo instanceof Int16Array", None),
        ("foo instanceof Uint16Array", None),
        ("foo instanceof Int32Array", None),
        ("foo instanceof Uint32Array", None),
        ("foo instanceof Float16Array", None),
        ("foo instanceof Float32Array", None),
        ("foo instanceof Float64Array", None),
        ("foo instanceof BigInt64Array", None),
        ("foo instanceof BigUint64Array", None),
        // Data types
        ("foo instanceof Object", None),
        // Regular Expressions
        ("foo instanceof RegExp", None),
        // Async and functions
        ("foo instanceof Promise", None),
        ("foo instanceof Proxy", None),
        // Other
        ("foo instanceof DataView", None),
        ("foo instanceof Date", None),
        ("foo instanceof SharedArrayBuffer", None),
        ("foo instanceof FinalizationRegistry", None),
    ];

    let fail = vec![
        ("foo instanceof String", None),
        ("foo instanceof Number", None),
        ("foo instanceof Boolean", None),
        ("foo instanceof BigInt", None),
        ("foo instanceof Symbol", None),
        ("foo instanceof Function", None),
        ("foo instanceof Array", None),
        (
            "fooErr instanceof Error",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "loose"}])),
        ),
        (
            "(fooErr) instanceof (Error)",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof Error",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof EvalError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof RangeError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof ReferenceError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof SyntaxError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof TypeError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof URIError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof InternalError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        (
            "err instanceof AggregateError",
            Some(serde_json::json!([{"useErrorIsError": true, "strategy": "strict"}])),
        ),
        ("fooInclude instanceof WebWorker", Some(serde_json::json!([{"include": ["WebWorker"]}]))),
        (
            "fooInclude instanceof HTMLElement",
            Some(serde_json::json!([{"include": ["HTMLElement"]}])),
        ),
        ("arr instanceof Array", None),
        ("[] instanceof Array", None),
        ("[1,2,3] instanceof Array === true", None),
        ("fun.call(1, 2, 3) instanceof Array", None),
        ("obj.arr instanceof Array", None),
        ("foo.bar[2] instanceof Array", None),
        ("(0, array) instanceof Array", None),
        ("function foo(){return[]instanceof Array}", None),
        (
            "(
				// comment
				((
					// comment
					(
						// comment
						foo
						// comment
					)
					// comment
				))
				// comment
			)
			// comment before instanceof
            instanceof
			// comment after instanceof
			(
				// comment
				(
					// comment
					Array
					// comment
				)
					// comment
			)
				// comment",
            None,
        ),
    ];

    Tester::new(NoInstanceofBuiltins::NAME, NoInstanceofBuiltins::PLUGIN, pass, fail)
        .test_and_snapshot();
}
