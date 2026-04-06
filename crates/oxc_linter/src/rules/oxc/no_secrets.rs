use lazy_regex::{Lazy, Regex, lazy_regex};
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const DEFAULT_TOLERANCE: f64 = 4.0;
const CHARSET: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+=!|*^@~`$%+?\"'_<>"; // keep aligned with eslint-plugin-no-secrets

fn high_entropy_diagnostic(span: Span, token: &str, entropy: f64) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Found a high-entropy string `{token}` ({entropy:.2})."))
        .with_help("Move the secret out of source code or replace it with a safe test value.")
        .with_label(span)
}

fn pattern_match_diagnostic(span: Span, name: &str, matched: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Found a string matching secret pattern `{name}`: `{matched}`."
    ))
    .with_help("Remove the embedded secret from source code or replace it with a redacted placeholder.")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoSecrets(Box<NoSecretsConfig>);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NoSecretsConfig {
    tolerance: f64,
    ignore_modules: bool,
}

impl Default for NoSecretsConfig {
    fn default() -> Self {
        Self { tolerance: DEFAULT_TOLERANCE, ignore_modules: true }
    }
}

impl std::ops::Deref for NoSecrets {
    type Target = NoSecretsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects likely hardcoded secrets in string literals, template text, and comments.
    ///
    /// ### Why is this bad?
    ///
    /// Credentials, tokens, and private keys committed to source code are a
    /// high-risk class of security issue and are difficult to fully remediate
    /// once leaked.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const awsKey = "AKIA1234567890ABCDEF";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const awsKey = process.env.AWS_ACCESS_KEY_ID;
    /// ```
    NoSecrets,
    oxc,
    suspicious,
    none,
    config = NoSecretsConfig
);

impl Rule for NoSecrets {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StringLiteral(string_literal) => {
                if self.ignore_modules && is_module_path_string(node.id(), string_literal.span, ctx)
                {
                    return;
                }
                check_text(string_literal.span, string_literal.value.as_str(), self.tolerance, ctx);
            }
            AstKind::TemplateElement(template_element) => {
                if let Some(cooked) = template_element.value.cooked {
                    check_text(template_element.span, cooked.as_str(), self.tolerance, ctx);
                }
            }
            _ => {}
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        for comment in ctx.comments() {
            let span = comment.content_span();
            check_text(span, ctx.source_range(span), self.tolerance, ctx);
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
    }
}

fn check_text(span: Span, value: &str, tolerance: f64, ctx: &LintContext<'_>) {
    if value.is_empty() {
        return;
    }

    let mut matched_pattern = false;
    for (name, regex) in patterns() {
        if let Some(found) = regex.find(value) {
            matched_pattern = true;
            ctx.diagnostic(pattern_match_diagnostic(span, name, found.as_str()));
        }
    }

    if matched_pattern {
        return;
    }

    for (token, entropy) in high_entropy_tokens(value, tolerance) {
        ctx.diagnostic(high_entropy_diagnostic(span, token, entropy));
    }
}

fn high_entropy_tokens(value: &str, tolerance: f64) -> Vec<(&str, f64)> {
    value
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .filter_map(|token| {
            let entropy = shannon_entropy(token);
            (entropy >= tolerance).then_some((token, entropy))
        })
        .collect()
}

fn shannon_entropy(value: &str) -> f64 {
    if value.is_empty() {
        return 0.0;
    }

    let length = value.len() as f64;
    let mut entropy = 0.0;

    for character in CHARSET.chars() {
        let count = value.chars().filter(|candidate| *candidate == character).count() as f64;
        if count > 0.0 {
            let ratio = count / length;
            entropy += -(ratio * ratio.log2());
        }
    }

    entropy
}

fn is_module_path_string(
    node_id: oxc_syntax::node::NodeId,
    span: Span,
    ctx: &LintContext<'_>,
) -> bool {
    match ctx.nodes().parent_kind(node_id) {
        AstKind::ImportDeclaration(_) => true,
        AstKind::ImportExpression(import_expr) => import_expr.source.span() == span,
        AstKind::CallExpression(call_expr) => {
            matches!(&call_expr.callee, oxc_ast::ast::Expression::Identifier(ident) if ident.name == "require")
                && call_expr
                    .arguments
                    .first()
                    .and_then(oxc_ast::ast::Argument::as_expression)
                    .is_some_and(|expression| expression.span() == span)
        }
        _ => false,
    }
}

fn patterns() -> &'static [(&'static str, &'static Lazy<Regex>)] {
    static PATTERNS: [(&str, &Lazy<Regex>); 11] = [
        ("Slack Token", &SLACK_TOKEN),
        ("RSA private key", &RSA_PRIVATE_KEY),
        ("SSH (OPENSSH) private key", &OPENSSH_PRIVATE_KEY),
        ("SSH (DSA) private key", &DSA_PRIVATE_KEY),
        ("SSH (EC) private key", &EC_PRIVATE_KEY),
        ("PGP private key block", &PGP_PRIVATE_KEY),
        ("AWS API Key", &AWS_API_KEY),
        ("Slack Webhook", &SLACK_WEBHOOK),
        ("Google (GCP) Service-account", &GCP_SERVICE_ACCOUNT),
        ("Twilio API Key", &TWILIO_API_KEY),
        ("Password in URL", &PASSWORD_IN_URL),
    ];

    &PATTERNS
}

static SLACK_TOKEN: Lazy<Regex> =
    lazy_regex!(r"(xox[p|b|o|a]-[0-9]{12}-[0-9]{12}-[0-9]{12}-[a-z0-9]{32})");
static RSA_PRIVATE_KEY: Lazy<Regex> = lazy_regex!(r"-----BEGIN RSA PRIVATE KEY-----");
static OPENSSH_PRIVATE_KEY: Lazy<Regex> = lazy_regex!(r"-----BEGIN OPENSSH PRIVATE KEY-----");
static DSA_PRIVATE_KEY: Lazy<Regex> = lazy_regex!(r"-----BEGIN DSA PRIVATE KEY-----");
static EC_PRIVATE_KEY: Lazy<Regex> = lazy_regex!(r"-----BEGIN EC PRIVATE KEY-----");
static PGP_PRIVATE_KEY: Lazy<Regex> = lazy_regex!(r"-----BEGIN PGP PRIVATE KEY BLOCK-----");
static AWS_API_KEY: Lazy<Regex> = lazy_regex!(r"AKIA[0-9A-Z]{16}");
static SLACK_WEBHOOK: Lazy<Regex> = lazy_regex!(
    r"https://hooks\.slack\.com/services/T[a-zA-Z0-9_]{8}/B[a-zA-Z0-9_]{8}/[a-zA-Z0-9_]{24}"
);
static GCP_SERVICE_ACCOUNT: Lazy<Regex> = lazy_regex!(r#""type": "service_account""#);
static TWILIO_API_KEY: Lazy<Regex> = lazy_regex!(r"SK[a-z0-9]{32}");
static PASSWORD_IN_URL: Lazy<Regex> =
    lazy_regex!(r#"[a-zA-Z]{3,10}://[^/\s:@]{3,20}:[^/\s:@]{3,20}@.{1,100}["'\s]"#);

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("const greeting = 'hello world';", None),
        ("const path = require('fs');", None),
        ("import x from './local-module';", None),
        ("const module = import('./dynamic-module');", None),
        ("const tmpl = `hello ${name}`;", None),
        ("// just a normal comment", None),
        ("const lowEntropy = 'aaaaaaaaaaaa';", Some(json!([{ "tolerance": 4.5 }]))),
    ];

    let fail = vec![
        ("const awsKey = 'AKIA1234567890ABCDEF';", None),
        (
            "const token = 'ZWVTjPQSdhwRgl204Hc51YCsritMIzn8B=/p9UyeX7xu6KkAGqfm3FJ+oObLDNEva';",
            Some(json!([{ "tolerance": 4.5 }])),
        ),
        (
            "const key = `https://hooks.slack.com/services/T12345678/B12345678/abcdefghijklmnopqrstuvwx`;",
            None,
        ),
        ("// -----BEGIN RSA PRIVATE KEY-----", None),
    ];

    Tester::new(NoSecrets::NAME, NoSecrets::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
