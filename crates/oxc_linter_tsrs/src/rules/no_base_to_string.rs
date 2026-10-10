// Port of internal/rules/no_base_to_string/no_base_to_string.go.

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Usefulness {
    Always,
    Never,
    Sometimes,
}

fn certainty_to_string(certainty: Usefulness) -> &'static str {
    match certainty {
        Usefulness::Always => "always",
        Usefulness::Never => "will",
        Usefulness::Sometimes => "may",
    }
}

fn build_base_array_join_message(name: &str, certainty: Usefulness) -> RuleMessage {
    RuleMessage::with_help(
        "baseArrayJoin",
        format!(
            "Using `join()` for {} {} use Object's default stringification format ('[object Object]') when stringified.",
            name,
            certainty_to_string(certainty)
        ),
        "Consider mapping the values to a meaningful string (e.g. pick a property or call a formatter) before calling `join()`, or implementing a custom `toString()`/`toLocaleString()` on the element type.",
    )
}

fn build_base_to_string_message(name: &str, certainty: Usefulness) -> RuleMessage {
    RuleMessage::with_help(
        "baseToString",
        format!(
            "'{}' {} use Object's default stringification format ('[object Object]') when stringified.",
            name,
            certainty_to_string(certainty)
        ),
        "Consider picking a property (e.g. `user.name`), using a formatter (or `JSON.stringify`), or implementing a custom `toString()`/`toLocaleString()` on the type.",
    )
}

#[derive(Default)]
struct CertaintyMemo {
    values: FxHashMap<P<Type>, Usefulness>,
    resolving: FxHashSet<P<Type>>,
}

pub struct NoBaseToString {
    check_unknown: bool,
    ignored_type_names: Vec<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let ignored_type_names = match m.get("ignoredTypeNames") {
        Some(serde_json::Value::Array(a)) => {
            a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
        }
        _ => ["Error", "RegExp", "URL", "URLSearchParams"].iter().map(|s| s.to_string()).collect(),
    };
    Ok(Box::new(NoBaseToString {
        check_unknown: opt_bool(&m, "checkUnknown", false),
        ignored_type_names,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::TemplateExpression),
];

impl Rule for NoBaseToString {
    fn name(&self) -> &'static str {
        "no-base-to-string"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            opts: self,
            to_string_memo: CertaintyMemo::default(),
            join_memo: CertaintyMemo::default(),
        })
    }
}

struct Visitor {
    opts: &'static NoBaseToString,
    to_string_memo: CertaintyMemo,
    join_memo: CertaintyMemo,
}

impl Visitor {
    fn check_expression(&mut self, ctx: &mut Ctx, node: P<Node>, t: Option<P<Type>>) {
        // TODO(port): boolean, null, etc?
        if ast::is_literal_expression(node) {
            return;
        }
        let t = t.unwrap_or_else(|| ctx.checker.get_type_at_location(node));
        let certainty = self.collect_to_string_certainty(ctx, t, &[]);
        if certainty == Usefulness::Always {
            return;
        }
        let text = tsrs_scanner::get_source_text_of_node_from_source_file(ctx.file, node, false);
        ctx.report_node(node, build_base_to_string_message(&text, certainty));
    }

    fn check_expression_for_array_join(&mut self, ctx: &mut Ctx, node: P<Node>, t: P<Type>) {
        let certainty = self.collect_join_certainty(ctx, t, &[]);
        if certainty == Usefulness::Always {
            return;
        }
        let text = tsrs_scanner::get_source_text_of_node_from_source_file(ctx.file, node, false);
        ctx.report_node(node, build_base_array_join_message(&text, certainty));
    }

    fn collect_union_type_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        sub: &mut dyn FnMut(&mut Self, &mut Ctx, P<Type>) -> Usefulness,
    ) -> Usefulness {
        let (mut all_never, mut all_always) = (true, true);
        for part in utils::union_type_parts(t) {
            let certainty = sub(self, ctx, part);
            all_never = all_never && certainty == Usefulness::Never;
            all_always = all_always && certainty == Usefulness::Always;
        }
        if all_never {
            return Usefulness::Never;
        }
        if all_always {
            return Usefulness::Always;
        }
        Usefulness::Sometimes
    }

    fn collect_intersection_type_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        sub: &mut dyn FnMut(&mut Self, &mut Ctx, P<Type>) -> Usefulness,
    ) -> Usefulness {
        for part in utils::intersection_type_parts(t) {
            if sub(self, ctx, part) == Usefulness::Always {
                return Usefulness::Always;
            }
        }
        Usefulness::Never
    }

    fn collect_tuple_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        visited: &[P<Type>],
    ) -> Usefulness {
        let type_args = ctx.checker.get_type_arguments(t);
        let (mut has_never, mut has_sometimes) = (false, false);
        for &type_arg in type_args {
            let certainty = self.collect_to_string_certainty(ctx, type_arg, visited);
            has_never = has_never || certainty == Usefulness::Never;
            has_sometimes = has_sometimes || certainty == Usefulness::Sometimes;
        }
        if has_never {
            return Usefulness::Never;
        }
        if has_sometimes {
            return Usefulness::Sometimes;
        }
        Usefulness::Always
    }

    fn collect_array_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        visited: &[P<Type>],
    ) -> Usefulness {
        let elem_type = utils::get_number_index_type(ctx.checker, t)
            .expect("array should have number index type");
        self.collect_to_string_certainty(ctx, elem_type, visited)
    }

    fn collect_join_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        visited: &[P<Type>],
    ) -> Usefulness {
        if let Some(&c) = self.join_memo.values.get(&t) {
            return c;
        }
        if self.join_memo.resolving.contains(&t) {
            return Usefulness::Always;
        }
        self.join_memo.resolving.insert(t);
        let certainty = self.compute_join_certainty(ctx, t, visited);
        self.join_memo.resolving.remove(&t);
        self.join_memo.values.insert(t, certainty);
        certainty
    }

    fn compute_join_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        visited: &[P<Type>],
    ) -> Usefulness {
        if utils::is_union_type(t) {
            return self.collect_union_type_certainty(ctx, t, &mut |s, ctx, t| {
                s.collect_join_certainty(ctx, t, visited)
            });
        }
        if utils::is_intersection_type(t) {
            return self.collect_intersection_type_certainty(ctx, t, &mut |s, ctx, t| {
                s.collect_join_certainty(ctx, t, visited)
            });
        }
        if t.is_tuple_type() {
            return self.collect_tuple_certainty(ctx, t, visited);
        }
        if ctx.checker.is_array_type(t) {
            return self.collect_array_certainty(ctx, t, visited);
        }
        Usefulness::Always
    }

    fn collect_to_string_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        visited: &[P<Type>],
    ) -> Usefulness {
        if visited.contains(&t) {
            // don't report if this is a self referencing array or tuple type
            return Usefulness::Always;
        }
        if let Some(&c) = self.to_string_memo.values.get(&t) {
            return c;
        }
        if self.to_string_memo.resolving.contains(&t) {
            return Usefulness::Always;
        }
        self.to_string_memo.resolving.insert(t);
        let certainty = self.compute_to_string_certainty(ctx, t, visited);
        self.to_string_memo.resolving.remove(&t);
        self.to_string_memo.values.insert(t, certainty);
        certainty
    }

    fn compute_to_string_certainty(
        &mut self,
        ctx: &mut Ctx,
        t: P<Type>,
        visited: &[P<Type>],
    ) -> Usefulness {
        if utils::is_type_parameter(t) {
            if let Some(constraint) = ctx.checker.get_base_constraint_of_type(t) {
                return self.collect_to_string_certainty(ctx, constraint, visited);
            }
            // unconstrained generic means `unknown`
            if self.opts.check_unknown {
                return Usefulness::Sometimes;
            }
            return Usefulness::Always;
        }

        // the Boolean type definition missing toString()
        if utils::is_type_flag_set(t, TypeFlags::BooleanLike) {
            return Usefulness::Always;
        }

        let ignored = &self.opts.ignored_type_names;
        if utils::matches_type_or_base_type(ctx.checker, t, &mut |c, t| {
            let name = utils::get_type_name(c, t);
            ignored.contains(&name)
        }) {
            return Usefulness::Always;
        }

        if utils::is_intersection_type(t) {
            return self.collect_intersection_type_certainty(ctx, t, &mut |s, ctx, t| {
                s.collect_to_string_certainty(ctx, t, visited)
            });
        }

        if utils::is_union_type(t) {
            return self.collect_union_type_certainty(ctx, t, &mut |s, ctx, t| {
                s.collect_to_string_certainty(ctx, t, visited)
            });
        }

        if t.is_tuple_type() {
            let mut v = visited.to_vec();
            v.push(t);
            return self.collect_tuple_certainty(ctx, t, &v);
        }

        if ctx.checker.is_array_type(t) {
            let mut v = visited.to_vec();
            v.push(t);
            return self.collect_array_certainty(ctx, t, &v);
        }

        let mut found_fallback_on_object = false;
        for property_name in ["toString", "toLocaleString", "valueOf"] {
            let Some(property) = ctx.checker.get_property_of_type(t, property_name) else {
                continue;
            };
            let declarations = property.declarations();
            if declarations.is_empty() {
                continue;
            }
            // If any declaration is not from the Object interface, this is user-defined (e.g.
            // overloaded toString/toLocaleString/valueOf).
            if declarations.iter().any(|&declaration| {
                !declaration.parent().is_some_and(|parent| {
                    ast::is_interface_declaration(parent)
                        && parent.name().is_some_and(|n| n.text() == "Object")
                })
            }) {
                return Usefulness::Always;
            }
            found_fallback_on_object = true;
        }

        if found_fallback_on_object {
            return Usefulness::Never;
        }

        // unknown
        if self.opts.check_unknown && utils::is_type_flag_set(t, TypeFlags::Unknown) {
            return Usefulness::Sometimes;
        }
        // e.g. any
        Usefulness::Always
    }

    fn is_built_in_string_call(ctx: &mut Ctx, node: P<Node>) -> bool {
        let expression = node.expression().unwrap();
        if ast::is_identifier(expression)
            && expression.text() == "String"
            && !node.arguments().is_empty()
        {
            let tt = ctx.checker.get_type_at_location(expression);
            let s = utils::is_builtin_symbol_like(ctx.program, ctx.checker, tt, &["String"]);
            let sc =
                utils::is_builtin_symbol_like(ctx.program, ctx.checker, tt, &["StringConstructor"]);
            return s || sc;
        }
        false
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::BinaryExpression => {
                let expr = node.as_binary_expression();
                let op = expr.operator_token.kind();
                if op != Kind::PlusToken && op != Kind::PlusEqualsToken {
                    return;
                }
                let left = expr.left;
                let right = expr.right.get();
                let left_type = ctx.checker.get_type_at_location(left);
                let right_type = ctx.checker.get_type_at_location(right);
                if utils::get_type_name(ctx.checker, left_type) == "string" {
                    self.check_expression(ctx, right, Some(right_type));
                } else if utils::get_type_name(ctx.checker, right_type) == "string"
                    && left.kind() != Kind::PrivateIdentifier
                {
                    self.check_expression(ctx, left, Some(left_type));
                }
            }
            Kind::CallExpression => {
                let args = node.arguments();
                if Self::is_built_in_string_call(ctx, node) && args[0].kind() != Kind::SpreadElement
                {
                    self.check_expression(ctx, args[0], None);
                    return;
                }
                let callee = node.expression().unwrap();
                if ast::is_property_access_expression(callee) {
                    let property_name = callee.name().unwrap().text();
                    let object = callee.expression().unwrap();
                    if property_name == "join" {
                        let t = utils::get_constrained_type_at_location(ctx.checker, object);
                        self.check_expression_for_array_join(ctx, object, t);
                    } else if property_name == "toLocaleString" || property_name == "toString" {
                        self.check_expression(ctx, object, None);
                    }
                }
            }
            Kind::TemplateExpression => {
                if node.parent().is_some_and(ast::is_tagged_template_expression) {
                    return;
                }
                for &span in node.as_template_expression().template_spans().nodes() {
                    self.check_expression(ctx, span.expression().unwrap(), None);
                }
            }
            _ => {}
        }
    }
}
