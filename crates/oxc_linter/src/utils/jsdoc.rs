use oxc_ast::{
    ast::{BindingPattern, BindingPatternKind, Expression, FormalParameters},
    AstKind,
};
use oxc_semantic::JSDoc;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{config::JSDocPluginSettings, context::LintContext, AstNode};

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
        exempted_tag_names.iter().map(String::as_str).collect::<FxHashSet<_>>();
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

#[derive(Debug, Clone)]
pub struct Param {
    pub span: Span,
    pub name: String,
    pub is_rest: bool,
}

#[derive(Debug, Clone)]
pub enum ParamKind {
    Single(Param),
    Nested(Vec<Param>),
}

pub fn collect_params(params: &FormalParameters) -> Vec<ParamKind> {
    // NOTE: Property level `is_rest` is implemented.
    //   - fn(a, { b1, ...b2 })
    //                 ^^^^^
    // But Object|Array level `is_rest` is not implemented
    //   - fn(a, ...{ b })
    //           ^^^^   ^
    // Tests are not covering these cases...
    fn get_param_name(pattern: &BindingPattern, is_rest: bool) -> ParamKind {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                ParamKind::Single(Param { span: ident.span, name: ident.name.to_string(), is_rest })
            }
            BindingPatternKind::ObjectPattern(obj_pat) => {
                let mut collected = vec![];

                for prop in &obj_pat.properties {
                    let Some(name) = prop.key.name() else { continue };

                    match get_param_name(&prop.value, false) {
                        ParamKind::Single(param) => {
                            collected.push(Param { name: format!("{name}"), ..param });
                        }
                        ParamKind::Nested(params) => {
                            collected.push(Param {
                                span: prop.span,
                                name: format!("{name}"),
                                is_rest: false,
                            });

                            for param in params {
                                collected.push(Param {
                                    name: format!("{name}.{}", param.name),
                                    ..param
                                });
                            }
                        }
                    }
                }

                if let Some(rest) = &obj_pat.rest {
                    match get_param_name(&rest.argument, true) {
                        ParamKind::Single(param) => collected.push(param),
                        ParamKind::Nested(params) => collected.extend(params),
                    }
                }

                ParamKind::Nested(collected)
            }
            BindingPatternKind::ArrayPattern(arr_pat) => {
                let mut collected = vec![];

                for (idx, elm) in arr_pat.elements.iter().enumerate() {
                    let name = format!("\"{idx}\"");

                    if let Some(pat) = elm {
                        match get_param_name(pat, false) {
                            ParamKind::Single(param) => collected.push(Param { name, ..param }),
                            ParamKind::Nested(params) => collected.extend(params),
                        }
                    }
                }

                if let Some(rest) = &arr_pat.rest {
                    match get_param_name(&rest.argument, true) {
                        ParamKind::Single(param) => collected.push(param),
                        ParamKind::Nested(params) => collected.extend(params),
                    }
                }

                ParamKind::Nested(collected)
            }
            BindingPatternKind::AssignmentPattern(assign_pat) => match &assign_pat.right {
                Expression::Identifier(_) => get_param_name(&assign_pat.left, false),
                _ => {
                    // TODO: If `config.useDefaultObjectProperties` = true,
                    // collect default parameters from `assign_pat.right` like:
                    // { prop = { a: 1, b: 2 }} => [prop, prop.a, prop.b]
                    //     get_param_name(&assign_pat.left, false)
                    // }
                    get_param_name(&assign_pat.left, false)
                }
            },
        }
    }

    let mut collected =
        params.items.iter().map(|param| get_param_name(&param.pattern, false)).collect::<Vec<_>>();

    if let Some(rest) = &params.rest {
        match get_param_name(&rest.argument, true) {
            ParamKind::Single(param) => collected.push(ParamKind::Single(param)),
            ParamKind::Nested(params) => collected.push(ParamKind::Nested(params)),
        }
    }

    collected
}
