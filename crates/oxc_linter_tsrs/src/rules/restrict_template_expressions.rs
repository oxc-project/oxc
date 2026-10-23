// Port of internal/rules/restrict_template_expressions/restrict_template_expressions.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils::{self, TypeOrValueSpecifier, TypeOrValueSpecifierFrom};

fn build_invalid_type_diagnostic(pos: i32, end: i32, t: &str) -> RuleDiagnostic {
    RuleDiagnostic {
        pos,
        end,
        message: RuleMessage::new(
            "invalidType",
            "Invalid type used in template literal expression.",
        ),
        labeled_ranges: vec![LabeledRange { label: format!("Type: {t}"), pos, end }],
    }
}

pub struct RestrictTemplateExpressions {
    allow: Vec<TypeOrValueSpecifier>,
    allowed_flags: TypeFlags,
    allow_reg_exp: bool,
    allow_array: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let allow = match m.get("allow") {
        None | Some(serde_json::Value::Null) => vec![TypeOrValueSpecifier {
            from: TypeOrValueSpecifierFrom::Lib,
            name: vec!["Error".into(), "URL".into(), "URLSearchParams".into()],
            path: String::new(),
            package: String::new(),
        }],
        v => utils::unmarshal_type_or_value_specifiers(v)?,
    };
    let mut allowed_flags = TypeFlags::StringLike;
    if opt_bool(&m, "allowAny", true) {
        allowed_flags |= TypeFlags::Any;
    }
    if opt_bool(&m, "allowBoolean", true) {
        allowed_flags |= TypeFlags::BooleanLike;
    }
    if opt_bool(&m, "allowNullish", true) {
        allowed_flags |= TypeFlags::Nullable;
    }
    if opt_bool(&m, "allowNumber", true) {
        allowed_flags |= TypeFlags::NumberLike | TypeFlags::BigIntLike;
    }
    if opt_bool(&m, "allowNever", false) {
        allowed_flags |= TypeFlags::Never;
    }
    Ok(Box::new(RestrictTemplateExpressions {
        allow,
        allowed_flags,
        allow_reg_exp: opt_bool(&m, "allowRegExp", true),
        allow_array: opt_bool(&m, "allowArray", false),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::TemplateExpression)];

impl Rule for RestrictTemplateExpressions {
    fn name(&self) -> &'static str {
        "restrict-template-expressions"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static RestrictTemplateExpressions,
}

impl Visitor {
    fn is_type_allowed(&self, ctx: &mut Ctx, inner_type: P<Type>) -> bool {
        let opts = self.rule;
        let global_regexp_type = ctx.checker.global_reg_exp_type;
        utils::union_type_parts(inner_type).into_iter().all(|t| {
            utils::intersection_type_parts(t).into_iter().any(|t| {
                utils::is_type_flag_set(t, opts.allowed_flags)
                    || (opts.allow_reg_exp && t == global_regexp_type)
                    || (!opts.allow.is_empty()
                        && utils::matches_type_or_base_type(ctx.checker, t, &mut |_c, t| {
                            utils::type_matches_some_specifier(t, &opts.allow, ctx.program)
                        }))
                    || (opts.allow_array && ctx.checker.is_array_or_tuple_type(t) && {
                        let elem = utils::get_number_index_type(ctx.checker, t).unwrap();
                        self.is_type_allowed(ctx, elem)
                    })
            })
        })
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        // don't check tagged template literals
        if node.parent().is_some_and(ast::is_tagged_template_expression) {
            return;
        }
        for &span in node.as_template_expression().template_spans().nodes() {
            let expression = span.expression().unwrap();
            let expression_type = utils::get_constrained_type_at_location(ctx.checker, expression);
            if !self.is_type_allowed(ctx, expression_type) {
                let (pos, end) = ctx.trim(expression);
                let type_string = crate::utils::type_to_string(ctx.checker, expression_type);
                ctx.report_diagnostic(build_invalid_type_diagnostic(pos, end, &type_string));
            }
        }
    }
}
