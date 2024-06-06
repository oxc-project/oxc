use crate::{config::JSDocPluginSettings, context::LintContext, AstNode};
use oxc_ast::AstKind;
use oxc_semantic::JSDoc;
use rustc_hash::FxHashSet;

/// JSDoc is often attached on the parent node of a function.
///
/// ```js
/// /** VariableDeclaration > VariableDeclarator > FunctionExpression */
/// const foo = function() {}
///
/// /** VariableDeclaration > VariableDeclarator > ArrowFunctionExpression */
/// const bar = () => {},
///       /** VariableDeclarator > ArrowFunctionExpression */
///       baz = () => {};
///
/// /** MethodDefinition > FunctionExpression */
/// class X { qux() {} }
///
/// /** PropertyDefinition > ArrowFunctionExpression */
/// class Y { qux = () => {} }
/// ```
pub fn get_function_nearest_jsdoc_node<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    let mut current_node = node;
    // Whether the node has attached JSDoc or not is determined by `JSDocBuilder`
    while ctx.jsdoc().get_all_by_node(current_node).is_none() {
        // Tie-breaker, otherwise every loop will end at `Program` node!
        // Maybe more checks should be added
        match current_node.kind() {
            AstKind::VariableDeclaration(_)
            | AstKind::MethodDefinition(_)
            | AstKind::PropertyDefinition(_)
            // /** This JSDoc should NOT found for `ArrowFunctionExpression` callback */
            // function outer() { inner(() => {}) }
            | AstKind::CallExpression(_)
            // /** This JSDoc should NOT found for `ArrowFunctionExpression` callback */
            // new Promise(() => {})
            | AstKind::NewExpression(_)
            // /** This JSDoc should NOT found for inner `Function` */
            // function outer() { return function inner() {} }
            | AstKind::ReturnStatement(_)
            => {
                // /** This JSDoc should NOT found for `VariableDeclaration` */
                // export const foo = () => {}
                let parent_node = ctx.nodes().parent_node(current_node.id())?;
                match parent_node.kind() {
                    AstKind::ExportDefaultDeclaration(_) | AstKind::ExportNamedDeclaration(_) => return Some(parent_node),
                    _ => return None
                }
            },
            _ => current_node = ctx.nodes().parent_node(current_node.id())?,
        }
    }

    Some(current_node)
}

pub fn should_ignore_as_internal(jsdoc: &JSDoc, settings: &JSDocPluginSettings) -> bool {
    if settings.ignore_internal {
        let resolved_internal_tag_name = settings.resolve_tag_name("internal");

        for tag in jsdoc.tags() {
            if tag.kind.parsed() == resolved_internal_tag_name {
                return true;
            }
        }
    }

    false
}

pub fn should_ignore_as_private(jsdoc: &JSDoc, settings: &JSDocPluginSettings) -> bool {
    if settings.ignore_private {
        let resolved_private_tag_name = settings.resolve_tag_name("private");
        let resolved_access_tag_name = settings.resolve_tag_name("access");

        for tag in jsdoc.tags() {
            let tag_name = tag.kind.parsed();
            if tag_name == resolved_private_tag_name
                || tag_name == resolved_access_tag_name && tag.comment().parsed() == "private"
            {
                return true;
            }
        }
    }

    false
}

pub fn should_ignore_as_avoid(
    jsdoc: &JSDoc,
    settings: &JSDocPluginSettings,
    exempted_tag_names: &[String],
) -> bool {
    let mut ignore_tag_names =
        exempted_tag_names.iter().map(std::convert::Into::into).collect::<FxHashSet<_>>();
    if settings.ignore_replaces_docs {
        ignore_tag_names.insert(settings.resolve_tag_name("ignore"));
    }
    if settings.override_replaces_docs {
        ignore_tag_names.insert(settings.resolve_tag_name("override"));
    }
    if settings.augments_extends_replaces_docs {
        ignore_tag_names.insert(settings.resolve_tag_name("augments"));
        ignore_tag_names.insert(settings.resolve_tag_name("extends"));
    }
    if settings.implements_replaces_docs {
        ignore_tag_names.insert(settings.resolve_tag_name("implements"));
    }

    jsdoc.tags().iter().any(|tag| ignore_tag_names.contains(tag.kind.parsed()))
}
