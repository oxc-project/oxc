use oxc_ast::{
    AstKind,
    ast::{Argument, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    AstNode,
    ast_util::is_new_expression,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn never_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Remove the `./` prefix from the relative URL.").with_label(span)
}

fn always_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Add a `./` prefix to the relative URL.").with_label(span)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RelativeUrlStyleConfig {
    #[default]
    /// Never use a `./` prefix.
    Never,
    /// Always add a `./` prefix to the relative URL when possible.
    Always,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
pub struct RelativeUrlStyle(RelativeUrlStyleConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent relative URL style.
    ///
    /// ### Why is this bad?
    ///
    /// When using a relative URL in `new URL()`, the URL should either never or always use the `./` prefix consistently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"never"` option:
    /// ```js
    /// new URL("./foo", base);
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"never"` option:
    /// ```js
    /// new URL("foo", base);
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `"always"` option:
    /// ```js
    /// new URL("foo", base);
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"always"` option:
    /// ```js
    /// new URL("./foo", base);
    /// ```
    RelativeUrlStyle,
    unicorn,
    style,
    fix_suggestion,
    config = RelativeUrlStyleConfig,
);

const DOT_SLASH: &str = "./";
const TEST_URL_BASES: [&str; 2] = ["https://example.com/a/b/", "https://example.com/a/b.html"];

impl Rule for RelativeUrlStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        if !is_new_expression(new_expr, &["URL"], Some(2), Some(2)) {
            return;
        }

        if new_expr.arguments.last().is_some_and(oxc_ast::ast::Argument::is_spread) {
            return;
        }

        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };

        match first_arg {
            Argument::StringLiteral(str_lit) => {
                let url = str_lit.value.as_str();

                match self.0 {
                    RelativeUrlStyleConfig::Never => {
                        let raw = str_lit.raw.as_ref().map_or(url, |r| {
                            let s = r.as_str();
                            // remove surrounding quotes
                            &s[1..s.len() - 1]
                        });

                        if can_remove_dot_slash(raw, new_expr) {
                            ctx.diagnostic_with_fix(never_diagnostic(str_lit.span), |fixer| {
                                let dot_slash_start = str_lit.span.start + 1;
                                let dot_slash_span =
                                    Span::new(dot_slash_start, dot_slash_start + 2);
                                fixer
                                    .delete_range(dot_slash_span)
                                    .with_message("Remove leading `./`")
                            });
                        }
                    }
                    RelativeUrlStyleConfig::Always => {
                        if can_add_dot_slash(url, new_expr) {
                            ctx.diagnostic_with_fix(always_diagnostic(str_lit.span), |fixer| {
                                let insert_pos = str_lit.span.start + 1;
                                let insert_span = Span::new(insert_pos, insert_pos);
                                fixer
                                    .replace(insert_span, DOT_SLASH)
                                    .with_message("Add `./` prefix")
                            });
                        }
                    }
                }
            }
            Argument::TemplateLiteral(template_lit) => {
                if !matches!(self.0, RelativeUrlStyleConfig::Never) {
                    return;
                }

                let Some(first_quasi) = template_lit.quasis.first() else {
                    return;
                };

                if first_quasi.value.raw.starts_with(DOT_SLASH) {
                    ctx.diagnostic_with_suggestion(never_diagnostic(template_lit.span), |fixer| {
                        let dot_slash_start = template_lit.span.start + 1;
                        let dot_slash_span = Span::new(dot_slash_start, dot_slash_start + 2);
                        fixer.delete_range(dot_slash_span).with_message("Remove leading `./`")
                    });
                }
            }
            _ => (),
        }
    }
}

fn can_add_dot_slash(url: &str, new_expr: &NewExpression) -> bool {
    // don't add if already starts with ./ or . or /
    if url.starts_with(DOT_SLASH) || url.starts_with('.') || url.starts_with('/') {
        return false;
    }

    if let Some(Argument::StringLiteral(base_lit)) = new_expr.arguments.get(1) {
        let base = base_lit.value.as_str();
        if is_safe_to_add_dot_slash(url, &[base]) {
            return true;
        }
    }

    is_safe_to_add_dot_slash(url, &TEST_URL_BASES)
}

fn can_remove_dot_slash(url: &str, new_expr: &NewExpression) -> bool {
    if !url.starts_with(DOT_SLASH) {
        return false;
    }

    if let Some(Argument::StringLiteral(base_lit)) = new_expr.arguments.get(1) {
        let base = base_lit.value.as_str();
        if is_safe_to_remove_dot_slash(url, &[base]) {
            return true;
        }
    }

    is_safe_to_remove_dot_slash(url, &TEST_URL_BASES)
}

fn is_safe_to_add_dot_slash_to_url(url: &str, base: &str) -> bool {
    let Ok(base_url) = Url::parse(base) else {
        return false;
    };

    let Ok(original) = base_url.join(url) else {
        return false;
    };

    let Ok(with_dot_slash) = base_url.join(&format!("{DOT_SLASH}{url}")) else {
        return false;
    };

    original.as_str() == with_dot_slash.as_str()
}

fn is_safe_to_add_dot_slash(url: &str, bases: &[&str]) -> bool {
    bases.iter().all(|base| is_safe_to_add_dot_slash_to_url(url, base))
}

fn is_safe_to_remove_dot_slash(url: &str, bases: &[&str]) -> bool {
    let Some(url_without_prefix) = url.strip_prefix(DOT_SLASH) else {
        return false;
    };

    bases.iter().all(|base| is_safe_to_add_dot_slash_to_url(url_without_prefix, base))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Default "never" option
        (r#"URL("./foo", base)"#, None),
        (r#"new URL(...["./foo"], base)"#, None),
        (r#"new URL(["./foo"], base)"#, None),
        (r#"new URL("./foo")"#, None),
        (r#"new URL("./foo", base, extra)"#, None),
        (r#"new URL("./foo", ...[base])"#, None),
        (r#"new NOT_URL("./foo", base)"#, None),
        (r#"new NOT_URL("./", base)"#, None),
        (r#"new URL("./", base)"#, None),
        (r#"new URL("./", "https://example.com/a/b/c.html")"#, None),
        (r#"const base = new URL("./", import.meta.url)"#, None),
        (r"new URL", None),
        (r"new URL(0, base)", None),
        // Not checking this case
        (r#"new globalThis.URL("./foo", base)"#, None),
        (r#"const foo = "./foo"; new URL(foo, base)"#, None),
        (r#"const foo = "/foo"; new URL(`.${foo}`, base)"#, None),
        (r"new URL(`.${foo}`, base)", None),
        (r#"new URL(".", base)"#, None),
        (r#"new URL(".././foo", base)"#, None),
        // We don't check cooked value
        (r"new URL(`\u002E/${foo}`, base)", None),
        // We don't check escaped string
        (r#"new URL("\u002E/foo", base)"#, None),
        (r"new URL('\u002E/foo', base)", None),
        // "always" option
        (r#"URL("foo", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL(...["foo"], base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL(["foo"], base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("foo")"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("foo", base, extra)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("foo", ...[base])"#, Some(serde_json::json!(["always"]))),
        (r#"new NOT_URL("foo", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("", "https://example.com/a/b.html")"#, Some(serde_json::json!(["always"]))),
        (r"/* 2 */ new URL", Some(serde_json::json!(["always"]))),
        (r"new URL(0, base2)", Some(serde_json::json!(["always"]))),
        // Not checking this case
        (r#"new globalThis.URL("foo", base)"#, Some(serde_json::json!(["always"]))),
        (r"new URL(`${foo}`, base2)", Some(serde_json::json!(["always"]))),
        (r"new URL(`.${foo}`, base2)", Some(serde_json::json!(["always"]))),
        (r#"new URL(".", base2)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("//example.org", "https://example.com")"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("//example.org", "ftp://example.com")"#, Some(serde_json::json!(["always"]))),
        (
            r#"new URL("ftp://example.org", "https://example.com")"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"new URL("https://example.org:65536", "https://example.com")"#,
            Some(serde_json::json!(["always"])),
        ),
        (r#"new URL("/", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("/foo", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("../foo", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL(".././foo", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("C:\foo", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("\u002E/foo", base)"#, Some(serde_json::json!(["always"]))),
        (r#"new URL("\u002Ffoo", base)"#, Some(serde_json::json!(["always"]))),
    ];

    let fail = vec![
        // Default "never" option
        (r#"new URL("./foo", base)"#, None),
        (r"new URL('./foo', base)", None),
        (r#"new URL("././a", base)"#, None),
        (r"new URL(`./${foo}`, base)", None),
        (r#"new URL("./", "https://example.com/a/b/")"#, None),
        // "always" option
        (r#"new URL("foo", base)"#, Some(serde_json::json!(["always"]))),
        (r"new URL('foo', base)", Some(serde_json::json!(["always"]))),
        (r#"new URL("", "https://example.com/a/b/")"#, Some(serde_json::json!(["always"]))),
    ];

    let fix = vec![
        (r#"new URL("./foo", base)"#, r#"new URL("foo", base)"#, None),
        (r"new URL('./foo', base)", r"new URL('foo', base)", None),
        (r#"new URL("././a", base)"#, r#"new URL("./a", base)"#, None),
        (r"new URL(`./${foo}`, base)", r"new URL(`${foo}`, base)", None),
        (
            r#"new URL("./", "https://example.com/a/b/")"#,
            r#"new URL("", "https://example.com/a/b/")"#,
            None,
        ),
        (
            r#"new URL("foo", base)"#,
            r#"new URL("./foo", base)"#,
            Some(serde_json::json!(["always"])),
        ),
        (r"new URL('foo', base)", r"new URL('./foo', base)", Some(serde_json::json!(["always"]))),
        (
            r#"new URL("", "https://example.com/a/b/")"#,
            r#"new URL("./", "https://example.com/a/b/")"#,
            Some(serde_json::json!(["always"])),
        ),
    ];

    Tester::new(RelativeUrlStyle::NAME, RelativeUrlStyle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
