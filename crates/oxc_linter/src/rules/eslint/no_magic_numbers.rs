use oxc_ast::{
    ast::{AssignmentTarget, Expression, MemberExpression, VariableDeclarationKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodes;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

enum NoMagicNumberReportReason {
    MustUseConst,
    NoMagicNumber,
}

fn must_use_const_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Number constants declarations must use 'const'.").with_label(span)
}

fn no_magic_number_diagnostic(span: Span, raw: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("No magic number: {raw}")).with_label(span)
}

#[derive(Debug, Default, Clone)]

pub struct NoMagicNumbers(Box<NoMagicNumbersConfig>);

impl std::ops::Deref for NoMagicNumbers {
    type Target = NoMagicNumbersConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoMagicNumbersNumber {
    Float(f64),
    BigInt(String),
}

#[derive(Debug, Default, Clone)]
pub struct NoMagicNumbersConfig {
    ignore: Vec<NoMagicNumbersNumber>,
    ignore_array_indexes: bool,
    ignore_default_values: bool,
    ignore_class_field_initial_values: bool,
    enforce_const: bool,
    detect_objects: bool,
    ignore_enums: bool,
    ignore_numeric_literal_types: bool,
    ignore_readonly_class_properties: bool,
    ignore_type_indexes: bool,
}

impl TryFrom<&serde_json::Value> for NoMagicNumbersConfig {
    type Error = OxcDiagnostic;

    fn try_from(raw: &serde_json::Value) -> Result<Self, Self::Error> {
        if raw.is_null() {
            return Ok(NoMagicNumbersConfig::default());
        }

        raw.get(0).map_or_else(
            || {
                Err(OxcDiagnostic::warn(
                    "Expecting object for typescript/no-magic-numbers configuration",
                ))
            },
            |object| {
                fn get_bool_property(object: &serde_json::Value, index: &str) -> bool {
                    object.get(index).and_then(serde_json::Value::as_bool).unwrap_or_default()
                }
                Ok(Self {
                    ignore: object
                        .get("ignore")
                        .and_then(serde_json::Value::as_array)
                        .map(|v| {
                            v.iter()
                                .map(|v| {
                                    if v.is_number() {
                                        NoMagicNumbersNumber::Float(v.as_f64().unwrap())
                                    } else {
                                        NoMagicNumbersNumber::BigInt(v.as_str().unwrap().to_owned())
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    ignore_array_indexes: get_bool_property(object, "ignoreArrayIndexes"),
                    ignore_default_values: get_bool_property(object, "ignoreDefaultValues"),
                    ignore_class_field_initial_values: get_bool_property(
                        object,
                        "ignoreClassFieldInitialValues",
                    ),
                    enforce_const: get_bool_property(object, "enforceConst"),
                    detect_objects: get_bool_property(object, "detectObjects"),
                    ignore_enums: get_bool_property(object, "ignoreEnums"),
                    ignore_numeric_literal_types: get_bool_property(
                        object,
                        "ignoreNumericLiteralTypes",
                    ),
                    ignore_readonly_class_properties: get_bool_property(
                        object,
                        "ignoreReadonlyClassProperties",
                    ),
                    ignore_type_indexes: get_bool_property(object, "ignoreTypeIndexes"),
                })
            },
        )
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The no-magic-numbers rule aims to make code more readable and refactoring easier by ensuring that special numbers are declared as constants to make their meaning explicit.
    /// The current implementation does not support BigInt numbers inside array indexes.
    ///
    /// ### Why is this bad?
    ///
    /// ‘Magic numbers’ are numbers that occur multiple times in code without an explicit meaning. They should preferably be replaced by named constants.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    ///
    /// var dutyFreePrice = 100;
    /// var finalPrice = dutyFreePrice + (dutyFreePrice * 0.25);
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignore":
    /// ```javascript
    /// /*typescript no-magic-numbers: ["error", { "ignore": [1] }]*/
    /// var data = ['foo', 'bar', 'baz'];
    /// var dataLast = data.length && data[data.length - 1];
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignoreArrayIndexes":
    /// ```javascript
    /// /*typescript no-magic-numbers: ["error", { "ignoreArrayIndexes": true }]*/
    /// var item = data[2];
    /// data[100] = a;
    /// f(data[0]);
    /// a = data[-0]; // same as data[0], -0 will be coerced to "0"
    /// a = data[0xAB];
    /// a = data[5.6e1];
    /// a = data[4294967294]; // max array index
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignoreDefaultValues":
    /// ```javascript
    /// /*typescript no-magic-numbers: ["error", { "ignoreDefaultValues": true }]*/
    /// const { tax = 0.25 } = accountancy;
    /// function mapParallel(concurrency = 3) { /***/ }
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignoreClassFieldInitialValues":
    /// ```javascript
    /// /*typescript no-magic-numbers: ["error", { "ignoreClassFieldInitialValues": true }]*/
    /// class C {
    ///     foo = 2;
    ///     bar = -3;
    ///     #baz = 4;
    ///     static qux = 5;
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with option "enforceConst":
    /// ```javascript
    /// /*typescript no-magic-numbers: ["error", { "enforceConst": true }]*/
    /// var TAX = 0.25;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with option "detectObjects":
    /// ```javascript
    /// /*typescript no-magic-numbers: ["error", { "detectObjects": true }]*/
    /// var magic = {
    ///     tax: 0.25
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "detectObjects":
    /// ```javascript
    /// /*typescript no-magic-numbers: ["error", { "detectObjects": true }]*/
    /// var TAX = 0.25;
    ///
    /// var magic = {
    ///     tax: TAX
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignoreEnums":
    /// ```typescript
    /// /*typescript no-magic-numbers: ["error", { "ignoreEnums": true }]*/
    /// enum foo {
    ///     SECOND = 1000,
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignoreNumericLiteralTypes":
    /// ```typescript
    /// /*typescript no-magic-numbers: ["error", { "ignoreNumericLiteralTypes": true }]*/
    /// type SmallPrimes = 2 | 3 | 5 | 7 | 11;
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignoreReadonlyClassProperties":
    /// ```typescript
    /// /*typescript no-magic-numbers: ["error", { "ignoreReadonlyClassProperties": true }]*/
    /// class Foo {
    ///     readonly A = 1;
    ///     readonly B = 2;
    ///     public static readonly C = 1;
    ///     static readonly D = 1;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with option "ignoreTypeIndexes":
    /// ```typescript
    /// /*typescript no-magic-numbers: ["error", { "ignoreTypeIndexes": true }]*/
    /// type Foo = Bar[0];
    /// type Baz = Parameters<Foo>[2];
    /// ```
    NoMagicNumbers,
    eslint,
    style,
    pending // TODO: enforceConst, probably copy from https://github.com/oxc-project/oxc/pull/5144
);

#[derive(Debug)]
struct InternConfig<'a> {
    node: &'a AstNode<'a>,
    value: NoMagicNumbersNumber,
    raw: String,
}

impl InternConfig<'_> {
    pub fn from<'a>(node: &'a AstNode<'a>, parent_node: &'a AstNode<'a>) -> InternConfig<'a> {
        let is_unary = matches!(parent_node.kind(), AstKind::UnaryExpression(_));
        let is_negative = matches!(parent_node.kind(), AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation);

        match node.kind() {
            AstKind::NumericLiteral(numeric) => {
                if is_negative {
                    InternConfig {
                        node: parent_node,
                        value: NoMagicNumbersNumber::Float(0.0 - numeric.value),
                        raw: format!("-{}", numeric.raw.as_ref().unwrap()),
                    }
                } else {
                    InternConfig {
                        node: if is_unary { parent_node } else { node },
                        value: NoMagicNumbersNumber::Float(numeric.value),
                        raw: numeric.raw.as_ref().unwrap().as_str().into(),
                    }
                }
            }
            AstKind::BigIntLiteral(bigint) => {
                let big_int_string = bigint.raw.into_string();
                if is_negative {
                    let raw = format!("-{big_int_string}");

                    InternConfig {
                        node: parent_node,
                        value: NoMagicNumbersNumber::BigInt(raw.clone()),
                        raw,
                    }
                } else {
                    InternConfig {
                        node: if is_unary { parent_node } else { node },
                        value: NoMagicNumbersNumber::BigInt(big_int_string.clone()),
                        raw: big_int_string,
                    }
                }
            }
            _ => {
                unreachable!(
                    "expected AstKind BingIntLiteral or NumericLiteral, got {:?}",
                    node.kind().debug_name()
                )
            }
        }
    }
}

impl Rule for NoMagicNumbers {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(NoMagicNumbersConfig::try_from(&value).unwrap()))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !matches!(node.kind(), AstKind::NumericLiteral(_) | AstKind::BigIntLiteral(_)) {
            return;
        }

        let nodes = ctx.semantic().nodes();
        let config = InternConfig::from(node, nodes.parent_node(node.id()).unwrap());

        if self.is_skipable(&config, nodes) {
            return;
        }

        let parent_kind = nodes.parent_node(config.node.id()).unwrap().kind();
        let span = config.node.kind().span();

        let Some(reason) = self.get_report_reason(&parent_kind) else {
            return;
        };

        ctx.diagnostic(match reason {
            NoMagicNumberReportReason::MustUseConst => must_use_const_diagnostic(span),
            NoMagicNumberReportReason::NoMagicNumber => {
                no_magic_number_diagnostic(span, &config.raw)
            }
        });
    }
}

fn is_default_value(parent_kind: &AstKind<'_>) -> bool {
    matches!(parent_kind, AstKind::AssignmentTargetWithDefault(_) | AstKind::AssignmentPattern(_))
}

fn is_class_field_initial_value(parent_kind: &AstKind<'_>) -> bool {
    matches!(parent_kind, AstKind::PropertyDefinition(_))
}

fn is_detectable_object(parent_kind: &AstKind<'_>) -> bool {
    matches!(parent_kind, AstKind::ObjectExpression(_) | AstKind::ObjectProperty(_))
}

fn is_parse_int_radix(parent_parent_node: &AstNode<'_>) -> bool {
    let AstKind::CallExpression(expression) = parent_parent_node.kind() else {
        return false;
    };

    let callee = expression.callee.without_parentheses();

    callee.is_specific_id("parseInt") || callee.is_specific_member_access("Number", "parseInt")
}

/// Returns whether the given node is used as an array index.
/// Value must coerce to a valid array index name: "0", "1", "2" ... "4294967294".
///
/// All other values, like "-1", "2.5", or "4294967295", are just "normal" object properties,
/// which can be created and accessed on an array in addition to the array index properties,
/// but they don't affect array's length and are not considered by methods such as .map(), .forEach() etc.
///
/// The maximum array length by the specification is 2 ** 32 - 1 = 4294967295,
/// thus the maximum valid index is 2 ** 32 - 2 = 4294967294.
///
/// All notations are allowed, as long as the value coerces to one of "0", "1", "2" ... "4294967294".
///
/// This current implementation does not check for BigInt Numbers, they will always accept them as array_index
/// Valid examples:
/// ```javascript
/// a[0], a[1], a[1.2e1], a[0xAB], a[0n], a[1n]
/// a[-0] // (same as a[0] because -0 coerces to "0")
/// a[-0n] // (-0n evaluates to 0n)
/// ```
///
/// Invalid examples:
/// ```javascript
/// a[-1], a[-0xAB], a[-1n], a[2.5], a[1.23e1], a[12e-1]
/// a[4294967295] // (above the max index, it's an access to a regular property a["4294967295"])
/// a[999999999999999999999] // (even if it wasn't above the max index, it would be a["1e+21"])
/// a[1e310] // (same as a["Infinity"])
/// ```
fn is_array_index<'a>(ast_kind: &AstKind<'a>, parent_kind: &AstKind<'a>) -> bool {
    match ast_kind {
        AstKind::BigIntLiteral(_) => true,
        AstKind::UnaryExpression(unary) => match &unary.argument {
            Expression::BigIntLiteral(_) => true,
            Expression::NumericLiteral(numeric)
                if unary.operator == UnaryOperator::UnaryNegation =>
            {
                numeric.value == 0.0
            }
            _ => false,
        },
        AstKind::NumericLiteral(numeric) => match parent_kind {
            AstKind::MemberExpression(expression) => {
                if let MemberExpression::ComputedMemberExpression(computed_expression) = expression
                {
                    return computed_expression.expression.is_number_value(numeric.value)
                        && numeric.value.fract() == 0.0
                        && numeric.value < f64::from(u32::MAX);
                }

                false
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_ts_enum(parent_kind: &AstKind<'_>) -> bool {
    matches!(parent_kind, AstKind::TSEnumMember(_))
}

fn is_ts_numeric_literal<'a>(parent_parent_node: &AstNode<'a>, nodes: &AstNodes<'a>) -> bool {
    let mut node = parent_parent_node;

    while matches!(
        node.kind(),
        AstKind::TSUnionType(_) | AstKind::TSIntersectionType(_) | AstKind::TSParenthesizedType(_)
    ) {
        node = nodes.parent_node(node.id()).unwrap();
    }

    matches!(node.kind(), AstKind::TSTypeAliasDeclaration(_))
}

fn is_ts_readonly_property(parent_kind: &AstKind<'_>) -> bool {
    matches!(parent_kind, AstKind::PropertyDefinition(property) if property.readonly)
}

fn is_ts_indexed_access_type<'a>(parent_parent_node: &AstNode<'a>, nodes: &AstNodes<'a>) -> bool {
    let mut node = parent_parent_node;

    while matches!(
        node.kind(),
        AstKind::TSUnionType(_) | AstKind::TSIntersectionType(_) | AstKind::TSParenthesizedType(_)
    ) {
        node = nodes.parent_node(node.id()).unwrap();
    }

    matches!(node.kind(), AstKind::TSIndexedAccessType(_))
}

impl NoMagicNumbers {
    fn is_skipable<'a>(&self, config: &InternConfig<'a>, nodes: &AstNodes<'a>) -> bool {
        if self.ignore.contains(&config.value) {
            return true;
        }

        let parent = nodes.parent_node(config.node.id()).unwrap();
        let parent_kind = parent.kind();

        if self.ignore_enums && is_ts_enum(&parent_kind) {
            return true;
        }

        if self.ignore_readonly_class_properties && is_ts_readonly_property(&parent_kind) {
            return true;
        }

        if self.ignore_default_values && is_default_value(&parent_kind) {
            return true;
        }

        if self.ignore_class_field_initial_values && is_class_field_initial_value(&parent_kind) {
            return true;
        }

        if !self.detect_objects && is_detectable_object(&parent_kind) {
            return true;
        }

        let parent_parent = nodes.parent_node(parent.id()).unwrap();

        if is_parse_int_radix(parent_parent) {
            return true;
        }

        if self.ignore_numeric_literal_types && is_ts_numeric_literal(parent_parent, nodes) {
            return true;
        }

        if self.ignore_type_indexes && is_ts_indexed_access_type(parent_parent, nodes) {
            return true;
        }

        // test it at the end because of false positives with BigInt values
        if self.ignore_array_indexes && is_array_index(&config.node.kind(), &parent_kind) {
            return true;
        }

        false
    }

    fn get_report_reason<'a>(
        &self,
        parent_kind: &'a AstKind<'a>,
    ) -> Option<NoMagicNumberReportReason> {
        match parent_kind {
            AstKind::VariableDeclarator(declarator) => {
                if self.enforce_const && declarator.kind != VariableDeclarationKind::Const {
                    return Some(NoMagicNumberReportReason::MustUseConst);
                }

                None
            }
            AstKind::AssignmentExpression(expression) => {
                if matches!(expression.left, AssignmentTarget::AssignmentTargetIdentifier(_)) {
                    return Some(NoMagicNumberReportReason::NoMagicNumber);
                }

                None
            }
            AstKind::JSXExpressionContainer(_) => None,
            _ => Some(NoMagicNumberReportReason::NoMagicNumber),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let ignore_array_indexes = Some(serde_json::json!([{"ignoreArrayIndexes": true}]));
    let ignore_default_values = Some(serde_json::json!([{"ignoreDefaultValues": true}]));
    let enforce_const = Some(serde_json::json!([{ "enforceConst": true}]));
    let ignore_class_field_initial_values =
        Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }]));
    let ignore_numeric_literal_types =
        Some(serde_json::json!([{ "ignoreNumericLiteralTypes": true }]));
    let ignore_typed_index_arrays = Some(serde_json::json!([{ "ignoreTypeIndexes": true }]));

    let pass = vec![
        ("var x = parseInt(y, 10);", None),
        ("var x = parseInt(y, -10);", None),
        ("var x = Number.parseInt(y, 10);", None),
        ("const foo = 42;", None), // { "ecmaVersion": 6 },
        (
            "var foo = 42;",
            Some(serde_json::json!([{                "enforceConst": false            }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = -42;", None),
        (
            "var foo = 0 + 1 - 2 + -2;",
            Some(serde_json::json!([{                "ignore": [0, 1, 2, -2]            }])),
        ),
        (
            "var foo = 0 + 1 + 2 + 3 + 4;",
            Some(serde_json::json!([{                "ignore": [0, 1, 2, 3, 4]            }])),
        ),
        ("var foo = { bar:10 }", None),
        (
            "setTimeout(function() {return 1;}, 0);",
            Some(serde_json::json!([{                "ignore": [0, 1]            }])),
        ),
        ("var data = ['foo', 'bar', 'baz']; var third = data[3];", ignore_array_indexes.clone()),
        ("foo[0]", ignore_array_indexes.clone()),
        ("foo[-0]", ignore_array_indexes.clone()),
        ("foo[1]", ignore_array_indexes.clone()),
        ("foo[100]", ignore_array_indexes.clone()),
        ("foo[200.00]", ignore_array_indexes.clone()),
        ("foo[3e4]", ignore_array_indexes.clone()),
        ("foo[1.23e2]", ignore_array_indexes.clone()),
        ("foo[230e-1]", ignore_array_indexes.clone()),
        ("foo[0b110]", ignore_array_indexes.clone()), // { "ecmaVersion": 2015 },
        ("foo[0o71]", ignore_array_indexes.clone()),  // { "ecmaVersion": 2015 },
        ("foo[0xABC]", ignore_array_indexes.clone()),
        ("foo[0123]", ignore_array_indexes.clone()), // {                "sourceType": "script"            },
        ("foo[5.0000000000000001]", ignore_array_indexes.clone()),
        ("foo[4294967294]", ignore_array_indexes.clone()),
        ("foo[0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[-0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[100n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[0xABn]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[4294967294n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("var a = <input maxLength={10} />;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("var a = <div objectProp={{ test: 1}}></div>;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("f(100n)", Some(serde_json::json!([{ "ignore": ["100n"] }]))), // { "ecmaVersion": 2020 },
        ("f(-100n)", Some(serde_json::json!([{ "ignore": ["-100n"] }]))), // { "ecmaVersion": 2020 },
        ("const { param = 123 } = sourceObject;", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("const func = (param = 123) => {}", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("const func = ({ param = 123 }) => {}", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("const [one = 1, two = 2] = []", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("var one, two; [one = 1, two = 2] = []", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("var x = parseInt?.(y, 10);", None), // { "ecmaVersion": 2020 },
        ("var x = Number?.parseInt(y, 10);", None), // { "ecmaVersion": 2020 },
        ("var x = (Number?.parseInt)(y, 10);", None), // { "ecmaVersion": 2020 },
        ("foo?.[777]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("class C { foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { foo = -2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { static foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { #foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { static #foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 }
        ("const FOO = 10;", ignore_numeric_literal_types.clone()),
        ("type Foo = 'bar';", None),
        ("type Foo = true;", None),
        ("type Foo = 1;", ignore_numeric_literal_types.clone()),
        ("type Foo = -1;", ignore_numeric_literal_types.clone()),
        ("type Foo = 1 | 2 | 3;", ignore_numeric_literal_types.clone()),
        ("type Foo = 1 | -1;", ignore_numeric_literal_types.clone()),
        (
            "
			        enum foo {
			          SECOND = 1000,
			          NUM = '0123456789',
			          NEG = -1,
			          POS = +1,
			        }
			      ",
            Some(serde_json::json!([{ "ignoreEnums": true }])),
        ),
        (
            "
			class Foo {
			  readonly A = 1;
			  readonly B = 2;
			  public static readonly C = 1;
			  static readonly D = 1;
			  readonly E = -1;
			  readonly F = +1;
			  private readonly G = 100n;
			}
			      ",
            Some(serde_json::json!([{ "ignoreReadonlyClassProperties": true }])),
        ),
        ("type Foo = Bar[0];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[-1];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[0xab];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[5.6e1];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[10n];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[1 | -2];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[1 & -2];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[1 & number];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar[((1 & -2) | 3) | 4];", ignore_typed_index_arrays.clone()),
        ("type Foo = Parameters<Bar>[2];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar['baz'];", ignore_typed_index_arrays.clone()),
        ("type Foo = Bar['baz'];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        (
            "
			type Others = [['a'], ['b']];

			type Foo = {
			  [K in keyof Others[0]]: Others[K];
			};
			      ",
            Some(serde_json::json!([{ "ignoreTypeIndexes": true }])),
        ),
        ("type Foo = 1;", Some(serde_json::json!([{ "ignore": [1] }]))),
        ("type Foo = -2;", Some(serde_json::json!([{ "ignore": [-2] }]))),
        ("type Foo = 3n;", Some(serde_json::json!([{ "ignore": ["3n"] }]))),
        ("type Foo = -4n;", Some(serde_json::json!([{ "ignore": ["-4n"] }]))),
        ("type Foo = 5.6;", Some(serde_json::json!([{ "ignore": [5.6] }]))),
        ("type Foo = -7.8;", Some(serde_json::json!([{ "ignore": [-7.8] }]))),
        ("type Foo = 0x0a;", Some(serde_json::json!([{ "ignore": [0x0a] }]))),
        ("type Foo = -0xbc;", Some(serde_json::json!([{ "ignore": [-0xbc] }]))),
        ("type Foo = 1e2;", Some(serde_json::json!([{ "ignore": [1e2] }]))),
        ("type Foo = -3e4;", Some(serde_json::json!([{ "ignore": [-3e4] }]))),
        ("type Foo = 5e-6;", Some(serde_json::json!([{ "ignore": [5e-6] }]))),
        ("type Foo = -7e-8;", Some(serde_json::json!([{ "ignore": [-7e-8] }]))),
        ("type Foo = 1.1e2;", Some(serde_json::json!([{ "ignore": [1.1e2] }]))),
        ("type Foo = -3.1e4;", Some(serde_json::json!([{ "ignore": [-3.1e4] }]))),
        ("type Foo = 5.1e-6;", Some(serde_json::json!([{ "ignore": [5.1e-6] }]))),
        ("type Foo = -7.1e-8;", Some(serde_json::json!([{ "ignore": [-7.1e-8] }]))),
        (
            "
			interface Foo {
			  bar: 1;
			}
			      ",
            Some(serde_json::json!([{ "ignoreNumericLiteralTypes": true, "ignore": [1] }])),
        ),
        (
            "
			enum foo {
			  SECOND = 1000,
			  NUM = '0123456789',
			  NEG = -1,
			  POS = +2,
			}
			      ",
            Some(serde_json::json!([{ "ignoreEnums": false, "ignore": [1000, -1, 2] }])),
        ),
        (
            "
			class Foo {
			  readonly A = 1;
			  readonly B = 2;
			  public static readonly C = 3;
			  static readonly D = 4;
			  readonly E = -5;
			  readonly F = +6;
			  private readonly G = 100n;
			  private static readonly H = -2000n;
			}
			      ",
            Some(
                serde_json::json!([        {          "ignoreReadonlyClassProperties": false,          "ignore": [1, 2, 3, 4, -5, 6, "100n", "-2000n"],        },      ]),
            ),
        ),
        (
            "type Foo = Bar[0];",
            Some(serde_json::json!([{ "ignoreTypeIndexes": false, "ignore": [0] }])),
        ),
        (
            "
			type Other = {
			  [0]: 3;
			};

			type Foo = {
			  [K in keyof Other]: `${K & number}`;
			};
			      ",
            Some(serde_json::json!([{ "ignoreTypeIndexes": true, "ignore": [0, 3] }])),
        ),
    ];

    let fail = vec![
        ("var foo = 42", enforce_const.clone()), // { "ecmaVersion": 6 },
        ("var foo = 0 + 1;", None),
        ("var foo = 42n", enforce_const.clone()), // {                "ecmaVersion": 2020            },
        ("var foo = 0n + 1n;", None), // {                "ecmaVersion": 2020            },
        ("a = a + 5;", None),
        ("a += 5;", None),
        ("var foo = 0 + 1 + -2 + 2;", None),
        (
            "var foo = 0 + 1 + 2;",
            Some(serde_json::json!([{                "ignore": [0, 1]            }])),
        ),
        (
            "var foo = { bar:10 }",
            Some(serde_json::json!([{                "detectObjects": true            }])),
        ),
        ("console.log(0x1A + 0x02); console.log(071);", None), // {                "sourceType": "script"            },
        (
            "var stats = {avg: 42};",
            Some(serde_json::json!([{                "detectObjects": true            }])),
        ),
        ("var colors = {}; colors.RED = 2; colors.YELLOW = 3; colors.BLUE = 4 + 5;", None),
        ("function getSecondsInMinute() {return 60;}", None),
        ("function getNegativeSecondsInMinute() {return -60;}", None),
        ("var data = ['foo', 'bar', 'baz']; var third = data[3];", None),
        ("var data = ['foo', 'bar', 'baz']; var third = data[3];", Some(serde_json::json!([{}]))),
        (
            "var data = ['foo', 'bar', 'baz']; var third = data[3];",
            Some(serde_json::json!([{"ignoreArrayIndexes": false}])),
        ),
        ("foo[-100]", ignore_array_indexes.clone()),
        ("foo[-1.5]", ignore_array_indexes.clone()),
        ("foo[-1]", ignore_array_indexes.clone()),
        ("foo[-0.1]", ignore_array_indexes.clone()),
        ("foo[-0b110]", ignore_array_indexes.clone()), // { "ecmaVersion": 2015 },
        ("foo[-0o71]", ignore_array_indexes.clone()),  // { "ecmaVersion": 2015 },
        ("foo[-0x12]", ignore_array_indexes.clone()),
        ("foo[-012]", ignore_array_indexes.clone()), // { "sourceType": "script" },
        ("foo[0.1]", ignore_array_indexes.clone()),
        ("foo[0.12e1]", ignore_array_indexes.clone()),
        ("foo[1.5]", ignore_array_indexes.clone()),
        ("foo[1.678e2]", ignore_array_indexes.clone()),
        ("foo[56e-1]", ignore_array_indexes.clone()),
        ("foo[5.000000000000001]", ignore_array_indexes.clone()),
        ("foo[100.9]", ignore_array_indexes.clone()),
        ("foo[4294967295]", ignore_array_indexes.clone()),
        ("foo[1e300]", ignore_array_indexes.clone()),
        ("foo[1e310]", ignore_array_indexes.clone()),
        ("foo[-1e310]", ignore_array_indexes.clone()),
        // ("foo[-1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[-100n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[-0x12n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[4294967295n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[+0]", ignore_array_indexes.clone()),
        ("foo[+1]", ignore_array_indexes.clone()),
        ("foo[-(-1)]", ignore_array_indexes.clone()),
        // ("foo[+0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[+1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[- -1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("100 .toString()", ignore_array_indexes.clone()),
        ("200[100]", ignore_array_indexes.clone()),
        ("var a = <div arrayProp={[1,2,3]}></div>;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("var min, max, mean; min = 1; max = 10; mean = 4;", Some(serde_json::json!([{}]))),
        ("f(100n)", Some(serde_json::json!([{ "ignore": [100] }]))), // { "ecmaVersion": 2020 },
        ("f(-100n)", Some(serde_json::json!([{ "ignore": ["100n"] }]))), // { "ecmaVersion": 2020 },
        ("f(100n)", Some(serde_json::json!([{ "ignore": ["-100n"] }]))), // { "ecmaVersion": 2020 },
        ("f(100)", Some(serde_json::json!([{ "ignore": ["100n"] }]))),
        (
            "const func = (param = 123) => {}",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        ("const { param = 123 } = sourceObject;", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("const { param = 123 } = sourceObject;", None), // { "ecmaVersion": 6 },
        (
            "const { param = 123 } = sourceObject;",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "const [one = 1, two = 2] = []",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "var one, two; [one = 1, two = 2] = []",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        ("class C { foo = 2; }", None),                  // { "ecmaVersion": 2022 },
        ("class C { foo = 2; }", Some(serde_json::json!([{}]))), // { "ecmaVersion": 2022 },
        (
            "class C { foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { foo = -2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { #foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static #foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { foo = 2 + 3; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { [2]; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 }
        ("type Foo = 1;", Some(serde_json::json!([{ "ignoreNumericLiteralTypes": false }]))),
        ("type Foo = -1;", Some(serde_json::json!([{ "ignoreNumericLiteralTypes": false }]))),
        (
            "type Foo = 1 | 2 | 3;",
            Some(serde_json::json!([{ "ignoreNumericLiteralTypes": false }])),
        ),
        ("type Foo = 1 | -1;", Some(serde_json::json!([{ "ignoreNumericLiteralTypes": false }]))),
        (
            "
			interface Foo {
			  bar: 1;
			}
			      ",
            Some(serde_json::json!([{ "ignoreNumericLiteralTypes": true }])),
        ),
        (
            "
			enum foo {
			  SECOND = 1000,
			  NUM = '0123456789',
			  NEG = -1,
			  POS = +1,
			}
			      ",
            Some(serde_json::json!([{ "ignoreEnums": false }])),
        ),
        (
            "
			class Foo {
			  readonly A = 1;
			  readonly B = 2;
			  public static readonly C = 3;
			  static readonly D = 4;
			  readonly E = -5;
			  readonly F = +6;
			  private readonly G = 100n;
			}
			      ",
            Some(serde_json::json!([{ "ignoreReadonlyClassProperties": false }])),
        ),
        ("type Foo = Bar[0];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        ("type Foo = Bar[-1];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        ("type Foo = Bar[0xab];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        ("type Foo = Bar[5.6e1];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        // ("type Foo = Bar[10n];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        ("type Foo = Bar[1 | -2];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        ("type Foo = Bar[1 & -2];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        ("type Foo = Bar[1 & number];", Some(serde_json::json!([{ "ignoreTypeIndexes": false }]))),
        (
            "type Foo = Bar[((1 & -2) | 3) | 4];",
            Some(serde_json::json!([{ "ignoreTypeIndexes": false }])),
        ),
        (
            "type Foo = Parameters<Bar>[2];",
            Some(serde_json::json!([{ "ignoreTypeIndexes": false }])),
        ),
        (
            "
			type Others = [['a'], ['b']];

			type Foo = {
			  [K in keyof Others[0]]: Others[K];
			};
			      ",
            Some(serde_json::json!([{ "ignoreTypeIndexes": false }])),
        ),
        (
            "
			type Other = {
			  [0]: 3;
			};

			type Foo = {
			  [K in keyof Other]: `${K & number}`;
			};
			      ",
            Some(serde_json::json!([{ "ignoreTypeIndexes": true }])),
        ),
        (
            "
			type Foo = {
			  [K in 0 | 1 | 2]: 0;
			};
			      ",
            Some(serde_json::json!([{ "ignoreTypeIndexes": true }])),
        ),
        ("type Foo = 1;", Some(serde_json::json!([{ "ignore": [-1] }]))),
        ("type Foo = -2;", Some(serde_json::json!([{ "ignore": [2] }]))),
        ("type Foo = 3n;", Some(serde_json::json!([{ "ignore": ["-3n"] }]))),
        ("type Foo = -4n;", Some(serde_json::json!([{ "ignore": ["4n"] }]))),
        ("type Foo = 5.6;", Some(serde_json::json!([{ "ignore": [-5.6] }]))),
        ("type Foo = -7.8;", Some(serde_json::json!([{ "ignore": [7.8] }]))),
        ("type Foo = 0x0a;", Some(serde_json::json!([{ "ignore": [-0x0a] }]))),
        ("type Foo = -0xbc;", Some(serde_json::json!([{ "ignore": [0xbc] }]))),
        ("type Foo = 1e2;", Some(serde_json::json!([{ "ignore": [-1e2] }]))),
        ("type Foo = -3e4;", Some(serde_json::json!([{ "ignore": [3e4] }]))),
        ("type Foo = 5e-6;", Some(serde_json::json!([{ "ignore": [-5e-6] }]))),
        ("type Foo = -7e-8;", Some(serde_json::json!([{ "ignore": [7e-8] }]))),
        ("type Foo = 1.1e2;", Some(serde_json::json!([{ "ignore": [-1.1e2] }]))),
        ("type Foo = -3.1e4;", Some(serde_json::json!([{ "ignore": [3.1e4] }]))),
        ("type Foo = 5.1e-6;", Some(serde_json::json!([{ "ignore": [-5.1e-6] }]))),
        ("type Foo = -7.1e-8;", Some(serde_json::json!([{ "ignore": [7.1e-8] }]))),
    ];

    Tester::new(NoMagicNumbers::NAME, NoMagicNumbers::PLUGIN, pass, fail).test_and_snapshot();
}
