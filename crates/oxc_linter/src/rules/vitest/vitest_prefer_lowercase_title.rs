use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        parse_expect_and_typeof_vitest_fn_call, parse_jest_fn_call, parse_vitest_fn_call, JestFnKind, JestGeneralFnKind, PossibleJestNode
    },
};

fn prefer_lowercase_title_diagnostic(title: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce lowercase titles")
        .with_help(format!("`{title:?}`s should begin with lowercase"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct VitestPreferLowercaseTitleConfig {
    allowed_prefixes: Vec<CompactStr>,
    ignore: Vec<CompactStr>,
    ignore_top_level_describe: bool,
    lowercase_first_character_only: bool,
}

impl std::ops::Deref for VitestPreferLowercaseTitle {
    type Target = VitestPreferLowercaseTitleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct VitestPreferLowercaseTitle(Box<VitestPreferLowercaseTitleConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce `it`, `test`, and `describe` to have descriptions that begin with a
    /// lowercase letter.
    ///
    /// ### Why is this bad?
    ///
    /// Capitalized `it`, `test`, and `describe` descriptions may result in less
    /// readable test failures.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('It works', () => {
    ///     ...
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('it works', () => {
    ///     ...
    /// })
    /// ```
    VitestPreferLowercaseTitle,
    style,
    fix
);

impl Rule for VitestPreferLowercaseTitle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        let ignore_top_level_describe = obj
            .and_then(|config| config.get("ignoreTopLevelDescribe"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let lowercase_first_character_only = obj
            .and_then(|config| config.get("lowercaseFirstCharacterOnly"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let ignore = obj
            .and_then(|config| config.get("ignore"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();
        let allowed_prefixes = obj
            .and_then(|config| config.get("allowedPrefixes"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(VitestPreferLowercaseTitleConfig {
            allowed_prefixes,
            ignore,
            ignore_top_level_describe,
            lowercase_first_character_only,
        }))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_vitest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_vitest_node.node;
        println!("node: {node:?}");
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        println!("call_expr: {call_expr:?}");
        let Some(vitest_fn_call) =
            parse_vitest_fn_call(call_expr, possible_vitest_node, ctx)
        else {
            return;
        };

        println!("vitest_fn_call: {vitest_fn_call:?}");

        let scopes = ctx.scopes();

        // TODO: populate ignores
        // let ignores = Self::populate_ignores(&self.ignore);

        // if ignores.contains(&vitest_fn_call.name.as_ref()) {
        //     return;
        // }

        
        if matches!(vitest_fn_call, JestGeneralFnKind::Describe) {
            if self.ignore_top_level_describe && scopes.get_flags(node.scope_id()).is_top() {
                return;
            }
        } else if !matches!(
            vitest_fn_call,
            JestGeneralFnKind::Test | JestGeneralFnKind::VitestBench
        ) {
            return;
        }

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        if let Argument::StringLiteral(string_expr) = arg {
            self.lint_string(ctx, string_expr.value.as_str(), string_expr.span);
        } else if let Argument::TemplateLiteral(template_expr) = arg {
            let Some(template_string) = template_expr.quasi() else {
                return;
            };
            dbg!("template_string: {template_string}");
            self.lint_string(ctx, template_string.as_str(), template_expr.span);
        }
    }
}

impl VitestPreferLowercaseTitle {
    fn lint_string<'a>(&self, ctx: &LintContext<'a>, literal: &'a str, span: Span) {
        dbg!("literal: {literal}");

        if literal.is_empty()
            || self.allowed_prefixes.iter().any(|name| literal.starts_with(name.as_str()))
        {
            return;
        }

        if self.lowercase_first_character_only {
            let Some(first_char) = literal.chars().next() else {
                return;
            };

            let lower = first_char.to_ascii_lowercase();
            if first_char == lower {
                return;
            }
        } else {
            for n in 0..literal.chars().count() {
                dbg!("n: {n}");
                let Some(next_char) = literal.chars().nth(n) else {
                    return;
                };

                dbg!("next_char: {next_char}");

                let next_lower = next_char.to_ascii_lowercase();

                if next_char != next_lower {
                    break;
                }
            }
        }

        let replacement = if self.lowercase_first_character_only {
            // safety: we know this is a valid char because we checked it above.
            literal.chars().nth(0).unwrap().to_ascii_lowercase().to_string()
        } else {
            literal.to_ascii_lowercase()
        };

        let replacement_len = replacement.len() as u32;

        // dbg!("replacement: {replacement}");

        ctx.diagnostic_with_fix(prefer_lowercase_title_diagnostic(literal, span), |fixer| {
            fixer.replace(Span::sized(span.start + 1, replacement_len), replacement)
        });
            }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
        ("it.each()", None),
        ("it.each()(1)", None),
        ("it.todo();", None),
        (r#"describe("oo", function () {})"#, None),
        (r#"test("foo", function () {})"#, None),
        ("test(`123`, function () {})", None),
    ];

    let fail: Vec<(&str, Option<serde_json::Value>)> = vec![
        (r#"it("Foo MM mm", function () {})"#, None),
        ("test(`Foo MM mm`, function () {})", None),
        (
            "test(`SFC Compile`, function () {})",
            Some(
                serde_json::json!([        {          "lowercaseFirstCharacterOnly": false        }      ]),
            ),
        ),
        ("bench(`Foo MM mm`, function () {})", None),
    ];

    let fix: Vec<(&str, &str, Option<serde_json::Value>)> = vec![
        (r#"it("Foo MM mm", function () {})"#, r#"it("foo MM mm", function () {})"#, None),
        ("test(`Foo MM mm`, function () {})", "test(`foo MM mm`, function () {})", None),
        (
            "test(`SFC Compile`, function () {})",
            "test(`sfc compile`, function () {})",
            Some(
                serde_json::json!([        {          "lowercaseFirstCharacterOnly": false        }      ]),
            ),
        ),
        ("bench(`Foo MM mm`, function () {})", "bench(`foo MM mm`, function () {})", None),
    ];

    // TODO: figure out how to prefix this name.
    Tester::new(VitestPreferLowercaseTitle::NAME, VitestPreferLowercaseTitle::CATEGORY, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .with_snapshot_suffix("vitest")
        .test_and_snapshot();
}
