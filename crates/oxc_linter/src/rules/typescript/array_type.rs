use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_ast::{ast::{TSType, TSTypeName, TSTypeOperator, TSTypeReference}, AstKind};
use oxc_semantic::AstNode;
use oxc_span::Span;
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, fixer::Fix, rule::Rule};

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
    style,
);

#[derive(Debug, Diagnostic, Error)]
pub enum ArrayTypeDiagnostic {
    #[error("Array type using '{0}{2}[]' is forbidden. Use '{1}<{2}>' instead.")]
    #[diagnostic(severity(warning))]
    // readonlyPrefix className type
    ErrorStringGeneric(String, String, String, #[label] Span),

    #[error("Array type using '{0}{2}[]' is forbidden for non-simple types. Use '{1}<{2}>' instead.")]
    #[diagnostic(severity(warning))]
    // readonlyPrefix className type
    ErrorStringGenericSimple(String, String, String, #[label] Span),

    #[error("Array type using '{1}<{2}>' is forbidden. Use '{0}{2}[]' instead.")]
    #[diagnostic(severity(warning))]
    // readonlyPrefix className type
    ErrorStringArray(String, String, String, #[label] Span),

    #[error("Array type using '{1}<{2}>' is forbidden for simple types. Use '{0}{2}[]' instead.")]
    #[diagnostic(severity(warning))]
    // readonlyPrefix className type
    ErrorStringArraySimple(String, String, String, #[label] Span),
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
            .map_or_else(|| ArrayOption::Array, |s| match s {
                "array" => ArrayOption::Array,
                "generic" => ArrayOption::Generic,
                _ => ArrayOption::ArraySimple,
            }),
            readonly: value
            .get(0)
            .and_then(|v| v.get("readonly"))
            .and_then(serde_json::Value::as_str)
            .map_or_else(|| None, |s| match s {
                "array" => Some(ArrayOption::Array),
                "generic" => Some(ArrayOption::Generic),
                _ => Some(ArrayOption::ArraySimple),
            }),
        }))
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let default_config = &self.default;
        let readonly_config: &ArrayOption = &self.readonly.clone().unwrap_or(default_config.clone());

        match node.kind() {
            AstKind::TSTypeAnnotation(ts_type_annotation) => {
                if let TSType::TSArrayType(array_type) = &ts_type_annotation.type_annotation {
                    check_and_report_error_generic(
                        default_config,
                        array_type.span,
                        &array_type.element_type,
                        ctx,
                        false,
                    );
                }

                if let TSType::TSTypeOperatorType(ts_operator_type) = &ts_type_annotation.type_annotation {
                    if let TSTypeOperator::Readonly = &ts_operator_type.operator {
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

                if let TSType::TSTypeReference(ts_type_reference) = &ts_type_annotation.type_annotation {
                    check_and_report_error_array(
                        default_config,
                        readonly_config, 
                        ts_type_reference, 
                        ctx,
                    );
                }
            },
            _ => {}
        }
    }
}


fn type_needs_parentheses(type_param: &TSType) -> bool {
    match type_param {
        TSType::TSTypeReference(node) => {
            // TODO: 
            println!("node===> {:#?}", node);
            // return type_needs_parentheses(ts_type_ref.type_name);
            true

        },
        TSType::TSUnionType(_) => true,
        TSType::TSFunctionType(_) => true,
        TSType::TSIntersectionType(_) => true,
        TSType::TSTypeOperatorType(_) => true,
        TSType::TSInferType(_) => true,
        TSType::TSConstructorType(_) => true,
        _ => false,
    }
}

fn check_and_report_error_generic(
    config: &ArrayOption,
    type_reference_span: Span,
    element_type: &TSType, 
    ctx: &LintContext,
    is_readonly: bool,
) {
    if let ArrayOption::Array = config {
        return;
    }
    if let ArrayOption::ArraySimple = config {
        if is_simple_type(element_type) {
            return;
        }
    }
    let source_text = ctx.source_text().to_string();

    let readonly_prefix = if is_readonly { "readonly " } else { "" };
    let class_name = if is_readonly { "ReadonlyArray" } else { "Array" };
    let message_type = get_message_type(element_type, &source_text);

    let diagnostic = if let ArrayOption::Generic = config {
        ArrayTypeDiagnostic::ErrorStringGeneric(
            readonly_prefix.to_string(),
            class_name.to_string(),
            message_type.to_string(),
            type_reference_span,
        )
    } else {
        ArrayTypeDiagnostic::ErrorStringGenericSimple(
            readonly_prefix.to_string(),
            class_name.to_string(),
            message_type.to_string(),
            type_reference_span,
        )
    };
    let element_type_span = get_ts_element_type_span(&element_type);
    let Some(element_type_span) = element_type_span else { return };

    ctx.diagnostic_with_fix(diagnostic, || {
        let type_text = &source_text[element_type_span.start as usize..element_type_span.end as usize];
        let array_type_identifier = if is_readonly { "ReadonlyArray" } else { "Array" };
        
        Fix::new(
            array_type_identifier.to_string() + "<" + type_text + ">", 
            Span { start: type_reference_span.start, end: type_reference_span.end }
        )
    })
}

fn check_and_report_error_array(
    default_config: &ArrayOption,
    readonly_config: &ArrayOption,
    ts_type_reference: &TSTypeReference,
    ctx: &LintContext,
) {
    let TSTypeName::IdentifierReference(ident_ref_type_name) = &ts_type_reference.type_name else { return };

    if ident_ref_type_name.name.as_str() != "ReadonlyArray" && ident_ref_type_name.name.as_str() != "Array" {
        return;
    }
    let is_readonly_array_type = ident_ref_type_name.name == "ReadonlyArray";
    let config = if is_readonly_array_type { readonly_config } else { default_config };
    if let ArrayOption::Generic = config {
        return;
    }
    let readonly_prefix: &str = if is_readonly_array_type { "readonly " } else { "" };
    let class_name = if is_readonly_array_type { "ReadonlyArray" } else { "Array" };
    let type_params = &ts_type_reference.type_parameters;

    if type_params.is_none() || type_params.as_ref().unwrap().params.len() == 0 {
        let diagnostic = if let ArrayOption::Array = config {
            ArrayTypeDiagnostic::ErrorStringArray(
                readonly_prefix.to_string(),
                class_name.to_string(),
                "any".to_string(),
                ts_type_reference.span,
            )
        } else {
            ArrayTypeDiagnostic::ErrorStringArraySimple(
                readonly_prefix.to_string(),
                ident_ref_type_name.name.to_string(),
                "any".to_string(),
                ts_type_reference.span,
            )
        };
        ctx.diagnostic_with_fix(diagnostic, || {
            Fix::new(
                readonly_prefix.to_string() + "any[]", 
                ts_type_reference.span,
            )
        });
        return;
    }
    if type_params.as_ref().unwrap().params.len() != 1 {
        return;
    }
    let first_type_param = type_params.as_ref().unwrap().params.get(0).unwrap();
    if let ArrayOption::ArraySimple = config {
        if !is_simple_type(first_type_param) {
            return;
        }
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
        
    let element_type_span = get_ts_element_type_span(&first_type_param);
    let Some(element_type_span) = element_type_span else { return };

    let type_text = &ctx.source_text()[element_type_span.start as usize..element_type_span.end as usize];

    let mut start = String::from(if parent_parens {"("} else {""});
    start.push_str(readonly_prefix);
    start.push_str(if type_parens {"("} else {""});

    let mut end = String::from(if type_parens {")"} else {""});
    end.push_str("[]");
    end.push_str(if parent_parens {")"} else {""});

    let message_type = get_message_type(first_type_param, ctx.source_text());
    let diagnostic = if let ArrayOption::Array = config {
            ArrayTypeDiagnostic::ErrorStringArray(
                readonly_prefix.to_string(),
                class_name.to_string(),
                message_type.to_string(),
                ts_type_reference.span,
            )
        } else {
            ArrayTypeDiagnostic::ErrorStringArraySimple(
                readonly_prefix.to_string(),
                ident_ref_type_name.name.to_string(),
                message_type.to_string(),
                ts_type_reference.span,
            )
        };
        ctx.diagnostic_with_fix(diagnostic, || {
            Fix::new(
                start + type_text + end.as_str(), 
                ts_type_reference.span,
            )
        });
}

// Check whatever node can be considered as simple type
fn is_simple_type(element_type: &TSType) -> bool {
    match element_type {
        TSType::TSAnyKeyword(_) => true,
        TSType::TSBooleanKeyword(_) => true,
        TSType::TSNeverKeyword(_) => true,
        TSType::TSNumberKeyword(_) => true,
        TSType::TSBigIntKeyword(_) => true,
        TSType::TSObjectKeyword(_) => true,
        TSType::TSStringKeyword(_) => true,
        TSType::TSSymbolKeyword(_) => true,
        TSType::TSUnknownKeyword(_) => true,
        TSType::TSVoidKeyword(_) => true,
        TSType::TSNullKeyword(_) => true,
        TSType::TSArrayType(_) => true,
        TSType::TSUndefinedKeyword(_) => true,
        TSType::TSQualifiedName(_) => true,
        TSType::TSThisKeyword(_) => true,
        TSType::TSTypeReference(node) => {
            println!("TSTypeReference ===> {:#?}", node);
            // TODO:
            true
        },
        _ => false,
    }

}

fn get_message_type<'a>(element_type: &'a TSType, source_text: &'a str) -> &'a str {
    if is_simple_type(element_type) {
        let element_type_span = get_ts_element_type_span(element_type);
        let Some(element_type_span) = element_type_span else { return "T" };
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
        TSType::TSThisKeyword(t) => Some(t.span),
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
        ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array" }]))),
        ("let a: (string | number)[] = [];", Some(serde_json::json!([{ "default": "array" }]))),
        ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "array" }]))),
        ("let a: readonly (string | number)[] = [];", Some(serde_json::json!([{ "default": "array" }]))),
        ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array" }]))),
        ("let a: (string | number)[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array" }]))),
        // ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array" }]))),
        // ("let a: readonly (string | number)[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array" }]))),
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array-simple" }]))),
        // ("let a: (string | number)[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array-simple" }]))),
        // ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array-simple" }]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "array", "readonly": "array-simple" }]))),
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "generic" }]))),
        // ("let a: (string | number)[] = [];", Some(serde_json::json!([{ "default": "array", "readonly": "generic" }]))),
        // ("let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{ "default": "array", "readonly": "generic" }]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "array", "readonly": "generic" }]))),
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array-simple" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "array-simple" }]))),
        // ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "array-simple" }]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "array-simple" }]))),
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array" }]))),
        // ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array" }]))),
        // ("let a: readonly (string | number)[] = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array" }]))),
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array-simple" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array-simple" }]))),
        // ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array-simple" }]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "array-simple" }]))),
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "generic" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "generic" }]))),
        // ("let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "generic" }]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "array-simple", "readonly": "generic" }]))),
        // ("let a: Array<number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
        // ("let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
        // ("let a: Array<number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "generic" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "generic" }]))),
        // ("let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "generic" }]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "generic" }]))),
        // ("let a: Array<number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array" }]))),
        // ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array" }]))),
        // ("let a: readonly (string | number)[] = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array" }]))),
        // ("let a: Array<number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array-simple" }]))),
        // ("let a: Array<string | number> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array-simple" }]))),
        // ("let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "generic",}]))),
        // ("let a: ReadonlyArray<string | number> = [];", Some(serde_json::json!([{ "default": "generic",}]))),
        // ("let a: Array<bigint> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array" }]))),
        // ("let a: readonly bigint[] = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array" }]))),
        // ("let a: readonly (string | bigint)[] = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array" }]))),
        // ("let a: Array<bigint> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array-simple" }]))),
        // ("let a: Array<string | bigint> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array-simple" }]))),
        // ("let a: readonly bigint[] = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array-simple" }]))),
        // ("let a: ReadonlyArray<string | bigint> = [];", Some(serde_json::json!([{ "default": "generic", "readonly": "array-simple" }]))),
    ];
    let fail: Vec<(&str, Option<serde_json::Value>)> = vec![
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "generic" }]))),
    ];
    let fix = vec![
        ("let a: Array<number> = [];", "let a: number[] = [];", Some(serde_json::json!([{ "default": "array" }]))),
        ("let a: Array<string | number> = [];", "let a: (string | number)[] = [];", Some(serde_json::json!([{ "default": "array" }]))),
        ("let a: ReadonlyArray<number> = [];", "let a: readonly number[] = [];", Some(serde_json::json!([{ "default": "array" }]))),
        ("let a: ReadonlyArray<string | number> = [];", "let a: readonly (string | number)[] = [];", Some(serde_json::json!([{ "default": "array" }]))),

        ("let a: number[] = [];", "let a: Array<number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
        ("let a: readonly number[] = [];", "let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
        ("let x: Array<undefined> = [undefined] as undefined[];", "let x: undefined[] = [undefined] as undefined[];", Some(serde_json::json!([{ "default": "array-simple" }]))),
        ("let z: Array = [3, '4'];", "let z: any[] = [3, '4'];", Some(serde_json::json!([{ "default": "array-simple" }]))),
    ];

    Tester::new(ArrayType::NAME, pass, fail).expect_fix(fix).test();
}
