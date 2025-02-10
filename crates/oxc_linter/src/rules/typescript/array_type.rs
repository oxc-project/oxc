use oxc_ast::{
    ast::{TSType, TSTypeName, TSTypeOperatorOperator, TSTypeReference},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
};

#[derive(Debug, Default, Clone)]
pub struct ArrayType(Box<ArrayTypeConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Require consistently using either `T[]` or `Array<T>` for arrays.
    ///
    /// ### Why is this bad?
    /// Using the `Array` type directly is not idiomatic. Instead, use the array type `T[]` or `Array<T>`.
    ///
    /// ### Example
    /// ```typescript
    /// const arr: Array<number> = new Array<number>();
    /// const arr: number[] = new Array<number>();
    /// ```
    ArrayType,
    typescript,
    style,
    fix
);

fn generic(readonly_prefix: &str, name: &str, type_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Array type using '{readonly_prefix}{type_name}[]' is forbidden. Use '{name}<{type_name}>' instead."
    ))
    .with_label(span)
}

fn generic_simple(readonly_prefix: &str, name: &str, type_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Array type using '{readonly_prefix}{type_name}[]' is forbidden for non-simple types. Use '{name}<{type_name}>' instead."
    ))
    .with_label(span)
}

fn array(readonly_prefix: &str, type_name: &str, generic_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Array type using '{type_name}<{generic_name}>' is forbidden. Use '{readonly_prefix}{generic_name}[]' instead."
    ))
    .with_label(span)
}

fn array_simple(
    readonly_prefix: &str,
    type_name: &str,
    generic_name: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Array type using '{type_name}<{generic_name}>' is forbidden for simple types. Use '{readonly_prefix}{generic_name}[]' instead."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ArrayTypeConfig {
    // The array type expected for mutable cases.
    default: ArrayOption,
    // The array type expected for readonly cases. If omitted, the value for `default` will be used.
    readonly: Option<ArrayOption>,
}

impl std::ops::Deref for ArrayType {
    type Target = ArrayTypeConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Debug, Default, Clone)]
pub enum ArrayOption {
    #[default]
    Array,
    ArraySimple,
    Generic,
}

impl Rule for ArrayType {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(ArrayTypeConfig {
            default: value
                .get(0)
                .and_then(|v| v.get("default"))
                .and_then(serde_json::Value::as_str)
                .map_or_else(
                    || ArrayOption::Array,
                    |s| match s {
                        "array" => ArrayOption::Array,
                        "generic" => ArrayOption::Generic,
                        _ => ArrayOption::ArraySimple,
                    },
                ),
            readonly: value
                .get(0)
                .and_then(|v| v.get("readonly"))
                .and_then(serde_json::Value::as_str)
                .map_or_else(
                    || None,
                    |s| match s {
                        "array" => Some(ArrayOption::Array),
                        "generic" => Some(ArrayOption::Generic),
                        _ => Some(ArrayOption::ArraySimple),
                    },
                ),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let default_config = &self.default;
        let readonly_config: &ArrayOption =
            &self.readonly.clone().unwrap_or_else(|| default_config.clone());

        match node.kind() {
            AstKind::TSTypeAnnotation(ts_type_annotation) => {
                check(&ts_type_annotation.type_annotation, default_config, readonly_config, ctx);
            }
            // for example: type barUnion = (string | number | boolean)[];
            AstKind::TSTypeAliasDeclaration(ts_alias_annotation) => {
                check(&ts_alias_annotation.type_annotation, default_config, readonly_config, ctx);
            }
            // for example: let ya = [[1, '2']] as [number, string][];
            AstKind::TSAsExpression(ts_as_expression) => {
                check(&ts_as_expression.type_annotation, default_config, readonly_config, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn check(
    type_annotation: &TSType,
    default_config: &ArrayOption,
    readonly_config: &ArrayOption,
    ctx: &LintContext,
) {
    if let TSType::TSArrayType(array_type) = &type_annotation {
        check_and_report_error_generic(
            default_config,
            array_type.span,
            &array_type.element_type,
            ctx,
            false,
        );
    }

    if let TSType::TSTypeOperatorType(ts_operator_type) = &type_annotation {
        if matches!(&ts_operator_type.operator, TSTypeOperatorOperator::Readonly) {
            if let TSType::TSArrayType(array_type) = &ts_operator_type.type_annotation {
                check_and_report_error_generic(
                    readonly_config,
                    ts_operator_type.span,
                    &array_type.element_type,
                    ctx,
                    true,
                );
            }
        }
    }

    if let TSType::TSTypeReference(ts_type_reference) = &type_annotation {
        check_and_report_error_reference(default_config, readonly_config, ts_type_reference, ctx);
    }
}

fn type_needs_parentheses(type_param: &TSType) -> bool {
    match type_param {
        TSType::TSTypeReference(node) => {
            if let TSTypeName::IdentifierReference(identifier_reference) = &node.type_name {
                return identifier_reference.name.as_str() == "ReadonlyArray";
            }
            true
        }
        TSType::TSUnionType(_)
        | TSType::TSFunctionType(_)
        | TSType::TSIntersectionType(_)
        | TSType::TSTypeOperatorType(_)
        | TSType::TSInferType(_)
        | TSType::TSConstructorType(_) => true,
        _ => false,
    }
}

fn check_and_report_error_generic(
    config: &ArrayOption,
    type_reference_span: Span,
    type_param: &TSType,
    ctx: &LintContext,
    is_readonly: bool,
) {
    if matches!(config, ArrayOption::Array) {
        return;
    }
    let type_param = type_param.without_parenthesized();
    if matches!(config, ArrayOption::ArraySimple) && is_simple_type(type_param) {
        return;
    }
    let source_text = ctx.source_text().to_string();

    let readonly_prefix = if is_readonly { "readonly " } else { "" };
    let class_name = if is_readonly { "ReadonlyArray" } else { "Array" };
    let message_type = get_message_type(type_param, &source_text);

    let diagnostic = match config {
        ArrayOption::Generic => {
            generic(readonly_prefix, class_name, message_type, type_reference_span)
        }
        _ => generic_simple(readonly_prefix, class_name, message_type, type_reference_span),
    };
    let element_type_span = get_ts_element_type_span(type_param);
    let Some(element_type_span) = element_type_span else {
        return;
    };

    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        let type_text =
            &source_text[element_type_span.start as usize..element_type_span.end as usize];
        let array_type_identifier = if is_readonly { "ReadonlyArray" } else { "Array" };

        fixer.replace(type_reference_span, format!("{array_type_identifier}<{type_text}>"))
    });
}

fn check_and_report_error_reference(
    default_config: &ArrayOption,
    readonly_config: &ArrayOption,
    ts_type_reference: &TSTypeReference,
    ctx: &LintContext,
) {
    if let TSTypeName::IdentifierReference(ident_ref_type_name) = &ts_type_reference.type_name {
        if ident_ref_type_name.name.as_str() == "ReadonlyArray"
            || ident_ref_type_name.name.as_str() == "Array"
        {
            check_and_report_error_array(default_config, readonly_config, ts_type_reference, ctx);
        } else if ident_ref_type_name.name.as_str() == "Promise" {
            if let Some(type_params) = &ts_type_reference.type_parameters {
                if type_params.params.len() == 1 {
                    if let Some(type_param) = type_params.params.first() {
                        if let TSType::TSArrayType(array_type) = &type_param {
                            check_and_report_error_generic(
                                default_config,
                                array_type.span,
                                &array_type.element_type,
                                ctx,
                                false,
                            );
                        }

                        if let TSType::TSTypeOperatorType(ts_operator_type) = &type_param {
                            if matches!(
                                &ts_operator_type.operator,
                                TSTypeOperatorOperator::Readonly
                            ) {
                                if let TSType::TSArrayType(array_type) =
                                    &ts_operator_type.type_annotation
                                {
                                    check_and_report_error_generic(
                                        readonly_config,
                                        ts_operator_type.span,
                                        &array_type.element_type,
                                        ctx,
                                        true,
                                    );
                                }
                            }
                        }

                        if let TSType::TSTypeReference(ts_type_reference) = &type_param {
                            check_and_report_error_reference(
                                default_config,
                                readonly_config,
                                ts_type_reference,
                                ctx,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn check_and_report_error_array(
    default_config: &ArrayOption,
    readonly_config: &ArrayOption,
    ts_type_reference: &TSTypeReference,
    ctx: &LintContext,
) {
    let TSTypeName::IdentifierReference(ident_ref_type_name) = &ts_type_reference.type_name else {
        return;
    };

    let is_readonly_array_type = ident_ref_type_name.name == "ReadonlyArray";
    let config = if is_readonly_array_type { readonly_config } else { default_config };
    if matches!(config, ArrayOption::Generic) {
        return;
    }
    let readonly_prefix: &'static str = if is_readonly_array_type { "readonly " } else { "" };
    let class_name = if is_readonly_array_type { "ReadonlyArray" } else { "Array" };
    let type_params = &ts_type_reference.type_parameters;

    if type_params.is_none() || type_params.as_ref().unwrap().params.len() == 0 {
        let diagnostic = match config {
            ArrayOption::Array => array(readonly_prefix, class_name, "any", ts_type_reference.span),
            _ => array_simple(
                readonly_prefix,
                &ident_ref_type_name.name,
                "any",
                ts_type_reference.span,
            ),
        };
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            fixer.replace(ts_type_reference.span, readonly_prefix.to_string() + "any[]")
        });
        return;
    }
    if type_params.as_ref().unwrap().params.len() != 1 {
        return;
    }
    let first_type_param = type_params.as_ref().unwrap().params.first().unwrap();
    if matches!(config, ArrayOption::ArraySimple) && !is_simple_type(first_type_param) {
        return;
    }

    let type_parens = type_needs_parentheses(first_type_param);
    // TODO: support example: type Foo = ReadonlyArray<object>[]; -> type Foo = (readonly object[])[];
    // let mut parent_parens: bool = readonly_prefix != "";
    // if let Some(parent) = ctx.nodes().parent_node(node.id()) {
    //     if let AstKind::TSTypeAnnotation(parent_node) = parent.kind() {}
    // } else {
    //     parent_parens = false
    // };
    let parent_parens = false;

    let element_type_span = get_ts_element_type_span(first_type_param);
    let Some(element_type_span) = element_type_span else {
        return;
    };

    let message_type = get_message_type(first_type_param, ctx.source_text());
    let diagnostic = match config {
        ArrayOption::Array => {
            array(readonly_prefix, class_name, message_type, ts_type_reference.span)
        }
        _ => array_simple(
            readonly_prefix,
            &ident_ref_type_name.name,
            message_type,
            ts_type_reference.span,
        ),
    };
    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        let mut start = String::from(if parent_parens { "(" } else { "" });
        start.push_str(readonly_prefix);
        start.push_str(if type_parens { "(" } else { "" });

        let mut end = String::from(if type_parens { ")" } else { "" });
        end.push_str("[]");
        end.push_str(if parent_parens { ")" } else { "" });

        let type_text = fixer.source_range(element_type_span);
        fixer.replace(ts_type_reference.span, start + type_text + end.as_str())
    });
}

// Check whatever node can be considered as simple type
fn is_simple_type(ts_type: &TSType) -> bool {
    match ts_type {
        TSType::TSAnyKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSNeverKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSBigIntKeyword(_)
        | TSType::TSObjectKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSUnknownKeyword(_)
        | TSType::TSVoidKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSArrayType(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSQualifiedName(_)
        | TSType::TSThisType(_) => true,
        TSType::TSTypeReference(node) => {
            let type_name = TSTypeName::get_identifier_reference(&node.type_name);
            if type_name.name.as_str() == "Array" {
                if node.type_parameters.is_none() {
                    return true;
                }
                if node.type_parameters.as_ref().unwrap().params.len() == 1 {
                    return is_simple_type(
                        node.type_parameters.as_ref().unwrap().params.first().unwrap(),
                    );
                }
            } else {
                if node.type_parameters.is_some() {
                    return false;
                }
                if let TSTypeName::IdentifierReference(_) = &node.type_name {
                    return true;
                }
                return false;
            }
            false
        }
        _ => false,
    }
}

// Get the type name from the type node. for example: `Array<string>` -> `string`
fn get_message_type<'a>(type_param: &'a TSType, source_text: &'a str) -> &'a str {
    if is_simple_type(type_param) {
        let element_type_span = get_ts_element_type_span(type_param);
        let Some(element_type_span) = element_type_span else {
            return "T";
        };
        return &source_text[element_type_span.start as usize..element_type_span.end as usize];
    }
    "T"
}

fn get_ts_element_type_span(ts_type: &TSType) -> Option<Span> {
    match ts_type {
        TSType::TSAnyKeyword(t) => Some(t.span),
        TSType::TSNumberKeyword(t) => Some(t.span),
        TSType::TSStringKeyword(t) => Some(t.span),
        TSType::TSBigIntKeyword(t) => Some(t.span),
        TSType::TSBooleanKeyword(t) => Some(t.span),
        TSType::TSNeverKeyword(t) => Some(t.span),
        TSType::TSObjectKeyword(t) => Some(t.span),
        TSType::TSSymbolKeyword(t) => Some(t.span),
        TSType::TSUnknownKeyword(t) => Some(t.span),
        TSType::TSVoidKeyword(t) => Some(t.span),
        TSType::TSNullKeyword(t) => Some(t.span),
        TSType::TSThisType(t) => Some(t.span),
        TSType::TSUndefinedKeyword(t) => Some(t.span),

        TSType::TSArrayType(t) => Some(t.span),
        TSType::TSConditionalType(t) => Some(t.span),
        TSType::TSConstructorType(t) => Some(t.span),
        TSType::TSFunctionType(t) => Some(t.span),
        TSType::TSImportType(t) => Some(t.span),
        TSType::TSIndexedAccessType(t) => Some(t.span),
        TSType::TSInferType(t) => Some(t.span),
        TSType::TSIntersectionType(t) => Some(t.span),
        TSType::TSLiteralType(t) => Some(t.span),
        TSType::TSMappedType(t) => Some(t.span),
        TSType::TSNamedTupleMember(t) => Some(t.span),
        TSType::TSQualifiedName(t) => Some(t.span),
        TSType::TSTemplateLiteralType(t) => Some(t.span),
        TSType::TSTupleType(t) => Some(t.span),
        TSType::TSTypeLiteral(t) => Some(t.span),
        TSType::TSTypeOperatorType(t) => Some(t.span),
        TSType::TSTypePredicate(t) => Some(t.span),
        TSType::TSTypeQuery(t) => Some(t.span),
        TSType::TSTypeReference(t) => Some(t.span),
        TSType::TSUnionType(t) => Some(t.span),

        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
        ("let a: number[] = [];", Some(serde_json::json!([{"default":"array"}]))),
        ("let a: (string | number)[] = [];", Some(serde_json::json!([{"default":"array"}]))),
        ("let a: readonly number[] = [];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        ("let a: number[] = [];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        ("let a: readonly number[] = [];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        ("let a: Array<number> = [];", Some(serde_json::json!([{"default":"generic"}]))),
        ("let a: Array<string | number> = [];", Some(serde_json::json!([{"default":"generic"}]))),
        ("let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: readonly bigint[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: readonly (string | bigint)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: Array<bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<string | bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly bigint[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<string | bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        ("let a = new Array();", Some(serde_json::json!([{"default":"array"}]))),
        ("let a: { foo: Bar[] }[] = [];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "function foo(a: Array<Bar>): Array<Bar> {}",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let yy: number[][] = [[4, 5], [6]];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function fooFunction(foo: Array<ArrayClass<string>>) {
            return foo.map(e => e.foo);
        }",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "
        function bazFunction(baz: Arr<ArrayClass<String>>) {
        return baz.map(e => e.baz);
        }
            ",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let fooVar: Array<(c: number) => number>;",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "type fooUnion = Array<string | number | boolean>;",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "type fooIntersection = Array<string & number>;",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "
        namespace fooName {
        type BarType = { bar: string };
        type BazType<T> = Arr<T>;
        }
            ",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "
        interface FooInterface {
        '.bar': { baz: string[] };
        }
            ",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        ("let yy: number[][] = [[4, 5], [6]];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "let ya = [[1, '2']] as [number, string][];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "
        function barFunction(bar: ArrayClass<String>[]) {
        return bar.map(e => e.bar);
        }
            ",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function bazFunction(baz: Arr<ArrayClass<String>>) {
        return baz.map(e => e.baz);
        }",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        ("let barVar: ((c: number) => number)[];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "type barUnion = (string | number | boolean)[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type barIntersection = (string & number)[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "
        interface FooInterface {
        '.bar': { baz: string[] };
        }",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type Unwrap<T> = T extends (infer E)[] ? E : T;",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let xx: Array<Array<number>> = [[1, 2], [3]];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        ("type Arr<T> = Array<T>;", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "function fooFunction(foo: Array<ArrayClass<string>>) { return foo.map(e => e.foo); }",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function bazFunction(baz: Arr<ArrayClass<String>>) { return baz.map(e => e.baz) }",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let fooVar: Array<(c: number) => number>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type fooUnion = Array<string | number | boolean>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type fooIntersection = Array<string & number>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type Unwrap<T> = T extends Array<infer E> ? E : T;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let a: ReadonlyArray<number[]> = [[]];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: readonly Array<number>[] = [[]];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
    ];

    let fail = vec![
        ("let a: Array<number> = [];", Some(serde_json::json!([{"default":"array"}]))),
        ("let a: Array<string | number> = [];", Some(serde_json::json!([{"default":"array"}]))),
        ("let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        ("let a: Array<number> = [];", Some(serde_json::json!([{"default":"array-simple"}]))),
        ("let a: (string | number)[] = [];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        ("let a: number[] = [];", Some(serde_json::json!([{"default":"generic"}]))),
        ("let a: (string | number)[] = [];", Some(serde_json::json!([{"default":"generic"}]))),
        ("let a: readonly number[] = [];", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: bigint[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | bigint)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | bigint)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly bigint[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | bigint)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        ("let a: { foo: Array<Bar> }[] = [];", Some(serde_json::json!([{"default":"array"}]))),
        ("let a: Array<{ foo: Bar[] }> = [];", Some(serde_json::json!([{"default":"generic"}]))),
        // ("let a: Array<{ foo: Foo | Bar[] }> = [];", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "function foo(a: Array<Bar>): Array<Bar> {}",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let x: Array<undefined> = [undefined] as undefined[];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let y: string[] = <Array<string>>['2'];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        ("let z: Array = [3, '4'];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let ya = [[1, '2']] as [number, string][];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        ("type Arr<T> = Array<T>;", Some(serde_json::json!([{"default":"array-simple"}]))),
        // ("
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Arr<string>>[]> = [[[['2']]]];
        //     ", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "
        interface ArrayClass<T> {
        foo: Array<T>;
        bar: T[];
        baz: Arr<T>;
        xyz: this[];
        }
            ",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "
        function barFunction(bar: ArrayClass<String>[]) {
        return bar.map(e => e.bar);
        }
            ",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let barVar: ((c: number) => number)[];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "type barUnion = (string | number | boolean)[];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "type barIntersection = (string & number)[];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        // ("let v: Array<fooName.BarType> = [{ bar: 'bar' }];", Some(serde_json::json!([{"default":"array-simple"}]))),
        // ("let w: fooName.BazType<string>[] = [['baz']];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let x: Array<undefined> = [undefined] as undefined[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        ("let y: string[] = <Array<string>>['2'];", Some(serde_json::json!([{"default":"array"}]))),
        ("let z: Array = [3, '4'];", Some(serde_json::json!([{"default":"array"}]))),
        ("type Arr<T> = Array<T>;", Some(serde_json::json!([{"default":"array"}]))),
        // ("
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Arr<string>>[]> = [[[['2']]]];
        //     ", Some(serde_json::json!([{"default":"array"}]))),
        (
            "
        interface ArrayClass<T> {
        foo: Array<T>;
        bar: T[];
        baz: Arr<T>;
        }
            ",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "
        function fooFunction(foo: Array<ArrayClass<string>>) {
        return foo.map(e => e.foo);
        }
            ",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let fooVar: Array<(c: number) => number>;",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type fooUnion = Array<string | number | boolean>;",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type fooIntersection = Array<string & number>;",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        ("let x: Array;", Some(serde_json::json!([{"default":"array"}]))),
        ("let x: Array<>;", Some(serde_json::json!([{"default":"array"}]))),
        ("let x: Array;", Some(serde_json::json!([{"default":"array-simple"}]))),
        ("let x: Array<>;", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let x: Array<number> = [1] as number[];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let y: string[] = <Array<string>>['2'];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let ya = [[1, '2']] as [number, string][];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        // ("
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Arr<string>>[]> = [[[['2']]]];
        //     ", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "
        interface ArrayClass<T> {
        foo: Array<T>;
        bar: T[];
        baz: Arr<T>;
        }
            ",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "
        function barFunction(bar: ArrayClass<String>[]) {
        return bar.map(e => e.bar);
        }
            ",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let barVar: ((c: number) => number)[];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type barUnion = (string | number | boolean)[];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type barIntersection = (string & number)[];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "
        interface FooInterface {
        '.bar': { baz: string[] };
        }
            ",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        // ("type Unwrap<T> = T extends Array<infer E> ? E : T;", Some(serde_json::json!([{"default":"array"}]))),
        // ("type Unwrap<T> = T extends (infer E)[] ? E : T;", Some(serde_json::json!([{"default":"generic"}]))),
        // ("type Foo = ReadonlyArray<object>[];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "const foo: Array<new (...args: any[]) => void> = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "const foo: ReadonlyArray<new (...args: any[]) => void> = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: Promise<string[]> = Promise.resolve([]);",
            Some(serde_json::json!([{"default": "generic"}])),
        ),
    ];

    let fix: Vec<(&str, &str, Option<serde_json::Value>)> = vec![
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array"}])),
        ),
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: Array<string | number> = [];",
            "let a: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: readonly number[] = [];",
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array","readonly":"generic"}])),
        ),
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array-simple"}])),
        ),
        (
            "let a: Array<number> = [];",
            "let a: number[] = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: readonly number[] = [];",
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "let a: number[] = [];",
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let a: readonly number[] = [];",
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let a: number[] = [];",
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: ReadonlyArray<string | number> = [];",
            "let a: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array"}])),
        ),
        (
            "let a: number[] = [];",
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<number> = [];",
            "let a: readonly number[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: number[] = [];",
            "let a: Array<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: (string | number)[] = [];",
            "let a: Array<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly number[] = [];",
            "let a: ReadonlyArray<number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | number)[] = [];",
            "let a: ReadonlyArray<string | number> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: bigint[] = [];",
            "let a: Array<bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | bigint)[] = [];",
            "let a: Array<string | bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: ReadonlyArray<bigint> = [];",
            "let a: readonly bigint[] = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"array-simple"}])),
        ),
        (
            "let a: (string | bigint)[] = [];",
            "let a: Array<string | bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly bigint[] = [];",
            "let a: ReadonlyArray<bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: readonly (string | bigint)[] = [];",
            "let a: ReadonlyArray<string | bigint> = [];",
            Some(serde_json::json!([{"default":"generic","readonly":"generic"}])),
        ),
        (
            "let a: { foo: Array<Bar> }[] = [];",
            "let a: { foo: Bar[] }[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: Array<{ foo: Bar[] }> = [];",
            "let a: Array<{ foo: Array<Bar> }> = [];",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        // ("let a: Array<{ foo: Foo | Bar[] }> = [];", "let a: Array<{ foo: Foo | Array<Bar> }> = [];", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "function foo(a: Array<Bar>): Array<Bar> {}",
            "function foo(a: Bar[]): Bar[] {}",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let x: Array<undefined> = [undefined] as undefined[];",
            "let x: undefined[] = [undefined] as undefined[];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        // ("let y: string[] = <Array<string>>['2'];", "let y: string[] = <string[]>['2'];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let z: Array = [3, '4'];",
            "let z: any[] = [3, '4'];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let ya = [[1, '2']] as [number, string][];",
            "let ya = [[1, '2']] as Array<[number, string]>;",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "type Arr<T> = Array<T>;",
            "type Arr<T> = T[];",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        // ("
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Arr<string>>[]> = [[[['2']]]];
        //     ", "
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Array<Arr<string>>>> = [[[['2']]]];
        //     ", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "
        interface ArrayClass<T> {
        foo: Array<T>;
        bar: T[];
        baz: Arr<T>;
        xyz: this[];
        }
            ",
            "
        interface ArrayClass<T> {
        foo: T[];
        bar: T[];
        baz: Arr<T>;
        xyz: this[];
        }
            ",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "
        function barFunction(bar: ArrayClass<String>[]) {
        return bar.map(e => e.bar);
        }
            ",
            "
        function barFunction(bar: Array<ArrayClass<String>>) {
        return bar.map(e => e.bar);
        }
            ",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let barVar: ((c: number) => number)[];",
            "let barVar: Array<(c: number) => number>;",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "type barUnion = (string | number | boolean)[];",
            "type barUnion = Array<string | number | boolean>;",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "type barIntersection = (string & number)[];",
            "type barIntersection = Array<string & number>;",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        // ("let v: Array<fooName.BarType> = [{ bar: 'bar' }];", "let v: fooName.BarType[] = [{ bar: 'bar' }];", Some(serde_json::json!([{"default":"array-simple"}]))),
        // ("let w: fooName.BazType<string>[] = [['baz']];", "let w: Array<fooName.BazType<string>> = [['baz']];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let x: Array<undefined> = [undefined] as undefined[];",
            "let x: undefined[] = [undefined] as undefined[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // ("let y: string[] = <Array<string>>['2'];", "let y: string[] = <string[]>['2'];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "let z: Array = [3, '4'];",
            "let z: any[] = [3, '4'];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type Arr<T> = Array<T>;",
            "type Arr<T> = T[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // ("
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Arr<string>>[]> = [[[['2']]]];
        //     ", "
        // // Ignore user defined aliases
        // let yyyy: Arr<Arr<string>[][]> = [[[['2']]]];
        //     ", Some(serde_json::json!([{"default":"array"}]))),
        (
            "
        interface ArrayClass<T> {
        foo: Array<T>;
        bar: T[];
        baz: Arr<T>;
        }
            ",
            "
        interface ArrayClass<T> {
        foo: T[];
        bar: T[];
        baz: Arr<T>;
        }
            ",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "
        function fooFunction(foo: Array<ArrayClass<string>>) {
        return foo.map(e => e.foo);
        }
            ",
            "
        function fooFunction(foo: ArrayClass<string>[]) {
        return foo.map(e => e.foo);
        }
            ",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let fooVar: Array<(c: number) => number>;",
            "let fooVar: ((c: number) => number)[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type fooUnion = Array<string | number | boolean>;",
            "type fooUnion = (string | number | boolean)[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type fooIntersection = Array<string & number>;",
            "type fooIntersection = (string & number)[];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        ("let x: Array;", "let x: any[];", Some(serde_json::json!([{"default":"array"}]))),
        ("let x: Array<>;", "let x: any[];", Some(serde_json::json!([{"default":"array"}]))),
        ("let x: Array;", "let x: any[];", Some(serde_json::json!([{"default":"array-simple"}]))),
        ("let x: Array<>;", "let x: any[];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let x: Array<number> = [1] as number[];",
            "let x: Array<number> = [1] as Array<number>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        // ("let y: string[] = <Array<string>>['2'];", "let y: Array<string> = <Array<string>>['2'];", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "let ya = [[1, '2']] as [number, string][];",
            "let ya = [[1, '2']] as Array<[number, string]>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        // ("
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Arr<string>>[]> = [[[['2']]]];
        //     ", "
        // // Ignore user defined aliases
        // let yyyy: Arr<Array<Array<Arr<string>>>> = [[[['2']]]];
        //     ", Some(serde_json::json!([{"default":"generic"}]))),
        (
            "
        interface ArrayClass<T> {
        foo: Array<T>;
        bar: T[];
        baz: Arr<T>;
        }
            ",
            "
        interface ArrayClass<T> {
        foo: Array<T>;
        bar: Array<T>;
        baz: Arr<T>;
        }
            ",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "
        function barFunction(bar: ArrayClass<String>[]) {
        return bar.map(e => e.bar);
        }
            ",
            "
        function barFunction(bar: Array<ArrayClass<String>>) {
        return bar.map(e => e.bar);
        }
            ",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "let barVar: ((c: number) => number)[];",
            "let barVar: Array<(c: number) => number>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type barUnion = (string | number | boolean)[];",
            "type barUnion = Array<string | number | boolean>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type barIntersection = (string & number)[];",
            "type barIntersection = Array<string & number>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "
        interface FooInterface {
        '.bar': { baz: string[] };
        }
            ",
            "
        interface FooInterface {
        '.bar': { baz: Array<string> };
        }
            ",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        // ("type Unwrap<T> = T extends Array<infer E> ? E : T;", "type Unwrap<T> = T extends (infer E)[] ? E : T;", Some(serde_json::json!([{"default":"array"}]))),
        // ("type Unwrap<T> = T extends (infer E)[] ? E : T;", "type Unwrap<T> = T extends Array<infer E> ? E : T;", Some(serde_json::json!([{"default":"generic"}]))),
        // ("type Foo = ReadonlyArray<object>[];", "type Foo = (readonly object[])[];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "const foo: Array<new (...args: any[]) => void> = [];",
            "const foo: (new (...args: any[]) => void)[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "const foo: ReadonlyArray<new (...args: any[]) => void> = [];",
            "const foo: readonly (new (...args: any[]) => void)[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let a: Promise<string[]> = Promise.resolve([]);",
            "let a: Promise<Array<string>> = Promise.resolve([]);",
            Some(serde_json::json!([{"default": "generic"}])),
        ),
    ];

    Tester::new(ArrayType::NAME, ArrayType::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
