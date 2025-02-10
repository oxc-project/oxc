use oxc_ast::{
    ast::{
        BindingPatternKind, ChainElement, Expression, FormalParameter, TSLiteral, TSType,
        TSTypeAnnotation, TSTypeName, UnaryOperator,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_inferrable_types_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Type can be trivially inferred from the initializer")
        .with_help("Remove the type annotation")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInferrableTypes {
    ignore_parameters: bool,
    ignore_properties: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow explicit type declarations for variables or parameters initialized to a number, string, or boolean
    ///
    /// ### Why is this bad?
    ///
    /// Explicitly typing variables or parameters that are initialized to a literal value is unnecessary because TypeScript can infer the type from the value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const a: number = 5;
    /// const b: string = 'foo';
    /// const c: boolean = true;
    /// const fn = (a: number = 5, b: boolean = true, c: string = 'foo') => {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const a = 5;
    /// const b = 'foo';
    /// const c = true;
    /// const fn = (a = 5, b = true, c = 'foo') => {};
    /// ```
    NoInferrableTypes,
    typescript,
    style,
    pending
);

impl Rule for NoInferrableTypes {
    fn from_configuration(value: serde_json::Value) -> Self {
        use serde_json::Value;
        let Some(config) = value.get(0).and_then(Value::as_object) else {
            return Self::default();
        };
        Self {
            ignore_parameters: config
                .get("ignoreParameters")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            ignore_properties: config
                .get("ignoreProperties")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(variable_decl) => {
                if let (Some(init), Some(type_annotation)) =
                    (&variable_decl.init, &variable_decl.id.type_annotation)
                {
                    if is_inferrable_type(type_annotation, init) {
                        ctx.diagnostic(no_inferrable_types_diagnostic(type_annotation.span()));
                    }
                }
            }
            AstKind::Function(function) => {
                self.check_formal_parameters(&function.params.items, ctx);
            }
            AstKind::ArrowFunctionExpression(arrow_function_expression) => {
                self.check_formal_parameters(&arrow_function_expression.params.items, ctx);
            }
            AstKind::PropertyDefinition(property_definition) => {
                // We ignore `readonly` because of Microsoft/TypeScript#14416
                // Essentially a readonly property without a type
                // will result in its value being the type, leading to
                // compile errors if the type is stripped.
                if self.ignore_properties
                    || property_definition.readonly
                    || property_definition.optional
                {
                    return;
                }
                if let (Some(init), Some(type_annotation)) =
                    (&property_definition.value, &property_definition.type_annotation)
                {
                    if is_inferrable_type(type_annotation, init) {
                        ctx.diagnostic(no_inferrable_types_diagnostic(type_annotation.span()));
                    }
                }
            }
            _ => {}
        }
    }
}

impl NoInferrableTypes {
    fn check_formal_parameters<'a>(
        &self,
        params: &oxc_allocator::Vec<'a, FormalParameter<'a>>,
        ctx: &LintContext<'a>,
    ) {
        if self.ignore_parameters {
            return;
        }

        for param in params {
            if let BindingPatternKind::AssignmentPattern(param_assignment_pat) = &param.pattern.kind
            {
                if let Some(type_annotation) = &param_assignment_pat.left.type_annotation {
                    if is_inferrable_type(type_annotation, &param_assignment_pat.right) {
                        ctx.diagnostic(no_inferrable_types_diagnostic(type_annotation.span()));
                    }
                }
            }
        }
    }
}

fn is_inferrable_type(type_annotation: &TSTypeAnnotation, init: &Expression) -> bool {
    match &type_annotation.type_annotation {
        TSType::TSLiteralType(ts_literal_type) => match &ts_literal_type.literal {
            TSLiteral::BooleanLiteral(_) => is_init_boolean(init),
            TSLiteral::NullLiteral(_) => is_init_null(init),
            TSLiteral::NumericLiteral(_) => is_init_number(init),
            TSLiteral::BigIntLiteral(_) => is_init_bigint(init),
            TSLiteral::StringLiteral(_) => is_init_string(init),
            TSLiteral::RegExpLiteral(_) => is_init_regexp(init),
            TSLiteral::TemplateLiteral(_) | TSLiteral::UnaryExpression(_) => false,
        },
        TSType::TSStringKeyword(_) => is_init_string(init),
        TSType::TSBigIntKeyword(_) => is_init_bigint(init),
        TSType::TSBooleanKeyword(_) => is_init_boolean(init),
        TSType::TSNumberKeyword(_) => is_init_number(init),
        TSType::TSNullKeyword(_) => is_init_null(init),
        TSType::TSSymbolKeyword(_) => {
            if is_chain_call_expression_with_name(init, "Symbol") {
                return true;
            }
            if let Expression::CallExpression(call_expr) = init.get_inner_expression() {
                call_expr.callee.get_identifier_reference().is_some_and(|id| id.name == "Symbol")
            } else {
                false
            }
        }
        TSType::TSTypeReference(type_reference) => {
            if let TSTypeName::IdentifierReference(ident) = &type_reference.type_name {
                if ident.name == "RegExp" {
                    return is_init_regexp(init);
                }
            }

            false
        }
        TSType::TSUndefinedKeyword(_) => match init.get_inner_expression() {
            Expression::Identifier(id) => id.name == "undefined",
            Expression::UnaryExpression(unary_expr) => {
                matches!(unary_expr.operator, UnaryOperator::Void)
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_chain_call_expression_with_name(init: &Expression, name: &str) -> bool {
    if let Expression::ChainExpression(chain_expr) = init {
        if let ChainElement::CallExpression(call_expr) = &chain_expr.expression {
            return call_expr.callee.get_identifier_reference().is_some_and(|id| id.name == name);
        }
    }
    false
}

fn is_init_bigint(init: &Expression) -> bool {
    let init = {
        let init = init.get_inner_expression();
        if let Expression::UnaryExpression(unary_expr) = init {
            if matches!(
                unary_expr.operator,
                UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) {
                unary_expr.argument.get_inner_expression()
            } else {
                init
            }
        } else {
            init
        }
    };

    if is_chain_call_expression_with_name(init, "BigInt") {
        return true;
    }

    match init {
        Expression::CallExpression(call_expr) => {
            call_expr.callee.get_identifier_reference().is_some_and(|id| id.name == "BigInt")
        }
        Expression::BigIntLiteral(_) => true,
        _ => false,
    }
}

fn is_init_boolean(init: &Expression) -> bool {
    if is_chain_call_expression_with_name(init, "Boolean") {
        return true;
    }
    match init.get_inner_expression() {
        Expression::UnaryExpression(unary_expr) => {
            matches!(unary_expr.operator, UnaryOperator::LogicalNot)
        }
        Expression::CallExpression(call_expr) => {
            call_expr.callee.get_identifier_reference().is_some_and(|id| id.name == "Boolean")
        }
        Expression::BooleanLiteral(_) => true,
        _ => false,
    }
}

fn is_init_null(init: &Expression) -> bool {
    let init = init.get_inner_expression();
    matches!(init, Expression::NullLiteral(_))
}

fn is_init_number(init: &Expression) -> bool {
    let init = {
        let init = init.get_inner_expression();
        if let Expression::UnaryExpression(unary_expr) = init {
            if matches!(
                unary_expr.operator,
                UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) {
                unary_expr.argument.get_inner_expression()
            } else {
                init
            }
        } else {
            init
        }
    };
    if is_chain_call_expression_with_name(init, "Number") {
        return true;
    }
    match init {
        Expression::Identifier(id) => id.name == "Infinity" || id.name == "NaN",
        Expression::CallExpression(call_expr) => {
            call_expr.callee.get_identifier_reference().is_some_and(|id| id.name == "Number")
        }
        Expression::NumericLiteral(_) => true,
        _ => false,
    }
}
fn is_init_string(init: &Expression) -> bool {
    if is_chain_call_expression_with_name(init, "String") {
        return true;
    }
    match init {
        Expression::CallExpression(call_expr) => {
            call_expr.callee.get_identifier_reference().is_some_and(|id| id.name == "String")
        }
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        _ => false,
    }
}
fn is_init_regexp(init: &Expression) -> bool {
    if is_chain_call_expression_with_name(init, "RegExp") {
        return true;
    }
    match init.get_inner_expression() {
        Expression::RegExpLiteral(_) => true,
        Expression::NewExpression(new_expr) => {
            if let Expression::Identifier(id) = new_expr.callee.get_inner_expression() {
                id.name == "RegExp"
            } else {
                false
            }
        }
        Expression::CallExpression(call_expr) => {
            if let Expression::Identifier(id) = call_expr.callee.get_inner_expression() {
                id.name == "RegExp"
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
        (r"const a = 10n", None),
        (r"const a = -10n", None),
        (r"const a = BigInt(10)", None),
        (r"const a = -BigInt(10)", None),
        (r"const a = BigInt?.(10)", None),
        (r"const a = -BigInt?.(10)", None),
        (r"const a = false", None),
        (r"const a = true", None),
        (r"const a = Boolean(null)", None),
        (r"const a = Boolean?.(null)", None),
        (r"const a = !0", None),
        (r"const a = 10", None),
        (r"const a = +10", None),
        (r"const a = -10", None),
        (r#"const a = Number("1")"#, None),
        (r#"const a = +Number("1")"#, None),
        (r#"const a = -Number("1")"#, None),
        (r#"const a = Number?.("1")"#, None),
        (r#"const a = +Number?.("1")"#, None),
        (r#"const a = -Number?.("1")"#, None),
        (r"const a = Infinity", None),
        (r"const a = +Infinity", None),
        (r"const a = -Infinity", None),
        (r"const a = NaN", None),
        (r"const a = +NaN", None),
        (r"const a = -NaN", None),
        (r"const a = null", None),
        (r"const a = /a/", None),
        (r#"const a = RegExp("a")"#, None),
        (r#"const a = RegExp?.("a")"#, None),
        (r#"const a = new RegExp("a")"#, None),
        (r#"const a = "str""#, None),
        (r"const a = 'str'", None),
        (r"const a = `str`", None),
        (r"const a = String(1)", None),
        (r"const a = String?.(1)", None),
        (r#"const a = Symbol("a")"#, None),
        (r#"const a = Symbol?.("a")"#, None),
        (r"const a = undefined", None),
        (r"const a = void someValue", None),
        ("const fn = (a = 5, b = true, c = 'foo') => {};", None),
        ("const fn = function (a = 5, b = true, c = 'foo') {};", None),
        ("function fn(a = 5, b = true, c = 'foo') {}", None),
        ("function fn(a: number, b: boolean, c: string) {}", None),
        (
            "
            class Foo {
              a = 5;
              b = true;
              c = 'foo';
            }",
            None,
        ),
        ("class Foo { readonly a: number = 5; }", None),
        ("const a: any = 5;", None),
        ("const fn = function (a: any = 5, b: any = true, c: any = 'foo') {};", None),
        (
            "const fn = (a: number = 5, b: boolean = true, c: string = 'foo') => {};",
            Some(serde_json::json!([{ "ignoreParameters": true }])),
        ),
        (
            "function fn(a: number = 5, b: boolean = true, c: string = 'foo') {}",
            Some(serde_json::json!([{ "ignoreParameters": true }])),
        ),
        (
            "const fn = function (a: number = 5, b: boolean = true, c: string = 'foo') {};",
            Some(serde_json::json!([{ "ignoreParameters": true }])),
        ),
        (
            "
            class Foo {
              a: number = 5;
              b: boolean = true;
              c: string = 'foo';
            }",
            Some(serde_json::json!([{ "ignoreProperties": true }])),
        ),
        (
            "
            class Foo {
              a?: number = 5;
              b?: boolean = true;
              c?: string = 'foo';
            }",
            None,
        ),
        ("class Foo { constructor(public a = true) {} }", None),
    ];

    let fail = vec![
        (r"const a: bigint = 10n", None),
        (r"const a: bigint = -10n", None),
        (r"const a: bigint = BigInt(10)", None),
        (r"const a: bigint = -BigInt(10)", None),
        (r"const a: bigint = BigInt?.(10)", None),
        (r"const a: bigint = -BigInt?.(10)", None),
        (r"const a: boolean = false", None),
        (r"const a: boolean = true", None),
        (r"const a: boolean = Boolean(null)", None),
        (r"const a: boolean = Boolean?.(null)", None),
        (r"const a: boolean = !0", None),
        (r"const a: number = 10", None),
        (r"const a: number = +10", None),
        (r"const a: number = -10", None),
        (r#"const a: number = Number("1")"#, None),
        (r#"const a: number = +Number("1")"#, None),
        (r#"const a: number = -Number("1")"#, None),
        (r#"const a: number = Number?.("1")"#, None),
        (r#"const a: number = +Number?.("1")"#, None),
        (r#"const a: number = -Number?.("1")"#, None),
        (r"const a: number = Infinity", None),
        (r"const a: number = +Infinity", None),
        (r"const a: number = -Infinity", None),
        (r"const a: number = NaN", None),
        (r"const a: number = +NaN", None),
        (r"const a: number = -NaN", None),
        (r"const a: null = null", None),
        (r"const a: RegExp = /a/", None),
        (r#"const a: RegExp = RegExp("a")"#, None),
        (r#"const a: RegExp = RegExp?.("a")"#, None),
        (r#"const a: RegExp = new RegExp("a")"#, None),
        (r#"const a: string = "str""#, None),
        (r"const a: string = 'str'", None),
        (r"const a: string = `str`", None),
        (r"const a: string = String(1)", None),
        (r"const a: string = String?.(1)", None),
        (r#"const a: symbol = Symbol("a")"#, None),
        (r#"const a: symbol = Symbol?.("a")"#, None),
        (r"const a: undefined = undefined", None),
        (r"const a: undefined = void someValue", None),
        (
            "const fn = (a?: number = 5) => {};",
            Some(serde_json::json!([{ "ignoreParameters": false }])),
        ),
        ("class A { a!: number = 1; }", Some(serde_json::json!([{ "ignoreProperties": false }]))),
        (
            "const fn = (a: number = 5, b: boolean = true, c: string = 'foo') => {};",
            Some(serde_json::json!([{ "ignoreParameters": false, "ignoreProperties": false }])),
        ),
        (
            "class Foo {
              a: number = 5;
              b: boolean = true;
              c: string = 'foo';
            }",
            Some(serde_json::json!([{ "ignoreParameters": false, "ignoreProperties": false }])),
        ),
        (
            "class Foo { constructor(public a: boolean = true) {} }",
            Some(serde_json::json!([{ "ignoreParameters": false, "ignoreProperties": false }])),
        ),
    ];

    let _fix = vec![
        (
            "const fn = (a?: number = 5) => {};",
            "const fn = (a = 5) => {};",
            Some(serde_json::json!([{ "ignoreParameters": false,        },      ])),
        ),
        (
            "
            class A {
              a!: number = 1;
            }
                  ",
            "
            class A {
              a = 1;
            }
                  ",
            Some(serde_json::json!([{ "ignoreProperties": false,        },      ])),
        ),
        (
            "const fn = (a: number = 5, b: boolean = true, c: string = 'foo') => {};",
            "const fn = (a = 5, b = true, c = 'foo') => {};",
            Some(
                serde_json::json!([{ "ignoreParameters": false,          "ignoreProperties": false,        },      ]),
            ),
        ),
        (
            "
            class Foo {
              a: number = 5;
              b: boolean = true;
              c: string = 'foo';
            }",
            "class Foo {
              a = 5;
              b = true;
              c = 'foo';
            }",
            Some(
                serde_json::json!([{ "ignoreParameters": false,          "ignoreProperties": false,        },      ]),
            ),
        ),
        (
            "class Foo { constructor(public a: boolean = true) {} }",
            "class Foo { constructor(public a = true) {} } ",
            Some(serde_json::json!([{ "ignoreParameters": false, "ignoreProperties": false, }, ])),
        ),
    ];
    Tester::new(NoInferrableTypes::NAME, NoInferrableTypes::PLUGIN, pass, fail)
        //.expect_fix(fix)
        .test_and_snapshot();
}
