// Port of internal/rules/promise_function_async/promise_function_async.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, ObjectFlags, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor, opt_bool, options_object,
};
use crate::utils;

fn missing_async() -> RuleMessage {
    RuleMessage::new("missingAsync", "Functions that return promises must be async.")
}
fn missing_async_hybrid_return() -> RuleMessage {
    RuleMessage::with_help(
        "missingAsyncHybridReturn",
        "Functions that return promises must be async.",
        "Consider adding an explicit return type annotation if the function is intended to return a union of promise and non-promise types.",
    )
}
fn missing_async_hybrid_return_suggestion() -> RuleMessage {
    RuleMessage::new("missingAsyncHybridReturnSuggestion", "Add `async` keyword to the function.")
}

pub struct PromiseFunctionAsync {
    allow_any: bool,
    allowed_promise_names: FxHashSet<String>,
    listeners: Vec<Listener>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let mut allowed_promise_names: FxHashSet<String> = FxHashSet::default();
    allowed_promise_names.insert("Promise".to_string());
    if let Some(names) = m.get("allowedPromiseNames").and_then(|v| v.as_array()) {
        for n in names {
            if let Some(s) = n.as_str() {
                allowed_promise_names.insert(s.to_string());
            }
        }
    }
    let mut listeners = Vec::new();
    if opt_bool(&m, "checkArrowFunctions", true) {
        listeners.push(Listener::Enter(Kind::ArrowFunction));
    }
    if opt_bool(&m, "checkFunctionDeclarations", true) {
        listeners.push(Listener::Enter(Kind::FunctionDeclaration));
    }
    if opt_bool(&m, "checkFunctionExpressions", true) {
        listeners.push(Listener::Enter(Kind::FunctionExpression));
    }
    if opt_bool(&m, "checkMethodDeclarations", true) {
        listeners.push(Listener::Enter(Kind::MethodDeclaration));
    }
    Ok(Box::new(PromiseFunctionAsync {
        allow_any: opt_bool(&m, "allowAny", true),
        allowed_promise_names,
        listeners,
    }))
}

impl Rule for PromiseFunctionAsync {
    fn name(&self) -> &'static str {
        "promise-function-async"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static PromiseFunctionAsync,
}

impl Visitor {
    fn contains_all_types_by_name(
        &self,
        c: &mut Checker,
        mut t: P<Type>,
        match_any_instead: bool,
    ) -> bool {
        if utils::is_type_flag_set(t, TypeFlags::AnyOrUnknown) {
            return false;
        }
        if utils::is_type_flag_set(t, TypeFlags::Object)
            && t.object_flags().intersects(ObjectFlags::Reference)
        {
            t = t.target().unwrap();
        }
        if let Some(symbol) = t.symbol() {
            if self.o.allowed_promise_names.contains(symbol.name()) {
                return true;
            }
        }
        if utils::is_union_type(t) || utils::is_intersection_type(t) {
            let types = t.types();
            return if match_any_instead {
                types.iter().all(|&t| self.contains_all_types_by_name(c, t, match_any_instead))
            } else {
                types.iter().any(|&t| self.contains_all_types_by_name(c, t, match_any_instead))
            };
        }
        if !t.object_flags().intersects(ObjectFlags::ClassOrInterface) {
            return false;
        }
        let bases = c.get_base_types(t);
        if match_any_instead {
            bases.iter().any(|&b| self.contains_all_types_by_name(c, b, match_any_instead))
        } else {
            !bases.is_empty()
                && bases.iter().all(|&b| self.contains_all_types_by_name(c, b, match_any_instead))
        }
    }

    fn validate_node(&self, ctx: &mut Ctx, node: P<Node>) {
        if utils::includes_modifier(node, Kind::AsyncKeyword) || node.body().is_none() {
            return;
        }
        let t = ctx.checker.get_type_at_location(node);
        let signatures = utils::get_call_signatures(ctx.checker, t);
        if signatures.is_empty() {
            return;
        }
        let has_explicit_return_type = node.type_node().is_some();
        let mut every_signature_returns_promise = true;
        for &sig in signatures {
            let return_type = ctx.checker.get_return_type_of_signature(sig);
            if !self.o.allow_any && utils::is_type_flag_set(return_type, TypeFlags::AnyOrUnknown) {
                ctx.report_node(node, missing_async());
                return;
            }
            every_signature_returns_promise = every_signature_returns_promise
                && self.contains_all_types_by_name(
                    ctx.checker,
                    return_type,
                    has_explicit_return_type,
                );
        }
        if !every_signature_returns_promise {
            return;
        }
        let mut is_hybrid_return_type = false;
        if !has_explicit_return_type {
            for &sig in signatures {
                let return_type = ctx.checker.get_return_type_of_signature(sig);
                if utils::is_union_type(return_type) {
                    let all = return_type
                        .types()
                        .iter()
                        .all(|&p| self.contains_all_types_by_name(ctx.checker, p, true));
                    if !all {
                        is_hybrid_return_type = true;
                        break;
                    }
                }
            }
        }
        let insert_async_fix = |ctx: &Ctx| {
            if ast::is_method_declaration(node) {
                return ctx.fix_insert_before(node.name().unwrap(), " async ");
            }
            if ast::is_function_declaration(node) {
                if let Some(last) = node.modifier_nodes().last() {
                    return ctx.fix_insert_after(*last, " async");
                }
            }
            ctx.fix_insert_before(node, " async ")
        };
        if is_hybrid_return_type {
            ctx.report_node_with_suggestions(node, missing_async_hybrid_return(), |ctx| {
                vec![RuleSuggestion {
                    message: missing_async_hybrid_return_suggestion(),
                    fixes: vec![insert_async_fix(ctx)],
                }]
            });
        } else {
            ctx.report_node_with_fixes(node, missing_async(), |ctx| vec![insert_async_fix(ctx)]);
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &self.o.listeners
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.kind() == Kind::MethodDeclaration
            && utils::includes_modifier(node, Kind::AbstractKeyword)
        {
            return;
        }
        self.validate_node(ctx, node);
    }
}
