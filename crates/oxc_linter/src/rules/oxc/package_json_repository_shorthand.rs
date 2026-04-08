use super::json_utils::{file_start_span, is_json_file};

use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

static BITBUCKET_REGEX: Lazy<Regex> =
    lazy_regex!(r"^(?:git\+)?(?:ssh://git@|https?://)?(?:www\.)?bitbucket\.org/");
static GIST_REGEX: Lazy<Regex> =
    lazy_regex!(r"^(?:git\+)?(?:ssh://git@|https?://)?(?:www\.)?gist\.github\.com/");
static GITHUB_REGEX: Lazy<Regex> =
    lazy_regex!(r"^(?:git\+)?(?:ssh://git@|https?://)?(?:www\.)?github\.com/");
static GITLAB_REGEX: Lazy<Regex> =
    lazy_regex!(r"^(?:git\+)?(?:ssh://git@|https?://)?(?:www\.)?gitlab\.com/");

fn prefer_object_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer an object locator for a repository.")
        .with_help("Use `{ \"type\": \"git\", \"url\": \"...\" }` for the `repository` field.")
        .with_label(span)
}

fn prefer_shorthand_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer a shorthand locator for a supported repository provider.")
        .with_help("Use provider shorthand such as `github:user/repo` when the repository URL can be normalized safely.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryForm {
    #[default]
    Object,
    Shorthand,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RepositoryShorthandConfig {
    form: RepositoryForm,
}

impl Default for RepositoryShorthandConfig {
    fn default() -> Self {
        Self { form: RepositoryForm::Object }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonRepositoryShorthand(Box<RepositoryShorthandConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces either object or shorthand declaration for `repository`.
    ///
    /// ### Why is this bad?
    ///
    /// Consistent repository metadata keeps package manifests easier to audit
    /// and aligns with npm normalization behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "repository": "npm/example" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "repository": { "type": "git", "url": "https://github.com/npm/example" } }
    /// ```
    PackageJsonRepositoryShorthand,
    oxc,
    style,
    fix,
    config = RepositoryShorthandConfig
);

impl Rule for PackageJsonRepositoryShorthand {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<RepositoryShorthandConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(object) = value.as_object() else {
            return;
        };
        let Some(repository) = object.get("repository") else {
            return;
        };

        match self.0.form {
            RepositoryForm::Object => self.run_object_form(ctx, source_text, &value, repository),
            RepositoryForm::Shorthand => {
                self.run_shorthand_form(ctx, source_text, &value, repository);
            }
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

impl PackageJsonRepositoryShorthand {
    #[expect(clippy::unused_self)]
    fn run_object_form(
        &self,
        ctx: &LintContext<'_>,
        source_text: &str,
        value: &Value,
        repository: &Value,
    ) {
        let Value::String(repository) = repository else {
            return;
        };

        let diagnostic = prefer_object_diagnostic(file_start_span(source_text));
        let Some(updated) = replace_repository(value, create_repository_object_value(repository))
        else {
            ctx.diagnostic(diagnostic);
            return;
        };
        let Some(expected) = serialize_updated_json(&updated, source_text) else {
            ctx.diagnostic(diagnostic);
            return;
        };

        #[expect(clippy::cast_possible_truncation)]
        let file_span = oxc_span::Span::new(0, source_text.len() as u32);
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            fixer.replace_full_source_range(file_span, expected)
        });
    }

    #[expect(clippy::unused_self)]
    fn run_shorthand_form(
        &self,
        ctx: &LintContext<'_>,
        source_text: &str,
        value: &Value,
        repository: &Value,
    ) {
        let Some(shorthand) = extract_shorthand_repository(repository) else {
            return;
        };

        let Some(updated) = replace_repository(value, Some(Value::String(shorthand))) else {
            return;
        };
        let Some(expected) = serialize_updated_json(&updated, source_text) else {
            return;
        };

        #[expect(clippy::cast_possible_truncation)]
        let file_span = oxc_span::Span::new(0, source_text.len() as u32);
        ctx.diagnostic_with_fix(
            prefer_shorthand_diagnostic(file_start_span(source_text)),
            |fixer| fixer.replace_full_source_range(file_span, expected),
        );
    }
}

fn create_repository_object_value(repository: &str) -> Option<Value> {
    if repository.split('/').filter(|segment| !segment.is_empty()).count() != 2 {
        return None;
    }

    let mut object = Map::with_capacity(2);
    object.insert("type".to_string(), Value::String("git".to_string()));
    object.insert("url".to_string(), Value::String(create_url(repository)));
    Some(Value::Object(object))
}

fn extract_shorthand_repository(repository: &Value) -> Option<String> {
    match repository {
        Value::String(value) => {
            let provider = get_provider_from_url(value)?;
            Some(create_shorthand(value, provider))
        }
        Value::Object(object) => {
            if object.contains_key("directory") {
                return None;
            }

            let type_value = object.get("type")?;
            if !matches!(type_value, Value::String(kind) if kind == "git") {
                return None;
            }

            let Value::String(url) = object.get("url")? else {
                return None;
            };
            let provider = get_provider_from_url(url)?;
            Some(create_shorthand(url, provider))
        }
        _ => None,
    }
}

fn replace_repository(value: &Value, repository: Option<Value>) -> Option<Value> {
    let repository = repository?;
    let mut object = value.as_object()?.clone();
    object.insert("repository".to_string(), repository);
    Some(Value::Object(object))
}

fn serialize_updated_json(value: &Value, source_text: &str) -> Option<String> {
    let indent = vec![b' '; 2];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut output = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut output, formatter);
    value.serialize(&mut serializer).ok()?;
    let mut formatted = String::from_utf8(output).ok()?;
    if source_text.ends_with('\n') {
        formatted.push('\n');
    }
    Some(formatted)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RepositoryProvider {
    Bitbucket,
    Gist,
    GitHub,
    GitLab,
}

fn get_provider_from_url(url: &str) -> Option<RepositoryProvider> {
    if BITBUCKET_REGEX.is_match(url) {
        Some(RepositoryProvider::Bitbucket)
    } else if GIST_REGEX.is_match(url) {
        Some(RepositoryProvider::Gist)
    } else if GITHUB_REGEX.is_match(url) {
        Some(RepositoryProvider::GitHub)
    } else if GITLAB_REGEX.is_match(url) {
        Some(RepositoryProvider::GitLab)
    } else {
        None
    }
}

fn create_shorthand(url: &str, provider: RepositoryProvider) -> String {
    let repo = clean_url(url, provider);
    format!("{}:{repo}", provider_name(provider))
}

fn clean_url(url: &str, provider: RepositoryProvider) -> String {
    let cleaned = match provider {
        RepositoryProvider::Bitbucket => BITBUCKET_REGEX.replace(url, ""),
        RepositoryProvider::Gist => GIST_REGEX.replace(url, ""),
        RepositoryProvider::GitHub => GITHUB_REGEX.replace(url, ""),
        RepositoryProvider::GitLab => GITLAB_REGEX.replace(url, ""),
    };
    cleaned.trim_end_matches(".git").to_string()
}

fn create_url(shorthand: &str) -> String {
    if let Some((provider, repo)) = shorthand.split_once(':')
        && let Some(provider) = parse_provider(provider)
    {
        return format!("{}{repo}", provider_url(provider));
    }

    format!("{}{}", provider_url(RepositoryProvider::GitHub), shorthand)
}

fn parse_provider(provider: &str) -> Option<RepositoryProvider> {
    match provider {
        "bitbucket" => Some(RepositoryProvider::Bitbucket),
        "gist" => Some(RepositoryProvider::Gist),
        "github" => Some(RepositoryProvider::GitHub),
        "gitlab" => Some(RepositoryProvider::GitLab),
        _ => None,
    }
}

fn provider_name(provider: RepositoryProvider) -> &'static str {
    match provider {
        RepositoryProvider::Bitbucket => "bitbucket",
        RepositoryProvider::Gist => "gist",
        RepositoryProvider::GitHub => "github",
        RepositoryProvider::GitLab => "gitlab",
    }
}

fn provider_url(provider: RepositoryProvider) -> &'static str {
    match provider {
        RepositoryProvider::Bitbucket => "https://bitbucket.org/",
        RepositoryProvider::Gist => "https://gist.github.com/",
        RepositoryProvider::GitHub => "https://github.com/",
        RepositoryProvider::GitLab => "https://gitlab.com/",
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"{"repository":{"type":"git","url":"https://github.com/npm/example"}}"#, None),
        (r#"{"repository":"github:npm/example"}"#, Some(json!([{ "form": "shorthand" }]))),
        (
            r#"{"repository":{"type":"git","url":"https://github.com/npm/cli","directory":"workspaces/libnpmpublish"}}"#,
            Some(json!([{ "form": "shorthand" }])),
        ),
        (r#"{"repository":"gist:11081aaa281"}"#, Some(json!([{ "form": "shorthand" }]))),
        (r#"{"name":"demo"}"#, None),
    ];

    let fail = vec![
        (r#"{"repository":"npm/example"}"#, None),
        (r#"{"repository":"github:npm/example"}"#, None),
        (
            r#"{"repository":{"type":"git","url":"https://github.com/npm/example"}}"#,
            Some(json!([{ "form": "shorthand" }])),
        ),
        (
            r#"{"repository":"https://gitlab.com/user/repo.git"}"#,
            Some(json!([{ "form": "shorthand" }])),
        ),
    ];

    let fix = vec![
        (
            r#"{"repository":"npm/example"}"#,
            "{\n  \"repository\": {\n    \"type\": \"git\",\n    \"url\": \"https://github.com/npm/example\"\n  }\n}",
            None,
        ),
        (
            r#"{"repository":"github:npm/example"}"#,
            "{\n  \"repository\": {\n    \"type\": \"git\",\n    \"url\": \"https://github.com/npm/example\"\n  }\n}",
            None,
        ),
        (
            r#"{"repository":{"type":"git","url":"https://github.com/npm/example"}}"#,
            "{\n  \"repository\": \"github:npm/example\"\n}",
            Some(json!([{ "form": "shorthand" }])),
        ),
        (
            r#"{"repository":"https://gitlab.com/user/repo.git"}"#,
            "{\n  \"repository\": \"gitlab:user/repo\"\n}",
            Some(json!([{ "form": "shorthand" }])),
        ),
    ];

    Tester::new(
        PackageJsonRepositoryShorthand::NAME,
        PackageJsonRepositoryShorthand::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .change_rule_path("package.json")
    .test_and_snapshot();
}
