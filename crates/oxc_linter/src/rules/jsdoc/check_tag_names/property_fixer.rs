use std::{borrow::Cow, fmt::Write as _, ops::Range};

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Declaration, Expression, PropertyKey, TSSignature, TSType},
};
use oxc_semantic::{JSDoc, JSDocTag};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{context::LintContext, fixer::Fix};

use super::remove_comment;

/// Move the property descriptions as a group. If a single target is ambiguous or is
/// already documented, keep the descriptions in the declaration's original comment.
pub(super) fn property_fix(
    ctx: &LintContext,
    jsdoc: &JSDoc,
    node: AstKind,
    include_prop: bool,
) -> Option<Vec<Fix>> {
    let is_property_tag = |tag: &JSDocTag| match tag.kind.parsed() {
        "property" => true,
        "prop" => include_prop,
        _ => false,
    };
    if !jsdoc.tags().iter().any(is_property_tag) {
        return None;
    }
    let mut properties = object_properties(node)?;
    let source = ctx.source_text();
    let span = Span::new(jsdoc.span.start - 3, jsdoc.span.end + 2);
    let mut offset = span.start as usize;
    let lines: Vec<_> = ctx
        .source_range(span)
        .split('\n')
        .map(|line| {
            let start = offset;
            offset += line.len() + 1;
            (start, line.strip_suffix('\r').unwrap_or(line))
        })
        .collect();
    let line_at = |pos: u32| lines.partition_point(|(start, _)| *start <= pos as usize) - 1;
    let mut tags = Vec::new();
    for (index, tag) in jsdoc.tags().iter().enumerate().filter(|(_, tag)| is_property_tag(tag)) {
        let start = line_at(tag.kind.span.start);
        let end = jsdoc.tags().get(index + 1).map_or_else(
            || lines.len() - usize::from(lines.last().is_some_and(|(_, text)| text.trim() == "*/")),
            |next| line_at(next.kind.span.start),
        );
        let (_, name, comment) = tag.type_name_comment();
        let raw_description = ctx.source_range(comment.span);
        // The comment parser includes the next line's indentation and star before
        // a subsequent tag. Normalize the delimiters while preserving paragraph breaks.
        let mut description = String::new();
        for (idx, line) in raw_description.lines().enumerate() {
            if idx != 0 {
                description.push('\n');
            }
            let text = line.trim_start();
            description.push_str(if idx == 0 {
                text
            } else {
                text.strip_prefix('*').unwrap_or(text).trim_start()
            });
        }
        let clear_until = name.as_ref().map_or(tag.kind.span.end, |name| name.span.start);
        let target = name.and_then(|name| properties.remove(name.parsed()));
        tags.push(PropertyTag {
            lines: start..end,
            clear: (tag.kind.span.start as usize - lines[start].0)
                ..(clear_until as usize - lines[start].0).min(lines[start].1.len()),
            target,
            description_on_tag_line: line_at(comment.span.start) == start,
            description,
        });
    }

    let line_break = if source.contains("\r\n") { "\r\n" } else { "\n" };
    let moves: Option<Vec<_>> =
        tags.iter().map(|tag| tag.move_to_property(ctx, line_break)).collect();

    let mut kept: Vec<_> = lines.iter().map(|(_, line)| Cow::Borrowed(*line)).collect();
    for tag in tags.iter().rev() {
        if moves.is_some() || tag.description.trim().is_empty() {
            kept.drain(tag.lines.clone());
        } else {
            kept[tag.lines.start].to_mut().replace_range(tag.clear.clone(), "");
        }
    }
    let has_text = kept
        .iter()
        .any(|line| line.chars().any(|ch| !matches!(ch, '*' | '/') && !ch.is_whitespace()));
    let mut fixes = moves.unwrap_or_default();
    fixes.push(if has_text {
        if kept.first().is_some_and(|line| !line.starts_with("/**")) {
            kept.insert(0, Cow::Borrowed("/**"));
        }
        if kept.last().is_some_and(|line| !line.ends_with("*/")) {
            kept.push(Cow::Borrowed(" */"));
        }
        Fix::new(kept.join(line_break), span)
    } else {
        remove_comment(source, span)
    });
    Some(fixes)
}

struct PropertyTag {
    lines: Range<usize>,
    clear: Range<usize>,
    target: Option<u32>,
    description_on_tag_line: bool,
    description: String,
}

impl PropertyTag {
    fn move_to_property(&self, ctx: &LintContext, line_break: &str) -> Option<Fix> {
        let start = self.target?;
        // A hyphen on the tag line separates the name from prose. On any
        // subsequent line, the hyphen can be part of a Markdown list.
        let text = if self.description_on_tag_line {
            self.description.strip_prefix('-').unwrap_or(&self.description)
        } else {
            &self.description
        }
        .trim();
        if text.is_empty() || has_preceding_jsdoc(ctx, start) {
            return None;
        }
        let prefix = &ctx.source_text()[..start as usize];
        let line = prefix.rsplit('\n').next().unwrap_or_default();
        let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
        let mut insertion = String::from("/**");
        for line in text.split('\n') {
            let _ = write!(insertion, "{line_break}{indent} *");
            if !line.is_empty() {
                insertion.push(' ');
                insertion.push_str(line);
            }
        }
        let _ = write!(insertion, "{line_break}{indent} */{line_break}{indent}");
        Some(Fix::new(insertion, Span::empty(start)))
    }
}

fn object_properties<'a>(node: AstKind<'a>) -> Option<FxHashMap<&'a str, u32>> {
    let declaration = match node {
        AstKind::VariableDeclaration(var) => var,
        AstKind::ExportDeclaration(export) => {
            let Declaration::VariableDeclaration(var) = &export.declaration else {
                return None;
            };
            var
        }
        _ => return None,
    };
    let declarator = declaration.declarations.first()?;
    let mut members = FxHashMap::default();
    let mut record = |key: &PropertyKey<'a>, span: Span| {
        let name = match key {
            PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
            PropertyKey::StaticStringLiteral(lit) | PropertyKey::StringLiteral(lit) => {
                lit.value.as_str()
            }
            _ => None,
        };
        if let Some(name) = name {
            members.entry(name).or_insert(span.start);
        }
    };
    if let Some(Expression::ObjectExpression(object)) =
        declarator.init.as_ref().map(Expression::without_parentheses)
    {
        for property in object.properties.iter().filter_map(|item| item.as_property()) {
            record(&property.key, property.span);
        }
    } else if let BindingPattern::BindingIdentifier(_) = &declarator.id
        && let TSType::TSTypeLiteral(literal) =
            &declarator.type_annotation.as_ref()?.type_annotation
    {
        for member in &literal.members {
            if let TSSignature::TSPropertySignature(property) = member {
                record(&property.key, property.span);
            }
        }
    } else {
        return None;
    }
    Some(members)
}

fn has_preceding_jsdoc(ctx: &LintContext, mut start: u32) -> bool {
    for comment in ctx.semantic().comments_range(..start).rev() {
        if !ctx.source_range(Span::new(comment.span.end, start)).trim().is_empty() {
            break;
        }
        if comment.is_block() && ctx.source_range(comment.span).starts_with("/**") {
            return true;
        }
        start = comment.span.start;
    }
    false
}
