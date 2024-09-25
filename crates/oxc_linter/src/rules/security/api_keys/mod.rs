mod entropy;
#[allow(unused_imports, unused_variables)]
mod secret;
mod secrets;

use std::{num::NonZeroU32, ops::Deref};

use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan};

use crate::{context::LintContext, rule::Rule, AstNode};
use entropy::Entropy;
use secret::{
    Secret, SecretScanner, SecretScannerMeta, SecretViolation, DEFAULT_MIN_ENTROPY, DEFAULT_MIN_LEN,
};
use secrets::{CustomSecret, SecretsEnum, ALL_RULES};

fn api_keys(violation: &SecretViolation) -> OxcDiagnostic {
    OxcDiagnostic::warn(violation.message().to_owned())
        .with_error_code_num(format!("api-keys/{}", violation.rule_name()))
        .with_label(violation.span())
        .with_help(
            "Use a secrets manager to store your API keys securely, then read them at runtime.",
        )
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows hard-coded API keys and other credentials.
    ///
    /// ### Why is this bad?
    ///
    /// Hard-coding API keys and committing them to source control is a serious
    /// security risk.
    ///
    /// 1. If your code is leaked, attackers can use your API keys to access your
    ///   services and data.
    /// 2. Accidental bundling of API keys can lead them to be exposed publicly
    ///    in your website, compriming your services.
    /// 3. Any developer or contractor you hire will have access to your
    ///    services, even after they lose access to your codebase.
    /// 4. Even after being deleted, they will be visible in your git repo's
    ///    commit history.
    /// 5. Key rotation requires a code change and redeployment, and can
    ///    therefore not be handled by security teams or by automated systems.
    /// 6. Many, many more reasons.
    ///
    /// ```ts
    /// const API_KEY = 'abcdef123456';
    /// const data = await fetch('/api/some/endpoint', {
    ///     headers: {
    ///         'Authorization': `Bearer ${API_KEY}`,
    ///     }
    /// });
    /// ```
    ///
    /// ### What To Do Instead
    /// :::warning
    /// The Oxc team are not security experts. We do not endorse any particular
    /// key management service or strategy. Do your research and choose the best
    /// solution/architecture for your use case.
    /// :::
    ///
    /// One possible alternative is to store secrets in a secure secrets manager
    /// (such as [AWS
    /// KMS](https://docs.aws.amazon.com/AWSJavaScriptSDK/v3/latest/client/kms/),
    /// [HashiCorp Vault](https://github.com/nodevault/node-vault/tree/master),
    /// [Pangea](https://pangea.cloud/docs/sdk/js/vault#retrieve), etc.) and
    /// request them when your application starts (e.g. a Docker container, an
    /// EC2).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const AWS_ACCESS_KEY_ID = 'AKIA1234X678C123B567';
    /// const OPENAI_API_KEY = 'sk_test_1234567890';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```js
    /// const AWS_ACCESS_KEY_ID = process.env.AWS_ACCESS_KEY_ID;
    /// const OPENAI_API_KEY = await getSecret('open-ai-api-key');
    /// ```
    ApiKeys,
    correctness
);

#[derive(Debug, Default, Clone)]
pub struct ApiKeys(Box<ApiKeysInner>);

#[derive(Debug, Clone)]
pub struct ApiKeysInner {
    /// Minimum length over all enabled secret rules.
    /// This is a performance optimization to avoid checking each rule for every string.
    min_len: NonZeroU32,
    /// Minimum entropy over all enabled secret rules.
    /// This is a performance optimization to avoid checking each rule for every string.
    min_entropy: f32,
    /// Credentials the user wants to check for.
    rules: Vec<SecretsEnum>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeysConfig {
    #[serde(default)]
    custom_patterns: Vec<CustomPattern>,
}

#[derive(Debug, Deserialize)]
struct CustomPattern {
    // required fields
    #[serde(rename = "ruleName")]
    rule_name: CompactStr,
    pattern: String,

    // optional fields
    #[serde(default)]
    message: Option<CompactStr>,
    #[serde(default)]
    entropy: Option<f32>,
    #[serde(default, rename = "minLength")]
    min_len: Option<NonZeroU32>,
    #[serde(default, rename = "maxLength")]
    max_len: Option<NonZeroU32>,
}

impl Default for ApiKeysInner {
    fn default() -> Self {
        Self::new(ALL_RULES.clone())
    }
}

impl ApiKeysInner {
    // TODO: allow configuring what rules are enabled/disabled
    // TODO: allow custom patterns
    pub fn new(rules: Vec<SecretsEnum>) -> Self {
        let min_len = rules.iter().map(secrets::SecretsEnum::min_len).min().unwrap();
        // can't use min() b/c f32 is not Ord
        let min_entropy = rules.iter().map(secrets::SecretsEnum::min_entropy).fold(0.0, f32::min);

        Self { min_len, min_entropy, rules }
    }
}

impl Deref for ApiKeys {
    type Target = ApiKeysInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ApiKeysInner {}

impl Rule for ApiKeys {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let string: &'a str = match node.kind() {
            AstKind::StringLiteral(string) => string.value.as_str(),
            AstKind::TemplateLiteral(string) => {
                let Some(string) = string.quasi() else {
                    return;
                };
                string.as_str()
            }
            _ => return,
        };

        // skip strings that are below the length/entropy threshold of _all_ rules. Perf
        // optimization, avoid O(n) len/entropy checks (for n rules)
        if string.len() < self.min_len.get() as usize {
            return;
        }
        let candidate = Secret::new(string, node.span(), None);
        if candidate.entropy() < self.min_entropy {
            return;
        }

        for rule in &self.rules {
            // order here is important: they're in order of cheapest to most expensive
            if candidate.len() < rule.min_len().get() as usize
                || candidate.entropy() < rule.min_entropy()
                || rule.max_len().is_some_and(|max_len| candidate.len() > max_len.get() as usize)
                || !rule.detect(&candidate)
            {
                continue;
            }

            // This clone allocs no memory and so is relatively cheap. rustc should optimize it
            // away anyways.
            let mut violation = SecretViolation::new(candidate.clone(), rule);
            if rule.verify(&mut violation) {
                ctx.diagnostic(api_keys(&violation));
                return;
            }
        }
    }

    fn from_configuration(value: Value) -> Self {
        let Some(obj) = value.get(0) else {
            return Self::default();
        };
        let config = serde_json::from_value::<ApiKeysConfig>(obj.clone()).unwrap();

        // TODO: Check if this is worth optimizing, then do so if needed.
        let mut rules = ALL_RULES.clone();
        rules.extend(config.custom_patterns.into_iter().map(|pattern| {
            let regex = Regex::new(&pattern.pattern).unwrap();
            SecretsEnum::Custom(CustomSecret {
                rule_name: pattern.rule_name,
                message: pattern.message.unwrap_or("Detected a hard-coded secret.".into()),
                entropy: pattern.entropy.unwrap_or(DEFAULT_MIN_ENTROPY),
                min_len: pattern.min_len.unwrap_or(DEFAULT_MIN_LEN),
                max_len: pattern.max_len,
                pattern: regex,
            })
        }));

        Self(Box::new(ApiKeysInner::new(rules)))
    }
}
