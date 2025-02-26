use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    Format, Prettier, array, dynamic_text, format::print::call_arguments, group, indent, ir::Doc,
    line, softline, text,
};

pub enum CallExpressionLike<'a, 'b> {
    CallExpression(&'b CallExpression<'a>),
    NewExpression(&'b NewExpression<'a>),
    V8Intrinsic(&'b V8IntrinsicExpression<'a>),
}

pub enum Callee<'a, 'b> {
    Expression(&'b Expression<'a>),
    V8IntrinsicName(&'b IdentifierName<'a>),
}

impl<'b> Callee<'b, '_> {
    fn format<'a>(&self, p: &mut Prettier<'a>) -> Doc<'a>
    where
        'b: 'a,
    {
        match self {
            Callee::Expression(expr) => expr.format(p),
            Callee::V8IntrinsicName(name) => {
                array!(p, [text!("%"), dynamic_text!(p, &name.name)])
            }
        }
    }

    fn is_call_expression(&self) -> bool {
        matches!(self, Callee::Expression(Expression::CallExpression(_)))
    }
}

impl<'a> CallExpressionLike<'a, '_> {
    fn is_new(&self) -> bool {
        matches!(self, CallExpressionLike::NewExpression(_))
    }

    pub fn callee(&self) -> Callee<'a, '_> {
        match self {
            CallExpressionLike::CallExpression(call) => Callee::Expression(&call.callee),
            CallExpressionLike::NewExpression(new) => Callee::Expression(&new.callee),
            CallExpressionLike::V8Intrinsic(expr) => Callee::V8IntrinsicName(&expr.name),
        }
    }

    fn optional(&self) -> bool {
        match self {
            CallExpressionLike::CallExpression(call) => call.optional,
            CallExpressionLike::NewExpression(new) => false,
            CallExpressionLike::V8Intrinsic(_) => false,
        }
    }

    pub fn arguments(&self) -> &Vec<'a, Argument<'a>> {
        match self {
            CallExpressionLike::CallExpression(call) => &call.arguments,
            CallExpressionLike::NewExpression(new) => &new.arguments,
            CallExpressionLike::V8Intrinsic(expr) => &expr.arguments,
        }
    }

    fn type_parameters(&self) -> Option<&oxc_allocator::Box<'a, TSTypeParameterInstantiation<'a>>> {
        match self {
            CallExpressionLike::CallExpression(call) => call.type_parameters.as_ref(),
            CallExpressionLike::NewExpression(new) => new.type_parameters.as_ref(),
            CallExpressionLike::V8Intrinsic(_) => None,
        }
    }
}

pub fn print_call_expression<'a>(
    p: &mut Prettier<'a>,
    expr: &CallExpressionLike<'a, '_>,
) -> Doc<'a> {
    // TODO:
    // if (
    //   isTemplateLiteralSingleArg ||
    //   // Dangling comments are not handled, all these special cases should have arguments #9668
    //   // We want to keep CommonJS- and AMD-style require calls, and AMD-style
    //   // define calls, as a unit.
    //   // e.g. `define(["some/lib"], (lib) => {`
    //   isCommonsJsOrAmdModuleDefinition(path) ||
    //   // Keep test declarations on a single line
    //   // e.g. `it('long name', () => {`
    //   isTestCall(node, path.parent)
    // ) {
    //   const printed = [];
    //   iterateCallArgumentsPath(path, () => printed.push(print()));
    //   if (!printed[0].label?.embed) {
    //     return [
    //       isNew ? "new " : "",
    //       printCallee(path, print),
    //       optional,
    //       printFunctionTypeParameters(path, options, print),
    //       "(",
    //       join(", ", printed),
    //       ")",
    //     ];
    //   }
    // }

    // TODO:
    // if (
    //   !isNew &&
    //   isMemberish(node.callee) &&
    //   !path.call(
    //     (path) => pathNeedsParens(path, options),
    //     "callee",
    //     ...(node.callee.type === "ChainExpression" ? ["expression"] : []),
    //   )
    // )
    //   return printMemberChain(path, options, print);

    let mut parts = Vec::new_in(p.allocator);

    if expr.is_new() {
        parts.push(text!("new "));
    };
    parts.push(expr.callee().format(p));
    if expr.optional() {
        parts.push(text!("?."));
    }
    if let Some(type_parameters) = expr.type_parameters() {
        parts.push(type_parameters.format(p));
    }
    parts.push(call_arguments::print_call_arguments(p, expr));

    if expr.callee().is_call_expression() {
        return group!(p, parts);
    }

    array!(p, parts)
}

// In Prettier, `printCallExpression()` has 4 branches.
// But for `ImportExpression`, it only passes the 1st and 3rd branches.
// - if (isTemplateLiteralSingleArg) return [callee, "(", source, ")"];
// - return group([callee, callArguments([source, arguments])]);
pub fn print_import_expression<'a>(p: &mut Prettier<'a>, expr: &ImportExpression<'a>) -> Doc<'a> {
    let callee_doc = {
        if let Some(phase) = &expr.phase {
            array!(p, [text!("import."), dynamic_text!(p, phase.as_str())]);
        }
        text!("import")
    };

    // TODO: isTemplateLiteralSingleArg branch
    // return [callee, "(", source, ")"];

    group!(p, [callee_doc, call_arguments::print_import_source_and_arguments(p, expr)])
}

/// <https://github.com/prettier/prettier/blob/7aecca5d6473d73f562ca3af874831315f8f2581/src/language-js/print/call-expression.js#L93-L116>
pub fn is_commons_js_or_amd_call<'a>(
    callee: &Expression<'a>,
    arguments: &Vec<'a, Argument<'a>>,
) -> bool {
    if let Expression::Identifier(callee) = callee {
        if callee.name == "require" {
            return arguments.len() == 1 && matches!(arguments[0], Argument::StringLiteral(_))
                || arguments.len() > 1;
        }
        if callee.name == "define" {
            // TODO: the parent node is ExpressionStatement
            return arguments.len() == 1
                || (arguments.len() == 2 && matches!(arguments[1], Argument::ArrayExpression(_)))
                || (arguments.len() == 3
                    && matches!(arguments[0], Argument::StringLiteral(_))
                    && matches!(arguments[1], Argument::ArrayExpression(_)));
        }
    }
    false
}
