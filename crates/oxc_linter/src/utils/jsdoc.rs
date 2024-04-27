use crate::{context::LintContext, AstNode};
use oxc_ast::AstKind;

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
