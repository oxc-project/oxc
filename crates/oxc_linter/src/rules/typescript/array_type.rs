use oxc_ast::{
    AstKind,
    ast::{TSType, TSTypeName, TSTypeOperatorOperator, TSTypeReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ArrayType(Box<ArrayTypeConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ArrayTypeConfig {
    /// The array type expected for mutable cases.
    default: ArrayOption,
    /// The array type expected for readonly cases. If omitted, the value for `default` will be used.
    readonly: Option<ReadonlyArrayOption>,
}

impl std::ops::Deref for ArrayType {
    type Target = ArrayTypeConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArrayOption {
    /// Enforce using `T[]` for all array types.
    ///
    /// Example of **incorrect** code for this option:
    /// ```ts
    /// const arr: Array<number> = new Array<number>();
    /// ```
    ///
    /// Example of **correct** code for this option:
    /// ```ts
    /// const arr: number[] = new Array<number>();
    /// ```
    #[default]
    Array,
    /// Enforce using `T[]` for simple types, and `Array<T>` for complex types.
    ///
    /// Example of **incorrect** code for this option:
    /// ```ts
    /// const a: (string | number)[] = ['a', 'b'];
    /// const b: { prop: string }[] = [{ prop: 'a' }];
    /// const c: Array<MyType> = ['a', 'b'];
    /// const d: Array<string> = ['a', 'b'];
    /// ```
    ///
    /// Example of **correct** code for this option:
    /// ```ts
    /// const a: Array<string | number> = ['a', 'b'];
    /// const b: Array<{ prop: string }> = [{ prop: 'a' }];
    /// const c: string[] = ['a', 'b'];
    /// const d: MyType[] = ['a', 'b'];
    /// ```
    ArraySimple,
    /// Enforce using `Array<T>` for all array types.
    ///
    /// Example of **incorrect** code for this option:
    /// ```ts
    /// const arr: number[] = new Array<number>();
    /// ```
    ///
    /// Example of **correct** code for this option:
    /// ```ts
    /// const arr: Array<number> = new Array<number>();
    /// ```
    Generic,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReadonlyArrayOption {
    /// Enforce using `readonly T[]` for all readonly array types.
    ///
    /// Example of **incorrect** code for this option:
    /// ```ts
    /// const arr: ReadonlyArray<number> = [];
    /// ```
    ///
    /// Example of **correct** code for this option:
    /// ```ts
    /// const arr: readonly number[] = [];
    /// ```
    #[default]
    Array,
    /// Enforce using `readonly T[]` for simple types, and `ReadonlyArray<T>` for complex types.
    ///
    /// Example of **incorrect** code for this option:
    /// ```ts
    /// const a: readonly (string | number)[] = [];
    /// const b: ReadonlyArray<number> = [];
    /// ```
    ///
    /// Example of **correct** code for this option:
    /// ```ts
    /// const a: ReadonlyArray<string | number> = [];
    /// const b: readonly number[] = [];
    /// ```
    ArraySimple,
    /// Enforce using `ReadonlyArray<T>` for all readonly array types.
    ///
    /// Example of **incorrect** code for this option:
    /// ```ts
    /// const arr: readonly number[] = [];
    /// const arr2: readonly (string | number)[] = [];
    /// ```
    ///
    /// Example of **correct** code for this option:
    /// ```ts
    /// const arr: ReadonlyArray<number> = [];
    /// const arr2: ReadonlyArray<string | number> = [];
    /// ```
    Generic,
}

impl From<ReadonlyArrayOption> for ArrayOption {
    fn from(value: ReadonlyArrayOption) -> Self {
        match value {
            ReadonlyArrayOption::Array => ArrayOption::Array,
            ReadonlyArrayOption::ArraySimple => ArrayOption::ArraySimple,
            ReadonlyArrayOption::Generic => ArrayOption::Generic,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require consistently using either `T[]` or `Array<T>` for arrays.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `Array` type directly is not idiomatic. Instead, use the array type `T[]` or `Array<T>`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (with default configuration):
    /// ```typescript
    /// const arr: Array<number> = new Array<number>();
    /// const readonlyArr: ReadonlyArray<number> = [1, 2, 3];
    /// ```
    ///
    /// Examples of **correct** code for this rule (with default configuration):
    /// ```typescript
    /// const arr: number[] = new Array<number>();
    /// const readonlyArr: readonly number[] = [1, 2, 3];
    /// ```
    ArrayType,
    typescript,
    style,
    fix,
    config = ArrayTypeConfig,
    version = "0.2.8",
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

impl Rule for ArrayType {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSArrayType(ts_array_type) => {
                check_array_type(
                    node,
                    ts_array_type.span,
                    &ts_array_type.element_type,
                    self.default_config(),
                    &self.readonly_config(),
                    ctx,
                );
            }
            AstKind::TSTypeReference(ts_type_reference)
                if ts_type_reference.type_name.get_identifier_reference().is_some_and(
                    |type_name| matches!(type_name.name.as_str(), "Array" | "ReadonlyArray"),
                ) =>
            {
                let readonly_config = self.readonly_config();
                if should_skip_type_reference(node, self.default_config(), &readonly_config, ctx) {
                    return;
                }
                check_and_report_error_reference(
                    self.default_config(),
                    &readonly_config,
                    ts_type_reference,
                    ctx,
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl ArrayType {
    fn default_config(&self) -> &ArrayOption {
        &self.default
    }

    fn readonly_config(&self) -> ArrayOption {
        if let Some(readonly) = &self.readonly {
            readonly.clone().into()
        } else {
            self.default.clone()
        }
    }
}

fn check_array_type<'a>(
    node: &AstNode<'a>,
    type_reference_span: Span,
    type_annotation: &TSType,
    default_config: &ArrayOption,
    readonly_config: &ArrayOption,
    ctx: &LintContext<'a>,
) {
    let (config, span, is_readonly) = if let Some(span) = readonly_array_span(node, ctx) {
        (readonly_config, span, true)
    } else {
        (default_config, type_reference_span, false)
    };
    check_and_report_error_generic(config, span, type_annotation, ctx, is_readonly);
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
    if let TSTypeName::IdentifierReference(ident_ref_type_name) = &ts_type_reference.type_name
        && (ident_ref_type_name.name.as_str() == "ReadonlyArray"
            || ident_ref_type_name.name.as_str() == "Array")
    {
        check_and_report_error_array(default_config, readonly_config, ts_type_reference, ctx);
    }
}

fn should_skip_type_reference<'a>(
    node: &AstNode<'a>,
    default_config: &ArrayOption,
    readonly_config: &ArrayOption,
    ctx: &LintContext<'a>,
) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::TSArrayType(ts_array_type) => {
                return array_type_would_report(
                    ancestor,
                    &ts_array_type.element_type,
                    default_config,
                    readonly_config,
                    ctx,
                );
            }
            AstKind::TSTypeAnnotation(_)
            | AstKind::TSTypeAliasDeclaration(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSTypeParameter(_)
            | AstKind::TSConditionalType(_)
            | AstKind::TSIndexedAccessType(_)
            | AstKind::TSMappedType(_) => return false,
            _ => {}
        }
    }

    false
}

fn readonly_array_span<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> Option<Span> {
    let parent = ctx.nodes().parent_node(node.id());
    match parent.kind() {
        AstKind::TSTypeOperator(ts_operator_type)
            if matches!(ts_operator_type.operator, TSTypeOperatorOperator::Readonly) =>
        {
            Some(ts_operator_type.span)
        }
        _ => None,
    }
}

fn array_type_would_report<'a>(
    node: &AstNode<'a>,
    type_annotation: &TSType,
    default_config: &ArrayOption,
    readonly_config: &ArrayOption,
    ctx: &LintContext<'a>,
) -> bool {
    let config =
        if readonly_array_span(node, ctx).is_some() { readonly_config } else { default_config };
    match config {
        ArrayOption::Array => false,
        ArrayOption::Generic => true,
        ArrayOption::ArraySimple => !is_simple_type(type_annotation.without_parenthesized()),
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
    let type_params = &ts_type_reference.type_arguments;

    if type_params.is_none() || type_params.as_ref().unwrap().params.is_empty() {
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
    let first_type_param = first_type_param.without_parenthesized();
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
        | TSType::TSThisType(_) => true,
        TSType::TSTypeReference(node) => {
            if let Some(type_name) = TSTypeName::get_identifier_reference(&node.type_name) {
                if type_name.name.as_str() == "Array" {
                    if node.type_arguments.is_none() {
                        return true;
                    }
                    if node.type_arguments.as_ref().unwrap().params.len() == 1 {
                        return is_simple_type(
                            node.type_arguments.as_ref().unwrap().params.first().unwrap(),
                        );
                    }
                } else {
                    if node.type_arguments.is_some() {
                        return false;
                    }
                    if node.type_name.is_identifier() || node.type_name.is_qualified_name() {
                        return true;
                    }
                    return false;
                }
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
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<string[]>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<Array<string>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<string[]>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<Array<{name: string}>>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test = testFn<string[], number[]>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test = testFn<Array<string>, Array<number>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<readonly string[]>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<ReadonlyArray<string>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<readonly string[]>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<ReadonlyArray<{name: string}>>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<string>('hello');",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test = testFn<{name: string}>({name: 'test'});",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "class MyClass<T> { constructor(public value: T) {} }
const instance = new MyClass<number>(42);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        // https://github.com/oxc-project/oxc/issues/12605
        ("let a: factories.User[] = [];", Some(serde_json::json!([{"default":"array-simple"}]))),
        ("let a: factories.TT.User[] = [];", Some(serde_json::json!([{"default":"array-simple"}]))),
        (
            "let z: readonly factories.User[] = [];",
            Some(serde_json::json!([{"readonly":"array-simple"}])),
        ),
        // https://github.com/oxc-project/oxc/issues/16897 - satisfies expression
        (
            "const arr = [] as const satisfies ReadonlyArray<string>;",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "const arr = [] as const satisfies readonly string[];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
    ];

    let fail = vec![
        ("let a: factories.User[] = [];", Some(serde_json::json!([{"default":"generic"}]))),
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
        ("interface I { b: string | string[]; }", Some(serde_json::json!([{"default":"generic"}]))),
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
        // (
        // "let y: string[] = <Array<string>>['2'];",
        // Some(serde_json::json!([{"default":"array-simple"}])),
        // ),
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
        // ("let y: string[] = <Array<string>>['2'];", Some(serde_json::json!([{"default":"array"}]))),
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
        // https://github.com/oxc-project/oxc/issues/11568
        ("type x = Array<number>[]", None),
        ("const arr: Array<Array<number>>[] = [];", None),
        ("export function fn4(arr: Array<number>[]) { return arr; }", None),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<{name: string}[]>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<Array<{name: string}>>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<string[]>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<(string | number)[]>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<readonly string[]>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<ReadonlyArray<string>>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test3 = testFn<string[], number[]>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test3 = testFn<Array<string>, Array<number>>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test4 = testFn<Promise<string[]>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test4 = testFn<Promise<Array<string>>>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test5 = testFn<(string & number)[]>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test5 = testFn<(() => void)[]>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        // Array of arrays in generic arguments
        // Note: When checking types in generic arguments, the rule checks recursively,
        // so string[][] will trigger errors for both the outer and inner array types.
        // This is different from checking a standalone type annotation where only the
        // outermost type is checked.
        // Class generic instantiation
        (
            "class MyClass<T> { constructor(public value: T) {} }
const instance = new MyClass<number[]>([1, 2, 3]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "class MyClass<T> { constructor(public value: T) {} }
const instance = new MyClass<Array<number>>([1, 2, 3]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // Type assertion with generic
        (
            "const value = {} as Map<string, number[]>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "const value = {} as Map<string, Array<number>>;",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "interface Container<T> { value: T; }
const container: Container<string[]> = { value: [] };",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "interface Container<T> { value: T; }
const container: Container<Array<string>> = { value: [] };",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test7 = testFn<readonly string[], readonly number[]>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test7 = testFn<ReadonlyArray<string>, ReadonlyArray<number>>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test8 = testFn<string[], Array<number>>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test8 = testFn<Array<string>, number[]>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type IsArray<T> = T extends any[] ? true : false;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type MakeArrays<T> = { [K in keyof T]: T[K][] };",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        ("let y = <Array<string>>['2'];", Some(serde_json::json!([{"default":"array"}]))),
        ("type Box<T extends string[]> = T;", Some(serde_json::json!([{"default":"generic"}]))),
        ("type Box<T extends Array<string>> = T;", Some(serde_json::json!([{"default":"array"}]))),
        ("type Box<T = string[]> = T;", Some(serde_json::json!([{"default":"generic"}]))),
        ("type Box<T = Array<string>> = T;", Some(serde_json::json!([{"default":"array"}]))),
        ("let x: Array<(string | number)> = [];", Some(serde_json::json!([{"default":"array"}]))),
        (
            "let x: ReadonlyArray<(string | number)> = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // https://github.com/oxc-project/oxc/issues/16897 - satisfies expression
        (
            "const arr = [] as const satisfies readonly string[];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "const arr = [] as const satisfies readonly SupportedDomainName[];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "const arr = [] as const satisfies ReadonlyArray<string>;",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "const arr = [] as const satisfies string[];",
            Some(serde_json::json!([{"default":"generic"}])),
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
            "interface I { b: string | string[]; }",
            "interface I { b: string | Array<string>; }",
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
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<{name: string}[]>([]);",
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<Array<{name: string}>>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<Array<{name: string}>>([]);",
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<{name: string}[]>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<string[]>([]);",
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<Array<string>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<(string | number)[]>([]);",
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<Array<string | number>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<readonly string[]>([]);",
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<ReadonlyArray<string>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<ReadonlyArray<string>>([]);",
            "function testFn<T>(param: T) { return param; }
export const test2 = testFn<readonly string[]>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // Multiple type parameters - fix tests
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test3 = testFn<string[], number[]>([]);",
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test3 = testFn<Array<string>, Array<number>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test3 = testFn<Array<string>, Array<number>>([]);",
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test3 = testFn<string[], number[]>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // Complex types in generic arguments - fix tests
        (
            "function testFn<T>(param: T) { return param; }
export const test5 = testFn<(string & number)[]>([]);",
            "function testFn<T>(param: T) { return param; }
export const test5 = testFn<Array<string & number>>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test5 = testFn<(() => void)[]>([]);",
            "function testFn<T>(param: T) { return param; }
export const test5 = testFn<Array<() => void>>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        // Note: Nested arrays in generic arguments are checked recursively,
        // so fixes are applied at each level independently
        // Class generic instantiation - fix tests
        (
            "class MyClass<T> { constructor(public value: T) {} }
const instance = new MyClass<number[]>([1, 2, 3]);",
            "class MyClass<T> { constructor(public value: T) {} }
const instance = new MyClass<Array<number>>([1, 2, 3]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "class MyClass<T> { constructor(public value: T) {} }
const instance = new MyClass<Array<number>>([1, 2, 3]);",
            "class MyClass<T> { constructor(public value: T) {} }
const instance = new MyClass<number[]>([1, 2, 3]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // Readonly arrays in multiple type parameters - fix tests
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test7 = testFn<readonly string[], readonly number[]>([]);",
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test7 = testFn<ReadonlyArray<string>, ReadonlyArray<number>>([]);",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test7 = testFn<ReadonlyArray<string>, ReadonlyArray<number>>([]);",
            "function testFn<T, U>(param1: T, param2: U) { return [param1, param2]; }
export const test7 = testFn<readonly string[], readonly number[]>([]);",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // array-simple with simple types in generics
        (
            "function testFn<T>(param: T) { return param; }
export const test9 = testFn<Array<string>>([]);",
            "function testFn<T>(param: T) { return param; }
export const test9 = testFn<string[]>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "function testFn<T>(param: T) { return param; }
export const test9 = testFn<ReadonlyArray<number>>([]);",
            "function testFn<T>(param: T) { return param; }
export const test9 = testFn<readonly number[]>([]);",
            Some(serde_json::json!([{"default":"array-simple"}])),
        ),
        (
            "let y = <Array<string>>['2'];",
            "let y = <string[]>['2'];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type Box<T extends string[]> = T;",
            "type Box<T extends Array<string>> = T;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type Box<T extends Array<string>> = T;",
            "type Box<T extends string[]> = T;",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "type Box<T = string[]> = T;",
            "type Box<T = Array<string>> = T;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
        (
            "type Box<T = Array<string>> = T;",
            "type Box<T = string[]> = T;",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let x: Array<(string | number)> = [];",
            "let x: (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        (
            "let x: ReadonlyArray<(string | number)> = [];",
            "let x: readonly (string | number)[] = [];",
            Some(serde_json::json!([{"default":"array"}])),
        ),
        // https://github.com/oxc-project/oxc/issues/16897 - satisfies expression
        (
            "const arr = [] as const satisfies readonly string[];",
            "const arr = [] as const satisfies ReadonlyArray<string>;",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "const arr = [] as const satisfies readonly SupportedDomainName[];",
            "const arr = [] as const satisfies ReadonlyArray<SupportedDomainName>;",
            Some(serde_json::json!([{"default":"array-simple","readonly":"generic"}])),
        ),
        (
            "const arr = [] as const satisfies ReadonlyArray<string>;",
            "const arr = [] as const satisfies readonly string[];",
            Some(serde_json::json!([{"default":"array-simple","readonly":"array"}])),
        ),
        (
            "const arr = [] as const satisfies string[];",
            "const arr = [] as const satisfies Array<string>;",
            Some(serde_json::json!([{"default":"generic"}])),
        ),
    ];

    Tester::new(ArrayType::NAME, ArrayType::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test_and_snapshot();
}
