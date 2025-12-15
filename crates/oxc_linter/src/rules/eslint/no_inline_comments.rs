use lazy_regex::Regex;
use oxc_ast::{AstKind, Comment};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;

use crate::{context::LintContext, rule::Rule};

fn no_inline_comments_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected comment inline with code")
        .with_help("Move the comment to a separate line")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInlineComments(Box<NoInlineCommentsConfig>);

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoInlineCommentsConfig {
    /// A regex pattern to ignore certain inline comments.
    ///
    /// Comments matching this pattern will not be reported.
    ///
    /// Example configuration:
    /// ```json
    /// {
    ///     "no-inline-comments": ["error", { "ignorePattern": "webpackChunkName" }]
    /// }
    /// ```
    #[schemars(skip)]
    ignore_pattern: Option<Regex>,
}

impl std::ops::Deref for NoInlineComments {
    type Target = NoInlineCommentsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows comments on the same line as code.
    ///
    /// ### Why is this bad?
    ///
    /// Comments placed at the end of a line of code can make code harder to read.
    /// They can easily be missed when scanning vertically, and they make lines longer.
    /// Moving comments to their own lines makes them more prominent and reduces line length.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var a = 1; // inline comment
    /// var b = 2; /* another inline comment */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // comment on its own line
    /// var a = 1;
    ///
    /// /* block comment on its own line */
    /// var b = 2;
    /// ```
    NoInlineComments,
    eslint,
    pedantic,
    config = NoInlineCommentsConfig
);

impl Rule for NoInlineComments {
    fn from_configuration(value: serde_json::Value) -> Self {
        let ignore_pattern = value
            .get(0)
            .and_then(|v| v.get("ignorePattern"))
            .and_then(|v| v.as_str())
            .and_then(|s| Regex::new(s).ok());

        Self(Box::new(NoInlineCommentsConfig { ignore_pattern }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let source_text = ctx.source_text();

        // Pre-collect JSXEmptyExpression spans for efficient lookup
        // Using Vec since JSXEmptyExpression nodes are typically few, and we need range containment checks
        let jsx_empty_expr_spans: Vec<Span> = ctx
            .nodes()
            .iter()
            .filter_map(|node| {
                if let AstKind::JSXEmptyExpression(expr) = node.kind() {
                    Some(expr.span)
                } else {
                    None
                }
            })
            .collect();

        for comment in ctx.semantic().comments() {
            let comment_span = comment.span;
            let comment_text = ctx.source_range(comment.content_span());

            // Check if this matches the ignorePattern
            if let Some(ref pattern) = self.ignore_pattern
                && pattern.is_match(comment_text)
            {
                continue;
            }

            // Check if this is an eslint directive comment (eslint-disable, eslint-disable-line, etc.)
            if is_directive_comment(comment, comment_text) {
                continue;
            }

            // Get the content before and after the comment on its line(s)
            let (preamble, postamble) = get_comment_context(source_text, comment_span);

            // A comment is inline if there's code on the same line before or after it
            let is_inline = !preamble.is_empty() || !postamble.is_empty();

            if !is_inline {
                continue;
            }

            // Check for JSX empty expression exception
            // Comments inside {/* comment */} are allowed if they're the only content
            if is_jsx_expression_comment(&jsx_empty_expr_spans, comment_span, preamble, postamble) {
                continue;
            }

            ctx.diagnostic(no_inline_comments_diagnostic(comment_span));
        }
    }
}

/// Checks if a comment text is an ESLint/OxLint directive or special comment
fn is_directive_comment(comment: &Comment, text: &str) -> bool {
    let trimmed = text.trim();

    // eslint-disable, eslint-disable-line, eslint-disable-next-line
    // oxlint-disable, oxlint-disable-line, oxlint-disable-next-line
    // eslint-enable, oxlint-enable
    if trimmed.starts_with("eslint-") || trimmed.starts_with("oxlint-") {
        return true;
    }

    // Block comments only: /* eslint ... */ (ESLint config directive with space)
    // This matches ESLint's ESLINT_DIRECTIVE_PATTERN which accepts "eslint " or "eslint-"
    if comment.is_block()
        && (trimmed.starts_with("eslint ")
            || trimmed.starts_with("eslint\t")
            || trimmed.starts_with("oxlint ")
            || trimmed.starts_with("oxlint\t"))
    {
        return true;
    }

    // Block comments only: global, globals, exported (ESLint only recognizes these in block comments)
    if comment.is_block()
        && (trimmed.starts_with("global ")
            || trimmed.starts_with("global\t")
            || trimmed.starts_with("globals ")
            || trimmed.starts_with("globals\t")
            || trimmed.starts_with("exported ")
            || trimmed.starts_with("exported\t"))
    {
        return true;
    }

    false
}

/// Returns (preamble, postamble) - the non-whitespace content before and after the comment on its lines
fn get_comment_context(source_text: &str, comment_span: Span) -> (&str, &str) {
    let start = comment_span.start as usize;
    let end = comment_span.end as usize;

    // Find the start of the line containing the comment start
    let line_start = source_text[..start].rfind('\n').map_or(0, |i| i + 1);

    // Find the end of the line containing the comment end
    let line_end = source_text[end..].find('\n').map_or(source_text.len(), |i| end + i);

    // Get content before the comment on its starting line
    let preamble = source_text[line_start..start].trim();

    // Get content after the comment on its ending line
    let postamble = source_text[end..line_end].trim();

    (preamble, postamble)
}

/// Checks if a comment is inside a JSX expression container and is the only content
/// This allows patterns like: `{/* comment */}` or `{ /* comment */ }`
/// But NOT: `{/* comment */}</div>` (where `</div>` is on the same line)
fn is_jsx_expression_comment(
    jsx_empty_expr_spans: &[Span],
    comment_span: Span,
    preamble: &str,
    postamble: &str,
) -> bool {
    // For JSX expression comments to be allowed:
    // - preamble should be empty or just "{" (not ending with other content before {)
    // - postamble should be empty or just "}" (not having content after })
    // This means `{/* comment */}` alone on a line is OK, but `{/* comment */}</div>` is not

    // Check preamble: should be empty or end with just "{"
    let preamble_valid = preamble.is_empty() || preamble == "{";

    // Check postamble: should be empty or be just "}"
    let postamble_valid = postamble.is_empty() || postamble == "}";

    if !preamble_valid || !postamble_valid {
        return false;
    }

    // Check if the comment is inside a JSXEmptyExpression using pre-collected spans
    jsx_empty_expr_spans.iter().any(|expr_span| expr_span.contains_inclusive(comment_span))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "// A valid comment before code
			var a = 1;",
            None,
        ),
        (
            "var a = 2;
			// A valid comment after code",
            None,
        ),
        ("// A solitary comment", None),
        ("var a = 1; // eslint-disable-line no-debugger", None),
        ("var a = 1; /* eslint-disable-line no-debugger */", None),
        // ESLint config directive with space (/* eslint ... */)
        (r#"var a = 1; /* eslint no-console: "off" */"#, None),
        (r"var a = 1; /* eslint-env node */", None),
        // global/globals/exported only in block comments
        ("foo(); /* global foo */", None),
        ("foo(); /* globals foo */", None),
        ("var foo; /* exported foo */", None),
        (
            "var a = (
			            <div>
			            {/*comment*/}
			            </div>
			        )",
            None,
        ),
        (
            "var a = (
			            <div>
			            { /* comment */ }
			            <h1>Some heading</h1>
			            </div>
			        )",
            None,
        ),
        (
            "var a = (
			            <div>
			            {// comment
			            }
			            </div>
			        )",
            None,
        ),
        (
            "var a = (
			            <div>
			            { // comment
			            }
			            </div>
			        )",
            None,
        ),
        (
            "var a = (
			            <div>
			            {/* comment 1 */
			            /* comment 2 */}
			            </div>
			        )",
            None,
        ),
        (
            "var a = (
			            <div>
			            {/*
			              * comment 1
			              */
			             /*
			              * comment 2
			              */}
			            </div>
			        )",
            None,
        ),
        (
            "var a = (
			            <div>
			            {/*
			               multi
			               line
			               comment
			            */}
			            </div>
			        )",
            None,
        ),
        (
            r#"import(/* webpackChunkName: "my-chunk-name" */ './locale/en');"#,
            Some(serde_json::json!([{ "ignorePattern": "(?:webpackChunkName):\\s.+" }])),
        ),
        (
            "var foo = 2; // Note: This comment is legal.",
            Some(serde_json::json!([{ "ignorePattern": "Note:" }])),
        ),
    ];

    let fail = vec![
        ("var a = 1; /*A block comment inline after code*/", None),
        ("/*A block comment inline before code*/ var a = 2;", None),
        // Line comments with global/globals/exported are NOT directive (only block comments are)
        ("foo(); // global foo", None),
        ("foo(); // globals foo", None),
        ("var foo; // exported foo", None),
        ("/* something */ var a = 2;", Some(serde_json::json!([{ "ignorePattern": "otherthing" }]))),
        ("var a = 3; //A comment inline with code", None),
        ("var a = 3; // someday use eslint-disable-line here", None),
        ("var a = 3; // other line comment", Some(serde_json::json!([{ "ignorePattern": "something" }]))),
        ("var a = 4;
			/**A
			 * block
			 * comment
			 * inline
			 * between
			 * code*/ var foo = a;", None),
("var a =
			{/**/}", None),
("var a = (
			                <div>{/* comment */}</div>
			            )", None),
("var a = (
			                <div>{// comment
			                }
			                </div>
			            )", None),
("var a = (
			                <div>{/* comment */
			                }
			                </div>
			            )", None),
("var a = (
			                <div>{/*
			                       * comment
			                       */
			                }
			                </div>
			            )", None),
("var a = (
			                <div>{/*
			                       * comment
			                       */}
			                </div>
			            )", None),
("var a = (
			                <div>{/*
			                       * comment
			                       */}</div>
			            )", None),
("var a = (
			                <div>
			                {/*
			                  * comment
			                  */}</div>
			            )", None),
("var a = (
			                <div>
			                {
			                 /*
			                  * comment
			                  */}</div>
			            )", None),
("var a = (
			                <div>
			                {
			                /* comment */}</div>
			            )", None),
("var a = (
			                <div>
			                {b/* comment */}
			                </div>
			            )", None),
("var a = (
			                <div>
			                {/* comment */b}
			                </div>
			            )", None),
("var a = (
			                <div>
			                {// comment
			                    b
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {/* comment */
			                    b
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {/*
			                  * comment
			                  */
			                    b
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    b// comment
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    /* comment */b
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    b/* comment */
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    b
			                /*
			                 * comment
			                 */}
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    b
			                /* comment */}
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    { /* this is an empty object literal, not braces for js code! */ }
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    {// comment
			                    }
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    {
			                    /* comment */}
			                }
			                </div>
			            )", None),
("var a = (
			                <div>
			                { /* two comments on the same line... */ /* ...are not allowed, same as with a non-JSX code */}
			                </div>
			            )", None),
("var a = (
			                <div>
			                {
			                    /* overlapping
			                    */ /*
			                       lines */
			                }
			                </div>
			            )", None)
    ];

    Tester::new(NoInlineComments::NAME, NoInlineComments::PLUGIN, pass, fail).test_and_snapshot();
}
