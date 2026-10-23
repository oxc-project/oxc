// Port of internal/rules/restrict_plus_operands/restrict_plus_operands.go.

use tsrs_ast::{Kind, Node};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

type Range = (i32, i32);

fn build_bigint_and_number_diagnostic(
    expr_range: Range,
    left_range: Range,
    right_range: Range,
    left_type: &str,
    right_type: &str,
) -> RuleDiagnostic {
    RuleDiagnostic {
        pos: expr_range.0,
        end: expr_range.1,
        message: RuleMessage::new(
            "bigintAndNumber",
            "Numeric '+' operations must either be both bigints or both numbers.",
        ),
        labeled_ranges: vec![
            LabeledRange {
                label: format!("Type: {left_type}"),
                pos: left_range.0,
                end: left_range.1,
            },
            LabeledRange {
                label: format!("Type: {right_type}"),
                pos: right_range.0,
                end: right_range.1,
            },
        ],
    }
}

fn build_invalid_diagnostic(
    expr_range: Range,
    invalid_type: &str,
    string_like: &str,
) -> RuleDiagnostic {
    RuleDiagnostic {
        pos: expr_range.0,
        end: expr_range.1,
        message: RuleMessage::with_help(
            "invalid",
            format!("Invalid operand of type '{invalid_type}' for a '+' operation."),
            format!("Operands must each be a number or {string_like}."),
        ),
        labeled_ranges: Vec::new(),
    }
}

fn build_mismatched_diagnostic(
    expr_range: Range,
    left_range: Range,
    right_range: Range,
    string_like: &str,
    left_type: &str,
    right_type: &str,
) -> RuleDiagnostic {
    RuleDiagnostic {
        pos: expr_range.0,
        end: expr_range.1,
        message: RuleMessage::with_help(
            "mismatched",
            "Operands of '+' operations must be of the same type.",
            format!("Operands must both be a number or both be {string_like}."),
        ),
        labeled_ranges: vec![
            LabeledRange {
                label: format!("Type: {left_type}"),
                pos: left_range.0,
                end: left_range.1,
            },
            LabeledRange {
                label: format!("Type: {right_type}"),
                pos: right_range.0,
                end: right_range.1,
            },
        ],
    }
}

pub struct RestrictPlusOperands {
    allow_any: bool,
    allow_number_and_string: bool,
    allow_reg_exp: bool,
    skip_compound_assignments: bool,
    string_like: String,
    invalid_flags: TypeFlags,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let allow_any = opt_bool(&m, "allowAny", true);
    let allow_boolean = opt_bool(&m, "allowBoolean", true);
    let allow_nullish = opt_bool(&m, "allowNullish", true);
    let allow_number_and_string = opt_bool(&m, "allowNumberAndString", true);
    let allow_reg_exp = opt_bool(&m, "allowRegExp", true);
    let skip_compound_assignments = opt_bool(&m, "skipCompoundAssignments", false);

    let mut string_likes: Vec<&str> = Vec::with_capacity(5);
    if allow_any {
        string_likes.push("`any`");
    }
    if allow_boolean {
        string_likes.push("`boolean`");
    }
    if allow_nullish {
        string_likes.push("`null`");
    }
    if allow_reg_exp {
        string_likes.push("`RegExp`");
    }
    if allow_nullish {
        string_likes.push("`undefined`");
    }
    let string_like = match string_likes.len() {
        0 => "string".to_string(),
        1 => format!("string, allowing a string + {}", string_likes[0]),
        _ => format!("string, allowing a string + any of: {}", string_likes.join(", ")),
    };

    let mut invalid_flags = TypeFlags::ESSymbolLike | TypeFlags::Never | TypeFlags::Unknown;
    if !allow_any {
        invalid_flags |= TypeFlags::Any;
    }
    if !allow_boolean {
        invalid_flags |= TypeFlags::BooleanLike;
    }
    if !allow_nullish {
        invalid_flags |= TypeFlags::Nullable;
    }

    Ok(Box::new(RestrictPlusOperands {
        allow_any,
        allow_number_and_string,
        allow_reg_exp,
        skip_compound_assignments,
        string_like,
        invalid_flags,
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::BinaryExpression)];

impl Rule for RestrictPlusOperands {
    fn name(&self) -> &'static str {
        "restrict-plus-operands"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { opts: self })
    }
}

struct Visitor {
    opts: &'static RestrictPlusOperands,
}

impl Visitor {
    fn get_type_constrained(ctx: &mut Ctx, node: P<Node>) -> P<Type> {
        let t = utils::get_constrained_type_at_location(ctx.checker, node);
        ctx.checker.get_base_type_of_literal_type_exported(t)
    }

    fn check_invalid_plus_operand(
        &self,
        ctx: &mut Ctx,
        base_type: P<Type>,
        other_type: P<Type>,
    ) -> (TypeFlags, bool) {
        let opts = self.opts;
        let global_regexp_type = ctx.checker.global_reg_exp_type;
        let mut found_invalid = false;
        let mut flags = TypeFlags::empty();
        for part in utils::union_type_parts(base_type) {
            flags |= part.flags();
            if utils::is_type_flag_set(part, opts.invalid_flags) {
                return (flags, true);
            }
            // RegExps also contain TypeFlags::Any & TypeFlags::Object
            if part == global_regexp_type {
                if opts.allow_reg_exp && !utils::is_type_flag_set(other_type, TypeFlags::NumberLike)
                {
                    continue;
                }
            } else if (opts.allow_any || !utils::is_type_any_type(part))
                && !utils::intersection_type_parts(part).into_iter().all(utils::is_object_type)
            {
                continue;
            }
            found_invalid = true;
        }
        (flags, found_invalid)
    }

    fn check_plus_operands(&self, ctx: &mut Ctx, node: P<Node>) {
        let opts = self.opts;
        let expr = node.as_binary_expression();
        let left = expr.left;
        let right = expr.right.get();
        let left_type = Self::get_type_constrained(ctx, left);
        let right_type = Self::get_type_constrained(ctx, right);

        if left_type == right_type
            && utils::is_type_flag_set(
                left_type,
                TypeFlags::BigIntLike | TypeFlags::NumberLike | TypeFlags::StringLike,
            )
        {
            return;
        }

        let (left_type_flags, left_invalid) =
            self.check_invalid_plus_operand(ctx, left_type, right_type);
        let (right_type_flags, right_invalid) =
            self.check_invalid_plus_operand(ctx, right_type, left_type);

        if left_invalid {
            let range = ctx.trim(left);
            let ts = utils::type_to_string(ctx.checker, left_type);
            ctx.report_diagnostic(build_invalid_diagnostic(range, &ts, &opts.string_like));
        }
        if right_invalid {
            let range = ctx.trim(right);
            let ts = utils::type_to_string(ctx.checker, right_type);
            ctx.report_diagnostic(build_invalid_diagnostic(range, &ts, &opts.string_like));
        }
        if left_invalid || right_invalid {
            return;
        }

        let check_mismatched_plus_operands =
            |ctx: &mut Ctx, base_type_flags: TypeFlags, other_type_flags: TypeFlags| -> bool {
                if !opts.allow_number_and_string
                    && base_type_flags.intersects(TypeFlags::StringLike)
                    && other_type_flags.intersects(TypeFlags::NumberLike | TypeFlags::BigIntLike)
                {
                    let (er, lr, rr) = (ctx.trim(node), ctx.trim(left), ctx.trim(right));
                    let lt = utils::type_to_string(ctx.checker, left_type);
                    let rt = utils::type_to_string(ctx.checker, right_type);
                    ctx.report_diagnostic(build_mismatched_diagnostic(
                        er,
                        lr,
                        rr,
                        &opts.string_like,
                        &lt,
                        &rt,
                    ));
                    return true;
                }
                if base_type_flags.intersects(TypeFlags::NumberLike)
                    && other_type_flags.intersects(TypeFlags::BigIntLike)
                {
                    let (er, lr, rr) = (ctx.trim(node), ctx.trim(left), ctx.trim(right));
                    let lt = utils::type_to_string(ctx.checker, left_type);
                    let rt = utils::type_to_string(ctx.checker, right_type);
                    ctx.report_diagnostic(build_bigint_and_number_diagnostic(er, lr, rr, &lt, &rt));
                    return true;
                }
                false
            };

        if check_mismatched_plus_operands(ctx, left_type_flags, right_type_flags) {
            return;
        }
        check_mismatched_plus_operands(ctx, right_type_flags, left_type_flags);
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let op = node.as_binary_expression().operator_token.kind();
        if op == Kind::PlusToken
            || (!self.opts.skip_compound_assignments && op == Kind::PlusEqualsToken)
        {
            self.check_plus_operands(ctx, node);
        }
    }
}
