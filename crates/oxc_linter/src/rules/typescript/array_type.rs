use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_ast::{ast::{TSArrayType, TSType, TSTypeOperator}, AstKind};
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
        let readonly_config = &self.readonly.clone().unwrap_or(default_config.clone());

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

                // Array<string> 泛型
                if let TSType::TSTypeReference(ts_type_reference) = &ts_type_annotation.type_annotation {
                    println!("{:#?}", ts_type_reference);
                }
            },
            _ => {}
        }
    }
}

fn check_and_report_error_generic(
    config: &ArrayOption,
    type_annotation_span: Span,
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
            type_annotation_span,
        )
    } else {
        ArrayTypeDiagnostic::ErrorStringGenericSimple(
            readonly_prefix.to_string(),
            class_name.to_string(),
            message_type.to_string(),
            type_annotation_span,
        )
    };
    let element_type_span = get_element_type_span(&element_type);
    let Some(element_type_span) = element_type_span else { return };

    ctx.diagnostic_with_fix(diagnostic, || {
        let element_type_span = &source_text[element_type_span.start as usize..element_type_span.end as usize];
        let array_type_identifier = if is_readonly { "ReadonlyArray" } else { "Array" };
        
        Fix::new(
            array_type_identifier.to_string() + "<" + element_type_span + ">", 
            Span { start: type_annotation_span.start, end: type_annotation_span.end }
        )
    })
}

// Check whatever node can be considered as simple type
fn is_simple_type(element_type: &TSType) -> bool {
    match element_type {
        // TODO: miss TSThisType Identifier
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
        TSType::TSTypeReference(_) => {
            // TODO:
            true
        },
        _ => false,
    }

}

fn get_message_type<'a>(element_type: &'a TSType, source_text: &'a str) -> &'a str {
    if is_simple_type(element_type) {
        let element_type_span = get_element_type_span(element_type);
        let Some(element_type_span) = element_type_span else { return "T" };
        return &source_text[element_type_span.start as usize..element_type_span.end as usize];
    }
    "T"
}

fn get_element_type_span(element_type: &TSType) -> Option<Span> {
    match element_type {
        // TODO: add more type
        TSType::TSNumberKeyword(t) => Some(t.span),
        TSType::TSStringKeyword(t) => Some(t.span),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
        ("let a: number[] = [];", None),
        // ("const y: readonly string[] = ['a', 'b'];", None),
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "array" }])))
    ];
    let fail: Vec<(&str, Option<serde_json::Value>)> = vec![
        // ("let a: number[] = [];", Some(serde_json::json!([{ "default": "generic" }]))),
    ];
    let fix = vec![
        ("let a: number[] = [];", "let a: Array<number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
        ("let a: readonly number[] = [];", "let a: ReadonlyArray<number> = [];", Some(serde_json::json!([{ "default": "generic" }]))),
    ];

    Tester::new(ArrayType::NAME, pass, fail).expect_fix(fix).test();
}
