use lazy_regex::regex;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::JSDoc;
use oxc_span::Span;

use super::CheckTagNames;
use crate::context::LintContext;

pub(super) fn check(rule: &CheckTagNames, ctx: &LintContext, jsdoc: &JSDoc) {
    if !ctx.source_range(jsdoc.span).contains("{@") {
        return;
    }
    let check_description = |span: Span| {
        let text = ctx.source_range(span);
        // In `[label]{@link Target}`, the bracketed label replaces the trailing display text.
        // A closing bracket directly before the tag belongs to this form only.
        let prefixed = regex!(r"\[(?:[^\\\]]|\\(?s:.))+\]\{@([^}\s]+)\s?[^}\s|]*\}");
        let suffixed = regex!(r"\{@([^}\s]+)(?:\s(?:[^}\\]|\\(?s:.))*)?\}");
        let suffixed = suffixed
            .captures_iter(text)
            .filter(|cap| !text[..cap.get(0).unwrap().start()].ends_with(']'));
        for captures in prefixed.captures_iter(text).chain(suffixed) {
            let name = captures.get(1).unwrap();
            if rule.0.inline_tags.iter().any(|allowed| allowed == name.as_str()) {
                continue;
            }
            let start = span.start + u32::try_from(name.start()).unwrap();
            let end = span.start + u32::try_from(name.end()).unwrap();
            ctx.diagnostic(
                OxcDiagnostic::warn(format!("Invalid JSDoc inline tag name \"{}\"", name.as_str()))
                    .with_label(Span::new(start - 1, end)),
            );
        }
    };
    check_description(jsdoc.comment().span);
    for tag in jsdoc.tags() {
        // Only descriptions can contain inline tags. Annotations and parameter names are opaque.
        let description = match tag.kind.parsed() {
            "default" | "defaultvalue" | "description" | "example" | "file" | "fileoverview"
            | "license" | "overview" | "summary" => tag.comment(),
            "see" if ctx.source_range(tag.comment().span).contains("{@link") => tag.comment(),
            "access" | "author" | "exception" | "kind" | "return" | "returns" | "since"
            | "throws" | "variation" | "version" => tag.type_comment().1,
            _ => tag.type_name_comment().2,
        };
        check_description(description.span);
    }
}
