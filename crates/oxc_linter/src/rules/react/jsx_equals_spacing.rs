use std::borrow::Cow;

use itertools::Itertools;
use oxc_ast::{AstKind, ast::JSXAttributeItem};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn jsx_equals_spacing_diagnostic(span: Span, mode: Mode, msg: Cow<'static, str>) -> OxcDiagnostic {
    let help_msg = if mode == Mode::Never {
        "Disallows spaces around the equal sign"
    } else {
        "Requires spaces around the equal sign"
    };
    OxcDiagnostic::warn(msg).with_help(help_msg).with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum Mode {
    #[default]
    Never,
    Always,
}

impl Mode {
    pub fn from(raw: &str) -> Self {
        if raw == "always" { Self::Always } else { Self::Never }
    }
}

#[derive(Debug, Default, Clone)]
pub struct JsxEqualsSpacing {
    mode: Mode,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule will enforce consistency of spacing around equal signs in JSX attributes,
    /// by requiring or disallowing one or more spaces before and after =.
    ///
    /// ### Why is this bad?
    ///
    /// Some style guides require or disallow spaces around equal signs.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `never` option:
    /// ```jsx
    /// <Hello name = {firstname} />;
    /// <Hello name ={firstname} />;
    /// <Hello name= {firstname} />;
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `never` option:
    /// ```jsx
    /// <Hello name={firstname} />;
    /// <Hello name />;
    /// <Hello {...props} />;
    /// ```
    /// Examples of **incorrect** code for this rule with the `always` option:
    /// ```jsx
    /// <Hello name={firstname} />;
    /// <Hello name ={firstname} />;
    /// <Hello name= {firstname} />;
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `always` option:
    /// ```jsx
    /// <Hello name = {firstname} />;
    /// <Hello name />;
    /// <Hello {...props} />;
    /// ```
    /// ### Options
    ///
    /// This rule has a single string option:
    ///
    /// `{ type: string, default: "never" }`
    ///
    /// * `always` enforces spaces around the equal sign
    /// * `never` disallows spaces around the equal sign (default)
    ///
    /// Example:
    /// ```json
    /// {
    ///   "react/jsx-equals-spacing": ["error", "always"]
    /// }
    /// ```
    JsxEqualsSpacing,
    react,
    style,
    fix
);

impl Rule for JsxEqualsSpacing {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);

        Self { mode: obj.and_then(Value::as_str).map(Mode::from).unwrap_or_default() }
    }
    #[expect(clippy::cast_possible_truncation)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(attr)) = node.kind() else {
            return;
        };
        let Some(attr_value) = &attr.value else {
            return;
        };
        let name_end = attr.name.span().end;
        let value_start = attr_value.span().start;
        let check_span = Span::new(name_end, value_start);
        // Maybe we can ignore the situation where there are comments before "name" and "value",
        // just like this e.g. "<App foo /** comments */= {bar}>"
        if ctx.has_comments_between(check_span) {
            return;
        }

        let Some(equal_token_pos) = ctx
            .source_range(check_span)
            .chars()
            .find_position(|c| *c == '=')
            .map(|(i, _)| (i as u32) + name_end)
        else {
            return;
        };

        // e.g. "<App foo ={bar}>"
        let is_space_before_equal = name_end < equal_token_pos;
        // e.g. "<App foo= {bar}>"
        let is_space_after_equal = value_start > equal_token_pos + 1;
        if self.mode == Mode::Never {
            if is_space_before_equal {
                ctx.diagnostic_with_fix(
                    jsx_equals_spacing_diagnostic(
                        attr.name.span(),
                        self.mode,
                        "There should be no space before '='".into(),
                    ),
                    |fixer| fixer.delete_range(Span::new(name_end, equal_token_pos)),
                );
            }
            if is_space_after_equal {
                ctx.diagnostic_with_fix(
                    jsx_equals_spacing_diagnostic(
                        attr.name.span(),
                        self.mode,
                        "There should be no space after '='".into(),
                    ),
                    |fixer| fixer.delete_range(Span::new(equal_token_pos + 1, value_start)),
                );
            }
        } else if self.mode == Mode::Always {
            if !is_space_before_equal {
                ctx.diagnostic_with_fix(
                    jsx_equals_spacing_diagnostic(
                        attr.name.span(),
                        self.mode,
                        "A space is required before '='".into(),
                    ),
                    |fixer| fixer.insert_text_after(&attr.name, " "),
                );
            }
            if !is_space_after_equal {
                ctx.diagnostic_with_fix(
                    jsx_equals_spacing_diagnostic(
                        attr.name.span(),
                        self.mode,
                        "A space is required after '='".into(),
                    ),
                    |fixer| fixer.insert_text_before(attr_value, " "),
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<App />", None),
        ("<App foo />", None),
        (r#"<App foo="bar" />"#, None),
        ("<App foo={e => bar(e)} />", None),
        ("<App {...props} />", None),
        ("<App />", Some(serde_json::json!(["never"]))),
        ("<App foo />", Some(serde_json::json!(["never"]))),
        (r#"<App foo="bar" />"#, Some(serde_json::json!(["never"]))),
        ("<App foo={e => bar(e)} />", Some(serde_json::json!(["never"]))),
        ("<App {...props} />", Some(serde_json::json!(["never"]))),
        ("<App />", Some(serde_json::json!(["always"]))),
        ("<App foo />", Some(serde_json::json!(["always"]))),
        (r#"<App foo = "bar" />"#, Some(serde_json::json!(["always"]))),
        ("<App foo = {e => bar(e)} />", Some(serde_json::json!(["always"]))),
        ("<App {...props} />", Some(serde_json::json!(["always"]))),
        ("<App foo/** 123123 */= /** 123 */ {2} />", None),
        ("<App foo/** 123123 */= {bar} bar = {baz} />", Some(serde_json::json!(["always"]))),
    ];

    let fail = vec![
        ("<App foo ={bar} />", Some(serde_json::json!(["never"]))),
        ("<App foo= {bar} />", Some(serde_json::json!(["never"]))),
        ("<App foo= {bar} bar = {baz} />", Some(serde_json::json!(["never"]))),
        ("<App foo ={bar} />", Some(serde_json::json!(["always"]))),
        ("<App foo= {bar} />", Some(serde_json::json!(["always"]))),
        ("<App foo={bar} bar ={baz} />", Some(serde_json::json!(["always"]))),
    ];

    let fix = vec![
        ("<App foo ={bar} />", "<App foo={bar} />", Some(serde_json::json!(["never"]))),
        ("<App foo= {bar} />", "<App foo={bar} />", Some(serde_json::json!(["never"]))),
        ("<App foo=        {bar} />", "<App foo={bar} />", Some(serde_json::json!(["never"]))),
        ("<App name = {bar} />", "<App name={bar} />", Some(serde_json::json!(["never"]))),
        ("<App foo ={bar} />", "<App foo = {bar} />", Some(serde_json::json!(["always"]))),
        ("<App foo= {bar} />", "<App foo = {bar} />", Some(serde_json::json!(["always"]))),
        (
            "<App foo={bar} bar ={baz} />",
            "<App foo = {bar} bar = {baz} />",
            Some(serde_json::json!(["always"])),
        ),
        (
            "<App foo= {bar} bar = {baz} />",
            "<App foo={bar} bar={baz} />",
            Some(serde_json::json!(["never"])),
        ),
    ];
    Tester::new(JsxEqualsSpacing::NAME, JsxEqualsSpacing::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
