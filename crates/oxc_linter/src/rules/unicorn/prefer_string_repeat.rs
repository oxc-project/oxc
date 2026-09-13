use oxc_ast::{
    AstKind,
    ast::{
        AccessorPropertyType, MethodDefinitionType, ModuleExportName, PropertyDefinitionType,
        TSLiteral,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::keyword::RESERVED_KEYWORDS;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_string_repeat_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `String#repeat()` for repeated whitespace.")
        .with_help("Replace the repeated whitespace with a call to `String#repeat()`.")
        .with_label(span)
}

const DEFAULT_MINIMUM_REPETITIONS: usize = 3;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferStringRepeat {
    /// The minimum number of repetitions required before reporting a string.
    #[serde(deserialize_with = "deserialize_minimum_repetitions")]
    #[schemars(range(min = 2))]
    minimum_repetitions: usize,
}

fn deserialize_minimum_repetitions<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let minimum_repetitions = usize::deserialize(deserializer)?;
    if minimum_repetitions < 2 {
        return Err(serde::de::Error::custom("minimumRepetitions must be at least 2"));
    }
    Ok(minimum_repetitions)
}

impl Default for PreferStringRepeat {
    fn default() -> Self {
        Self { minimum_repetitions: DEFAULT_MINIMUM_REPETITIONS }
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers `String#repeat()` over literal strings containing repeated whitespace.
    ///
    /// ### Why is this bad?
    ///
    /// `String#repeat()` makes the intended number and kind of repeated whitespace explicit,
    /// instead of requiring readers to count visually indistinguishable characters.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const indentation = "    ";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const indentation = " ".repeat(4);
    /// ```
    PreferStringRepeat,
    unicorn,
    style,
    fix,
    config = PreferStringRepeat,
    version = "next",
    short_description = "Prefer `String#repeat()` over repeated whitespace.",
);

impl Rule for PreferStringRepeat {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (span, value) = match node.kind() {
            AstKind::StringLiteral(literal) => (literal.span, literal.value.as_str()),
            AstKind::TemplateLiteral(literal) => {
                if !literal.expressions.is_empty()
                    || literal.quasis.len() != 1
                    || matches!(
                        ctx.nodes().parent_kind(node.id()),
                        AstKind::TaggedTemplateExpression(_)
                    )
                {
                    return;
                }

                let Some(value) = literal.quasis[0].value.cooked.as_ref() else {
                    return;
                };
                (literal.span, value.as_str())
            }
            _ => return,
        };

        if is_restricted_context(node, span, ctx) {
            return;
        }

        let mut chars = value.chars();
        let Some(whitespace) = chars.next() else {
            return;
        };
        if !is_ecmascript_whitespace(whitespace) {
            return;
        }

        let repetitions = 1 + chars.take_while(|&ch| ch == whitespace).count();
        if repetitions < self.minimum_repetitions || repetitions != value.chars().count() {
            return;
        }

        let repeated = readable_whitespace(whitespace);
        ctx.diagnostic_with_fix(prefer_string_repeat_diagnostic(span), |fixer| {
            let source_text = ctx.source_text();
            let leading_space = if needs_leading_space(span, source_text) { " " } else { "" };
            let trailing_space = if needs_trailing_space(span, source_text) { " " } else { "" };
            fixer.replace(
                span,
                format!("{leading_space}{repeated}.repeat({repetitions}){trailing_space}"),
            )
        });
    }
}

/// Mirrors the contexts excluded by `eslint-plugin-unicorn` because replacing a literal with an
/// expression would either be invalid syntax or change its meaning.
///
/// Reference: <https://github.com/sindresorhus/eslint-plugin-unicorn/blob/main/rules/prefer-string-repeat.js>
fn is_restricted_context(node: &AstNode<'_>, span: Span, ctx: &LintContext<'_>) -> bool {
    match ctx.nodes().parent_kind(node.id()) {
        AstKind::Directive(_)
        | AstKind::TaggedTemplateExpression(_)
        | AstKind::ExportAllDeclaration(_)
        | AstKind::TSEnumMember(_)
        | AstKind::JSXAttribute(_) => true,
        AstKind::TSLiteralType(literal_type) => match &literal_type.literal {
            TSLiteral::StringLiteral(literal) => literal.span == span,
            TSLiteral::TemplateLiteral(literal) => literal.span == span,
            _ => false,
        },
        AstKind::ImportDeclaration(declaration) => declaration.source.span == span,
        AstKind::ExportFromDeclaration(declaration) => declaration.source.span == span,
        AstKind::ImportSpecifier(specifier) => {
            module_export_name_span(&specifier.imported) == Some(span)
        }
        AstKind::ExportSpecifier(specifier) => {
            module_export_name_span(&specifier.local) == Some(span)
                || module_export_name_span(&specifier.exported) == Some(span)
        }
        AstKind::ImportAttribute(attribute) => {
            attribute.key.span() == span || attribute.value.span == span
        }
        AstKind::ObjectProperty(property) => !property.computed && property.key.span() == span,
        AstKind::MethodDefinition(method) => {
            method.key.span() == span
                && (!method.computed
                    || method.r#type == MethodDefinitionType::TSAbstractMethodDefinition)
        }
        AstKind::PropertyDefinition(property) => {
            property.key.span() == span
                && (!property.computed
                    || property.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition)
        }
        AstKind::AccessorProperty(property) => {
            property.key.span() == span
                && (!property.computed
                    || property.r#type == AccessorPropertyType::TSAbstractAccessorProperty)
        }
        AstKind::TSPropertySignature(property) => property.key.span() == span,
        AstKind::TSMethodSignature(method) => method.key.span() == span,
        AstKind::TSExternalModuleDeclaration(declaration) => declaration.id.span == span,
        AstKind::TSExternalModuleReference(reference) => reference.expression.span == span,
        AstKind::TSImportType(import_type) => import_type.source.span == span,
        AstKind::CallExpression(call) => is_jest_inline_snapshot(call, span),
        _ => false,
    }
}

fn is_jest_inline_snapshot(call: &oxc_ast::ast::CallExpression<'_>, span: Span) -> bool {
    let Some(member) = call.callee.as_member_expression() else {
        return false;
    };
    if call.optional
        || member.optional()
        || !matches!(
            member.static_property_name(),
            Some("toMatchInlineSnapshot" | "toThrowErrorMatchingInlineSnapshot")
        )
        || call.arguments.len() != 1
        || call.arguments[0].span() != span
    {
        return false;
    }

    let oxc_ast::ast::Expression::CallExpression(expect_call) = member.object() else {
        return false;
    };
    !expect_call.optional
        && expect_call.arguments.len() == 1
        && expect_call
            .callee
            .get_identifier_reference()
            .is_some_and(|identifier| identifier.name == "expect")
}

fn module_export_name_span(name: &ModuleExportName<'_>) -> Option<Span> {
    match name {
        ModuleExportName::StringLiteral(literal) => Some(literal.span),
        _ => None,
    }
}

fn is_ecmascript_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

fn readable_whitespace(ch: char) -> String {
    match ch {
        ' ' => "' '".to_string(),
        '\t' => r"'\t'".to_string(),
        '\n' => r"'\n'".to_string(),
        '\r' => r"'\r'".to_string(),
        '\u{000C}' => r"'\f'".to_string(),
        _ => format!(r"'\u{{{:X}}}'", ch as u32),
    }
}

fn needs_leading_space(span: Span, source_text: &str) -> bool {
    let before = &source_text[..span.start as usize];
    RESERVED_KEYWORDS.iter().any(|keyword| before.ends_with(keyword))
        || before.ends_with("of")
        || before.ends_with("await")
}

fn needs_trailing_space(span: Span, source_text: &str) -> bool {
    let after = &source_text[span.end as usize..];
    RESERVED_KEYWORDS.iter().any(|keyword| after.starts_with(keyword))
        || after.starts_with("of")
        || after.starts_with("await")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"const string = " ";"#, None),
        (r#"const string = "  ";"#, None),
        (r#"const string = "a";"#, None),
        (r#"const string = "aaa";"#, None),
        (r#"const string = "unicorn unicorn unicorn";"#, None),
        (r#"const string = " \t ";"#, None),
        (r#"const string = " \n ";"#, None),
        (r#"const string = "\r\n\r\n\r\n";"#, None),
        (r#"const string = " ".repeat(3);"#, None),
        ("const string = `  `;", None),
        ("const string = tag`   `;", None),
        (r"const string = `  ${value}`;", None),
        (r#"const object = {"   ": value};"#, None),
        (r#""   ";"#, None),
        (r#"expect(foo).toMatchInlineSnapshot("   ");"#, None),
        ("expect(foo).toMatchInlineSnapshot(`   `);", None),
        (r#"expect(foo).toThrowErrorMatchingInlineSnapshot("   ");"#, None),
        (
            r#"const string = "   "; // minimumRepetitions: 4"#,
            Some(serde_json::json!([{"minimumRepetitions": 4}])),
        ),
        (r#"import "   ";"#, None),
        (r#"export {} from "   ";"#, None),
        (r#"export * from "   ";"#, None),
        (r#"import "module" with {"   ": "value"};"#, None),
        (r#"export {} from "module" with {"   ": "value"};"#, None),
        (r#"import "module" with {key: "   "};"#, None),
        (r#"export {} from "module" with {key: "   "};"#, None),
        (r#"import {"   " as string} from "module";"#, None),
        (r#"export {"   " as string} from "module";"#, None),
        (r#"export {string as "   "} from "module";"#, None),
        (r#"export * as "   " from "module";"#, None),
        (r#"enum Enum {"   " = 1}"#, None),
        (r#"enum Enum {Key = "   "}"#, None),
        (r#"declare module "   " {}"#, None),
        (r#"import type Type = require("   ");"#, None),
        (r#"type Type = "   ";"#, None),
        ("type Type = `   `;", None),
        (r#"type Type = import("   ");"#, None),
        (r#"abstract class Class { abstract "   " }"#, None),
        (r#"abstract class Class { abstract "   "() }"#, None),
        ("abstract class Class { abstract [`   `](): void }", None),
        (r#"abstract class Class { abstract accessor "   " }"#, None),
        ("abstract class Class { abstract accessor [`   `]: string }", None),
        (r#"interface Interface { "   " }"#, None),
        (r#"interface Interface { "   "(): void }"#, None),
        ("interface Interface { [`   `](): void }", None),
        ("interface Interface { readonly [`   `]: string }", None),
        (r#"type Type = { "   "(): void }"#, None),
        ("type Type = { [`   `](): void }", None),
        (r#"class Class { "   " = 1 }"#, None),
        (r#"class Class { "   "() {} }"#, None),
        (r#"class Class { accessor "   " = 1 }"#, None),
        ("enum OtherEnum {Key = `   `}", None),
        (r#"<Component attribute="   " />"#, None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, }
    ];

    let fail = vec![
        (r#"const string = "   ";"#, None),
        (r#"const string = "    ";"#, None),
        (r#"const string = "\t\t\t";"#, None),
        (r#"const string = "\n\n\n";"#, None),
        (r#"const string = "\r\r\r";"#, None),
        (r#"const string = "\v\v\v";"#, None),
        (r#"const string = "\f\f\f";"#, None),
        (r#"const string = "\u00A0\u00A0\u00A0";"#, None),
        (r#"const string = "\u2003\u2003\u2003";"#, None),
        (r#"const string = "\uFEFF\uFEFF\uFEFF";"#, None),
        ("const string = `   `;", None),
        (r"const string = `\t\t\t`;", None),
        (r#"function foo() {return"   "}"#, None),
        (r#"foo(); "   ";"#, None),
        (r#"const object = {["   "]: value};"#, None),
        (
            r#"const string = "  "; // minimumRepetitions: 2"#,
            Some(serde_json::json!([{"minimumRepetitions": 2}])),
        ),
        (
            r#"const string = "    "; // minimumRepetitions: 4"#,
            Some(serde_json::json!([{"minimumRepetitions": 4}])),
        ),
        (r#"class Class { ["   "] = 1 }"#, None),
        (r#"class Class { ["   "]() {} }"#, None),
        (r#"class Class { accessor ["   "] = 1 }"#, None),
        (r#"formatter.toMatchInlineSnapshot("   ");"#, None),
        (r#"custom.toThrowErrorMatchingInlineSnapshot("   ");"#, None),
        (r#"expect(foo)?.toMatchInlineSnapshot("   ");"#, None),
        (r#"expect(foo).toMatchInlineSnapshot?.("   ");"#, None),
        (r#"expect?.(foo).toMatchInlineSnapshot("   ");"#, None),
        (r#"<Component attribute={"   "} />"#, None), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        (
            "function foo() {
                return'   ';
            }",
            None,
        ),
    ];

    let fix = vec![
        (r#"const string = "   ";"#, "const string = ' '.repeat(3);"),
        (r#"const string = "\t\t\t";"#, r"const string = '\t'.repeat(3);"),
        (r#"const string = "\u00A0\u00A0\u00A0";"#, r"const string = '\u{A0}'.repeat(3);"),
        ("const string = `   `;", "const string = ' '.repeat(3);"),
        (r#"const object = {["   "]: value};"#, "const object = {[' '.repeat(3)]: value};"),
        (r#"const result = "   "in object;"#, "const result = ' '.repeat(3) in object;"),
        (
            "function foo() {
                return'   ';
            }",
            "function foo() {
                return ' '.repeat(3);
            }",
        ),
    ];

    Tester::new(PreferStringRepeat::NAME, PreferStringRepeat::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn configuration_minimum_repetitions() {
    assert!(
        PreferStringRepeat::from_configuration(serde_json::json!([{"minimumRepetitions": 0}]))
            .is_err()
    );
    assert!(
        PreferStringRepeat::from_configuration(serde_json::json!([{"minimumRepetitions": 1}]))
            .is_err()
    );
    assert_eq!(
        PreferStringRepeat::from_configuration(serde_json::json!([{"minimumRepetitions": 2}]))
            .unwrap()
            .minimum_repetitions,
        2
    );
}
