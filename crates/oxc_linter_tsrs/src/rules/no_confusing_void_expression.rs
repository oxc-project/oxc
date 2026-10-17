// Port of internal/rules/no_confusing_void_expression/no_confusing_void_expression.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{ContextFlags, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor, opt_bool, options_object,
};
use crate::utils;

fn invalid_void_expr() -> RuleMessage {
    RuleMessage::with_help(
        "invalidVoidExpr",
        "Placing a void expression inside another expression is forbidden.",
        "Move it to its own statement instead.",
    )
}
fn invalid_void_expr_arrow() -> RuleMessage {
    RuleMessage::with_help(
        "invalidVoidExprArrow",
        "Returning a void expression from an arrow function shorthand is forbidden.",
        "Add braces to the arrow function.",
    )
}
fn invalid_void_expr_arrow_wrap_void() -> RuleMessage {
    RuleMessage::new(
        "invalidVoidExprArrowWrapVoid",
        "Void expressions returned from an arrow function shorthand must be marked explicitly with the `void` operator.",
    )
}
fn invalid_void_expr_return() -> RuleMessage {
    RuleMessage::with_help(
        "invalidVoidExprReturn",
        "Returning a void expression from a function is forbidden.",
        "Move it before the `return` statement.",
    )
}
fn invalid_void_expr_return_last() -> RuleMessage {
    RuleMessage::with_help(
        "invalidVoidExprReturnLast",
        "Returning a void expression from a function is forbidden.",
        "Remove the `return` statement.",
    )
}
fn invalid_void_expr_return_wrap_void() -> RuleMessage {
    RuleMessage::new(
        "invalidVoidExprReturnWrapVoid",
        "Void expressions returned from a function must be marked explicitly with the `void` operator.",
    )
}
fn invalid_void_expr_wrap_void() -> RuleMessage {
    RuleMessage::new(
        "invalidVoidExprWrapVoid",
        "Void expressions used inside another expression must be moved to its own statement or marked explicitly with the `void` operator.",
    )
}
fn void_expr_wrap_void() -> RuleMessage {
    RuleMessage::new("voidExprWrapVoid", "Mark with an explicit `void` operator.")
}

pub struct NoConfusingVoidExpression {
    ignore_arrow_shorthand: bool,
    ignore_void_operator: bool,
    ignore_void_returning_functions: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(NoConfusingVoidExpression {
        ignore_arrow_shorthand: opt_bool(&m, "ignoreArrowShorthand", false),
        ignore_void_operator: opt_bool(&m, "ignoreVoidOperator", false),
        ignore_void_returning_functions: opt_bool(&m, "ignoreVoidReturningFunctions", false),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::AwaitExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::TaggedTemplateExpression),
];

impl Rule for NoConfusingVoidExpression {
    fn name(&self) -> &'static str {
        "no-confusing-void-expression"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static NoConfusingVoidExpression,
}

fn can_fix(ctx: &mut Ctx, node: P<Node>) -> bool {
    let t = utils::get_constrained_type_at_location(ctx.checker, node);
    utils::is_type_flag_set(t, TypeFlags::VoidLike)
}

fn is_final_return(node: P<Node>) -> bool {
    let Some(block) = node.parent() else {
        return false;
    };
    if !ast::is_block(block) {
        return false;
    }
    if !ast::is_function_like_declaration(block.parent()) {
        return false;
    }
    let statements = block.statements();
    statements.last() == Some(&node)
}

fn is_semicolon_needed_token(kind: Kind) -> bool {
    matches!(kind, Kind::OpenParenToken | Kind::OpenBracketToken | Kind::BacktickToken)
}

impl Visitor {
    fn find_invalid_ancestor(&self, mut node: P<Node>) -> Option<P<Node>> {
        let mut parent = node;
        loop {
            node = parent;
            parent = parent.parent()?;
            match parent.kind() {
                Kind::ParenthesizedExpression => continue,
                Kind::BinaryExpression => {
                    let n = parent.as_binary_expression();
                    let op = n.operator_token.kind();
                    if ast::is_logical_or_coalescing_binary_operator(op) && n.right.get() == node {
                        // e.g. `x && console.log(x)`
                        // this is valid only if the next ancestor is valid
                        continue;
                    }
                    if op == Kind::CommaToken && n.left == node {
                        return None;
                    }
                }
                Kind::ExpressionStatement => {
                    // e.g. `{ console.log("foo"); }`
                    // this is always valid
                    return None;
                }
                Kind::ConditionalExpression => {
                    let n = parent.as_conditional_expression();
                    if n.when_true == node || n.when_false == node {
                        // e.g. `cond ? console.log(true) : console.log(false)`
                        // this is valid only if the next ancestor is valid
                        continue;
                    }
                }
                // e.g. `() => console.log("foo")` is valid with an appropriate option
                Kind::ArrowFunction if self.o.ignore_arrow_shorthand => return None,
                Kind::VoidExpression if self.o.ignore_void_operator => return None,
                _ => {}
            }
            break;
        }
        // Any other parent is invalid.
        Some(parent)
    }

    fn is_void_returning_function(&self, ctx: &mut Ctx, function_node: P<Node>) -> bool {
        if let Some(return_type_node) = function_node.type_node() {
            let return_type = ctx.checker.get_type_from_type_node(return_type_node);
            return utils::union_type_parts(return_type)
                .into_iter()
                .any(utils::is_intrinsic_void_type);
        }
        if !ast::is_arrow_function(function_node) && !ast::is_function_expression(function_node) {
            return false;
        }
        let Some(function_type) =
            ctx.checker.get_contextual_type(function_node, ContextFlags::None)
        else {
            return false;
        };
        for t in utils::union_type_parts(function_type) {
            for &s in utils::get_call_signatures(ctx.checker, t) {
                let return_type = ctx.checker.get_return_type_of_signature(s);
                if utils::union_type_parts(return_type)
                    .into_iter()
                    .any(utils::is_intrinsic_void_type)
                {
                    return true;
                }
            }
        }
        false
    }

    fn check_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        let Some(invalid_ancestor) = self.find_invalid_ancestor(node) else {
            // void expression is in valid position
            return;
        };
        let t = utils::get_constrained_type_at_location(ctx.checker, node);
        if !utils::is_type_flag_set(t, TypeFlags::VoidLike) {
            return;
        }
        let insert_void_fix = |ctx: &Ctx| ctx.fix_insert_before(node, "void ");

        if ast::is_arrow_function(invalid_ancestor) {
            if self.o.ignore_void_returning_functions
                && self.is_void_returning_function(ctx, invalid_ancestor)
            {
                return;
            }
            if self.o.ignore_void_operator {
                ctx.report_node_with_fixes(node, invalid_void_expr_arrow_wrap_void(), |ctx| {
                    vec![insert_void_fix(ctx)]
                });
                return;
            }
            ctx.report_node_with_fixes(node, invalid_void_expr_arrow(), |ctx| {
                let mut fixes = Vec::new();
                let body = invalid_ancestor.body().unwrap();
                if !ast::is_block(body) && can_fix(ctx, body) {
                    let without_parens = ast::skip_parentheses(body);
                    fixes = vec![
                        ctx.fix_replace_range(body.pos(), without_parens.pos(), "{ "),
                        ctx.fix_replace_range(without_parens.end(), body.end(), "; }"),
                    ];
                }
                fixes
            });
            return;
        }

        if ast::is_return_statement(invalid_ancestor) {
            if self.o.ignore_void_returning_functions {
                if let Some(function_node) = utils::get_parent_function_node(invalid_ancestor) {
                    if self.is_void_returning_function(ctx, function_node) {
                        return;
                    }
                }
            }
            if self.o.ignore_void_operator {
                ctx.report_node_with_fixes(node, invalid_void_expr_return_wrap_void(), |ctx| {
                    vec![insert_void_fix(ctx)]
                });
                return;
            }
            let expr = invalid_ancestor.expression().unwrap();
            if is_final_return(invalid_ancestor) {
                ctx.report_node_with_fixes(node, invalid_void_expr_return_last(), |ctx| {
                    let mut fixes = Vec::new();
                    if can_fix(ctx, expr) {
                        let next_token = tsrs_scanner::scan_token_at_position(ctx.file, expr.pos());
                        let replace_text =
                            if is_semicolon_needed_token(next_token) { ";" } else { "" };
                        let r = tsrs_scanner::get_range_of_token_at_position(
                            ctx.file,
                            invalid_ancestor.pos(),
                        );
                        fixes.push(ctx.fix_replace_range(r.pos(), r.end(), replace_text));
                    }
                    fixes
                });
                return;
            }
            ctx.report_node_with_fixes(node, invalid_void_expr_return(), |ctx| {
                let next_token = tsrs_scanner::scan_token_at_position(ctx.file, expr.pos());
                let replace_text = if is_semicolon_needed_token(next_token) { ";" } else { "" };
                let r =
                    tsrs_scanner::get_range_of_token_at_position(ctx.file, invalid_ancestor.pos());
                let mut fixes = vec![
                    ctx.fix_replace_range(r.pos(), r.end(), replace_text),
                    ctx.fix_insert_after(invalid_ancestor, "; return;"),
                ];
                if !invalid_ancestor.parent().is_some_and(ast::is_block) {
                    // e.g. `if (cond) return console.error();`
                    // add braces if not inside a block
                    fixes.push(ctx.fix_insert_before(invalid_ancestor, "{ "));
                    fixes.push(ctx.fix_insert_after(invalid_ancestor, " }"));
                }
                fixes
            });
            return;
        }

        if self.o.ignore_void_operator {
            ctx.report_node_with_suggestions(node, invalid_void_expr_wrap_void(), |ctx| {
                vec![RuleSuggestion {
                    message: void_expr_wrap_void(),
                    fixes: vec![insert_void_fix(ctx)],
                }]
            });
            return;
        }
        ctx.report_node(node, invalid_void_expr());
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        self.check_expression(ctx, node);
    }
}
