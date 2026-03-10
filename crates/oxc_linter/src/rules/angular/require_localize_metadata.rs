use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::Rule};

fn require_localize_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("$localize tagged messages should contain a description")
        .with_help(
            "Add a description to help translators understand the context. \
            Use the format: $localize`:description:text` or $localize`:meaning|description:text`",
        )
        .with_label(span)
}

fn require_localize_meaning_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("$localize tagged messages should contain a meaning")
        .with_help(
            "Add a meaning to distinguish identical text that needs different translations. \
            Use the format: $localize`:meaning|description:text`",
        )
        .with_label(span)
}

fn require_localize_custom_id_diagnostic(span: Span, pattern_message: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "$localize tagged messages should contain a custom id{pattern_message}"
    ))
    .with_help(
        "Add a custom ID for stable translation references. \
        Use the format: $localize`:description@@customId:text`",
    )
    .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[expect(clippy::struct_field_names)]
pub struct RequireLocalizeMetadataConfig {
    #[serde(default)]
    require_description: bool,
    #[serde(default)]
    require_meaning: bool,
    #[serde(default)]
    require_custom_id: RequireCustomIdOption,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
pub enum RequireCustomIdOption {
    #[default]
    Disabled,
    Enabled(bool),
    Pattern(String),
}

impl RequireCustomIdOption {
    fn is_required(&self) -> bool {
        match self {
            Self::Disabled => false,
            Self::Enabled(b) => *b,
            Self::Pattern(_) => true,
        }
    }

    fn get_pattern(&self) -> Option<&str> {
        match self {
            Self::Pattern(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[expect(clippy::struct_field_names)]
pub struct RequireLocalizeMetadata {
    require_description: bool,
    require_meaning: bool,
    require_custom_id: RequireCustomIdOption,
}

impl From<RequireLocalizeMetadataConfig> for RequireLocalizeMetadata {
    fn from(config: RequireLocalizeMetadataConfig) -> Self {
        Self {
            require_description: config.require_description,
            require_meaning: config.require_meaning,
            require_custom_id: config.require_custom_id,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that `$localize` tagged messages contain helpful metadata to aid with translations.
    ///
    /// ### Why is this bad?
    ///
    /// When internationalizing Angular applications with `@angular/localize`, adding metadata
    /// (description, meaning, and custom IDs) to `$localize` tagged strings is essential for
    /// high-quality translations:
    ///
    /// - **Description**: Helps translators understand the context and purpose of the text
    /// - **Meaning**: Distinguishes identical text that needs different translations
    /// - **Custom ID**: Provides stable references that persist across code changes
    ///
    /// Without this metadata, translators work blind, leading to poor translations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (with `requireDescription: true`):
    /// ```typescript
    /// const message = $localize`Hello World`;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// const message = $localize`:greeting|A friendly greeting:Hello World`;
    /// const message2 = $localize`:A friendly greeting@@greeting-id:Hello World`;
    /// ```
    ///
    /// ### Configuration
    ///
    /// ```json
    /// {
    ///   "angular/require-localize-metadata": ["error", {
    ///     "requireDescription": true,
    ///     "requireMeaning": false,
    ///     "requireCustomId": false
    ///   }]
    /// }
    /// ```
    RequireLocalizeMetadata,
    angular,
    pedantic,
    pending
);

impl Rule for RequireLocalizeMetadata {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }
        let config_value = value.get(0).unwrap_or(&value);
        serde_json::from_value::<RequireLocalizeMetadataConfig>(config_value.clone())
            .map(Into::into)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Skip if no requirements are configured
        if !self.require_description
            && !self.require_meaning
            && !self.require_custom_id.is_required()
        {
            return;
        }

        let AstKind::TaggedTemplateExpression(tagged) = node.kind() else {
            return;
        };

        // Check if the tag is $localize
        let tag_name = match &tagged.tag {
            oxc_ast::ast::Expression::Identifier(ident) => ident.name.as_str(),
            _ => return,
        };

        if tag_name != "$localize" {
            return;
        }

        // Get the first quasi (template element)
        let Some(first_quasi) = tagged.quasi.quasis.first() else {
            return;
        };

        let raw_text = first_quasi.value.raw.as_str().trim();
        let metadata = parse_localize_metadata(raw_text);

        if self.require_description && metadata.description.is_none() {
            ctx.diagnostic(require_localize_description_diagnostic(first_quasi.span));
        }

        if self.require_meaning && metadata.meaning.is_none() {
            ctx.diagnostic(require_localize_meaning_diagnostic(first_quasi.span));
        }

        if self.require_custom_id.is_required() {
            let id_valid = match (&metadata.custom_id, self.require_custom_id.get_pattern()) {
                (Some(id), Some(pattern)) => {
                    // Check if custom_id matches the pattern
                    lazy_regex::Regex::new(pattern).is_ok_and(|re| re.is_match(id))
                }
                (Some(_), None) => true,
                (None, _) => false,
            };

            if !id_valid {
                let pattern_message = match self.require_custom_id.get_pattern() {
                    Some(pattern) => {
                        format!(
                            " matching the pattern /{pattern}/ on '{}'",
                            metadata.custom_id.unwrap_or_default()
                        )
                    }
                    None => String::new(),
                };
                ctx.diagnostic(require_localize_custom_id_diagnostic(
                    first_quasi.span,
                    &pattern_message,
                ));
            }
        }
    }
}

/// Parsed metadata from a $localize tagged template
struct LocalizeMetadata<'a> {
    meaning: Option<&'a str>,
    description: Option<&'a str>,
    custom_id: Option<&'a str>,
}

/// Parse metadata from the raw text of a $localize tagged template.
/// Format: `:meaning|description@@customId:` or `:description@@customId:` or `:description:`
fn parse_localize_metadata(raw_text: &str) -> LocalizeMetadata<'_> {
    const BLOCK_MARKER: char = ':';
    const MEANING_SEPARATOR: char = '|';
    const ID_SEPARATOR: &str = "@@";

    let mut result = LocalizeMetadata { meaning: None, description: None, custom_id: None };

    // Must start with ':'
    if !raw_text.starts_with(BLOCK_MARKER) {
        return result;
    }

    // Find the end of the metadata block
    let Some(end_index) = raw_text[1..].find(BLOCK_MARKER) else {
        return result;
    };

    let text = &raw_text[1..=end_index];

    // Split by @@ to get custom ID
    let (meaning_and_desc, custom_id) = if let Some(idx) = text.find(ID_SEPARATOR) {
        (&text[..idx], Some(&text[idx + 2..]))
    } else {
        (text, None)
    };

    // Split by | to get meaning and description
    let (meaning, description) = if let Some(idx) = meaning_and_desc.find(MEANING_SEPARATOR) {
        (Some(&meaning_and_desc[..idx]), Some(&meaning_and_desc[idx + 1..]))
    } else {
        // If no separator, the whole thing is the description
        (None, Some(meaning_and_desc))
    };

    // Filter out empty strings
    result.meaning = meaning.filter(|s| !s.is_empty());
    result.description = description.filter(|s| !s.is_empty());
    result.custom_id = custom_id.filter(|s| !s.is_empty());

    result
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // With description (when required)
        (
            r"const msg = $localize`:A greeting message:Hello`;",
            Some(serde_json::json!([{ "requireDescription": true }])),
        ),
        // With meaning and description
        (
            r"const msg = $localize`:greeting|A friendly greeting:Hello`;",
            Some(serde_json::json!([{ "requireDescription": true, "requireMeaning": true }])),
        ),
        // With custom ID
        (
            r"const msg = $localize`:description@@custom-id:Hello`;",
            Some(serde_json::json!([{ "requireCustomId": true }])),
        ),
        // With all metadata
        (
            r"const msg = $localize`:meaning|description@@custom-id:Hello`;",
            Some(
                serde_json::json!([{ "requireDescription": true, "requireMeaning": true, "requireCustomId": true }]),
            ),
        ),
        // Custom ID with pattern
        (
            r"const msg = $localize`:desc@@MSG_123:Hello`;",
            Some(serde_json::json!([{ "requireCustomId": "^MSG_" }])),
        ),
        // No requirements (default)
        (r"const msg = $localize`Hello`;", None),
        // Not $localize
        (
            r"const msg = someTag`Hello`;",
            Some(serde_json::json!([{ "requireDescription": true }])),
        ),
    ];

    let fail = vec![
        // Missing description
        (
            r"const msg = $localize`Hello`;",
            Some(serde_json::json!([{ "requireDescription": true }])),
        ),
        // Missing meaning
        (
            r"const msg = $localize`:description:Hello`;",
            Some(serde_json::json!([{ "requireMeaning": true }])),
        ),
        // Missing custom ID
        (
            r"const msg = $localize`:description:Hello`;",
            Some(serde_json::json!([{ "requireCustomId": true }])),
        ),
        // Custom ID doesn't match pattern
        (
            r"const msg = $localize`:desc@@wrong-id:Hello`;",
            Some(serde_json::json!([{ "requireCustomId": "^MSG_" }])),
        ),
        // Empty description
        (
            r"const msg = $localize`::Hello`;",
            Some(serde_json::json!([{ "requireDescription": true }])),
        ),
    ];

    Tester::new(RequireLocalizeMetadata::NAME, RequireLocalizeMetadata::PLUGIN, pass, fail)
        .test_and_snapshot();
}
