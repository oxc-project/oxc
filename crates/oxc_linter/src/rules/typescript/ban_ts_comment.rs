use cow_utils::CowUtils;
use oxc_ast::CommentKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn comment(ts_comment_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not use @ts-{ts_comment_name} because it alters compilation errors."
    ))
    .with_label(span)
}

fn ignore_instead_of_expect_error(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use \"@ts-expect-error\" instead of @ts-ignore, as \"@ts-ignore\" will do nothing if the following line is error-free.")
        .with_help("Replace \"@ts-ignore\" with \"@ts-expect-error\".")
        .with_label(span)
}

fn comment_requires_description(ts_comment_name: &str, min_len: u64, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Include a description after the @ts-{ts_comment_name} directive to explain why the @ts-{ts_comment_name} is necessary. The description must be {min_len} characters or longer."
    ))
    .with_label(span)
}

fn comment_description_not_match_pattern(
    ts_comment_name: &str,
    pattern: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The description for the @ts-{ts_comment_name} directive must match the {pattern} format."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BanTsComment(Box<BanTsCommentConfig>);

#[derive(Debug, Clone)]
pub struct BanTsCommentConfig {
    ts_expect_error: DirectiveConfig,
    ts_ignore: DirectiveConfig,
    ts_nocheck: DirectiveConfig,
    ts_check: DirectiveConfig,
    minimum_description_length: u64,
}

impl std::ops::Deref for BanTsComment {
    type Target = BanTsCommentConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for BanTsCommentConfig {
    fn default() -> Self {
        Self {
            ts_expect_error: DirectiveConfig::RequireDescription,
            ts_ignore: DirectiveConfig::Boolean(true),
            ts_nocheck: DirectiveConfig::Boolean(true),
            ts_check: DirectiveConfig::Boolean(false),
            minimum_description_length: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DirectiveConfig {
    Boolean(bool),
    RequireDescription,
    DescriptionFormat(Option<Regex>),
}

impl DirectiveConfig {
    fn from_json(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::Bool(b) => Some(Self::Boolean(*b)),
            serde_json::Value::String(s) => {
                if s == "allow-with-description" {
                    Some(Self::RequireDescription)
                } else {
                    None
                }
            }
            serde_json::Value::Object(o) => {
                let re = o
                    .get("descriptionFormat")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|pattern| Regex::new(pattern).ok());
                Some(Self::DescriptionFormat(re))
            }
            _ => None,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule lets you set which directive comments you want to allow in your codebase.
    ///
    /// ### Why is this bad?
    /// Using TypeScript directives to suppress TypeScript compiler errors
    /// reduces the effectiveness of TypeScript overall.
    ///
    /// ### Example
    /// ```ts
    /// if (false) {
    ///   // @ts-ignore: Unreachable code error
    ///   console.log('hello');
    /// }
    /// ```
    BanTsComment,
    pedantic,
    conditional_fix
);

impl Rule for BanTsComment {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(BanTsCommentConfig {
            ts_expect_error: value
                .get(0)
                .and_then(|x| x.get("ts-expect-error"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::RequireDescription),
            ts_ignore: value
                .get(0)
                .and_then(|x| x.get("ts-ignore"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::Boolean(true)),
            ts_nocheck: value
                .get(0)
                .and_then(|x| x.get("ts-nocheck"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::Boolean(true)),
            ts_check: value
                .get(0)
                .and_then(|x| x.get("ts-check"))
                .and_then(DirectiveConfig::from_json)
                .unwrap_or(DirectiveConfig::Boolean(false)),
            minimum_description_length: value
                .get(0)
                .and_then(|x| x.get("minimumDescriptionLength"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(3),
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.semantic().comments();
        for comm in comments {
            let raw = ctx.source_range(comm.span);
            if let Some(captures) = find_ts_comment_directive(raw, comm.is_line()) {
                // safe to unwrap, if capture success, it can always capture one of the four directives
                let (directive, description) = (captures.0, captures.1);
                if CommentKind::Block == comm.kind
                    && (directive == "check" || directive == "nocheck")
                {
                    continue;
                }

                if raw.trim_start().starts_with('/')
                    && (directive == "check" || directive == "nocheck")
                {
                    continue;
                }

                match self.option(directive) {
                    DirectiveConfig::Boolean(on) => {
                        if *on {
                            if directive == "ignore" {
                                ctx.diagnostic_with_fix(
                                    ignore_instead_of_expect_error(comm.span),
                                    |fixer| {
                                        fixer.replace(
                                            comm.span,
                                            raw.cow_replace("@ts-ignore", "@ts-expect-error"),
                                        )
                                    },
                                );
                            } else {
                                ctx.diagnostic(comment(directive, comm.span));
                            }
                        }
                    }
                    config => {
                        let description_len = description.trim().len();
                        if (description_len as u64) < self.minimum_description_length {
                            ctx.diagnostic(comment_requires_description(
                                directive,
                                self.minimum_description_length,
                                comm.span,
                            ));
                        }

                        if let DirectiveConfig::DescriptionFormat(Some(ref re)) = config {
                            if !re.is_match(description) {
                                ctx.diagnostic(comment_description_not_match_pattern(
                                    directive,
                                    re.as_str(),
                                    comm.span,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl BanTsComment {
    /// get the option for a given directive, caller should guarantee
    /// the directive should be one of the ignore/check/nocheck/expect-error
    fn option(&self, directive: &str) -> &DirectiveConfig {
        match directive {
            "ignore" => &self.ts_ignore,
            "check" => &self.ts_check,
            "nocheck" => &self.ts_nocheck,
            "expect-error" => &self.ts_expect_error,
            _ => {
                unreachable!(
                    "Expected one of ignore/check/nocheck/expect-error, got {}.",
                    directive,
                );
            }
        }
    }
}

pub fn find_ts_comment_directive(raw: &str, single_line: bool) -> Option<(&str, &str)> {
    let prefix = "@ts-";

    let mut last_line_start = None;
    let mut char_indices = raw.char_indices().peekable();
    while let Some((_, c)) = char_indices.next() {
        if c == '\n' {
            last_line_start = char_indices.peek().map(|(i, _)| *i);
        }
    }

    let multi_len = last_line_start.unwrap_or(0);
    let line = &raw[multi_len..];

    // Check the content before the prefix
    let index = line.find(prefix)?;
    if !line[..index]
        .chars()
        .all(|c| c.is_whitespace() || if single_line { c == '/' } else { c == '*' || c == '/' })
    {
        return None;
    }

    let start = index + prefix.len();
    for directive in ["expect-error", "ignore", "nocheck", "check"] {
        if line.get(start..start + directive.len()) == Some(directive) {
            let start = multi_len + index + prefix.len();
            let end = start + directive.len();
            let (directive, description) = (&raw[start..end], &raw[end..]);

            debug_assert!(
                matches!(directive, "expect-error" | "ignore" | "nocheck" | "check"),
                "Expected one of ignore/check/nocheck/expect-error, got {directive}",
            );

            return Some((directive, description));
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;
    // A total of 51 test cases passed successfully.
    let pass = vec![
        // ts-expect-error
        ("// just a comment containing @ts-expect-error somewhere", None),
        (
            r"
            /*
            @ts-expect-error running with long description in a block
            */
		",
            None,
        ),
        (
            r"
            /* @ts-expect-error not on the last line
            */
        ",
            None,
        ),
        (
            r"
            /**
             * @ts-expect-error not on the last line
             */
        ",
            None,
        ),
        (
            r"
            /* not on the last line
            * @ts-expect-error
            */
        ",
            None,
        ),
        (
            r"
            /* @ts-expect-error
            * not on the last line */
        ",
            None,
        ),
        ("// @ts-expect-error", Some(serde_json::json!([{ "ts-expect-error": false }]))),
        (
            "// @ts-expect-error here is why the error is expected",
            Some(serde_json::json!([{"ts-expect-error": "allow-with-description"},])),
        ),
        (
            r"
            /*
            * @ts-expect-error here is why the error is expected */
        ",
            Some(serde_json::json!([{"ts-expect-error": "allow-with-description"},])),
        ),
        (
            "// @ts-expect-error exactly 21 characters",
            Some(serde_json::json!([
              {
                "ts-expect-error": "allow-with-description",
                "minimumDescriptionLength": 21,
              },
            ])),
        ),
        (
            r"
            /*
            * @ts-expect-error exactly 21 characters*/
        ",
            Some(serde_json::json!([{
                "ts-expect-error": "allow-with-description",
                "minimumDescriptionLength": 21,
            }])),
        ),
        (
            "// @ts-expect-error: TS1234 because xyz",
            Some(serde_json::json!([
                 {
                     "ts-expect-error": {
                         "descriptionFormat": "^: TS\\d+ because .+$",
                     },
                     "minimumDescriptionLength" : 10,
                 },
            ])),
        ),
        (
            r"
            /*
            * @ts-expect-error: TS1234 because xyz */
        ",
            Some(serde_json::json!([
                 {
                     "ts-expect-error": {
                         "descriptionFormat": "^: TS\\d+ because .+$",
                     },
                     "minimumDescriptionLength" : 10,
                 },
            ])),
        ),
        (
            "// @ts-expect-error 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦",
            Some(serde_json::json!([{ "ts-expect-error": "allow-with-description" }])),
        ),
        // ts-ignore
        ("// just a comment containing @ts-ignore somewhere", None),
        ("// @ts-ignore", Some(serde_json::json!([{ "ts-ignore": false}]))),
        (
            "// @ts-ignore I think that I am exempted from any need to follow the rules!",
            Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }])),
        ),
        (
            r"
         /*
          @ts-ignore running with long description in a block
         */
		",
            Some(serde_json::json!([
                {
                    "ts-ignore": "allow-with-description",
                    "minimumDescriptionLength": 21,
                },
            ])),
        ),
        (
            r"
            /*
             @ts-ignore
            */
        ",
            None,
        ),
        (
            r"
            /* @ts-ignore not on the last line
            */
        ",
            None,
        ),
        (
            r"
            /**
             * @ts-ignore not on the last line
             */
        ",
            None,
        ),
        (
            r"
            /* not on the last line
            * @ts-expect-error
            */
        ",
            None,
        ),
        (
            r"
            /* @ts-ignore
            * not on the last line */
        ",
            None,
        ),
        (
            "// @ts-ignore: TS1234 because xyz",
            Some(serde_json::json!([
                {
                    "ts-ignore": {
                        "descriptionFormat": "^: TS\\d+ because .+$",
                    },
                    "minimumDescriptionLength": 10,
                },
            ])),
        ),
        (
            "// @ts-ignore 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦",
            Some(serde_json::json!([
                {
                    "ts-ignore": "allow-with-description"
                },
            ])),
        ),
        (
            r"
            /*
            * @ts-ignore here is why the error is expected */
        ",
            Some(serde_json::json!([
                {
                    "ts-ignore": "allow-with-description"
                },
            ])),
        ),
        (
            "// @ts-ignore exactly 21 characters",
            Some(serde_json::json!([
                {
                    "ts-ignore": "allow-with-description",
                    "minimumDescriptionLength": 21,
                },
            ])),
        ),
        (
            r"
            /*
            * @ts-ignore exactly 21 characters*/
        ",
            Some(serde_json::json!([
                {
                    "ts-ignore": "allow-with-description",
                    "minimumDescriptionLength": 21,
                },
            ])),
        ),
        (
            r"
            /*
            * @ts-ignore: TS1234 because xyz */
        ",
            Some(serde_json::json!([
                {
                    "ts-ignore": {
                        "descriptionFormat": "^: TS\\d+ because .+$",
                    },
                    "minimumDescriptionLength": 10,
                },
            ])),
        ),
        // ts-nocheck
        ("// just a comment containing @ts-nocheck somewhere", None),
        ("// @ts-nocheck", Some(serde_json::json!([{ "ts-nocheck": false}]))),
        (
            "// @ts-nocheck no doubt, people will put nonsense here from time to time just to get the rule to stop reporting, perhaps even long messages with other nonsense in them like other // @ts-nocheck or // @ts-ignore things",
            Some(serde_json::json!([{ "ts-nocheck": "allow-with-description" }])),
        ),
        (
            r"
        /*
            @ts-nocheck running with long description in a block
        */",
            Some(serde_json::json!([
                {
                "ts-nocheck": "allow-with-description",
                "minimumDescriptionLength": 21,
                },
            ])),
        ),
        (
            "// @ts-nocheck: TS1234 because xyz",
            Some(serde_json::json!([
                {
                "ts-nocheck": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                },
                "minimumDescriptionLength": 10,
                },
            ])),
        ),
        (
            "// @ts-nocheck 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦",
            Some(serde_json::json!([
                {
                    "ts-nocheck": "allow-with-description",
                },
            ])),
        ),
        ("//// @ts-nocheck - pragma comments may contain 2 or 3 leading slashes", None),
        (
            r"
            /**
             @ts-nocheck
            */
        ",
            None,
        ),
        (
            r"
            /*
             @ts-nocheck
            */
        ",
            None,
        ),
        ("/** @ts-nocheck */", None),
        ("/* @ts-nocheck */", None),
        // ts-check
        ("// just a comment containing @ts-check somewhere", None),
        (
            r"
        /*
            @ts-check running with long description in a block
        */
        ",
            None,
        ),
        ("// @ts-check", Some(serde_json::json!([{ "ts-check": false}]))),
        (
            "// @ts-check with a description and also with a no-op // @ts-ignore",
            Some(serde_json::json!([
                {"ts-check": "allow-with-description", "minimumDescriptionLength": 3 },
            ])),
        ),
        (
            "// @ts-check: TS1234 because xyz",
            Some(serde_json::json!([
                {
                "ts-check": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                },
                "minimumDescriptionLength": 10,
                },
            ])),
        ),
        (
            "// @ts-check 👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦",
            Some(serde_json::json!([
                {
                    "ts-check": "allow-with-description",
                },
            ])),
        ),
        (
            "//// @ts-check - pragma comments may contain 2 or 3 leading slashes",
            Some(serde_json::json!([
                {
                    "ts-check": true,
                },
            ])),
        ),
        (
            r"
            /**
             @ts-check
            */
        ",
            Some(serde_json::json!([
                {
                    "ts-check": true,
                },
            ])),
        ),
        (
            r"
            /*
             @ts-check
            */
        ",
            Some(serde_json::json!([
                {
                    "ts-check": true,
                },
            ])),
        ),
        (
            "/** @ts-check */",
            Some(serde_json::json!([
                {
                    "ts-check": true,
                },
            ])),
        ),
        (
            "/* @ts-check */",
            Some(serde_json::json!([
                {
                    "ts-check": true,
                },
            ])),
        ),
    ];

    // A total of 57 test cases failed.
    let fail = vec![
        // ts-expect-error
        ("// @ts-expect-error", Some(serde_json::json!([{ "ts-expect-error": true }]))),
        ("/* @ts-expect-error */", Some(serde_json::json!([{ "ts-expect-error": true}]))),
        (
            r"
/*
 @ts-expect-error */
        ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
/** on the last line
 @ts-expect-error */
        ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
/** on the last line
 * @ts-expect-error */
        ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
/**
 * @ts-expect-error: TODO */
        ",
            Some(
                serde_json::json!([{ "ts-expect-error": "allow-with-description", "minimumDescriptionLength": 10}]),
            ),
        ),
        (
            r"
/**
 * @ts-expect-error: TS1234 because xyz */
        ",
            Some(serde_json::json!([{
            "ts-expect-error": {
                "descriptionFormat": "^: TS\\d+ because .+$",
              },
              "minimumDescriptionLength": 25
            }])),
        ),
        (
            r"
/**
 * @ts-expect-error: TS1234 */
        ",
            Some(serde_json::json!([{
            "ts-expect-error": {
                "descriptionFormat": "^: TS\\d+ because .+$",
              },
            }])),
        ),
        (
            r"
/**
 * @ts-expect-error    : TS1234 */
        ",
            Some(serde_json::json!([{
            "ts-expect-error": {
                "descriptionFormat": "^: TS\\d+ because .+$",
              },
            }])),
        ),
        ("/** @ts-expect-error */", Some(serde_json::json!([{ "ts-expect-error": true}]))),
        (
            "// @ts-expect-error: Suppress next line",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            "/////@ts-expect-error: Suppress next line",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            r"
if (false) {
    // @ts-expect-error: Unreachable code error
    console.log('hello');
}
          ",
            Some(serde_json::json!([{ "ts-expect-error": true}])),
        ),
        (
            "// @ts-expect-error",
            Some(serde_json::json!([
              {
                "ts-expect-error": "allow-with-description",
              },
            ])),
        ),
        (
            "// @ts-expect-error: TODO",
            Some(serde_json::json!([
              {
                "ts-expect-error": "allow-with-description",
                "minimumDescriptionLength": 10,
              },
            ])),
        ),
        (
            "// @ts-expect-error: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-expect-error": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength": 25,
              },
            ])),
        ),
        (
            "// @ts-expect-error: TS1234",
            Some(serde_json::json!([
              {
                "ts-expect-error": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-expect-error    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-expect-error": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        // ts-ignore
        (
            "// @ts-ignore",
            Some(serde_json::json!([{ "ts-ignore": true, "ts-expect-error": true }])),
        ),
        (
            "// @ts-ignore",
            Some(
                serde_json::json!([{ "ts-ignore": true, "ts-expect-error": "allow-with-description" }]),
            ),
        ),
        ("// @ts-ignore", None),
        ("/* @ts-ignore */", Some(serde_json::json!([{ "ts-ignore": true}]))),
        (
            r"
/*
 @ts-ignore */
            ",
            Some(serde_json::json!([{ "ts-ignore": true}])),
        ),
        (
            r"
/** on the last line
 @ts-ignore */
            ",
            Some(serde_json::json!([{ "ts-ignore": true}])),
        ),
        (
            r"
/** on the last line
 * @ts-ignore */
            ",
            Some(serde_json::json!([{ "ts-ignore": true}])),
        ),
        (
            "/** @ts-ignore */",
            Some(serde_json::json!([{ "ts-ignore": true, "ts-expect-error": false }])),
        ),
        (
            r"
/**
 * @ts-ignore: TODO */
            ",
            Some(
                serde_json::json!([{ "ts-expect-error": "allow-with-description", "minimumDescriptionLength": 10 }]),
            ),
        ),
        (
            r"
/**
 * @ts-ignore: TS1234 because xyz */
            ",
            Some(serde_json::json!([{
                "ts-expect-error": {
                    "descriptionFormat": "^: TS\\d+ because .+$",
                  },
                  "minimumDescriptionLength": 25
            }])),
        ),
        ("// @ts-ignore: Suppress next line", None),
        ("/////@ts-ignore: Suppress next line", None),
        (
            r"
if (false) {
    // @ts-ignore: Unreachable code error
    console.log('hello');
}
            ",
            None,
        ),
        ("// @ts-ignore", Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }]))),
        (
            "// @ts-ignore         ",
            Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }])),
        ),
        (
            "// @ts-ignore    .",
            Some(serde_json::json!([{ "ts-ignore": "allow-with-description" }])),
        ),
        (
            "// @ts-ignore: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-ignore": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength": 25,
              },
            ])),
        ),
        (
            "// @ts-ignore: TS1234",
            Some(serde_json::json!([
              {
                "ts-ignore": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-ignore    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-ignore": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        // ts-nocheck
        ("// @ts-nocheck", Some(serde_json::json!([{ "ts-nocheck": true}]))),
        ("// @ts-nocheck", None),
        ("// @ts-nocheck: Suppress next line", None),
        (
            r"
if (false) {
    // @ts-nocheck: Unreachable code error
    console.log('hello');
}
            ",
            None,
        ),
        ("// @ts-nocheck", Some(serde_json::json!([{ "ts-nocheck": "allow-with-description" }]))),
        (
            "// @ts-nocheck: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-nocheck": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength": 25,
              },
            ])),
        ),
        (
            "// @ts-nocheck: TS1234",
            Some(serde_json::json!([
              {
                "ts-nocheck": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-nocheck    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-nocheck": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        // ts-check
        ("// @ts-check", Some(serde_json::json!([{ "ts-check": true}]))),
        ("// @ts-check: Suppress next line", Some(serde_json::json!([{ "ts-check":true}]))),
        (
            r"
if (false) {
    // @ts-check: Unreachable code error
    console.log('hello');
}
            ",
            Some(serde_json::json!([{ "ts-check":true}])),
        ),
        ("// @ts-check", Some(serde_json::json!([{ "ts-check": "allow-with-description" }]))),
        (
            "// @ts-check: TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-check": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
               "minimumDescriptionLength" : 25,
              },
            ])),
        ),
        (
            "// @ts-check: TS1234",
            Some(serde_json::json!([
              {
                "ts-check": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
        (
            "// @ts-check    : TS1234 because xyz",
            Some(serde_json::json!([
              {
                "ts-check": {
                 "descriptionFormat": "^: TS\\d+ because .+$",
                },
              },
            ])),
        ),
    ];

    let fix = vec![
        ("// @ts-ignore", r"// @ts-expect-error"),
        ("// @ts-ignore: TS1234 because xyz", r"// @ts-expect-error: TS1234 because xyz"),
        ("// @ts-ignore: TS1234", r"// @ts-expect-error: TS1234"),
        ("// @ts-ignore    : TS1234 because xyz", r"// @ts-expect-error    : TS1234 because xyz"),
    ];

    Tester::new(BanTsComment::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
