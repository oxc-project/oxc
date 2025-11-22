use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule, utils::get_element_type};

fn media_has_caption_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Missing `<track>` element with captions inside `<audio>` or `<video>` element",
    )
    .with_help("Media elements such as `<audio>` and `<video>` must have a `<track>` for captions.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MediaHasCaption(Box<MediaHasCaptionConfig>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaHasCaptionConfig {
    /// Element names to treat as `<audio>` elements
    audio: Vec<Cow<'static, str>>,
    /// Element names to treat as `<video>` elements
    video: Vec<Cow<'static, str>>,
    /// Element names to treat as `<track>` elements
    track: Vec<Cow<'static, str>>,
}

impl Default for MediaHasCaptionConfig {
    fn default() -> Self {
        Self {
            audio: vec![Cow::Borrowed("audio")],
            video: vec![Cow::Borrowed("video")],
            track: vec![Cow::Borrowed("track")],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks if `<audio>` and `<video>` elements have a `<track>` element for captions.
    /// This ensures media content is accessible to all users, including those with hearing impairments.
    ///
    /// ### Why is this bad?
    ///
    /// Without captions, audio and video content is not accessible to users who are deaf or hard of hearing.
    /// Captions are also useful for users in noisy environments or where audio is not available.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <audio></audio>
    /// <video></video>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <audio><track kind="captions" src="caption_file.vtt" /></audio>
    /// <video><track kind="captions" src="caption_file.vtt" /></video>
    /// ```
    MediaHasCaption,
    jsx_a11y,
    correctness,
    config = MediaHasCaptionConfig,
);

impl Rule for MediaHasCaption {
    fn from_configuration(value: Value) -> Self {
        let mut config = MediaHasCaptionConfig::default();

        if let Some(arr) = value.as_array() {
            for v in arr {
                let serde_json::Value::Object(rule_config) = v else {
                    continue;
                };

                if let Some(audio) = rule_config.get("audio").and_then(Value::as_array) {
                    config.audio.extend(
                        audio.iter().filter_map(Value::as_str).map(String::from).map(Into::into),
                    );
                }
                if let Some(video) = rule_config.get("video").and_then(Value::as_array) {
                    config.video.extend(
                        video.iter().filter_map(Value::as_str).map(String::from).map(Into::into),
                    );
                }
                if let Some(track) = rule_config.get("track").and_then(Value::as_array) {
                    config.track.extend(
                        track.iter().filter_map(Value::as_str).map(String::from).map(Into::into),
                    );
                }
                break;
            }
        }

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_name = get_element_type(ctx, jsx_el);

        let is_audio_or_video =
            self.0.audio.contains(&element_name) || self.0.video.contains(&element_name);

        // Bail out if the element is not an <audio /> or <video /> element.
        if !is_audio_or_video {
            return;
        }

        let muted = jsx_el.attributes.iter().any(|attr_item| {
            let JSXAttributeItem::Attribute(attr) = attr_item else { return false };

            let JSXAttributeName::Identifier(iden) = &attr.name else { return false };

            if iden.name == "muted" {
                return match &attr.value {
                    Some(JSXAttributeValue::ExpressionContainer(exp)) => match &exp.expression {
                        JSXExpression::BooleanLiteral(boolean) => boolean.value,
                        _ => false,
                    },
                    Some(JSXAttributeValue::StringLiteral(lit)) => lit.value == "true",
                    None => true, // e.g. <video muted></video>
                    _ => false,
                };
            }

            false
        });

        // Bail out if the element is muted as captions are not required for muted media. (e.g <video muted />)
        if muted {
            return;
        }

        let AstKind::JSXElement(parent) = ctx.nodes().parent_kind(node.id()) else {
            return;
        };

        let has_caption = if parent.children.is_empty() {
            ctx.diagnostic(media_has_caption_diagnostic(parent.opening_element.span));
            false
        } else {
            parent.children.iter().any(|child| match child {
                JSXChild::Element(child_el) => {
                    let child_name = get_element_type(ctx, &child_el.opening_element);

                    self.0.track.contains(&child_name)
                        && child_el.opening_element.attributes.iter().any(|attr| {
                            let JSXAttributeItem::Attribute(attr) = attr else { return false };
                            let JSXAttributeName::Identifier(iden) = &attr.name else {
                                return false;
                            };
                            if let Some(JSXAttributeValue::StringLiteral(s)) = &attr.value {
                                return iden.name == "kind"
                                    && s.value.eq_ignore_ascii_case("captions");
                            }
                            false
                        })
                }
                _ => false,
            })
        };

        let span = parent.span;

        if !has_caption {
            ctx.diagnostic(media_has_caption_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn config() -> serde_json::Value {
        serde_json::json!([{
            "audio": [ "Audio" ],
            "video": [ "Video" ],
            "track": [ "Track" ],
        }])
    }

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "polymorphicPropName": "as",
                "components": {
                    "Audio": "audio",
                    "Video": "video",
                    "Track": "track",
                },
            } }
        })
    }

    let pass = vec![
        (r"<div />;", None, None),
        (r"<MyDiv />;", None, None),
        (r"<audio><track kind='captions' /></audio>", None, None),
        (r"<audio><track kind='Captions' /></audio>", None, None),
        (r"<audio><track kind='Captions' /><track kind='subtitles' /></audio>", None, None),
        (r"<video><track kind='captions' /></video>", None, None),
        (r"<video><track kind='Captions' /></video>", None, None),
        (r"<video><track kind='Captions' /><track kind='subtitles' /></video>", None, None),
        (r"<audio muted={true}></audio>", None, None),
        (r"<video muted={true}></video>", None, None),
        (r"<video muted></video>", None, None),
        (r"<Audio><track kind='captions' /></Audio>", Some(config()), None),
        (r"<audio><Track kind='captions' /></audio>", Some(config()), None),
        (r"<Video><track kind='captions' /></Video>", Some(config()), None),
        (r"<video><Track kind='captions' /></video>", Some(config()), None),
        (r"<Audio><Track kind='captions' /></Audio>", Some(config()), None),
        (r"<Video><Track kind='captions' /></Video>", Some(config()), None),
        (r"<Video muted></Video>", Some(config()), None),
        (r"<Video muted={true}></Video>", Some(config()), None),
        (r"<Audio muted></Audio>", Some(config()), None),
        (r"<Audio muted={true}></Audio>", Some(config()), None),
        (r"<Audio><track kind='captions' /></Audio>", None, Some(settings())),
        (r"<audio><Track kind='captions' /></audio>", None, Some(settings())),
        (r"<Video><track kind='captions' /></Video>", None, Some(settings())),
        (r"<video><Track kind='captions' /></video>", None, Some(settings())),
        (r"<Audio><Track kind='captions' /></Audio>", None, Some(settings())),
        (r"<Video><Track kind='captions' /></Video>", None, Some(settings())),
        (r"<Video muted></Video>", None, Some(settings())),
        (r"<Video muted={true}></Video>", None, Some(settings())),
        (r"<Audio muted></Audio>", None, Some(settings())),
        (r"<Audio muted={true}></Audio>", None, Some(settings())),
        (r"<Box as='audio' muted={true}></Box>", None, Some(settings())),
    ];

    let fail = vec![
        (r"<audio><track /></audio>", None, None),
        (r"<audio><track kind='subtitles' /></audio>", None, None),
        (r"<audio />", None, None),
        (r"<video><track /></video>", None, None),
        (r"<video><track kind='subtitles' /></video>", None, None),
        (r"<Audio muted={false}></Audio>", Some(config()), None),
        (r"<Video muted={false}></Video>", Some(config()), None),
        (r"<Audio muted={false}></Audio>", None, Some(settings())),
        (r"<Video muted={false}></Video>", None, Some(settings())),
        (r"<video />", None, None),
        (r"<audio>Foo</audio>", None, None),
        (r"<video>Foo</video>", None, None),
        (r"<Audio />", Some(config()), None),
        (r"<Video />", Some(config()), None),
        (r"<Audio />", None, Some(settings())),
        (r"<Video />", None, Some(settings())),
        (r"<audio><Track /></audio>", Some(config()), None),
        (r"<video><Track /></video>", Some(config()), None),
        (r"<Audio><Track kind='subtitles' /></Audio>", Some(config()), None),
        (r"<Video><Track kind='subtitles' /></Video>", Some(config()), None),
        (r"<Audio><Track kind='subtitles' /></Audio>", None, Some(settings())),
        (r"<Video><Track kind='subtitles' /></Video>", None, Some(settings())),
        (r"<Box as='audio'><Track kind='subtitles' /></Box>", None, Some(settings())),
    ];

    Tester::new(MediaHasCaption::NAME, MediaHasCaption::PLUGIN, pass, fail).test_and_snapshot();
}
