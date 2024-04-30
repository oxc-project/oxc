use crate::{config::JSDocPluginSettings, context::LintContext, AstNode};
use oxc_ast::AstKind;
use oxc_semantic::JSDoc;

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
        match current_node.kind() {
            AstKind::VariableDeclaration(_)
            | AstKind::MethodDefinition(_)
            | AstKind::PropertyDefinition(_) => return None,
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
