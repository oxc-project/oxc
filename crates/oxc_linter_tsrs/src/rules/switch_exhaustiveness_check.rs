// Port of internal/rules/switch_exhaustiveness_check/switch_exhaustiveness_check.go.

use regex::Regex;
use tsrs_ast::{self as ast, CommentRange, Kind, Node, SourceFile};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_core::{LanguageVariant, P, TextRange};

use crate::rule::{
    Ctx, Listener, Rule, RuleFix, RuleMessage, RuleSuggestion, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

fn build_add_missing_cases_message() -> RuleMessage {
    RuleMessage::new("addMissingCases", "Add branches for missing cases.")
}

fn build_dangerous_default_case_message() -> RuleMessage {
    RuleMessage::new(
        "dangerousDefaultCase",
        "The switch statement is exhaustive, so the default case is unnecessary.",
    )
}

fn build_switch_is_not_exhaustive_message(missing_branches: &str) -> RuleMessage {
    RuleMessage::new(
        "switchIsNotExhaustive",
        format!("Switch is not exhaustive. Cases not matched: {missing_branches}"),
    )
}

struct SwitchMetadata {
    contains_non_literal_type: bool,
    // Both are None if there is no default case or matching default-case comment.
    default_case: Option<P<Node>>,
    default_case_comment: Option<CommentRange>,
    missing_literal_branch_types: Vec<P<Type>>,
    symbol_name: String,
}

impl SwitchMetadata {
    fn has_default_case(&self) -> bool {
        self.default_case.is_some() || self.default_case_comment.is_some()
    }
}

fn is_literal_like_type(t: P<Type>) -> bool {
    utils::is_type_flag_set(
        t,
        TypeFlags::Literal | TypeFlags::Undefined | TypeFlags::Null | TypeFlags::UniqueESSymbol,
    )
}

/// Default cases are never superfluous in switches with non-literal types.
fn does_type_contain_non_literal_type(t: P<Type>) -> bool {
    utils::union_type_parts(t)
        .into_iter()
        .any(|t| utils::intersection_type_parts(t).into_iter().all(|t| !is_literal_like_type(t)))
}

fn get_comment_default_case(
    source_file: P<SourceFile>,
    node: P<Node>,
    comment_pattern: &Regex,
) -> Option<CommentRange> {
    let case_block = node.as_switch_statement().case_block;
    let cases = case_block.as_case_block().clauses.nodes();
    let last = *cases.last()?;
    let comment_range = TextRange::new(last.end(), case_block.end());
    for comment in utils::get_comments_in_range(source_file, comment_range) {
        let mut comment_text = &source_file.text()[comment.pos() as usize..comment.end() as usize];
        match comment.kind {
            Kind::SingleLineCommentTrivia => {
                comment_text = comment_text.strip_prefix("//").unwrap_or(comment_text);
            }
            Kind::MultiLineCommentTrivia => {
                comment_text = comment_text.strip_prefix("/*").unwrap_or(comment_text);
                comment_text = comment_text.strip_suffix("*/").unwrap_or(comment_text);
            }
            _ => {}
        }
        if comment_pattern.is_match(comment_text.trim()) {
            return Some(comment);
        }
    }
    None
}

fn get_switch_metadata(
    source_file: P<SourceFile>,
    checker: &mut Checker,
    node: P<Node>,
    comment_pattern: &Regex,
) -> SwitchMetadata {
    let stmt = node.as_switch_statement();
    let cases = stmt.case_block.as_case_block().clauses.nodes();
    let default_case = cases.iter().copied().find(|&c| ast::is_default_clause(c));
    let default_case_comment = if default_case.is_none() {
        get_comment_default_case(source_file, node, comment_pattern)
    } else {
        None
    };

    let discriminant_type = utils::get_constrained_type_at_location(checker, stmt.expression);
    let symbol_name = discriminant_type.symbol().map(|s| s.name().to_string()).unwrap_or_default();

    let mut case_type_set: Vec<P<Type>> = Vec::with_capacity(cases.len());
    let mut has_undefined_case = false;
    for &c in cases {
        if c.kind() == Kind::DefaultClause {
            continue;
        }
        let expr = c.as_case_or_default_clause().expression.unwrap();
        let case_type = utils::get_constrained_type_at_location(checker, expr);
        case_type_set.push(case_type);
        if utils::is_type_flag_set(case_type, TypeFlags::Undefined) {
            has_undefined_case = true;
        }
    }

    let contains_non_literal_type = does_type_contain_non_literal_type(discriminant_type);

    let mut missing_literal_branch_types = Vec::with_capacity(10);
    utils::type_recurser(discriminant_type, &mut |t| {
        if case_type_set.contains(&t) || !is_literal_like_type(t) {
            return false;
        }
        // "missing", "optional" and "undefined" types are different runtime objects,
        // but all of them have TypeFlags.Undefined type flag
        if has_undefined_case && utils::is_type_flag_set(t, TypeFlags::Undefined) {
            return false;
        }
        missing_literal_branch_types.push(t);
        false
    });

    SwitchMetadata {
        contains_non_literal_type,
        default_case,
        default_case_comment,
        missing_literal_branch_types,
        symbol_name,
    }
}

fn requires_quoting(text: &str) -> bool {
    !tsrs_scanner::is_identifier_text(text, LanguageVariant::Standard)
}

fn get_node_indent(source_file: P<SourceFile>, node: P<Node>) -> String {
    let (pos, _) = utils::trim_node_text_range(source_file, node);
    let (_, column) =
        tsrs_scanner::get_ecma_line_and_utf16_character_of_position(&*source_file, pos);
    let column = column as i64;
    if column <= 0 {
        return String::new();
    }
    " ".repeat(column as usize)
}

fn build_case_test(
    checker: &mut Checker,
    missing_branch_type: P<Type>,
    symbol_name: &str,
) -> String {
    let missing_branch_name =
        missing_branch_type.symbol().map(|s| s.name().to_string()).unwrap_or_default();
    let case_test = if utils::is_type_flag_set(missing_branch_type, TypeFlags::ESSymbolLike) {
        missing_branch_name.clone()
    } else {
        utils::type_to_string(checker, missing_branch_type)
    };
    if !symbol_name.is_empty() && requires_quoting(&missing_branch_name) {
        return format!(
            "{symbol_name}[{}]",
            utils::quote_single_string_literal(&missing_branch_name)
        );
    }
    case_test
}

fn build_missing_case_line(
    checker: &mut Checker,
    missing_branch_type: Option<P<Type>>,
    symbol_name: &str,
) -> String {
    let Some(missing_branch_type) = missing_branch_type else {
        return "default: { throw new Error('default case') }".to_string();
    };
    let case_test = build_case_test(checker, missing_branch_type, symbol_name);
    let escaped_case = case_test.replace('\\', "\\\\").replace('\'', "\\'");
    format!("case {case_test}: {{ throw new Error('Not implemented yet: {escaped_case} case') }}")
}

fn apply_missing_cases(
    ctx: &Ctx,
    node: P<Node>,
    default_case: Option<P<Node>>,
    missing_cases: &[String],
) -> Vec<RuleFix> {
    let source_file = ctx.file;
    let case_block = node.as_switch_statement().case_block;
    let cases = case_block.as_case_block().clauses.nodes();
    let last_case = cases.last().copied();
    let case_indent = match last_case {
        Some(last) => get_node_indent(source_file, last),
        None => get_node_indent(source_file, node),
    };
    let fix_string = missing_cases
        .iter()
        .map(|code| format!("{case_indent}{code}"))
        .collect::<Vec<_>>()
        .join("\n");
    if let Some(last_case) = last_case {
        if let Some(default_case) = default_case {
            let mut before = String::new();
            for code in missing_cases {
                before.push_str(code);
                before.push('\n');
                before.push_str(&case_indent);
            }
            return vec![ctx.fix_insert_before(default_case, before)];
        }
        return vec![ctx.fix_insert_after(last_case, format!("\n{fix_string}"))];
    }
    vec![ctx.fix_replace(case_block, ["{", &fix_string, &format!("{case_indent}}}")].join("\n"))]
}

fn fix_switch(
    ctx: &mut Ctx,
    node: P<Node>,
    missing_branch_types: &[Option<P<Type>>],
    default_case: Option<P<Node>>,
    symbol_name: &str,
) -> Vec<RuleFix> {
    let missing_cases: Vec<String> = missing_branch_types
        .iter()
        .map(|&t| build_missing_case_line(ctx.checker, t, symbol_name))
        .collect();
    apply_missing_cases(ctx, node, default_case, &missing_cases)
}

pub struct SwitchExhaustivenessCheck {
    allow_default_case_for_exhaustive_switch: bool,
    consider_default_exhaustive_for_unions: bool,
    require_default_for_non_union: bool,
    comment_pattern: Regex,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let comment_pattern = match m.get("defaultCaseCommentPattern") {
        Some(serde_json::Value::String(p)) => Regex::new(p).map_err(|e| {
            format!("switch-exhaustiveness-check: invalid defaultCaseCommentPattern: {e}")
        })?,
        _ => Regex::new("(?i)^no default$").unwrap(),
    };
    Ok(Box::new(SwitchExhaustivenessCheck {
        allow_default_case_for_exhaustive_switch: opt_bool(
            &m,
            "allowDefaultCaseForExhaustiveSwitch",
            true,
        ),
        consider_default_exhaustive_for_unions: opt_bool(
            &m,
            "considerDefaultExhaustiveForUnions",
            false,
        ),
        require_default_for_non_union: opt_bool(&m, "requireDefaultForNonUnion", false),
        comment_pattern,
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::SwitchStatement)];

impl Rule for SwitchExhaustivenessCheck {
    fn name(&self) -> &'static str {
        "switch-exhaustiveness-check"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { opts: self })
    }
}

struct Visitor {
    opts: &'static SwitchExhaustivenessCheck,
}

impl Visitor {
    fn check_switch_exhaustive(&self, ctx: &mut Ctx, node: P<Node>, metadata: &SwitchMetadata) {
        // If considerDefaultExhaustiveForUnions is enabled, the presence of a default case
        // always makes the switch exhaustive.
        if self.opts.consider_default_exhaustive_for_unions && metadata.has_default_case() {
            return;
        }
        if metadata.missing_literal_branch_types.is_empty() {
            return;
        }
        let mut missing_branches = Vec::with_capacity(metadata.missing_literal_branch_types.len());
        for &missing_type in &metadata.missing_literal_branch_types {
            if utils::is_type_flag_set(missing_type, TypeFlags::ESSymbolLike) {
                if let Some(symbol) = missing_type.symbol() {
                    missing_branches.push(format!("typeof {}", symbol.name()));
                    continue;
                }
            }
            missing_branches.push(utils::type_to_string(ctx.checker, missing_type));
        }
        let expression = node.as_switch_statement().expression;
        ctx.report_node_with_suggestions(
            expression,
            build_switch_is_not_exhaustive_message(&missing_branches.join(" | ")),
            |ctx| {
                let types: Vec<Option<P<Type>>> =
                    metadata.missing_literal_branch_types.iter().map(|&t| Some(t)).collect();
                vec![RuleSuggestion {
                    message: build_add_missing_cases_message(),
                    fixes: fix_switch(
                        ctx,
                        node,
                        &types,
                        metadata.default_case,
                        &metadata.symbol_name,
                    ),
                }]
            },
        );
    }

    fn check_switch_unnecessary_default_case(&self, ctx: &mut Ctx, metadata: &SwitchMetadata) {
        if self.opts.allow_default_case_for_exhaustive_switch {
            return;
        }
        if metadata.missing_literal_branch_types.is_empty()
            && metadata.has_default_case()
            && !metadata.contains_non_literal_type
        {
            if let Some(comment) = &metadata.default_case_comment {
                ctx.report_range(
                    comment.pos(),
                    comment.end(),
                    build_dangerous_default_case_message(),
                );
            } else if let Some(default_case) = metadata.default_case {
                ctx.report_node(default_case, build_dangerous_default_case_message());
            }
        }
    }

    fn check_switch_no_union_default_case(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        metadata: &SwitchMetadata,
    ) {
        if !self.opts.require_default_for_non_union {
            return;
        }
        if metadata.contains_non_literal_type && !metadata.has_default_case() {
            let expression = node.as_switch_statement().expression;
            ctx.report_node_with_suggestions(
                expression,
                build_switch_is_not_exhaustive_message("default"),
                |ctx| {
                    vec![RuleSuggestion {
                        message: build_add_missing_cases_message(),
                        fixes: fix_switch(
                            ctx,
                            node,
                            &[None],
                            metadata.default_case,
                            &metadata.symbol_name,
                        ),
                    }]
                },
            );
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let metadata = get_switch_metadata(ctx.file, ctx.checker, node, &self.opts.comment_pattern);
        self.check_switch_exhaustive(ctx, node, &metadata);
        self.check_switch_unnecessary_default_case(ctx, &metadata);
        self.check_switch_no_union_default_case(ctx, node, &metadata);
    }
}
