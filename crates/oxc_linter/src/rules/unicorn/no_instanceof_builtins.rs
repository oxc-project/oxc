use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;
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

/// ECMAScript built-ins (2024-05 edition).
const DEFAULT_BUILTINS: &[&str] = &[
    "Array",
    "ArrayBuffer",
    "Boolean",
    "DataView",
    "Date",
    "Error",
    "EvalError",
    "FinalizationRegistry",
    "Float32Array",
    "Float64Array",
    "Function",
    "Int8Array",
    "Int16Array",
    "Int32Array",
    "Map",
    "Number",
    "Object",
    "Promise",
    "RangeError",
    "ReferenceError",
    "RegExp",
    "Set",
    "SharedArrayBuffer",
    "String",
    "Symbol",
    "SyntaxError",
    "TypeError",
    "Uint8Array",
    "Uint8ClampedArray",
    "Uint16Array",
    "Uint32Array",
    "URIError",
    "WeakMap",
    "WeakRef",
    "WeakSet",
];

#[derive(Debug, Clone, Default)]
pub struct NoInstanceofBuiltinsConfig {
    include: Vec<String>,
    exclude: Vec<String>,
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
    pending
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

        let cfg = &self.0;

        if cfg.exclude.iter().any(|s| s == ctor_name) {
            return;
        }

        let is_forbidden_default = DEFAULT_BUILTINS.contains(&ctor_name);
        let is_forbidden_extra = cfg.include.iter().any(|s| s == ctor_name);

        if is_forbidden_default || is_forbidden_extra {
            ctx.diagnostic(no_instanceof_builtins_diagnostic(bin.span));
        }
    }

    fn from_configuration(value: Value) -> Self {
        let mut include = Vec::<String>::new();
        let mut exclude = Vec::<String>::new();

        if let Value::Array(arr) = value {
            for item in arr {
                if let Value::Object(map) = item {
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
                }
            }
        }

        Self(Box::new(NoInstanceofBuiltinsConfig { include, exclude }))
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
    ];

    let fail = vec![
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
