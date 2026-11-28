use std::iter;

use oxc_ast::AstKind;
use oxc_semantic::{JSDoc, JSDocTag};
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{Fix, RuleFix},
};

use super::{
    ALWAYS_INVALID_TAGS_IF_TYPED, OUTSIDE_AMBIENT_INVALID_TAGS_IF_TYPED,
    check_tag_names_diagnostic, remove_comment,
};

pub(super) fn check(
    ctx: &LintContext,
    jsdoc: &JSDoc,
    index: usize,
    node: Option<AstKind>,
    enable_fixer: bool,
) -> bool {
    let tag = &jsdoc.tags()[index];
    let name = tag.kind.parsed();
    let access_tag = matches!(name, "private" | "protected" | "public");
    let always_redundant = ALWAYS_INVALID_TAGS_IF_TYPED.contains(&name)
        && (!access_tag
            || node.is_some_and(|kind| {
                matches!(ctx.nodes().parent_kind(kind.node_id()), AstKind::ClassBody(_))
            }));
    let message = if always_redundant {
        format!("'@{name}' is redundant when using a type system.")
    } else if OUTSIDE_AMBIENT_INVALID_TAGS_IF_TYPED.contains(&name)
        && node.is_some_and(|kind| !is_ambient(ctx, kind))
    {
        format!(
            "'@{name}' is redundant outside of ambient (`declare`/`.d.ts`) contexts when using a type system."
        )
    } else if name == "template" && tag.type_name_comment().1.is_none() {
        format!("'@{name}' without a name is redundant when using a type system.")
    } else {
        return false;
    };

    let diagnostic = check_tag_names_diagnostic(tag.kind.span, message);
    if enable_fixer {
        ctx.diagnostic_with_fix(diagnostic, |_| {
            RuleFix::from(typed_fix(ctx.source_text(), jsdoc, index, always_redundant))
                .with_message(format!("Remove the redundant `@{name}` annotation."))
        });
    } else {
        ctx.diagnostic(diagnostic);
    }
    true
}

fn is_ambient(ctx: &LintContext, kind: AstKind) -> bool {
    ctx.source_type().is_typescript_definition()
        || iter::once(kind).chain(ctx.nodes().ancestor_kinds(kind.node_id())).any(is_declare)
}

fn is_declare(kind: AstKind) -> bool {
    match kind {
        AstKind::VariableDeclaration(decl) => decl.declare,
        AstKind::Function(decl) => decl.declare,
        AstKind::Class(decl) => decl.declare,
        AstKind::TSEnumDeclaration(decl) => decl.declare,
        AstKind::TSGlobalDeclaration(decl) => decl.declare,
        AstKind::TSExternalModuleDeclaration(decl) => decl.declare,
        AstKind::TSNamespaceDeclaration(decl) => decl.declare,
        AstKind::TSInterfaceDeclaration(decl) => decl.declare,
        AstKind::TSTypeAliasDeclaration(decl) => decl.declare,
        AstKind::ExportDeclaration(export) => export.declaration.declare(),
        _ => false,
    }
}

fn typed_fix(source: &str, jsdoc: &JSDoc, index: usize, remove_tag: bool) -> Option<Fix> {
    let tag = &jsdoc.tags()[index];
    let full = Span::new(jsdoc.span.start - 3, jsdoc.span.end + 2);
    let comment = full.source_text(source);
    let tag_start = (tag.kind.span.start - full.start) as usize;

    // Keep the name and prose of tags with a description. Other redundant tags
    // and their continuation lines can be removed together.
    if has_description(tag) {
        let type_part = tag.r#type();
        let start = if remove_tag { tag.kind.span.start } else { type_part.as_ref()?.span.start };
        let end = type_part.map_or(tag.kind.span.end, |ty| ty.span.end);
        let after = &source[end as usize..jsdoc.span.end as usize];
        let spaces = after.len() - after.trim_start_matches([' ', '\t']).len();
        let mut replacement = comment.to_string();
        replacement
            .replace_range((start - full.start) as usize..(end - full.start) as usize + spaces, "");
        return Some(Fix::new(replacement, full));
    }

    if jsdoc.tags().len() == 1 && jsdoc.comment().parsed().is_empty() {
        return Some(remove_comment(source, full));
    }

    let mut retained = String::new();
    if let Some(start_of_line) = comment[..tag_start].rfind('\n') {
        retained.push_str(&comment[..=start_of_line]);
    } else {
        retained.push_str("/**");
        retained.push_str(if comment.contains("\r\n") { "\r\n" } else { "\n" });
    }

    if let Some(next) = jsdoc.tags().get(index + 1) {
        let next_start = (next.kind.span.start - full.start) as usize;
        let start_of_line = comment[..next_start].rfind('\n').map_or(0, |pos| pos + 1);
        retained.push_str(&comment[start_of_line..]);
    } else {
        let closing_line = comment.rsplit('\n').next().unwrap_or(comment);
        if closing_line.trim_start() == "*/" {
            retained.push_str(closing_line);
        } else {
            let opening_line =
                source[..full.start as usize].rsplit('\n').next().unwrap_or_default();
            if opening_line.bytes().all(|b| matches!(b, b' ' | b'\t')) {
                retained.push_str(opening_line);
            }
            retained.push_str(" */");
        }
    }
    Some(Fix::new(retained, full))
}

fn has_description(tag: &JSDocTag) -> bool {
    // @access has no name argument, so all text after its type is prose.
    if tag.kind.parsed() == "access" {
        !tag.type_comment().1.parsed().is_empty()
    } else {
        !tag.type_name_comment().2.parsed().is_empty()
    }
}
