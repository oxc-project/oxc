use cow_utils::CowUtils;
use lazy_regex::Regex;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::deserialize_required_regex_option,
};

fn comment(ts_comment_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not use @ts-{ts_comment_name} because it alters compilation errors."
    ))
    .with_help(format!("Remove the @ts-{ts_comment_name} directive and fix the underlying TypeScript error instead. If you must suppress an error, consider using @ts-expect-error with a descriptive comment explaining why it's necessary."))
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
    .with_help(format!("Add a description after @ts-{ts_comment_name} that is at least {min_len} characters long, explaining why the directive is necessary. For example: `// @ts-{ts_comment_name}: TS2345 - This is a known limitation with third-party types`"))
    .with_note("Requiring descriptions ensures that developers document why they're suppressing TypeScript errors, making it easier for future maintainers to understand the context and decide if the suppression is still necessary.")
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
    .with_help(format!("Update the description after @ts-{ts_comment_name} to match the required pattern: {pattern}."))
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct BanTsComment(Box<BanTsCommentConfig>);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
/// This rule allows you to specify how different TypeScript directive comments
/// should be handled.
///
/// For each directive (`@ts-expect-error`, `@ts-ignore`, `@ts-nocheck`, `@ts-check`), you can choose one of the following options:
/// - `true`: Disallow the directive entirely, preventing its use in the entire codebase.
/// - `false`: Allow the directive without any restrictions.
/// - `"allow-with-description"`: Allow the directive only if it is followed by a description explaining its use. The description must meet the minimum length specified by `minimumDescriptionLength`.
/// - `{ "descriptionFormat": "<regex>" }`: Allow the directive only if the description matches the specified regex pattern.
///
/// For example:
/// ```json
/// {
///   "ts-expect-error": "allow-with-description",
///   "ts-ignore": true,
///   "ts-nocheck": { "descriptionFormat": "^: TS\\d+ because .+$" },
///   "ts-check": false,
///   "minimumDescriptionLength": 3
/// }
/// ```
pub struct BanTsCommentConfig {
    /// How to handle the `@ts-expect-error` directive.
    #[schemars(with = "DirectiveConfigSchema")]
    ts_expect_error: DirectiveConfig,
    /// How to handle the `@ts-ignore` directive.
    #[schemars(with = "DirectiveConfigSchema")]
    ts_ignore: DirectiveConfig,
    /// How to handle the `@ts-nocheck` directive.
    #[schemars(with = "DirectiveConfigSchema")]
    ts_nocheck: DirectiveConfig,
    /// How to handle the `@ts-check` directive.
    #[schemars(with = "DirectiveConfigSchema")]
    ts_check: DirectiveConfig,
    /// Minimum description length required when using directives with `allow-with-description`.
    #[serde(rename = "minimumDescriptionLength")]
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

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DirectiveConfig {
    Boolean(bool),
    #[serde(rename = "allow-with-description")]
    RequireDescription,
    DescriptionFormat(Option<Regex>),
}

#[derive(Debug, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[expect(unused)]
enum DirectiveConfigSchema {
    Boolean(bool),
    RequireDescription(RequireDescription),
    DescriptionFormat(DescriptionFormatConfig),
}

#[derive(Debug, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[expect(unused)]
enum RequireDescription {
    AllowWithDescription,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DescriptionFormatConfig {
    #[serde(
        default,
        rename = "descriptionFormat",
        deserialize_with = "deserialize_required_regex_option"
    )]
    description_format: Option<Regex>,
}

impl<'de> Deserialize<'de> for DirectiveConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::Bool(value) => Ok(Self::Boolean(value)),
            serde_json::Value::String(value) if value == "allow-with-description" => {
                Ok(Self::RequireDescription)
            }
            value @ serde_json::Value::Object(_) => DescriptionFormatConfig::deserialize(value)
                .map(|config| Self::DescriptionFormat(config.description_format))
                .map_err(D::Error::custom),
            _ => Err(D::Error::custom(
                "expected a boolean, `allow-with-description`, or a descriptionFormat object",
            )),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule lets you set which directive comments you want to allow in your codebase.
    ///
    /// ### Why is this bad?
    ///
    /// Using TypeScript directives to suppress TypeScript compiler errors
    /// reduces the effectiveness of TypeScript overall.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// if (false) {
    ///   // @ts-ignore: Unreachable code error
    ///   console.log('hello');
    /// }
    /// ```
    BanTsComment,
    typescript,
    pedantic,
    conditional_fix,
    config = BanTsCommentConfig,
    version = "0.0.8",
    short_description = "This rule lets you set which directive comments you want to allow in your codebase.",
);

impl Rule for BanTsComment {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.comments();
        for comm in comments {
            let raw = ctx.source_range(comm.content_span());
            if let Some(captures) = find_ts_comment_directive(raw, comm.is_line()) {
                // safe to unwrap, if capture success, it can always capture one of the four directives
                let (directive, description) = (captures.0, captures.1);
                if comm.is_block() && (directive == "check" || directive == "nocheck") {
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
                                    ignore_instead_of_expect_error(comm.content_span()),
                                    |fixer| {
                                        fixer.replace(
                                            comm.content_span(),
                                            raw.cow_replace("@ts-ignore", "@ts-expect-error")
                                                .into_owned(),
                                        )
                                    },
                                );
                            } else {
                                ctx.diagnostic(comment(directive, comm.content_span()));
                            }
                        }
                    }
                    config => {
                        let description_len = description.trim().len();
                        if (description_len as u64) < self.minimum_description_length {
                            ctx.diagnostic(comment_requires_description(
                                directive,
                                self.minimum_description_length,
                                comm.content_span(),
                            ));
                        }

                        if let DirectiveConfig::DescriptionFormat(Some(re)) = config
                            && !re.is_match(description)
                        {
                            ctx.diagnostic(comment_description_not_match_pattern(
                                directive,
                                re.as_str(),
                                comm.content_span(),
                            ));
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

    if !raw.contains(prefix) {
        return None;
    }

    let multi_len = if single_line { 0 } else { raw.rfind('\n').map_or(0, |i| i + 1) };
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
        ("/* @ts-ignore */", r"/* @ts-expect-error */"),
        ("// @ts-ignore: TS1234 because xyz", r"// @ts-expect-error: TS1234 because xyz"),
        ("// @ts-ignore: TS1234", r"// @ts-expect-error: TS1234"),
        ("// @ts-ignore    : TS1234 because xyz", r"// @ts-expect-error    : TS1234 because xyz"),
    ];

    Tester::new(BanTsComment::NAME, BanTsComment::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn invalid_description_format_is_rejected() {
    let result = BanTsComment::from_configuration(serde_json::json!([{
        "ts-expect-error": {
            "descriptionFormat": "^(unclosed",
        },
    }]));

    let error = result.expect_err("invalid descriptionFormat should be rejected");
    assert!(error.to_string().contains("regex parse error"), "unexpected error: {error}");
}
