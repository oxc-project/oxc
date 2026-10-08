// Port of internal/rules/prefer_optional_chain/prefer_optional_chain.go (and options.go).

use std::cell::RefCell;
use std::rc::Rc;

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{LiteralValue, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleFix, RuleMessage, RuleSuggestion, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

fn build_prefer_optional_chain_message() -> RuleMessage {
    RuleMessage::new(
        "preferOptionalChain",
        "Prefer using an optional chain expression instead, as it's more concise and easier to read.",
    )
}

fn build_optional_chain_suggest_message() -> RuleMessage {
    RuleMessage::new("optionalChainSuggest", "Change to an optional chain.")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OperandType {
    Invalid,
    Plain,
    #[expect(dead_code, reason = "tsgolint never constructs OperandTypeNotEqualNull either")]
    NotEqualNull,
    NotStrictEqualNull,
    NotStrictEqualUndef,
    // Loose inequality (!= null or != undefined), which checks for BOTH null and undefined.
    NotEqualBoth,
    Not,
    NegatedAndOperand,
    TypeofCheck,
    Comparison,
    // Loose equality (== null or == undefined), which checks for BOTH null and undefined.
    EqualNull,
    StrictEqualNull,
    StrictEqualUndef,
}

impl OperandType {
    fn is_nullish_check(self) -> bool {
        matches!(
            self,
            OperandType::NotStrictEqualNull
                | OperandType::NotStrictEqualUndef
                | OperandType::NotEqualBoth
                | OperandType::StrictEqualNull
                | OperandType::EqualNull
                | OperandType::StrictEqualUndef
                | OperandType::TypeofCheck
        )
    }

    fn is_strict_nullish_check(self) -> bool {
        self == OperandType::NotStrictEqualNull || self == OperandType::NotStrictEqualUndef
    }

    fn is_loose_nullish_check(self) -> bool {
        self == OperandType::NotEqualBoth || self == OperandType::EqualNull
    }

    fn is_trailing_comparison(self) -> bool {
        matches!(
            self,
            OperandType::NotStrictEqualNull
                | OperandType::NotStrictEqualUndef
                | OperandType::NotEqualBoth
                | OperandType::Comparison
        )
    }

    fn is_comparison_or_null_check(self) -> bool {
        self == OperandType::Comparison || self.is_nullish_check()
    }
}

#[derive(Clone, Copy)]
struct Operand {
    typ: OperandType,
    node: P<Node>,
    compared_expr: Option<P<Node>>,
}

impl Operand {
    fn new(typ: OperandType, node: P<Node>, compared_expr: Option<P<Node>>) -> Operand {
        Operand { typ, node, compared_expr }
    }
    fn invalid(node: P<Node>) -> Operand {
        Operand::new(OperandType::Invalid, node, None)
    }
    fn cexpr(&self) -> P<Node> {
        self.compared_expr.unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NodeComparisonResult {
    Equal,
    Subset,
    Superset,
    Invalid,
}

fn is_and_operator(op: Kind) -> bool {
    op == Kind::AmpersandAmpersandToken
}

/// The components of a binary expression after unwrapping parentheses.
struct BinaryParts {
    operator: Kind,
    left: P<Node>,  // parentheses-skipped
    right: P<Node>, // parentheses-skipped
}

fn unwrap_binary(node: Option<P<Node>>) -> Option<BinaryParts> {
    let node = node?;
    let unwrapped = ast::skip_parentheses(node);
    if !ast::is_binary_expression(unwrapped) {
        return None;
    }
    let b = unwrapped.as_binary_expression();
    Some(BinaryParts {
        operator: b.operator_token.kind(),
        left: ast::skip_parentheses(b.left),
        right: ast::skip_parentheses(b.right.get()),
    })
}

fn is_comparison_against(op: &Operand, predicate: fn(Option<P<Node>>) -> bool) -> bool {
    if op.typ != OperandType::Comparison {
        return false;
    }
    let Some(bin) = unwrap_binary(Some(op.node)) else {
        return false;
    };
    if bin.operator != Kind::EqualsEqualsToken && bin.operator != Kind::EqualsEqualsEqualsToken {
        return false;
    }
    predicate(Some(bin.left)) || predicate(Some(bin.right))
}

fn is_nullish_comparison(op: &Operand) -> bool {
    is_comparison_against(op, utils::is_nullish_literal)
}

fn is_strict_null_comparison(op: &Operand) -> bool {
    is_comparison_against(op, utils::is_null_literal)
}

fn is_strict_null_check_for_or_chain(op: &Operand) -> bool {
    match op.typ {
        OperandType::StrictEqualNull | OperandType::NotStrictEqualNull => true,
        OperandType::Comparison => {
            if let Some(bin) = unwrap_binary(Some(op.node)) {
                return bin.operator == Kind::EqualsEqualsEqualsToken
                    && (utils::is_null_literal(Some(bin.left))
                        || utils::is_null_literal(Some(bin.right)));
            }
            false
        }
        _ => false,
    }
}

fn is_strict_undefined_check_for_or_chain(op: &Operand) -> bool {
    match op.typ {
        OperandType::StrictEqualUndef
        | OperandType::NotStrictEqualUndef
        | OperandType::TypeofCheck => true,
        OperandType::Comparison => {
            if let Some(bin) = unwrap_binary(Some(op.node)) {
                return bin.operator == Kind::EqualsEqualsEqualsToken
                    && (utils::is_undefined_literal(Some(bin.left))
                        || utils::is_undefined_literal(Some(bin.right)));
            }
            false
        }
        _ => false,
    }
}

fn is_or_chain_nullish_check(op: &Operand) -> bool {
    match op.typ {
        OperandType::StrictEqualNull | OperandType::StrictEqualUndef | OperandType::EqualNull => {
            true
        }
        OperandType::Comparison => is_nullish_comparison(op),
        _ => false,
    }
}

fn is_nullish_check_operand(op: &Operand) -> bool {
    match op.typ {
        OperandType::NotStrictEqualNull
        | OperandType::NotStrictEqualUndef
        | OperandType::NotEqualBoth
        | OperandType::StrictEqualNull
        | OperandType::StrictEqualUndef
        | OperandType::EqualNull => true,
        OperandType::Comparison => {
            if let Some(bin) = unwrap_binary(Some(op.node)) {
                return utils::is_null_literal(Some(bin.right))
                    || utils::is_undefined_identifier(Some(bin.right))
                    || utils::is_null_literal(Some(bin.left))
                    || utils::is_undefined_identifier(Some(bin.left));
            }
            false
        }
        _ => false,
    }
}

fn is_non_nullish_trailing_comparison(op: &Operand) -> bool {
    op.typ == OperandType::Comparison && !is_nullish_comparison(op)
}

/// What kinds of null/undefined checks are present in a chain.
#[derive(Default)]
struct NullishCheckAnalysis {
    has_null_check: bool,
    has_undefined_check: bool,
    has_both_check: bool,
}

impl NullishCheckAnalysis {
    fn has_only_null_check(&self) -> bool {
        self.has_null_check && !self.has_undefined_check && !self.has_both_check
    }
    fn has_only_undefined_check(&self) -> bool {
        !self.has_null_check && self.has_undefined_check && !self.has_both_check
    }
    fn has_incomplete_check(&self) -> bool {
        self.has_only_null_check() || self.has_only_undefined_check()
    }
}

// or_chain_mode: true for OR chains (=== null), false for AND chains (!== null)
fn analyze_nullish_checks(chain: &[Operand], or_chain_mode: bool) -> NullishCheckAnalysis {
    let mut analysis = NullishCheckAnalysis::default();
    for op in chain {
        match op.typ {
            OperandType::NotStrictEqualNull | OperandType::StrictEqualNull => {
                analysis.has_null_check = true
            }
            OperandType::NotStrictEqualUndef | OperandType::StrictEqualUndef => {
                analysis.has_undefined_check = true
            }
            OperandType::NotEqualBoth | OperandType::EqualNull => analysis.has_both_check = true,
            OperandType::TypeofCheck => analysis.has_undefined_check = true,
            OperandType::Not if or_chain_mode => analysis.has_both_check = true,
            // Plain truthiness checks only count as both in AND chain mode
            OperandType::Plain if !or_chain_mode => analysis.has_both_check = true,
            OperandType::Comparison => {
                if let Some(bin) = unwrap_binary(Some(op.node)) {
                    let is_null = utils::is_null_literal(Some(bin.left))
                        || utils::is_null_literal(Some(bin.right));
                    let is_undefined = utils::is_undefined_literal(Some(bin.left))
                        || utils::is_undefined_literal(Some(bin.right));
                    if bin.operator == Kind::EqualsEqualsToken && (is_null || is_undefined) {
                        analysis.has_both_check = true;
                    } else if bin.operator == Kind::EqualsEqualsEqualsToken {
                        if is_null {
                            analysis.has_null_check = true;
                        }
                        if is_undefined {
                            analysis.has_undefined_check = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    analysis
}

fn unwrap_for_comparison(mut n: P<Node>) -> P<Node> {
    loop {
        if ast::is_parenthesized_expression(n) {
            n = n.as_parenthesized_expression().expression.get();
        } else if ast::is_non_null_expression(n) {
            n = n.as_non_null_expression().expression;
        } else {
            return n;
        }
    }
}

fn node_lists_equal(a: &[P<Node>], b: &[P<Node>]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| are_nodes_structurally_equal(x, y))
}

// Ignores parentheses and non-null assertions.
fn are_nodes_structurally_equal(a: P<Node>, b: P<Node>) -> bool {
    let a = unwrap_for_comparison(a);
    let b = unwrap_for_comparison(b);

    if a.kind() != b.kind() {
        return false;
    }

    match a.kind() {
        Kind::Identifier => a.text() == b.text(),
        Kind::ThisKeyword | Kind::NullKeyword | Kind::SuperKeyword => true,
        Kind::PropertyAccessExpression => {
            let ap = a.as_property_access_expression();
            let bp = b.as_property_access_expression();
            ap.name.text() == bp.name.text()
                && are_nodes_structurally_equal(ap.expression, bp.expression)
        }
        Kind::ElementAccessExpression => {
            let ae = a.as_element_access_expression();
            let be = b.as_element_access_expression();
            are_nodes_structurally_equal(ae.argument_expression, be.argument_expression)
                && are_nodes_structurally_equal(ae.expression, be.expression)
        }
        Kind::CallExpression => {
            let ac = a.as_call_expression();
            let bc = b.as_call_expression();
            if !are_nodes_structurally_equal(ac.expression, bc.expression) {
                return false;
            }
            if !node_lists_equal(ac.arguments.nodes(), bc.arguments.nodes()) {
                return false;
            }
            match (ac.type_arguments(), bc.type_arguments()) {
                (None, None) => true,
                (Some(x), Some(y)) => node_lists_equal(x.nodes(), y.nodes()),
                _ => false,
            }
        }
        Kind::AsExpression => are_nodes_structurally_equal(
            a.as_as_expression().expression,
            b.as_as_expression().expression,
        ),
        Kind::TypeAssertionExpression => are_nodes_structurally_equal(
            a.as_type_assertion().expression,
            b.as_type_assertion().expression,
        ),
        Kind::StringLiteral
        | Kind::NumericLiteral
        | Kind::BigIntLiteral
        | Kind::NoSubstitutionTemplateLiteral => a.text() == b.text(),
        Kind::TrueKeyword | Kind::FalseKeyword => true,
        // Type keywords (for type arguments comparison)
        Kind::StringKeyword
        | Kind::NumberKeyword
        | Kind::BooleanKeyword
        | Kind::AnyKeyword
        | Kind::UnknownKeyword
        | Kind::NeverKeyword
        | Kind::VoidKeyword
        | Kind::UndefinedKeyword
        | Kind::ObjectKeyword
        | Kind::SymbolKeyword
        | Kind::BigIntKeyword => true,
        Kind::TypeReference => are_nodes_structurally_equal(
            a.as_type_reference_node().type_name,
            b.as_type_reference_node().type_name,
        ),
        Kind::QualifiedName => {
            let aq = a.as_qualified_name();
            let bq = b.as_qualified_name();
            are_nodes_structurally_equal(aq.left, bq.left)
                && are_nodes_structurally_equal(aq.right, bq.right)
        }
        Kind::MetaProperty => {
            // Handles import.meta and new.target
            let am = a.as_meta_property();
            let bm = b.as_meta_property();
            am.keyword_token == bm.keyword_token && am.name.text() == bm.name.text()
        }
        Kind::BinaryExpression => {
            let ab = a.as_binary_expression();
            let bb = b.as_binary_expression();
            if ab.operator_token.kind() != bb.operator_token.kind() {
                return false;
            }
            are_nodes_structurally_equal(ab.left, bb.left)
                && are_nodes_structurally_equal(ab.right.get(), bb.right.get())
        }
        Kind::PrefixUnaryExpression => {
            let ap = a.as_prefix_unary_expression();
            let bp = b.as_prefix_unary_expression();
            if ap.operator != bp.operator {
                return false;
            }
            are_nodes_structurally_equal(ap.operand, bp.operand)
        }
        Kind::TypeOfExpression => are_nodes_structurally_equal(
            a.as_type_of_expression().expression,
            b.as_type_of_expression().expression,
        ),
        Kind::TemplateExpression => {
            let at = a.as_template_expression();
            let bt = b.as_template_expression();
            if at.head.text() != bt.head.text() {
                return false;
            }
            let a_spans = at.template_spans.nodes();
            let b_spans = bt.template_spans.nodes();
            if a_spans.len() != b_spans.len() {
                return false;
            }
            for (x, y) in a_spans.iter().zip(b_spans) {
                let xs = x.as_template_span();
                let ys = y.as_template_span();
                if !are_nodes_structurally_equal(xs.expression, ys.expression) {
                    return false;
                }
                if xs.literal.text() != ys.literal.text() {
                    return false;
                }
            }
            true
        }
        Kind::AwaitExpression => are_nodes_structurally_equal(
            a.as_await_expression().expression,
            b.as_await_expression().expression,
        ),
        _ => false,
    }
}

// e.g., foo.bar is a prefix of foo.bar.baz
fn is_node_prefix_of(shorter: P<Node>, longer: P<Node>) -> bool {
    let shorter = unwrap_for_comparison(shorter);
    let longer = unwrap_for_comparison(longer);

    if are_nodes_structurally_equal(shorter, longer) {
        return true;
    }

    let base = if ast::is_property_access_expression(longer) {
        longer.as_property_access_expression().expression
    } else if ast::is_element_access_expression(longer) {
        longer.as_element_access_expression().expression
    } else if ast::is_call_expression(longer) {
        longer.as_call_expression().expression
    } else if ast::is_non_null_expression(longer) {
        longer.as_non_null_expression().expression
    } else {
        return false;
    };

    is_node_prefix_of(shorter, base)
}

// In JSX, foo && foo.bar has different semantics than foo?.bar:
// foo && foo.bar returns false/null/undefined, while foo?.bar returns undefined.
fn is_inside_jsx(node: P<Node>) -> bool {
    let mut current = Some(node);
    while let Some(c) = current {
        if ast::is_jsx_expression(c)
            || ast::is_jsx_attribute(c)
            || ast::is_jsx_attributes(c)
            || ast::is_jsx_element(c)
            || ast::is_jsx_self_closing_element(c)
            || ast::is_jsx_opening_element(c)
            || ast::is_jsx_closing_element(c)
            || ast::is_jsx_fragment(c)
        {
            return true;
        }
        current = c.parent();
    }
    false
}

fn get_base_identifier(node: P<Node>) -> P<Node> {
    let mut current = node;
    loop {
        current = if ast::is_property_access_expression(current) {
            current.as_property_access_expression().expression
        } else if ast::is_element_access_expression(current) {
            current.as_element_access_expression().expression
        } else if ast::is_call_expression(current) {
            current.as_call_expression().expression
        } else if ast::is_non_null_expression(current) {
            current.as_non_null_expression().expression
        } else if ast::is_parenthesized_expression(current) {
            current.as_parenthesized_expression().expression.get()
        } else if current.kind() == Kind::AsExpression {
            current.as_as_expression().expression
        } else {
            return current;
        };
    }
}

fn has_side_effects(node: Option<P<Node>>) -> bool {
    let Some(node) = node else { return false };

    if ast::is_prefix_unary_expression(node) {
        let op = node.as_prefix_unary_expression().operator;
        if op == Kind::PlusPlusToken || op == Kind::MinusMinusToken {
            return true;
        }
    }

    if node.kind() == Kind::PostfixUnaryExpression {
        return true;
    }

    if ast::is_yield_expression(node) {
        return true;
    }

    // NOTE: Await expressions are NOT checked here for side effects (see the Go source).

    if ast::is_binary_expression(node) {
        let op = node.as_binary_expression().operator_token.kind();
        if matches!(
            op,
            Kind::EqualsToken
                | Kind::PlusEqualsToken
                | Kind::MinusEqualsToken
                | Kind::AsteriskEqualsToken
                | Kind::SlashEqualsToken
        ) {
            return true;
        }
    }

    if ast::is_property_access_expression(node) {
        return has_side_effects(Some(node.as_property_access_expression().expression));
    }
    if ast::is_element_access_expression(node) {
        let elem = node.as_element_access_expression();
        return has_side_effects(Some(elem.expression))
            || has_side_effects(Some(elem.argument_expression));
    }
    if ast::is_call_expression(node) {
        return has_side_effects(Some(node.as_call_expression().expression));
    }
    if ast::is_parenthesized_expression(node) {
        return has_side_effects(Some(node.as_parenthesized_expression().expression.get()));
    }

    false
}

#[derive(Clone, Default)]
struct ChainPart {
    text: String,
    optional: bool,
    requires_dot: bool,
    is_private: bool,
    has_non_null: bool,
    is_call: bool,
}

impl ChainPart {
    fn base_text(&self) -> String {
        if self.has_non_null && self.text.ends_with('!') {
            return self.text[..self.text.len() - 1].to_string();
        }
        self.text.clone()
    }
}

/// Go returns the cached slice itself, so callers that write to its elements write to the cache.
type Parts = Rc<RefCell<Vec<ChainPart>>>;

#[derive(Default)]
struct TypeInfo {
    parts: Vec<P<Type>>,
    has_null: bool,
    has_undefined: bool,
    has_void: bool,
    has_any: bool,
    has_unknown: bool,
    has_bool_literal: bool,
    has_num_literal: bool,
    has_str_literal: bool,
    has_big_int_literal: bool,
    has_big_int_like: bool,
    has_boolean_like: bool,
    has_number_like: bool,
    has_string_like: bool,
}

impl TypeInfo {
    fn is_nullish(&self) -> bool {
        self.has_null || self.has_undefined || self.has_any || self.has_unknown
    }
    // Excludes any/unknown (which are implicitly nullish).
    fn has_explicit_nullish(&self) -> bool {
        self.has_null || self.has_undefined
    }
    fn is_any_or_unknown(&self) -> bool {
        self.has_any || self.has_unknown
    }
    fn has_both_null_and_undefined(&self) -> bool {
        self.has_null && self.has_undefined
    }
    fn has_no_nullable_types(&self) -> bool {
        !self.has_null && !self.has_undefined && !self.has_any && !self.has_unknown
    }
    fn can_be_undefined_like(&self) -> bool {
        self.parts.is_empty()
            || self.has_undefined
            || self.has_void
            || self.has_any
            || self.has_unknown
    }
    fn can_be_nullish_like(&self) -> bool {
        self.parts.is_empty()
            || self.has_null
            || self.has_undefined
            || self.has_void
            || self.has_any
            || self.has_unknown
    }
    fn is_always_undefined_like(&self) -> bool {
        if self.parts.is_empty() || self.is_any_or_unknown() {
            return false;
        }
        self.parts.iter().all(|&p| utils::is_type_undefined_type(p) || utils::is_type_void_type(p))
    }
    fn is_always_nullish_like(&self) -> bool {
        if self.parts.is_empty() || self.is_any_or_unknown() {
            return false;
        }
        self.parts.iter().all(|&p| {
            utils::is_type_null_type(p)
                || utils::is_type_undefined_type(p)
                || utils::is_type_void_type(p)
        })
    }
}

#[derive(Clone, Copy)]
struct PreferOptionalChainOptions {
    allow_potentially_unsafe_fixes_that_modify_the_return_type_i_know_what_im_doing: bool,
    check_any: bool,
    check_big_int: bool,
    check_boolean: bool,
    check_number: bool,
    check_string: bool,
    check_unknown: bool,
    require_nullish: bool,
}

pub struct PreferOptionalChain {
    opts: PreferOptionalChainOptions,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(PreferOptionalChain {
        opts: PreferOptionalChainOptions {
            allow_potentially_unsafe_fixes_that_modify_the_return_type_i_know_what_im_doing:
                opt_bool(
                    &m,
                    "allowPotentiallyUnsafeFixesThatModifyTheReturnTypeIKnowWhatImDoing",
                    false,
                ),
            check_any: opt_bool(&m, "checkAny", true),
            check_big_int: opt_bool(&m, "checkBigInt", true),
            check_boolean: opt_bool(&m, "checkBoolean", true),
            check_number: opt_bool(&m, "checkNumber", true),
            check_string: opt_bool(&m, "checkString", true),
            check_unknown: opt_bool(&m, "checkUnknown", true),
            require_nullish: opt_bool(&m, "requireNullish", false),
        },
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::BinaryExpression)];

impl Rule for PreferOptionalChain {
    fn name(&self) -> &'static str {
        "prefer-optional-chain"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(ChainProcessor {
            opts: self.opts,
            seen_ranges: FxHashSet::default(),
            type_cache: FxHashMap::default(),
            flatten_cache: FxHashMap::default(),
            call_sig_cache: FxHashMap::default(),
            optional_chain_cache: FxHashMap::default(),
        })
    }
}

struct ChainProcessor {
    opts: PreferOptionalChainOptions,
    seen_ranges: FxHashSet<(i32, i32)>,
    type_cache: FxHashMap<P<Node>, Rc<TypeInfo>>,
    flatten_cache: FxHashMap<P<Node>, Parts>,
    call_sig_cache: FxHashMap<P<Node>, Rc<FxHashMap<String, String>>>,
    optional_chain_cache: FxHashMap<P<Node>, bool>,
}

impl RuleVisitor for ChainProcessor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let operator = node.as_binary_expression().operator_token.kind();
        match operator {
            Kind::AmpersandAmpersandToken | Kind::BarBarToken | Kind::QuestionQuestionToken => {
                self.process_chain(ctx, node, operator)
            }
            _ => {}
        }
    }
}

fn node_text(ctx: &Ctx, node: P<Node>) -> String {
    let (pos, end) = ctx.trim(node);
    ctx.text()[pos as usize..end as usize].to_string()
}

fn src(ctx: &Ctx, pos: i32, end: i32) -> &'static str {
    &ctx.text()[pos as usize..end as usize]
}

fn unsafe_fixes(opts: PreferOptionalChainOptions) -> bool {
    opts.allow_potentially_unsafe_fixes_that_modify_the_return_type_i_know_what_im_doing
}

impl ChainProcessor {
    fn unsafe_ok(&self) -> bool {
        unsafe_fixes(self.opts)
    }

    fn get_type_info(&mut self, ctx: &mut Ctx, node: P<Node>) -> Rc<TypeInfo> {
        if let Some(info) = self.type_cache.get(&node) {
            return Rc::clone(info);
        }
        let node_type = ctx.checker.get_type_at_location(node);
        let parts = utils::union_type_parts(node_type);
        let mut info = TypeInfo::default();
        for &part in &parts {
            if utils::is_type_null_type(part) {
                info.has_null = true;
            }
            if utils::is_type_undefined_type(part) {
                info.has_undefined = true;
            }
            if utils::is_type_void_type(part) {
                info.has_void = true;
            }
            if utils::is_type_any_type(part) {
                info.has_any = true;
            }
            if utils::is_type_unknown_type(part) {
                info.has_unknown = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::BooleanLiteral) {
                info.has_bool_literal = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::NumberLiteral) {
                info.has_num_literal = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::StringLiteral) {
                info.has_str_literal = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::BigIntLiteral) {
                info.has_big_int_literal = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::BigIntLike) {
                info.has_big_int_like = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::BooleanLike) {
                info.has_boolean_like = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::NumberLike) {
                info.has_number_like = true;
            }
            if utils::is_type_flag_set(part, TypeFlags::StringLike) {
                info.has_string_like = true;
            }
        }
        info.parts = parts;
        let info = Rc::new(info);
        self.type_cache.insert(node, Rc::clone(&info));
        info
    }

    fn extract_call_signatures(
        &mut self,
        ctx: &Ctx,
        node: P<Node>,
    ) -> Rc<FxHashMap<String, String>> {
        if let Some(cached) = self.call_sig_cache.get(&node) {
            return Rc::clone(cached);
        }
        let mut signatures: FxHashMap<String, String> = FxHashMap::default();
        let mut n = Some(node);
        while let Some(cur) = n {
            n = if ast::is_call_expression(cur) {
                let call = cur.as_call_expression();
                signatures.insert(node_text(ctx, call.expression), node_text(ctx, cur));
                Some(call.expression)
            } else if ast::is_property_access_expression(cur) {
                Some(cur.as_property_access_expression().expression)
            } else if ast::is_element_access_expression(cur) {
                Some(cur.as_element_access_expression().expression)
            } else if ast::is_non_null_expression(cur) {
                Some(cur.as_non_null_expression().expression)
            } else {
                None
            };
        }
        let signatures = Rc::new(signatures);
        self.call_sig_cache.insert(node, Rc::clone(&signatures));
        signatures
    }

    fn is_chain_already_seen(&self, ctx: &Ctx, node: P<Node>) -> bool {
        self.seen_ranges.contains(&ctx.trim(node))
    }

    fn report_chain_with_fixes(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        fixes: Vec<RuleFix>,
        use_suggestion: bool,
    ) {
        if use_suggestion {
            ctx.report_node_with_suggestions(node, build_prefer_optional_chain_message(), |_| {
                vec![RuleSuggestion { message: build_optional_chain_suggest_message(), fixes }]
            });
        } else {
            ctx.report_node_with_fixes(node, build_prefer_optional_chain_message(), |_| fixes);
        }
    }
}

fn is_valid_operand_for_chain_type(op: &Operand, operator_kind: Kind) -> bool {
    if is_and_operator(operator_kind) {
        return op.typ != OperandType::Invalid;
    }
    matches!(
        op.typ,
        OperandType::Not
            | OperandType::Comparison
            | OperandType::Plain
            | OperandType::TypeofCheck
            | OperandType::NotStrictEqualNull
            | OperandType::NotStrictEqualUndef
            | OperandType::NotEqualBoth
            | OperandType::StrictEqualNull
            | OperandType::StrictEqualUndef
            | OperandType::EqualNull
    )
}

impl ChainProcessor {
    // Catches patterns like `foo != null || foo.bar` which have opposite semantics to optional chaining,
    // and comparisons against non-nullish values like `foo.bar === false || foo.bar === undefined`.
    fn is_invalid_or_chain_starting_operand(&self, op: &Operand) -> bool {
        if op.typ != OperandType::Comparison {
            return false;
        }
        let Some(bin) = unwrap_binary(Some(op.node)) else {
            return false;
        };

        let is_left_nullish = utils::is_nullish_literal(Some(bin.left));
        let is_right_nullish = utils::is_nullish_literal(Some(bin.right));

        if bin.operator == Kind::EqualsEqualsEqualsToken && !is_left_nullish && !is_right_nullish {
            return true;
        }

        if bin.operator != Kind::ExclamationEqualsToken
            && bin.operator != Kind::ExclamationEqualsEqualsToken
        {
            return false;
        }

        if !is_left_nullish && !is_right_nullish {
            return false;
        }

        let checked_expr = if is_right_nullish { bin.left } else { bin.right };
        ast::is_identifier(checked_expr) || checked_expr.kind() == Kind::ThisKeyword
    }

    // Allows extending through a call expression when compare_nodes returns Invalid.
    fn should_allow_call_chain_extension(
        &self,
        prev_op: &Operand,
        current_op: &Operand,
        operator_kind: Kind,
    ) -> bool {
        if self.unsafe_ok() {
            return true;
        }
        if is_and_operator(operator_kind) {
            return prev_op.typ == OperandType::Plain && current_op.typ == OperandType::Plain;
        }
        let is_negation_chain =
            prev_op.typ == OperandType::Not && current_op.typ == OperandType::Not;
        let is_nullish_comparison_chain =
            is_or_chain_nullish_check(prev_op) && is_or_chain_nullish_check(current_op);
        is_negation_chain || is_nullish_comparison_chain
    }

    fn try_extend_through_call_expression(
        &self,
        last_expr: Option<P<Node>>,
        current_op: &Operand,
        first_op_expr: Option<P<Node>>,
        operator_kind: Kind,
    ) -> NodeComparisonResult {
        let Some(last_expr) = last_expr else {
            return NodeComparisonResult::Invalid;
        };
        let last_unwrapped = ast::skip_parentheses(last_expr);
        if !ast::is_call_expression(last_unwrapped) && !ast::is_new_expression(last_unwrapped) {
            return NodeComparisonResult::Invalid;
        }

        // Each `new X()` creates a fresh instance, so can't chain through it
        if is_and_operator(operator_kind) {
            if let Some(first_op_expr) = first_op_expr {
                let mut base_expr = first_op_expr;
                loop {
                    let unwrapped = ast::skip_parentheses(base_expr);
                    base_expr = if ast::is_property_access_expression(unwrapped) {
                        unwrapped.as_property_access_expression().expression
                    } else if ast::is_element_access_expression(unwrapped) {
                        unwrapped.as_element_access_expression().expression
                    } else if ast::is_call_expression(unwrapped) {
                        unwrapped.as_call_expression().expression
                    } else {
                        break;
                    };
                }
                if ast::is_new_expression(ast::skip_parentheses(base_expr)) {
                    return NodeComparisonResult::Invalid;
                }
            }
        }

        if is_node_prefix_of(last_expr, current_op.cexpr()) {
            return NodeComparisonResult::Subset;
        }
        NodeComparisonResult::Invalid
    }

    fn compare_nodes(&mut self, ctx: &Ctx, left: P<Node>, right: P<Node>) -> NodeComparisonResult {
        if has_side_effects(Some(left)) || has_side_effects(Some(right)) {
            return NodeComparisonResult::Invalid;
        }

        let left_unwrapped = ast::skip_parentheses(left);

        // Block standalone calls, new expressions and literals that create new instances, unless the
        // expression already contains optional chaining.
        if !self.contains_optional_chain(Some(left)) {
            let mut root_expr = left_unwrapped;
            loop {
                let unwrapped = ast::skip_parentheses(root_expr);
                root_expr = if ast::is_property_access_expression(unwrapped) {
                    unwrapped.as_property_access_expression().expression
                } else if ast::is_element_access_expression(unwrapped) {
                    unwrapped.as_element_access_expression().expression
                } else if ast::is_call_expression(unwrapped) {
                    unwrapped.as_call_expression().expression
                } else {
                    break;
                };
            }

            let is_rooted_in_new = ast::is_new_expression(ast::skip_parentheses(root_expr));

            let mut is_standalone_call = false;
            if ast::is_call_expression(left_unwrapped) {
                let callee = ast::skip_parentheses(left_unwrapped.as_call_expression().expression);
                is_standalone_call = ast::is_identifier(callee);
            } else if ast::is_new_expression(left_unwrapped) {
                is_standalone_call = true;
            }

            if is_standalone_call
                || is_rooted_in_new
                || ast::is_array_literal_expression(left_unwrapped)
                || ast::is_object_literal_expression(left_unwrapped)
                || ast::is_function_expression(left_unwrapped)
                || ast::is_arrow_function(left_unwrapped)
                || ast::is_class_expression(left_unwrapped)
                || ast::is_jsx_element(left_unwrapped)
                || ast::is_jsx_self_closing_element(left_unwrapped)
                || ast::is_jsx_fragment(left_unwrapped)
                || left_unwrapped.kind() == Kind::TemplateExpression
                || left_unwrapped.kind() == Kind::AwaitExpression
            {
                return NodeComparisonResult::Invalid;
            }
        }

        let left_sigs = self.extract_call_signatures(ctx, left);
        let right_sigs = self.extract_call_signatures(ctx, right);
        #[expect(
            clippy::iter_over_hash_type,
            reason = "the loop only asks whether any callee has different call text on each side; visiting order cannot change that"
        )]
        for (base_expr, left_sig) in left_sigs.iter() {
            if let Some(right_sig) = right_sigs.get(base_expr) {
                if left_sig != right_sig {
                    return NodeComparisonResult::Invalid;
                }
            }
        }

        if are_nodes_structurally_equal(left, right) {
            return NodeComparisonResult::Equal;
        }
        if is_node_prefix_of(left, right) {
            return NodeComparisonResult::Subset;
        }
        if is_node_prefix_of(right, left) {
            return NodeComparisonResult::Superset;
        }
        NodeComparisonResult::Invalid
    }

    fn includes_nullish(&mut self, ctx: &mut Ctx, node: P<Node>) -> bool {
        self.get_type_info(ctx, node).is_nullish()
    }

    fn includes_explicit_nullish(&mut self, ctx: &mut Ctx, node: P<Node>) -> bool {
        self.get_type_info(ctx, node).has_explicit_nullish()
    }

    fn type_is_any_or_unknown(&mut self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let info = self.get_type_info(ctx, node);
        if info.parts.is_empty() {
            return false;
        }
        info.parts.iter().all(|&p| utils::is_type_flag_set(p, TypeFlags::Any | TypeFlags::Unknown))
    }

    fn would_change_return_type(&mut self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let info = self.get_type_info(ctx, node);
        let has_falsy_non_nullish = info.has_bool_literal
            || info.has_num_literal
            || info.has_str_literal
            || info.has_big_int_literal;
        has_falsy_non_nullish && !info.has_explicit_nullish()
    }

    fn has_falsy_literal_type(&mut self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let info = self.get_type_info(ctx, node);
        for &union_part in &info.parts {
            for part in utils::intersection_type_parts(union_part) {
                if utils::is_type_flag_set(part, TypeFlags::BooleanLiteral) {
                    if part.as_literal_type().value() == Some(LiteralValue::Boolean(false)) {
                        return true;
                    }
                } else if utils::is_type_flag_set(part, TypeFlags::StringLiteral) {
                    if part.as_literal_type().value() == Some(LiteralValue::String("")) {
                        return true;
                    }
                } else if utils::is_type_flag_set(
                    part,
                    TypeFlags::NumberLiteral | TypeFlags::BigIntLiteral,
                ) {
                    // Go: part.AsLiteralType().String() (checker.ValueToString of the value)
                    let value = match part.as_literal_type().value() {
                        Some(v) => tsrs_checker::value_to_string(v),
                        None => String::new(),
                    };
                    if value == "0" || value == "0n" {
                        return true;
                    }
                }
            }
        }
        false
    }

    // void is falsy but not nullish - x?.() on void would TypeError.
    fn has_void_type(&mut self, ctx: &mut Ctx, node: P<Node>) -> bool {
        self.get_type_info(ctx, node).has_void
    }

    // Unsafe: === X (non-undefined), != null/undefined (since undefined != null is false!)
    fn is_or_chain_comparison_safe(&self, op: &Operand) -> bool {
        if op.typ != OperandType::Comparison {
            return true;
        }
        let Some(bin) = unwrap_binary(Some(op.node)) else {
            return true;
        };

        let value = if utils::is_access_expression(bin.left) {
            bin.right
        } else if utils::is_access_expression(bin.right) {
            bin.left
        } else {
            return true;
        };

        let is_null = utils::is_null_literal(Some(value));
        let is_undefined = utils::is_undefined_literal(Some(value));
        let is_literal = matches!(
            value.kind(),
            Kind::NumericLiteral
                | Kind::StringLiteral
                | Kind::TrueKeyword
                | Kind::FalseKeyword
                | Kind::ObjectLiteralExpression
                | Kind::ArrayLiteralExpression
        );

        match bin.operator {
            // !== undefined is NOT safe: undefined !== undefined is false
            Kind::ExclamationEqualsEqualsToken => is_literal || is_null,
            Kind::EqualsEqualsEqualsToken => is_undefined,
            Kind::ExclamationEqualsToken => {
                if is_null || is_undefined {
                    return false;
                }
                if ast::is_identifier(value) && !is_literal {
                    return false;
                }
                is_literal
            }
            Kind::EqualsEqualsToken => is_null || is_undefined,
            // Relational operators are unsafe because undefined/null comparisons produce
            // unexpected results (e.g., undefined <= 100 is false)
            Kind::LessThanToken
            | Kind::GreaterThanToken
            | Kind::LessThanEqualsToken
            | Kind::GreaterThanEqualsToken => false,
            _ => true,
        }
    }

    fn should_skip_by_type(&mut self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let base_node = get_base_identifier(node);
        let info = self.get_type_info(ctx, base_node);
        let o = self.opts;

        if o.require_nullish && info.has_explicit_nullish() {
            return false;
        }
        (info.has_any && !o.check_any)
            || (info.has_big_int_like && !o.check_big_int)
            || (info.has_boolean_like && !o.check_boolean)
            || (info.has_number_like && !o.check_number)
            || (info.has_string_like && !o.check_string)
            || (info.has_unknown && !o.check_unknown)
    }

    fn flatten_for_fix(&mut self, ctx: &Ctx, node: P<Node>) -> Parts {
        if let Some(cached) = self.flatten_cache.get(&node) {
            return Rc::clone(cached);
        }
        let mut parts: Vec<ChainPart> = Vec::new();
        flatten_visit(ctx, node, false, &mut parts);
        let parts = Rc::new(RefCell::new(parts));
        self.flatten_cache.insert(node, Rc::clone(&parts));
        parts
    }

    fn flatten_len(&mut self, ctx: &Ctx, node: P<Node>) -> usize {
        let len = self.flatten_for_fix(ctx, node).borrow().len();
        len
    }
}

fn flatten_visit(ctx: &Ctx, n: P<Node>, parent_is_non_null: bool, parts: &mut Vec<ChainPart>) {
    if ast::is_parenthesized_expression(n) {
        let inner = n.as_parenthesized_expression().expression.get();
        if ast::is_await_expression(inner) || ast::is_yield_expression(inner) {
            parts.push(ChainPart { text: node_text(ctx, n), ..Default::default() });
            return;
        }
        flatten_visit(ctx, inner, parent_is_non_null, parts);
    } else if ast::is_non_null_expression(n) {
        flatten_visit(ctx, n.as_non_null_expression().expression, true, parts);
    } else if ast::is_property_access_expression(n) {
        let prop_access = n.as_property_access_expression();
        flatten_visit(ctx, prop_access.expression, false, parts);
        let mut name_text = node_text(ctx, prop_access.name);
        let has_non_null = parent_is_non_null;
        if has_non_null {
            name_text.push('!');
        }
        let is_private = prop_access.name.kind() == Kind::PrivateIdentifier;
        parts.push(ChainPart {
            text: name_text,
            optional: prop_access.question_dot_token().is_some(),
            requires_dot: true,
            is_private,
            has_non_null,
            is_call: false,
        });
    } else if ast::is_element_access_expression(n) {
        let elem_access = n.as_element_access_expression();
        flatten_visit(ctx, elem_access.expression, false, parts);
        let arg_text = node_text(ctx, elem_access.argument_expression);
        let has_non_null = parent_is_non_null;
        let suffix = if has_non_null { "!" } else { "" };
        parts.push(ChainPart {
            text: format!("[{arg_text}]{suffix}"),
            optional: elem_access.question_dot_token().is_some(),
            requires_dot: false,
            is_private: false,
            has_non_null,
            is_call: false,
        });
    } else if ast::is_call_expression(n) {
        let call_expr = n.as_call_expression();
        flatten_visit(ctx, call_expr.expression, false, parts);

        let mut type_args_text = String::new();
        if let Some(type_args) = call_expr.type_arguments() {
            if !type_args.nodes().is_empty() {
                type_args_text = format!("<{}>", src(ctx, type_args.pos(), type_args.end()));
            }
        }

        let mut args_text = "()".to_string();
        if !call_expr.arguments.nodes().is_empty() {
            let args_start = call_expr.arguments.pos();
            let call_end = n.end();
            args_text = format!("({})", src(ctx, args_start, call_end - 1));
        }

        parts.push(ChainPart {
            text: type_args_text + &args_text,
            optional: call_expr.question_dot_token().is_some(),
            requires_dot: false,
            is_private: false,
            has_non_null: false,
            is_call: true,
        });
    } else {
        let mut text = node_text(ctx, n);
        if parent_is_non_null && ast::is_identifier(n) {
            text.push('!');
        }
        if n.kind() == Kind::AsExpression || n.kind() == Kind::TypeAssertionExpression {
            text = format!("({text})");
        }
        parts.push(ChainPart {
            text,
            optional: false,
            requires_dot: false,
            is_private: false,
            has_non_null: parent_is_non_null,
            is_call: false,
        });
    }
}

fn build_optional_chain(
    parts: &[ChainPart],
    checked_lengths: &FxHashSet<usize>,
    call_should_be_optional: bool,
    strip_non_null_assertions: bool,
) -> String {
    let max_checked_length = checked_lengths.iter().copied().max().unwrap_or(0);

    let mut optional_parts = vec![false; parts.len()];
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            if checked_lengths.contains(&i) || part.optional {
                optional_parts[i] = true;
            } else {
                let is_last_part = i == parts.len() - 1;
                if part.is_call && is_last_part && call_should_be_optional {
                    optional_parts[i] = true;
                }
            }
        }
        if optional_parts[i] && part.is_private {
            return String::new();
        }
    }

    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        let mut part_text: &str = &part.text;
        if strip_non_null_assertions
            && i < parts.len() - 1
            && optional_parts[i + 1]
            && part.has_non_null
            && i < max_checked_length
        {
            part_text = &part_text[..part_text.len() - 1];
        }

        if i > 0 && optional_parts[i] {
            result.push_str("?.");
        } else if i > 0 {
            if part.optional && i > max_checked_length {
                result.push_str("?.");
            } else if part.requires_dot {
                result.push('.');
            }
        }
        result.push_str(part_text);
    }
    result
}

impl ChainProcessor {
    fn contains_optional_chain(&mut self, n: Option<P<Node>>) -> bool {
        let Some(n) = n else { return false };
        if let Some(&cached) = self.optional_chain_cache.get(&n) {
            return cached;
        }
        let result = self.contains_optional_chain_uncached(n);
        self.optional_chain_cache.insert(n, result);
        result
    }

    fn contains_optional_chain_uncached(&mut self, n: P<Node>) -> bool {
        let unwrapped = ast::skip_parentheses(n);
        if ast::is_property_access_expression(unwrapped) {
            let p = unwrapped.as_property_access_expression();
            if p.question_dot_token().is_some() {
                return true;
            }
            return self.contains_optional_chain(Some(p.expression));
        }
        if ast::is_element_access_expression(unwrapped) {
            let e = unwrapped.as_element_access_expression();
            if e.question_dot_token().is_some() {
                return true;
            }
            return self.contains_optional_chain(Some(e.expression));
        }
        if ast::is_call_expression(unwrapped) {
            let c = unwrapped.as_call_expression();
            if c.question_dot_token().is_some() {
                return true;
            }
            return self.contains_optional_chain(Some(c.expression));
        }
        if ast::is_binary_expression(unwrapped) {
            let b = unwrapped.as_binary_expression();
            return self.contains_optional_chain(Some(b.left))
                || self.contains_optional_chain(Some(b.right.get()));
        }
        false
    }

    // Whether all operands in the chain (starting from index 1) already have optional chaining.
    fn all_subsequent_have_optional_chaining(&mut self, chain: &[Operand]) -> bool {
        for op in chain.iter().skip(1) {
            if op.compared_expr.is_some() && !self.contains_optional_chain(op.compared_expr) {
                return false;
            }
        }
        true
    }

    fn parse_operand(&self, node: P<Node>, operator_kind: Kind) -> Operand {
        let is_and_chain = is_and_operator(operator_kind);
        let unwrapped = unwrap_for_comparison(node);

        // Bare 'this' cannot be converted because it's not nullable in TypeScript.
        if unwrapped.kind() == Kind::ThisKeyword {
            return Operand::invalid(node);
        }

        let mut base_expr = unwrapped;
        loop {
            base_expr = if ast::is_property_access_expression(base_expr) {
                base_expr.as_property_access_expression().expression
            } else if ast::is_element_access_expression(base_expr) {
                base_expr.as_element_access_expression().expression
            } else if ast::is_call_expression(base_expr) {
                base_expr.as_call_expression().expression
            } else if ast::is_non_null_expression(base_expr) {
                base_expr.as_non_null_expression().expression
            } else if ast::is_parenthesized_expression(base_expr) {
                base_expr.as_parenthesized_expression().expression.get()
            } else if base_expr.kind() == Kind::AsExpression {
                base_expr.as_as_expression().expression
            } else if base_expr.kind() == Kind::TypeAssertionExpression {
                base_expr.as_type_assertion().expression
            } else {
                break;
            };
        }

        if ast::is_binary_expression(base_expr) {
            let bin_op = base_expr.as_binary_expression().operator_token.kind();
            if bin_op == Kind::AmpersandAmpersandToken || bin_op == Kind::BarBarToken {
                return Operand::invalid(node);
            }
        }

        if ast::is_binary_expression(unwrapped) {
            let bin_expr = unwrapped.as_binary_expression();
            let op = bin_expr.operator_token.kind();
            let (b_left, b_right) = (bin_expr.left, bin_expr.right.get());

            let mut expr_value: Option<(P<Node>, P<Node>)> = None;
            if utils::is_nullish_literal(Some(b_right)) || ast::is_string_literal(b_right) {
                expr_value = Some((b_left, b_right));
            } else if utils::is_nullish_literal(Some(b_left)) || ast::is_string_literal(b_left) {
                expr_value = Some((b_right, b_left));
            }

            if let Some((expr, value)) = expr_value {
                let expr = ast::skip_parentheses(expr);

                if ast::is_type_of_expression(expr) {
                    let typeof_expr = expr.as_type_of_expression();
                    if ast::is_string_literal(value) && value.text() == "undefined" {
                        if (op == Kind::ExclamationEqualsEqualsToken
                            || op == Kind::ExclamationEqualsToken)
                            && is_and_chain
                        {
                            return Operand::new(
                                OperandType::TypeofCheck,
                                node,
                                Some(typeof_expr.expression),
                            );
                        }
                        if (op == Kind::EqualsEqualsEqualsToken || op == Kind::EqualsEqualsToken)
                            && !is_and_chain
                        {
                            return Operand::new(
                                OperandType::TypeofCheck,
                                node,
                                Some(typeof_expr.expression),
                            );
                        }
                    }
                }

                let is_null = utils::is_null_literal(Some(value));
                let is_undefined = utils::is_undefined_literal(Some(value));
                let is_ident_or_this = ast::is_identifier(expr) || expr.kind() == Kind::ThisKeyword;

                if is_and_chain {
                    match op {
                        Kind::ExclamationEqualsEqualsToken => {
                            if is_null {
                                return Operand::new(
                                    OperandType::NotStrictEqualNull,
                                    node,
                                    Some(expr),
                                );
                            }
                            if is_undefined {
                                return Operand::new(
                                    OperandType::NotStrictEqualUndef,
                                    node,
                                    Some(expr),
                                );
                            }
                        }
                        Kind::ExclamationEqualsToken if is_null || is_undefined => {
                            return Operand::new(OperandType::NotEqualBoth, node, Some(expr));
                        }
                        Kind::EqualsEqualsEqualsToken => {
                            if is_null && is_ident_or_this {
                                return Operand::new(
                                    OperandType::StrictEqualNull,
                                    node,
                                    Some(expr),
                                );
                            }
                            if is_undefined && is_ident_or_this {
                                return Operand::new(
                                    OperandType::StrictEqualUndef,
                                    node,
                                    Some(expr),
                                );
                            }
                        }
                        Kind::EqualsEqualsToken
                            if (is_null || is_undefined) && is_ident_or_this =>
                        {
                            return Operand::new(OperandType::EqualNull, node, Some(expr));
                        }
                        _ => {}
                    }
                } else {
                    let is_property_or_element = utils::is_access_expression(expr);
                    if is_property_or_element && (is_null || is_undefined) {
                        return Operand::new(OperandType::Comparison, node, Some(expr));
                    } else if !is_property_or_element {
                        match op {
                            Kind::EqualsEqualsEqualsToken => {
                                if is_null {
                                    return Operand::new(
                                        OperandType::NotStrictEqualNull,
                                        node,
                                        Some(expr),
                                    );
                                }
                                if is_undefined {
                                    return Operand::new(
                                        OperandType::NotStrictEqualUndef,
                                        node,
                                        Some(expr),
                                    );
                                }
                            }
                            Kind::EqualsEqualsToken if is_null || is_undefined => {
                                return Operand::new(OperandType::NotEqualBoth, node, Some(expr));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if ast::is_prefix_unary_expression(unwrapped) {
            let prefix_expr = unwrapped.as_prefix_unary_expression();
            if prefix_expr.operator == Kind::ExclamationToken {
                if prefix_expr.operand.kind() == Kind::ThisKeyword {
                    return Operand::invalid(node);
                }
                if !is_and_chain {
                    return Operand::new(OperandType::Not, node, Some(prefix_expr.operand));
                }
                return Operand::new(
                    OperandType::NegatedAndOperand,
                    node,
                    Some(prefix_expr.operand),
                );
            }
        }

        if ast::is_binary_expression(unwrapped) {
            let bin_expr = unwrapped.as_binary_expression();

            // Relational operators should not participate in optional chaining.
            if matches!(
                bin_expr.operator_token.kind(),
                Kind::LessThanToken
                    | Kind::GreaterThanToken
                    | Kind::LessThanEqualsToken
                    | Kind::GreaterThanEqualsToken
                    | Kind::InKeyword
                    | Kind::InstanceOfKeyword
            ) {
                return Operand::invalid(node);
            }

            let left = ast::skip_parentheses(bin_expr.left);
            let right = ast::skip_parentheses(bin_expr.right.get());
            let left_is_access = utils::is_access_expression(left);
            let right_is_access = utils::is_access_expression(right);

            // If both sides are access expressions, we can't convert without changing when the
            // other side is evaluated.
            if left_is_access && right_is_access {
                return Operand::invalid(node);
            }

            let mut compared_expr = left;
            let mut has_property_access = left_is_access;
            if right_is_access {
                compared_expr = right;
                has_property_access = true;
            }

            if !has_property_access {
                return Operand::invalid(node);
            }

            return Operand::new(OperandType::Comparison, node, Some(compared_expr));
        }

        Operand::new(OperandType::Plain, node, Some(unwrapped))
    }

    fn collect_operands(&mut self, ctx: &Ctx, node: P<Node>, operator_kind: Kind) -> Vec<P<Node>> {
        let mut operand_nodes = Vec::new();
        self.collect(ctx, node, operator_kind, &mut operand_nodes);
        operand_nodes
    }

    fn collect(&mut self, ctx: &Ctx, n: P<Node>, operator_kind: Kind, out: &mut Vec<P<Node>>) {
        let unwrapped = ast::skip_parentheses(n);
        if ast::is_binary_expression(unwrapped)
            && unwrapped.as_binary_expression().operator_token.kind() == operator_kind
        {
            let bin_expr = unwrapped.as_binary_expression();
            self.collect(ctx, bin_expr.left, operator_kind, out);
            self.collect(ctx, bin_expr.right.get(), operator_kind, out);
            // Mark this binary expression as seen to prevent re-processing
            self.seen_ranges.insert(ctx.trim(unwrapped));
        } else {
            out.push(n);
        }
    }

    fn has_property_access_in_chain(&self, chain: &[Operand]) -> bool {
        chain.iter().any(|op| {
            op.compared_expr.is_some_and(|e| utils::is_access_expression(ast::skip_parentheses(e)))
        })
    }

    fn has_same_base_identifier(&self, chain: &[Operand]) -> bool {
        let mut first_base: Option<P<Node>> = None;
        for op in chain {
            let Some(e) = op.compared_expr else { continue };
            let base = get_base_identifier(e);
            match first_base {
                None => first_base = Some(base),
                Some(fb) => {
                    if !are_nodes_structurally_equal(fb, base) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn validate_chain(&mut self, ctx: &mut Ctx, chain: &[Operand], operator_kind: Kind) -> bool {
        if chain.len() < 2 {
            return false;
        }
        if !self.has_same_base_identifier(chain) {
            return false;
        }
        if !self.has_property_access_in_chain(chain) {
            return false;
        }
        if self.should_skip_for_require_nullish(ctx, chain, operator_kind) {
            return false;
        }
        true
    }

    fn should_skip_for_require_nullish(
        &mut self,
        ctx: &mut Ctx,
        chain: &[Operand],
        operator_kind: Kind,
    ) -> bool {
        if !self.opts.require_nullish {
            return false;
        }
        if !is_and_operator(operator_kind) && !chain.is_empty() && chain[0].typ == OperandType::Not
        {
            return true;
        }
        for (i, op) in chain.iter().enumerate() {
            if op.typ != OperandType::Plain {
                return false;
            }
            if is_and_operator(operator_kind) && i < chain.len() - 1 {
                if let Some(e) = op.compared_expr {
                    if self.includes_explicit_nullish(ctx, e) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn process_chain(&mut self, ctx: &mut Ctx, node: P<Node>, operator_kind: Kind) {
        if operator_kind == Kind::BarBarToken || operator_kind == Kind::QuestionQuestionToken {
            self.handle_empty_object_pattern(ctx, node);
        }
        if operator_kind == Kind::QuestionQuestionToken {
            return;
        }
        if is_inside_jsx(node) {
            return;
        }
        if self.is_chain_already_seen(ctx, node) {
            return;
        }

        let operand_nodes = self.collect_operands(ctx, node, operator_kind);
        if operand_nodes.len() < 2 {
            return;
        }

        let mut operands = Vec::with_capacity(operand_nodes.len());
        for (i, &n) in operand_nodes.iter().enumerate() {
            let mut op = self.parse_operand(n, operator_kind);
            let are_more_operands = i < operand_nodes.len() - 1;
            let disallow_falsy_literal = (is_and_operator(operator_kind)
                && op.typ == OperandType::Plain)
                || (operator_kind == Kind::BarBarToken && op.typ == OperandType::Not);
            // Truthiness guards narrow out non-nullish falsy literals, whereas optional chaining
            // does not. Keep the final operand as a result and invalidate unsafe guards.
            if are_more_operands
                && disallow_falsy_literal
                && self.has_falsy_literal_type(ctx, op.cexpr())
            {
                op = Operand::invalid(n);
            }
            operands.push(op);
        }

        let chains = self.build_chains(ctx, &operands, operator_kind);

        for chain in chains {
            let Some(validated_chain) =
                self.validate_chain_for_reporting(ctx, chain, operator_kind)
            else {
                continue;
            };
            self.generate_fix_and_report(
                ctx,
                node,
                &validated_chain,
                &operand_nodes,
                operator_kind,
            );
        }
    }

    fn build_chains(
        &mut self,
        ctx: &mut Ctx,
        operands: &[Operand],
        operator_kind: Kind,
    ) -> Vec<Vec<Operand>> {
        if is_and_operator(operator_kind) {
            return self.build_and_chains(ctx, operands);
        }
        self.build_or_chains(ctx, operands)
    }

    fn build_and_chains(&mut self, ctx: &mut Ctx, operands: &[Operand]) -> Vec<Vec<Operand>> {
        let mut all_chains: Vec<Vec<Operand>> = Vec::new();
        let mut current_chain: Vec<Operand> = Vec::new();
        let mut last_expr: Option<P<Node>> = None;
        let mut last_check_type = OperandType::Invalid;
        let mut chain_complete = false;

        let mut i = 0;
        while i < operands.len() {
            let op = operands[i];

            if matches!(
                op.typ,
                OperandType::Invalid
                    | OperandType::NegatedAndOperand
                    | OperandType::EqualNull
                    | OperandType::StrictEqualNull
                    | OperandType::StrictEqualUndef
            ) {
                if current_chain.len() >= 2 {
                    all_chains.push(std::mem::take(&mut current_chain));
                }
                current_chain.clear();
                last_expr = None;
                last_check_type = OperandType::Invalid;
                chain_complete = false;
                i += 1;
                continue;
            }

            if current_chain.is_empty() {
                if is_non_nullish_trailing_comparison(&op) {
                    i += 1;
                    continue;
                }
                current_chain.push(op);
                last_expr = op.compared_expr;
                if op.typ != OperandType::Plain {
                    last_check_type = op.typ;
                }
                chain_complete = false;
                i += 1;
                continue;
            }

            if chain_complete {
                if current_chain.len() >= 2 {
                    all_chains.push(std::mem::take(&mut current_chain));
                }
                current_chain = vec![op];
                last_expr = op.compared_expr;
                last_check_type = OperandType::Invalid;
                if op.typ != OperandType::Plain {
                    last_check_type = op.typ;
                }
                chain_complete = false;
                i += 1;
                continue;
            }

            let mut cmp = self.compare_nodes(ctx, last_expr.unwrap(), op.cexpr());

            if !current_chain.is_empty() {
                let prev_op = *current_chain.last().unwrap();
                if self.should_stop_at_strict_nullish_check(ctx, &prev_op) {
                    if current_chain.len() >= 2 {
                        all_chains.push(std::mem::take(&mut current_chain));
                    }
                    current_chain.clear();
                    break;
                }
            }

            if cmp == NodeComparisonResult::Invalid && !current_chain.is_empty() {
                let prev_op = *current_chain.last().unwrap();
                if self.should_allow_call_chain_extension(
                    &prev_op,
                    &op,
                    Kind::AmpersandAmpersandToken,
                ) {
                    let first_op_expr = current_chain[0].compared_expr;
                    if self.try_extend_through_call_expression(
                        last_expr,
                        &op,
                        first_op_expr,
                        Kind::AmpersandAmpersandToken,
                    ) == NodeComparisonResult::Subset
                    {
                        cmp = NodeComparisonResult::Subset;
                    }
                }
            }

            // Complementary pairs (e.g., x !== null && x !== undefined) checking the same expression
            if cmp == NodeComparisonResult::Equal && i + 1 < operands.len() {
                if let Some(pair) = self.try_merge_complementary_pair(
                    ctx,
                    &op,
                    &operands[i + 1],
                    Kind::AmpersandAmpersandToken,
                ) {
                    last_expr = pair.last().unwrap().compared_expr;
                    current_chain.extend(pair);
                    i += 2;
                    continue;
                }
            }

            if op.typ.is_nullish_check()
                && last_check_type != OperandType::Invalid
                && !self.are_nullish_checks_consistent(last_check_type, op.typ)
            {
                if current_chain.len() >= 2 {
                    all_chains.push(std::mem::take(&mut current_chain));
                }
                current_chain.clear();
                break;
            }

            if cmp == NodeComparisonResult::Subset || cmp == NodeComparisonResult::Equal {
                current_chain.push(op);
                last_expr = op.compared_expr;
                if op.typ != OperandType::Plain && op.typ.is_nullish_check() {
                    last_check_type = op.typ;
                }
                if is_non_nullish_trailing_comparison(&op) {
                    chain_complete = true;
                }
                i += 1;
                continue;
            }

            if current_chain.len() >= 2 {
                all_chains.push(std::mem::take(&mut current_chain));
            }
            current_chain = vec![op];
            last_expr = op.compared_expr;
            last_check_type = OperandType::Invalid;
            if op.typ != OperandType::Plain {
                last_check_type = op.typ;
            }
            i += 1;
        }

        if current_chain.len() >= 2 {
            all_chains.push(current_chain);
        }

        for chain in all_chains.iter_mut() {
            while chain.len() >= 2 {
                let last_op = chain[chain.len() - 1];
                let second_to_last_op = chain[chain.len() - 2];
                if last_op.typ == OperandType::Plain && second_to_last_op.typ == OperandType::Plain
                {
                    let cmp = self.compare_nodes(ctx, second_to_last_op.cexpr(), last_op.cexpr());
                    if cmp == NodeComparisonResult::Equal {
                        chain.pop();
                        continue;
                    }
                }
                break;
            }
        }

        all_chains
    }

    fn build_or_chains(&mut self, ctx: &mut Ctx, operands: &[Operand]) -> Vec<Vec<Operand>> {
        let mut chain: Vec<Operand> = Vec::new();
        let mut last_expr: Option<P<Node>> = None;

        for &op in operands {
            if !is_valid_operand_for_chain_type(&op, Kind::BarBarToken) {
                if chain.len() >= 2 {
                    break;
                }
                chain.clear();
                last_expr = None;
                continue;
            }

            if chain.is_empty() {
                if self.is_invalid_or_chain_starting_operand(&op) {
                    continue;
                }
                chain.push(op);
                last_expr = op.compared_expr;
                continue;
            }

            let mut cmp = self.compare_nodes(ctx, last_expr.unwrap(), op.cexpr());

            if cmp == NodeComparisonResult::Invalid && !chain.is_empty() {
                let prev_op = *chain.last().unwrap();
                if self.should_allow_call_chain_extension(&prev_op, &op, Kind::BarBarToken)
                    && self.try_extend_through_call_expression(
                        last_expr,
                        &op,
                        None,
                        Kind::BarBarToken,
                    ) == NodeComparisonResult::Subset
                {
                    cmp = NodeComparisonResult::Subset;
                }
            }

            if cmp == NodeComparisonResult::Subset || cmp == NodeComparisonResult::Equal {
                if cmp == NodeComparisonResult::Equal
                    && op.typ == OperandType::Comparison
                    && !chain.is_empty()
                    && !is_nullish_comparison(&op)
                {
                    let last_op = *chain.last().unwrap();
                    if last_op.typ == OperandType::Not
                        || last_op.typ == OperandType::NotStrictEqualNull
                        || last_op.typ == OperandType::NotStrictEqualUndef
                        || last_op.typ == OperandType::NotEqualBoth
                        || last_op.typ == OperandType::Plain
                        || (last_op.typ == OperandType::Comparison
                            && !is_nullish_comparison(&last_op))
                    {
                        if chain.len() >= 2 {
                            break;
                        }
                        // Two non-nullish comparisons on the same expression (x === 'a' || x === 'b')
                        // are not an optional chain pattern.
                        if last_op.typ == OperandType::Comparison
                            && !is_nullish_comparison(&last_op)
                        {
                            chain.clear();
                            last_expr = None;
                            continue;
                        }
                    }
                }

                chain.push(op);
                last_expr = op.compared_expr;
                continue;
            }

            if chain.len() >= 2 {
                break;
            }
            chain = vec![op];
            last_expr = op.compared_expr;
        }

        if chain.len() < 2 {
            return Vec::new();
        }
        vec![chain]
    }

    fn should_stop_at_strict_nullish_check(&mut self, ctx: &mut Ctx, prev_op: &Operand) -> bool {
        let Some(prev_expr) = prev_op.compared_expr else {
            return false;
        };
        if !prev_op.typ.is_strict_nullish_check() {
            return false;
        }

        let prev_unwrapped = ast::skip_parentheses(prev_expr);
        let is_call_or_new =
            ast::is_call_expression(prev_unwrapped) || ast::is_new_expression(prev_unwrapped);
        let is_element_access = ast::is_element_access_expression(prev_unwrapped);

        if !is_call_or_new && !is_element_access {
            return false;
        }

        let is_any_or_unknown = self.type_is_any_or_unknown(ctx, prev_expr);
        let type_info = self.get_type_info(ctx, prev_expr);
        let has_null = type_info.has_null || type_info.has_any || type_info.has_unknown;
        let has_undefined = type_info.has_undefined || type_info.has_any || type_info.has_unknown;

        let is_incomplete = !is_any_or_unknown && has_null && has_undefined;

        let mut is_mismatched = false;
        if !is_any_or_unknown {
            if prev_op.typ == OperandType::NotStrictEqualUndef && !has_undefined && !has_null {
                is_mismatched = true;
            }
            if prev_op.typ == OperandType::NotStrictEqualNull && !has_null && !has_undefined {
                is_mismatched = true;
            }
        }

        if is_call_or_new {
            return is_incomplete || is_mismatched;
        }
        if is_element_access {
            if is_mismatched {
                return true;
            }
            if is_incomplete && !self.unsafe_ok() {
                return true;
            }
        }
        false
    }

    fn try_merge_complementary_pair(
        &mut self,
        ctx: &mut Ctx,
        op1: &Operand,
        op2: &Operand,
        operator_kind: Kind,
    ) -> Option<Vec<Operand>> {
        let (Some(e1), Some(e2)) = (op1.compared_expr, op2.compared_expr) else {
            return None;
        };
        if self.compare_nodes(ctx, e1, e2) != NodeComparisonResult::Equal {
            return None;
        }

        let (is_op1_null, is_op1_undef, is_op2_null, is_op2_undef) =
            if is_and_operator(operator_kind) {
                (
                    op1.typ == OperandType::NotStrictEqualNull,
                    op1.typ == OperandType::NotStrictEqualUndef
                        || op1.typ == OperandType::TypeofCheck,
                    op2.typ == OperandType::NotStrictEqualNull,
                    op2.typ == OperandType::NotStrictEqualUndef
                        || op2.typ == OperandType::TypeofCheck,
                )
            } else {
                (
                    op1.typ == OperandType::StrictEqualNull,
                    op1.typ == OperandType::StrictEqualUndef,
                    op2.typ == OperandType::StrictEqualNull,
                    op2.typ == OperandType::StrictEqualUndef,
                )
            };
        if (is_op1_null && is_op2_undef) || (is_op1_undef && is_op2_null) {
            return Some(vec![*op1, *op2]);
        }
        None
    }

    fn are_nullish_checks_consistent(&self, type1: OperandType, type2: OperandType) -> bool {
        if type1.is_loose_nullish_check() || type2.is_loose_nullish_check() {
            return true;
        }
        let is_type1_null =
            type1 == OperandType::NotStrictEqualNull || type1 == OperandType::StrictEqualNull;
        let is_type1_undef = type1 == OperandType::NotStrictEqualUndef
            || type1 == OperandType::StrictEqualUndef
            || type1 == OperandType::TypeofCheck;
        let is_type2_null =
            type2 == OperandType::NotStrictEqualNull || type2 == OperandType::StrictEqualNull;
        let is_type2_undef = type2 == OperandType::NotStrictEqualUndef
            || type2 == OperandType::StrictEqualUndef
            || type2 == OperandType::TypeofCheck;

        if (is_type1_null && is_type2_undef) || (is_type1_undef && is_type2_null) {
            return true;
        }
        (is_type1_null && is_type2_null) || (is_type1_undef && is_type2_undef)
    }

    fn validate_chain_for_reporting(
        &mut self,
        ctx: &mut Ctx,
        chain: Vec<Operand>,
        operator_kind: Kind,
    ) -> Option<Vec<Operand>> {
        if !self.validate_chain(ctx, &chain, operator_kind) {
            return None;
        }
        if is_and_operator(operator_kind) {
            return self.validate_and_chain_for_reporting(ctx, chain);
        }
        self.validate_or_chain_for_reporting(ctx, chain)
    }

    fn validate_and_chain_for_reporting(
        &mut self,
        ctx: &mut Ctx,
        chain: Vec<Operand>,
    ) -> Option<Vec<Operand>> {
        let first_op = chain[0];
        if first_op.typ == OperandType::Plain
            && first_op.compared_expr.is_some()
            && self.contains_optional_chain(first_op.compared_expr)
        {
            return None;
        }

        if first_op.typ.is_strict_nullish_check()
            && first_op.compared_expr.is_some()
            && self.contains_optional_chain(first_op.compared_expr)
            && !self.is_split_strict_equals_pattern(&chain)
        {
            return None;
        }

        // All operands check the same expression (e.g., x !== undefined && x !== null): already optimal.
        if self.all_operands_check_same_expression(ctx, &chain) {
            return None;
        }

        if self.should_skip_optimal_strict_checks(ctx, &chain) {
            return None;
        }

        if chain.len() == 2 {
            let first_op = chain[0];
            let second_op = chain[1];

            if self.contains_optional_chain(second_op.compared_expr) {
                let first_parts = self.flatten_for_fix(ctx, first_op.cexpr());
                // For comparison operands, use compared_expr (the access expression) instead of node
                let second_expr = second_op.compared_expr.unwrap_or(second_op.node);
                let second_parts = self.flatten_for_fix(ctx, second_expr);
                let first_parts = first_parts.borrow().clone();
                let second_parts = second_parts.borrow().clone();

                let mut is_redundant_check = false;
                if second_parts.len() == first_parts.len() + 1 {
                    let all_match = first_parts
                        .iter()
                        .zip(&second_parts)
                        .all(|(f, s)| f.text == s.text && f.optional == s.optional);
                    if all_match && second_parts.last().unwrap().optional {
                        is_redundant_check = true;
                    }
                }

                if !is_redundant_check
                    && second_parts.len() > first_parts.len()
                    && !first_parts.is_empty()
                {
                    let bases_match =
                        first_parts.iter().zip(&second_parts).all(|(f, s)| f.text == s.text);
                    if bases_match {
                        let has_optional_in_extension =
                            second_parts[first_parts.len()..].iter().any(|p| p.optional);
                        if has_optional_in_extension {
                            return None;
                        }
                    }
                }

                // `foo && foo?.id === null` -> `foo?.id === null` would change the return type.
                if second_op.typ == OperandType::Comparison && is_redundant_check {
                    return None;
                }
            }
        }

        if !chain.is_empty() && chain[0].typ == OperandType::Plain {
            if let Some(e) = chain[0].compared_expr {
                if self.has_void_type(ctx, e) {
                    return None;
                }
            }
        }

        if !self.unsafe_ok() {
            for op in &chain {
                if ast::is_non_null_expression(op.node) {
                    return None;
                }
            }
            if self.has_incomplete_nullish_check(ctx, &chain) {
                return None;
            }
        }

        for (i, op) in chain.iter().enumerate() {
            if op.typ == OperandType::Plain && i == 0 {
                if self.should_skip_by_type(ctx, op.cexpr()) {
                    return None;
                }
                if self.would_change_return_type(ctx, op.cexpr()) && !self.unsafe_ok() {
                    return None;
                }
            }
            if op.typ == OperandType::TypeofCheck {
                if let Some(e) = op.compared_expr {
                    if !self.includes_nullish(ctx, e) && chain.len() == 2 {
                        let last_op = chain[chain.len() - 1];
                        if let Some(le) = last_op.compared_expr {
                            if ast::is_call_expression(ast::skip_parentheses(le)) {
                                return None;
                            }
                        }
                    }
                }
            }
        }

        if chain.len() >= 2 {
            let last_op = chain[chain.len() - 1];
            if self.is_unsafe_trailing_comparison(ctx, &chain, &last_op) {
                return None;
            }
        }

        Some(chain)
    }

    // Optional chaining checks for BOTH null AND undefined, so if the chain only checks
    // for one but not both, it's unsafe to convert.
    fn has_incomplete_nullish_check(&mut self, ctx: &mut Ctx, chain: &[Operand]) -> bool {
        let mut guard_operands = chain;
        if chain.len() >= 2 {
            let last_op = chain[chain.len() - 1];
            if last_op.typ.is_trailing_comparison() {
                guard_operands = &chain[..chain.len() - 1];
            } else if last_op.typ == OperandType::Plain && last_op.compared_expr.is_some() {
                let prev_op = chain[chain.len() - 2];
                if let Some(pe) = prev_op.compared_expr {
                    let last_len = self.flatten_len(ctx, last_op.cexpr());
                    let prev_len = self.flatten_len(ctx, pe);
                    if last_len > prev_len {
                        guard_operands = &chain[..chain.len() - 1];
                    }
                }
            }
        }

        let analysis = analyze_nullish_checks(guard_operands, false);

        let has_typeof_check = guard_operands.iter().any(|op| op.typ == OperandType::TypeofCheck);

        let mut has_trailing_both_check = false;
        if chain.len() >= 2
            && guard_operands.len() < chain.len()
            && chain[chain.len() - 1].typ == OperandType::NotEqualBoth
        {
            has_trailing_both_check = true;
        }

        let mut has_trailing_optional_chaining = false;
        if chain.len() >= 2 && guard_operands.len() < chain.len() {
            let last_op = chain[chain.len() - 1];
            if last_op.compared_expr.is_some()
                && self.contains_optional_chain(last_op.compared_expr)
            {
                has_trailing_optional_chaining = true;
            }
        }

        let mut first_op_not_nullish = false;
        if let Some(e) = guard_operands.first().and_then(|o| o.compared_expr) {
            if !self.includes_nullish(ctx, e) {
                first_op_not_nullish = true;
            }
        }

        let mut strict_check_is_complete = false;
        if let Some(e) = guard_operands.first().and_then(|o| o.compared_expr) {
            let info = self.get_type_info(ctx, e);
            if !info.is_any_or_unknown() {
                let has_only_null_check = analysis.has_null_check && !analysis.has_undefined_check;
                let has_only_undefined_check =
                    !analysis.has_null_check && analysis.has_undefined_check;
                if has_only_null_check && info.has_null && !info.has_undefined {
                    strict_check_is_complete = true;
                }
                if has_only_undefined_check && info.has_undefined && !info.has_null {
                    strict_check_is_complete = true;
                }
            }
        }

        let has_plain_truthiness_check = analysis.has_both_check;

        !has_plain_truthiness_check
            && !has_typeof_check
            && !has_trailing_both_check
            && !has_trailing_optional_chaining
            && !first_op_not_nullish
            && !strict_check_is_complete
            && analysis.has_incomplete_check()
    }

    fn is_unsafe_trailing_comparison(
        &mut self,
        ctx: &mut Ctx,
        chain: &[Operand],
        last_op: &Operand,
    ) -> bool {
        let mut is_trailing_comparison = last_op.typ == OperandType::Comparison;
        if !is_trailing_comparison
            && chain.len() >= 2
            && matches!(
                last_op.typ,
                OperandType::NotStrictEqualNull
                    | OperandType::NotStrictEqualUndef
                    | OperandType::NotEqualBoth
            )
        {
            let prev_op = chain[chain.len() - 2];
            if let (Some(le), Some(pe)) = (last_op.compared_expr, prev_op.compared_expr) {
                let last_len = self.flatten_len(ctx, le);
                let prev_len = self.flatten_len(ctx, pe);
                if last_len > prev_len {
                    is_trailing_comparison = true;
                }
            }
        }

        if is_trailing_comparison {
            let unwrapped_node = ast::skip_parentheses(last_op.node);
            if ast::is_binary_expression(unwrapped_node) {
                let bin_expr = unwrapped_node.as_binary_expression();
                let op = bin_expr.operator_token.kind();
                let left = ast::skip_parentheses(bin_expr.left);
                let right = ast::skip_parentheses(bin_expr.right.get());

                let value =
                    if last_op.compared_expr.is_some_and(|e| are_nodes_structurally_equal(e, left))
                    {
                        Some(right)
                    } else if last_op
                        .compared_expr
                        .is_some_and(|e| are_nodes_structurally_equal(e, right))
                    {
                        Some(left)
                    } else if utils::is_access_expression(left) {
                        Some(right)
                    } else if utils::is_access_expression(right) {
                        Some(left)
                    } else {
                        None
                    };

                if let Some(value) = value {
                    if !self.is_safe_trailing_comparison_value(ctx, op, value) && !self.unsafe_ok()
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_safe_trailing_comparison_value(
        &mut self,
        ctx: &mut Ctx,
        operator: Kind,
        value: P<Node>,
    ) -> bool {
        let info = self.get_type_info(ctx, value);
        match operator {
            Kind::EqualsEqualsToken => !info.can_be_nullish_like(),
            Kind::EqualsEqualsEqualsToken => !info.can_be_undefined_like(),
            Kind::ExclamationEqualsToken => info.is_always_nullish_like(),
            Kind::ExclamationEqualsEqualsToken => info.is_always_undefined_like(),
            Kind::LessThanToken
            | Kind::GreaterThanToken
            | Kind::LessThanEqualsToken
            | Kind::GreaterThanEqualsToken => false,
            _ => true,
        }
    }

    fn validate_or_chain_nullish_checks(
        &mut self,
        ctx: &mut Ctx,
        mut chain: Vec<Operand>,
    ) -> Option<Vec<Operand>> {
        if self.unsafe_ok() {
            return Some(chain);
        }

        let analysis = analyze_nullish_checks(&chain, true);

        let mut first_op_not_nullish = false;
        let mut has_trailing_optional_chaining = false;
        if let Some(e) = chain.first().and_then(|o| o.compared_expr) {
            if !self.includes_nullish(ctx, e) {
                first_op_not_nullish = true;
            }
        }
        if chain.len() >= 2 {
            let last_op = chain[chain.len() - 1];
            if last_op.compared_expr.is_some()
                && self.contains_optional_chain(last_op.compared_expr)
            {
                has_trailing_optional_chaining = true;
            }
        }

        let mut strict_check_is_complete = true;
        let mut has_any_nullable_operand = false;
        for op in &chain {
            let Some(e) = op.compared_expr else { continue };
            let info = self.get_type_info(ctx, e);
            if info.has_no_nullable_types() {
                continue;
            }
            has_any_nullable_operand = true;
            if info.is_any_or_unknown() || info.has_both_null_and_undefined() {
                strict_check_is_complete = false;
                break;
            }
            if analysis.has_only_null_check() && !info.has_null && info.has_undefined {
                strict_check_is_complete = false;
                break;
            }
            if analysis.has_only_undefined_check() && info.has_null && !info.has_undefined {
                strict_check_is_complete = false;
                break;
            }
        }
        if !has_any_nullable_operand {
            strict_check_is_complete = false;
        }

        if !analysis.has_both_check
            && !first_op_not_nullish
            && !has_trailing_optional_chaining
            && !strict_check_is_complete
            && analysis.has_incomplete_check()
        {
            return None;
        }

        let mut truncate_at: isize = -1;
        for (i, op) in chain.iter().enumerate() {
            if i == chain.len() - 1 {
                continue;
            }
            if op.compared_expr.is_some() && self.contains_optional_chain(op.compared_expr) {
                continue;
            }
            let Some(e) = op.compared_expr else { continue };
            let type_info = self.get_type_info(ctx, e);
            if !(type_info.has_null && type_info.has_undefined) {
                continue;
            }
            let next = (i + 1) as isize;
            match op.typ {
                OperandType::NotStrictEqualNull
                | OperandType::NotStrictEqualUndef
                | OperandType::StrictEqualNull
                | OperandType::StrictEqualUndef
                    if truncate_at == -1 || next < truncate_at =>
                {
                    truncate_at = next;
                }
                OperandType::Comparison => {
                    let unwrapped = ast::skip_parentheses(op.node);
                    if ast::is_binary_expression(unwrapped) {
                        let bin_expr = unwrapped.as_binary_expression();
                        if bin_expr.operator_token.kind() == Kind::EqualsEqualsEqualsToken {
                            let (l, r) = (bin_expr.left, bin_expr.right.get());
                            let is_strict_null_check =
                                utils::is_null_literal(Some(r)) || utils::is_null_literal(Some(l));
                            let is_strict_undef_check = utils::is_undefined_literal(Some(r))
                                || utils::is_undefined_literal(Some(l));
                            if ((is_strict_null_check && !is_strict_undef_check)
                                || (is_strict_undef_check && !is_strict_null_check))
                                && (truncate_at == -1 || next < truncate_at)
                            {
                                truncate_at = next;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if truncate_at > 0 && (truncate_at as usize) < chain.len() {
            chain.truncate(truncate_at as usize);
        }

        if chain.len() < 2 {
            return None;
        }

        // A nullish guard start requires every later operand to be a safe comparison.
        let is_nullish_guard_start = matches!(
            chain[0].typ,
            OperandType::TypeofCheck
                | OperandType::StrictEqualNull
                | OperandType::StrictEqualUndef
                | OperandType::EqualNull
        );
        if is_nullish_guard_start && !self.unsafe_ok() {
            for op in &chain[1..] {
                if op.typ == OperandType::Comparison && !self.is_or_chain_comparison_safe(op) {
                    return None;
                }
                if op.typ == OperandType::Plain || op.typ == OperandType::Not {
                    return None;
                }
            }
        }

        if analysis.has_null_check
            && !analysis.has_undefined_check
            && !analysis.has_both_check
            && !strict_check_is_complete
        {
            return None;
        }

        if chain[0].typ == OperandType::Comparison
            && is_nullish_comparison(&chain[0])
            && !self.unsafe_ok()
        {
            for op in &chain[1..] {
                if op.typ == OperandType::Comparison
                    && !self.is_or_chain_comparison_safe(op)
                    && !is_nullish_comparison(op)
                {
                    return None;
                }
            }
        }

        Some(chain)
    }

    fn has_unsafe_strict_nullish_guard_with_negated_access(
        &mut self,
        ctx: &mut Ctx,
        chain: &[Operand],
    ) -> bool {
        if chain.len() < 2 {
            return false;
        }
        for i in 0..chain.len() - 1 {
            let guard_op = chain[i];
            let Some(guard_expr) = guard_op.compared_expr else {
                continue;
            };
            if !guard_op.typ.is_strict_nullish_check() {
                continue;
            }
            let guard_type = self.get_type_info(ctx, guard_expr);
            if !guard_type.is_any_or_unknown() && !guard_type.has_both_null_and_undefined() {
                continue;
            }
            for candidate in &chain[i + 1..] {
                if candidate.typ != OperandType::Not {
                    continue;
                }
                let Some(cand_expr) = candidate.compared_expr else {
                    continue;
                };
                let cmp = self.compare_nodes(ctx, guard_expr, cand_expr);
                if cmp != NodeComparisonResult::Equal && cmp != NodeComparisonResult::Subset {
                    continue;
                }
                let unwrapped = ast::skip_parentheses(cand_expr);
                if ast::is_call_expression(unwrapped)
                    || ast::is_property_access_expression(unwrapped)
                    || ast::is_element_access_expression(unwrapped)
                {
                    return true;
                }
            }
        }
        false
    }

    fn validate_or_chain_for_reporting(
        &mut self,
        ctx: &mut Ctx,
        chain: Vec<Operand>,
    ) -> Option<Vec<Operand>> {
        let has_explicit_check = chain.iter().any(|op| op.typ != OperandType::Plain);
        if !has_explicit_check {
            return None;
        }

        if self.should_skip_or_chain_optimal_checks(ctx, &chain) {
            return None;
        }

        // All operands check the same expression at the same depth.
        if self.all_operands_check_same_expression(ctx, &chain) {
            return None;
        }

        let all_subsequent_have_optional_chaining =
            self.all_subsequent_have_optional_chaining(&chain);
        if all_subsequent_have_optional_chaining {
            let first_op = chain[0];
            let is_explicit_null_check = first_op.typ == OperandType::StrictEqualNull
                || first_op.typ == OperandType::EqualNull
                || is_strict_null_comparison(&first_op);
            if is_explicit_null_check {
                let mut any_type_has_null_or_undefined = false;
                for op in &chain {
                    if let Some(e) = op.compared_expr {
                        let type_info = self.get_type_info(ctx, e);
                        if type_info.has_null || type_info.has_undefined {
                            any_type_has_null_or_undefined = true;
                            break;
                        }
                    }
                }
                if !any_type_has_null_or_undefined {
                    return None;
                }
            }
        }

        let mut all_strict_checks = true;
        for op in &chain {
            if op.typ.is_loose_nullish_check()
                || op.typ == OperandType::Plain
                || op.typ == OperandType::Not
            {
                all_strict_checks = false;
                break;
            }
            if op.typ == OperandType::Comparison && ast::is_binary_expression(op.node) {
                let bin_expr = op.node.as_binary_expression();
                if bin_expr.operator_token.kind() == Kind::EqualsEqualsToken {
                    let left = ast::skip_parentheses(bin_expr.left);
                    let right = ast::skip_parentheses(bin_expr.right.get());
                    let is_nullish = utils::is_null_literal(Some(left))
                        || utils::is_null_literal(Some(right))
                        || utils::is_undefined_identifier(Some(left))
                        || utils::is_undefined_identifier(Some(right));
                    if is_nullish {
                        all_strict_checks = false;
                        break;
                    }
                }
            }
        }

        if all_subsequent_have_optional_chaining && all_strict_checks {
            let mut any_has_null = false;
            let mut any_has_undefined = false;
            for op in &chain {
                if let Some(e) = op.compared_expr {
                    let type_info = self.get_type_info(ctx, e);
                    if type_info.has_null {
                        any_has_null = true;
                    }
                    if type_info.has_undefined {
                        any_has_undefined = true;
                    }
                }
            }
            if !any_has_null && !any_has_undefined {
                return None;
            }
            if any_has_null && any_has_undefined {
                return None;
            }
        }

        if chain[0].typ == OperandType::Not && !self.unsafe_ok() {
            if let Some(first_expr) = chain[0].compared_expr {
                let unwrapped_first = ast::skip_parentheses(first_expr);
                let is_first_simple_negation = !utils::is_access_expression(unwrapped_first);

                if is_first_simple_negation {
                    let mut all_negated_or_safe_comparison_or_null_check = true;
                    let mut has_intermediate_nullish_comp = false;
                    for i in 1..chain.len() {
                        let is_comparison = chain[i].typ == OperandType::Comparison;
                        let is_safe_comparison =
                            is_comparison && self.is_or_chain_comparison_safe(&chain[i]);
                        let is_intermediate_nullish_comp = is_comparison
                            && is_nullish_comparison(&chain[i])
                            && i < chain.len() - 1;
                        if is_intermediate_nullish_comp {
                            has_intermediate_nullish_comp = true;
                        }
                        let is_allowed_plain_at_end = chain[i].typ == OperandType::Plain
                            && i == chain.len() - 1
                            && has_intermediate_nullish_comp;

                        if chain[i].typ != OperandType::Not
                            && !is_safe_comparison
                            && !is_intermediate_nullish_comp
                            && !chain[i].typ.is_nullish_check()
                            && !is_allowed_plain_at_end
                        {
                            all_negated_or_safe_comparison_or_null_check = false;
                            break;
                        }
                    }
                    if !all_negated_or_safe_comparison_or_null_check {
                        return None;
                    }
                } else {
                    for op in &chain[1..] {
                        if op.typ == OperandType::Comparison
                            && !self.is_or_chain_comparison_safe(op)
                        {
                            return None;
                        }
                    }
                }
            }
        }

        if !self.unsafe_ok()
            && matches!(
                chain[0].typ,
                OperandType::NotEqualBoth
                    | OperandType::NotStrictEqualNull
                    | OperandType::NotStrictEqualUndef
            )
        {
            let first_type_info = self.get_type_info(ctx, chain[0].cexpr());
            for i in 1..chain.len() {
                if chain[i].typ == OperandType::Comparison
                    && !self.is_or_chain_comparison_safe(&chain[i])
                {
                    let is_last_operand = i == chain.len() - 1;
                    if is_last_operand && is_nullish_comparison(&chain[i]) {
                        if first_type_info.has_null && first_type_info.has_undefined {
                            return None;
                        }
                        if first_type_info.has_any || first_type_info.has_unknown {
                            return None;
                        }
                    } else if !is_nullish_comparison(&chain[i]) {
                        return None;
                    }
                }
            }
        }

        if chain[0].typ == OperandType::Plain {
            if let Some(first_expr) = chain[0].compared_expr {
                if ast::skip_parentheses(first_expr).kind() == Kind::MetaProperty {
                    return None;
                }
            }
        }

        if chain[0].typ == OperandType::Plain && !self.unsafe_ok() {
            if let Some(first_expr) = chain[0].compared_expr {
                let is_first_simple_plain =
                    !utils::is_access_expression(ast::skip_parentheses(first_expr));
                if is_first_simple_plain
                    && chain[1..].iter().any(|op| op.typ == OperandType::Comparison)
                {
                    return None;
                }
            }
        }

        let chain = self.validate_or_chain_nullish_checks(ctx, chain)?;
        if chain.len() < 2 {
            return None;
        }

        if !self.unsafe_ok()
            && self.has_unsafe_strict_nullish_guard_with_negated_access(ctx, &chain)
        {
            return None;
        }

        for op in &chain {
            if (op.typ == OperandType::Plain || op.typ == OperandType::Not)
                && self.would_change_return_type(ctx, op.cexpr())
                && !self.unsafe_ok()
            {
                return None;
            }
        }

        for i in 0..chain.len() - 1 {
            if chain[i].typ != OperandType::Not {
                continue;
            }
            let negated_expr = chain[i].compared_expr;
            for later in &chain[i + 1..] {
                if later.typ == OperandType::Not {
                    continue;
                }
                if let Some(call_expr) = later.compared_expr {
                    let unwrapped = ast::skip_parentheses(call_expr);
                    if ast::is_call_expression(unwrapped) {
                        let call_base = unwrapped.as_call_expression().expression;
                        let cmp = self.compare_nodes(ctx, negated_expr.unwrap(), call_base);
                        if cmp == NodeComparisonResult::Equal {
                            return None;
                        }
                    }
                }
            }
        }

        Some(chain)
    }

    fn all_operands_check_same_expression(&mut self, ctx: &Ctx, chain: &[Operand]) -> bool {
        if chain.len() < 2 {
            return false;
        }
        let first_parts = self.flatten_for_fix(ctx, chain[0].cexpr());
        for op in &chain[1..] {
            let op_parts = self.flatten_for_fix(ctx, op.cexpr());
            let (fp, opp) = (first_parts.borrow(), op_parts.borrow());
            if opp.len() != fp.len() {
                return false;
            }
            if fp.iter().zip(opp.iter()).any(|(a, b)| a.text != b.text) {
                return false;
            }
        }
        true
    }

    fn is_split_strict_equals_pattern(&self, chain: &[Operand]) -> bool {
        if chain.len() != 2 {
            return false;
        }
        let first_op = chain[0];
        let last_op = chain[1];
        let is_first_undef = first_op.typ == OperandType::NotStrictEqualUndef
            || first_op.typ == OperandType::TypeofCheck;
        let is_first_null = first_op.typ == OperandType::NotStrictEqualNull;
        let is_last_undef = last_op.typ == OperandType::NotStrictEqualUndef
            || last_op.typ == OperandType::TypeofCheck;
        let is_last_null = last_op.typ == OperandType::NotStrictEqualNull;
        (is_first_undef && is_last_null) || (is_first_null && is_last_undef)
    }

    fn should_skip_optimal_strict_checks(&mut self, ctx: &mut Ctx, chain: &[Operand]) -> bool {
        if chain.len() < 2 {
            return false;
        }
        if !self.all_subsequent_have_optional_chaining(chain) {
            return false;
        }

        let mut all_strict_checks = true;
        let mut has_nullish_check = false;
        for op in chain {
            if op.typ == OperandType::Plain {
                continue;
            }
            if op.typ == OperandType::Not || op.typ == OperandType::NegatedAndOperand {
                all_strict_checks = false;
                break;
            }
            if op.typ.is_loose_nullish_check() {
                all_strict_checks = false;
                break;
            }
            if op.typ.is_strict_nullish_check() {
                has_nullish_check = true;
            }
        }

        if !has_nullish_check || !all_strict_checks {
            return false;
        }

        // Strict checks (=== null or === undefined) intentionally cover one - skip only when type has BOTH
        self.get_type_info(ctx, chain[0].cexpr()).has_both_null_and_undefined()
    }

    fn should_skip_or_chain_optimal_checks(&mut self, ctx: &mut Ctx, chain: &[Operand]) -> bool {
        if chain.len() < 2 {
            return false;
        }
        if !self.all_subsequent_have_optional_chaining(chain) {
            return false;
        }
        let first_op = chain[0];
        if first_op.typ != OperandType::StrictEqualNull {
            return false;
        }
        if let Some(e) = first_op.compared_expr {
            let type_info = self.get_type_info(ctx, e);
            if !type_info.has_undefined && !type_info.has_any && !type_info.has_unknown {
                return true;
            }
        }
        false
    }

    fn generate_fix_and_report(
        &mut self,
        ctx: &mut Ctx,
        node: P<Node>,
        chain: &[Operand],
        operand_nodes: &[P<Node>],
        operator_kind: Kind,
    ) {
        if is_and_operator(operator_kind) {
            self.generate_and_chain_fix_and_report(ctx, node, chain, operand_nodes);
        } else {
            self.generate_or_chain_fix_and_report(ctx, node, chain, operand_nodes);
        }
    }

    fn generate_and_chain_fix_and_report(
        &mut self,
        ctx: &mut Ctx,
        node: P<Node>,
        chain: &[Operand],
        operand_nodes: &[P<Node>],
    ) {
        let mut last_property_access: Option<P<Node>> = None;
        let mut has_trailing_comparison = false;
        let mut has_trailing_typeof_check = false;
        let mut has_complementary_null_check = false;
        let mut complementary_trailing_node: Option<P<Node>> = None;
        let mut has_loose_strict_with_trailing_plain = false;
        let mut loose_strict_trailing_plain_node: Option<P<Node>> = None;

        // Complementary pair (null + undefined checks on same expression): use second-to-last as
        // chain endpoint and append last as trailing text.
        if chain.len() >= 2 {
            let last_op = chain[chain.len() - 1];
            let second_last_op = chain[chain.len() - 2];
            if let (Some(le), Some(se)) = (last_op.compared_expr, second_last_op.compared_expr) {
                if self.compare_nodes(ctx, le, se) == NodeComparisonResult::Equal {
                    let is_last_undef = last_op.typ == OperandType::NotStrictEqualUndef
                        || last_op.typ == OperandType::TypeofCheck;
                    let is_last_null = last_op.typ == OperandType::NotStrictEqualNull;
                    let is_second_last_undef = second_last_op.typ
                        == OperandType::NotStrictEqualUndef
                        || second_last_op.typ == OperandType::TypeofCheck;
                    let is_second_last_null = second_last_op.typ == OperandType::NotStrictEqualNull;

                    if (is_last_undef && is_second_last_null)
                        || (is_last_null && is_second_last_undef)
                    {
                        has_complementary_null_check = true;
                        last_property_access = second_last_op.compared_expr;
                        complementary_trailing_node = Some(last_op.node);
                        has_trailing_comparison = true;
                        has_trailing_typeof_check = second_last_op.typ == OperandType::TypeofCheck;
                    }
                }
            }
        }

        // Loose+strict transition with trailing Plain:
        // foo && foo.bar != null && foo.bar.baz !== undefined && foo.bar.baz.buzz
        // -> foo?.bar?.baz !== undefined && foo.bar.baz.buzz
        if !has_complementary_null_check && chain.len() >= 3 {
            let last_op = chain[chain.len() - 1];
            let second_last_op = chain[chain.len() - 2];

            if last_op.typ == OperandType::Plain {
                let is_strict_check = second_last_op.typ == OperandType::NotStrictEqualNull
                    || second_last_op.typ == OperandType::NotStrictEqualUndef;

                if let (true, Some(second_last_expr)) =
                    (is_strict_check, second_last_op.compared_expr)
                {
                    let mut has_matching_strict_check = false;
                    for i in (0..chain.len() - 2).rev() {
                        let Some(ce) = chain[i].compared_expr else {
                            continue;
                        };
                        if self.compare_nodes(ctx, ce, second_last_expr)
                            == NodeComparisonResult::Equal
                        {
                            let is_second_last_null =
                                second_last_op.typ == OperandType::NotStrictEqualNull;
                            let is_second_last_undef =
                                second_last_op.typ == OperandType::NotStrictEqualUndef;
                            let is_other_null = chain[i].typ == OperandType::NotStrictEqualNull;
                            let is_other_undef = chain[i].typ == OperandType::NotStrictEqualUndef
                                || chain[i].typ == OperandType::TypeofCheck;
                            if (is_second_last_null && is_other_undef)
                                || (is_second_last_undef && is_other_null)
                            {
                                has_matching_strict_check = true;
                                break;
                            }
                        }
                    }

                    if !has_matching_strict_check {
                        let mut closest_loose_check_expr: Option<P<Node>> = None;
                        for i in (0..chain.len() - 2).rev() {
                            if chain[i].typ.is_loose_nullish_check() {
                                closest_loose_check_expr = chain[i].compared_expr;
                                break;
                            }
                        }

                        if let Some(cl) = closest_loose_check_expr {
                            if self.compare_nodes(ctx, cl, second_last_expr)
                                == NodeComparisonResult::Subset
                            {
                                has_loose_strict_with_trailing_plain = true;
                                last_property_access = second_last_op.compared_expr;
                                loose_strict_trailing_plain_node = Some(last_op.node);
                                has_trailing_comparison = true;
                                has_trailing_typeof_check = false;
                            }
                        }
                    }
                }
            }
        }

        if !has_complementary_null_check && !has_loose_strict_with_trailing_plain {
            for op in chain.iter().rev() {
                match op.typ {
                    OperandType::Plain => {
                        last_property_access = Some(op.node);
                        has_trailing_comparison = false;
                        has_trailing_typeof_check = false;
                        break;
                    }
                    OperandType::Comparison
                    | OperandType::NotStrictEqualNull
                    | OperandType::NotStrictEqualUndef
                    | OperandType::NotEqualBoth => {
                        last_property_access = op.compared_expr;
                        has_trailing_comparison = true;
                        has_trailing_typeof_check = false;
                        break;
                    }
                    OperandType::TypeofCheck => {
                        last_property_access = op.compared_expr;
                        has_trailing_comparison = true;
                        has_trailing_typeof_check = true;
                        break;
                    }
                    _ if op.compared_expr.is_some() => {
                        last_property_access = op.compared_expr;
                        has_trailing_comparison = false;
                        has_trailing_typeof_check = false;
                        break;
                    }
                    _ => {}
                }
            }
        }

        let Some(last_property_access) = last_property_access else {
            return;
        };

        let parts_rc = self.flatten_for_fix(ctx, last_property_access);

        if !chain.is_empty() && !parts_rc.borrow().is_empty() && chain[0].typ == OperandType::Plain
        {
            let first_parts = self.flatten_for_fix(ctx, chain[0].node);
            let first0 = {
                let fp = first_parts.borrow();
                if fp.len() == 1 && fp.len() <= parts_rc.borrow().len() {
                    Some(fp[0].clone())
                } else {
                    None
                }
            };
            if let Some(first0) = first0 {
                let longer = first0.text.len() > parts_rc.borrow()[0].text.len();
                if longer {
                    parts_rc.borrow_mut()[0] = first0;
                }
            }
        }

        let mut checked_lengths: FxHashSet<usize> = FxHashSet::default();

        let mut checks_to_consider: Vec<Operand> = Vec::new();
        for (i, op) in chain.iter().enumerate() {
            let is_last_operand = i == chain.len() - 1;
            let is_call_access = op.compared_expr.is_some_and(ast::is_call_expression);
            if is_last_operand
                && (op.typ == OperandType::Plain || (op.typ == OperandType::Not && is_call_access))
            {
                continue;
            }
            checks_to_consider.push(*op);
        }

        let has_non_typeof_check = checks_to_consider
            .iter()
            .any(|o| o.typ != OperandType::TypeofCheck && o.compared_expr.is_some());

        for operand in &checks_to_consider {
            let Some(e) = operand.compared_expr else {
                continue;
            };
            // Skip typeof checks when there are other checks - typeof verifies existence, not nullability.
            if operand.typ == OperandType::TypeofCheck && has_non_typeof_check {
                continue;
            }
            checked_lengths.insert(self.flatten_len(ctx, e));
        }

        // Fill in gaps: for single check at start, fill up to second-to-last part.
        let num_checks = checked_lengths.len();
        let min_checked = checked_lengths.iter().copied().min();
        let max_checked = checked_lengths.iter().copied().max();
        if let (Some(min_checked), Some(max_checked)) = (min_checked, max_checked) {
            if min_checked > 0 && num_checks == 1 && min_checked == 1 {
                let fill_up_to: isize = if !chain.is_empty()
                    && chain[chain.len() - 1].typ == OperandType::Plain
                {
                    let last_plain_parts = self.flatten_for_fix(ctx, chain[chain.len() - 1].node);
                    let lpp = last_plain_parts.borrow();
                    let mut f = lpp.len() as isize - 1;
                    if f > 0 && !lpp.is_empty() && lpp[lpp.len() - 1].is_call {
                        f -= 1;
                    }
                    f
                } else {
                    max_checked as isize
                };
                let mut i = min_checked as isize;
                while i <= fill_up_to {
                    checked_lengths.insert(i as usize);
                    i += 1;
                }
            }
        }

        if !checks_to_consider.is_empty() && parts_rc.borrow().len() > 1 {
            let mut max_checked_len = 0;
            for op in &checks_to_consider {
                if let Some(e) = op.compared_expr {
                    max_checked_len = max_checked_len.max(self.flatten_len(ctx, e));
                }
            }

            {
                let mut parts = parts_rc.borrow_mut();
                let n = parts.len();
                for part in parts.iter_mut().take(max_checked_len.min(n)) {
                    if part.has_non_null {
                        part.text = part.base_text();
                        part.has_non_null = false;
                    }
                }
            }

            let mut all_op_parts: Vec<Parts> = Vec::new();
            for op in &checks_to_consider {
                let Some(e) = op.compared_expr else { continue };
                let expr_to_flatten = if op.typ == OperandType::Plain { op.node } else { e };
                let op_parts = self.flatten_for_fix(ctx, expr_to_flatten);
                let is_prefix = {
                    let opp = op_parts.borrow();
                    let parts = parts_rc.borrow();
                    opp.len() <= parts.len()
                        && opp.iter().zip(parts.iter()).all(|(a, b)| a.base_text() == b.base_text())
                };
                if is_prefix {
                    all_op_parts.push(op_parts);
                }
            }

            let n = parts_rc.borrow().len();
            for i in 0..n {
                let mut shortest: Option<&Parts> = None;
                let mut shortest_len = 0;
                for op in &all_op_parts {
                    let len = op.borrow().len();
                    if i < len && (shortest.is_none() || len < shortest_len) {
                        shortest = Some(op);
                        shortest_len = len;
                    }
                }
                if let Some(op) = shortest {
                    let optional = op.borrow()[i].optional;
                    parts_rc.borrow_mut()[i].optional = optional;
                    let has_non_null = op.borrow()[i].has_non_null;
                    if has_non_null {
                        let mut parts = parts_rc.borrow_mut();
                        if !parts[i].has_non_null {
                            parts[i].text.push('!');
                        }
                        parts[i].has_non_null = true;
                    }
                }
            }
        }

        let mut call_should_be_optional = false;
        let (parts_len, last_is_call) = {
            let p = parts_rc.borrow();
            (p.len(), p.last().is_some_and(|l| l.is_call))
        };
        if parts_len > 0 && last_is_call {
            let parts_without_call = parts_len - 1;
            for op in &chain[..chain.len() - 1] {
                if let Some(e) = op.compared_expr {
                    if self.flatten_len(ctx, e) == parts_without_call {
                        call_should_be_optional = true;
                        break;
                    }
                }
            }
        }

        let mut new_code = build_optional_chain(
            &parts_rc.borrow(),
            &checked_lengths,
            call_should_be_optional,
            false,
        );
        if new_code.is_empty() {
            return;
        }

        if chain.len() > 1 {
            let mut leading_trivia = String::new();
            for op in &chain[1..] {
                let full_pos = op.node.pos();
                let trimmed_pos = ctx.trim(op.node).0;
                if full_pos < trimmed_pos {
                    leading_trivia.push_str(src(ctx, full_pos, trimmed_pos));
                }
            }
            if !leading_trivia.is_empty() {
                let trivia_str = leading_trivia.trim_start_matches([' ', '\t', '\n', '\r']);
                if !trivia_str.is_empty() {
                    new_code = format!("{trivia_str}{new_code}");
                }
            }
        }

        if has_trailing_comparison {
            let operand_for_comparison =
                if has_complementary_null_check || has_loose_strict_with_trailing_plain {
                    chain[chain.len() - 2]
                } else {
                    chain[chain.len() - 1]
                };

            if ast::is_binary_expression(operand_for_comparison.node) {
                let bin_expr = operand_for_comparison.node.as_binary_expression();
                let compared = operand_for_comparison.cexpr();

                if has_trailing_typeof_check {
                    let left_range = ctx.trim(bin_expr.left);
                    let compared_expr_range = ctx.trim(compared);
                    let typeof_prefix = src(ctx, left_range.0, compared_expr_range.0);
                    let bin_expr_end = ctx.trim(operand_for_comparison.node).1;
                    let comparison_suffix = src(ctx, compared_expr_range.1, bin_expr_end);
                    new_code = format!("{typeof_prefix}{new_code}{comparison_suffix}");
                } else {
                    let compared_expr_range = ctx.trim(compared);
                    let left_range = ctx.trim(bin_expr.left);
                    let is_yoda = compared_expr_range.0 > left_range.0;
                    if is_yoda {
                        let bin_expr_start = ctx.trim(operand_for_comparison.node).0;
                        let yoda_prefix = src(ctx, bin_expr_start, compared_expr_range.0);
                        new_code = format!("{yoda_prefix}{new_code}");
                    } else {
                        let bin_expr_end = ctx.trim(operand_for_comparison.node).1;
                        let comparison_suffix = src(ctx, compared_expr_range.1, bin_expr_end);
                        new_code.push_str(comparison_suffix);
                    }
                }
            }

            if has_complementary_null_check {
                if let Some(trailing) = complementary_trailing_node {
                    let second_last_range = ctx.trim(chain[chain.len() - 2].node);
                    let last_range = ctx.trim(trailing);
                    let between_text = src(ctx, second_last_range.1, last_range.0);
                    new_code = format!("{new_code}{between_text}{}", node_text(ctx, trailing));
                }
            }

            if has_loose_strict_with_trailing_plain {
                if let Some(trailing) = loose_strict_trailing_plain_node {
                    let second_last_range = ctx.trim(chain[chain.len() - 2].node);
                    let last_range = ctx.trim(trailing);
                    let between_text = src(ctx, second_last_range.1, last_range.0);
                    new_code = format!("{new_code}{between_text}{}", node_text(ctx, trailing));
                }
            }
        }

        // Preserve typeof check on undeclared variables to avoid ReferenceError.
        let mut effective_chain_start = 0;
        if chain.len() >= 2 && chain[0].typ == OperandType::TypeofCheck {
            let has_non_typeof_after_first =
                chain[1..].iter().any(|o| o.typ != OperandType::TypeofCheck);
            if has_non_typeof_after_first {
                if let Some(typeof_target) = chain[0].compared_expr {
                    let symbol = ctx.checker.get_symbol_at_location_exported(typeof_target);
                    let is_undeclared = symbol.is_none_or(|s| s.declarations().is_empty());
                    if is_undeclared {
                        effective_chain_start = 1;
                    }
                }
            }
        }

        let (replace_start, replace_end) = if effective_chain_start == 0
            && chain.len() == operand_nodes.len()
        {
            ctx.trim(node)
        } else {
            (ctx.trim(chain[effective_chain_start].node).0, ctx.trim(chain[chain.len() - 1].node).1)
        };

        let fixes = vec![ctx.fix_replace_range(replace_start, replace_end, new_code)];

        // Autofix is safe when: unsafe option enabled, trailing comparison present,
        // operand type includes undefined/any/unknown, or certain nullish comparison types.
        let mut use_suggestion = !self.unsafe_ok();

        if use_suggestion && !chain.is_empty() {
            let last_op = chain[chain.len() - 1];
            if matches!(
                last_op.typ,
                OperandType::EqualNull
                    | OperandType::NotEqualBoth
                    | OperandType::StrictEqualUndef
                    | OperandType::NotStrictEqualUndef
                    | OperandType::TypeofCheck
            ) {
                use_suggestion = false;
            }

            if use_suggestion && has_trailing_comparison {
                use_suggestion = false;
            }

            if use_suggestion {
                for op in chain {
                    if let Some(e) = op.compared_expr {
                        let info = self.get_type_info(ctx, e);
                        if info.has_undefined || info.is_any_or_unknown() {
                            use_suggestion = false;
                            break;
                        }
                    }
                }
            }

            if !use_suggestion {
                let first_op = chain[0];
                let last_op = chain[chain.len() - 1];
                let is_explicit_nullish_check = matches!(
                    first_op.typ,
                    OperandType::NotEqualBoth
                        | OperandType::NotStrictEqualNull
                        | OperandType::NotStrictEqualUndef
                );
                let is_plain_access = last_op.typ == OperandType::Plain;
                if is_explicit_nullish_check && is_plain_access {
                    if let Some(e) = first_op.compared_expr {
                        if !self.get_type_info(ctx, e).is_any_or_unknown() {
                            use_suggestion = true;
                        }
                    }
                }
            }

            if has_complementary_null_check && chain.len() >= 2 {
                let last_op = chain[chain.len() - 1];
                let second_last_op = chain[chain.len() - 2];
                if last_op.typ == OperandType::TypeofCheck {
                    use_suggestion = true;
                }
                if second_last_op.typ == OperandType::NotStrictEqualNull {
                    use_suggestion = true;
                }
            }
        }

        self.report_chain_with_fixes(ctx, node, fixes, use_suggestion);
    }

    fn generate_or_chain_fix_and_report(
        &mut self,
        ctx: &mut Ctx,
        node: P<Node>,
        chain: &[Operand],
        operand_nodes: &[P<Node>],
    ) {
        let mut has_trailing_comparison = false;
        if let Some(last_op) = chain.last() {
            has_trailing_comparison = last_op.typ.is_comparison_or_null_check();
        }

        let mut trailing_plain_operand = String::new();
        let mut chain_for_optional: &[Operand] = chain;
        if chain.len() >= 3 && chain[chain.len() - 1].typ == OperandType::Plain && !self.unsafe_ok()
        {
            let last_op = chain[chain.len() - 1];
            let second_last_op = chain[chain.len() - 2];
            if is_nullish_check_operand(&second_last_op) {
                if let (Some(le), Some(se)) = (last_op.compared_expr, second_last_op.compared_expr)
                {
                    let last_len = self.flatten_len(ctx, le);
                    let second_last_len = self.flatten_len(ctx, se);
                    if last_len > second_last_len {
                        trailing_plain_operand = node_text(ctx, last_op.node);
                        chain_for_optional = &chain[..chain.len() - 1];
                    }
                }
            }
        }

        if chain_for_optional.len() == 1 && !trailing_plain_operand.is_empty() {
            let single_op = chain_for_optional[0];
            if single_op.compared_expr.is_some()
                && self.contains_optional_chain(single_op.compared_expr)
            {
                return;
            }
        }

        if chain.len() == 2 && trailing_plain_operand.is_empty() {
            let first_op = chain[0];
            if first_op.compared_expr.is_some()
                && self.contains_optional_chain(first_op.compared_expr)
            {
                return;
            }
            if self.contains_optional_chain(Some(first_op.node)) {
                return;
            }
        }

        let last_op = chain_for_optional[chain_for_optional.len() - 1];
        let last_property_access =
            if last_op.typ == OperandType::Plain { last_op.node } else { last_op.cexpr() };
        let parts_rc = self.flatten_for_fix(ctx, last_property_access);

        let mut checked_lengths: FxHashSet<usize> = FxHashSet::default();

        let checks_to_consider =
            if chain_for_optional[chain_for_optional.len() - 1].typ == OperandType::Plain {
                &chain_for_optional[..chain_for_optional.len() - 1]
            } else {
                chain_for_optional
            };

        for operand in checks_to_consider {
            if let Some(e) = operand.compared_expr {
                checked_lengths.insert(self.flatten_len(ctx, e));
            }
        }

        let mut call_should_be_optional = false;
        let (parts_len, last_is_call) = {
            let p = parts_rc.borrow();
            (p.len(), p.last().is_some_and(|l| l.is_call))
        };
        if parts_len > 0 && last_is_call {
            let parts_without_call = parts_len - 1;
            for op in &chain_for_optional[..chain_for_optional.len() - 1] {
                if self.flatten_len(ctx, op.node) == parts_without_call {
                    call_should_be_optional = true;
                    break;
                }
            }
        }

        let optional_chain_code = build_optional_chain(
            &parts_rc.borrow(),
            &checked_lengths,
            call_should_be_optional,
            true,
        );
        if optional_chain_code.is_empty() {
            return;
        }

        let last_op_for_fix = chain_for_optional[chain_for_optional.len() - 1];
        let has_trailing_comparison_for_fix = last_op_for_fix.typ.is_comparison_or_null_check();

        let mut new_code;
        if has_trailing_comparison_for_fix {
            if ast::is_binary_expression(last_op_for_fix.node) {
                let bin_expr = last_op_for_fix.node.as_binary_expression();
                let compared_expr_range = ctx.trim(last_op_for_fix.cexpr());
                let left_range = ctx.trim(bin_expr.left);
                let is_yoda = compared_expr_range.0 > left_range.0;
                if is_yoda {
                    let op_text = node_text(ctx, bin_expr.operator_token);
                    let value_text = src(ctx, left_range.0, left_range.1).trim();
                    new_code = format!("{optional_chain_code} {op_text} {value_text}");
                } else {
                    let op_start = bin_expr.operator_token.pos();
                    let right_end = bin_expr.right.get().end();
                    let comparison_text = src(ctx, op_start, right_end);
                    new_code = format!("{optional_chain_code}{comparison_text}");
                }
            } else {
                new_code = optional_chain_code;
            }
        } else if last_op_for_fix.typ == OperandType::Not {
            // De Morgan: A || B || ... || Z == !(!A && !B && ... && !Z)
            new_code = format!("!{optional_chain_code}");
        } else {
            new_code = optional_chain_code;
        }

        if !trailing_plain_operand.is_empty() {
            let last_chain_end = ctx.trim(chain[chain.len() - 2].node).1;
            let trailing_start = ctx.trim(chain[chain.len() - 1].node).0;
            let separator = src(ctx, last_chain_end, trailing_start);
            new_code = format!("{new_code}{separator}{trailing_plain_operand}");
        }

        let (replace_start, replace_end) = if chain.len() == operand_nodes.len() {
            ctx.trim(node)
        } else {
            (ctx.trim(chain[0].node).0, ctx.trim(chain[chain.len() - 1].node).1)
        };

        let fixes = vec![ctx.fix_replace_range(replace_start, replace_end, new_code)];

        let mut use_suggestion = !self.unsafe_ok();

        if use_suggestion && !chain.is_empty() {
            let last_op = chain[chain.len() - 1];
            if matches!(
                last_op.typ,
                OperandType::Not
                    | OperandType::EqualNull
                    | OperandType::NotEqualBoth
                    | OperandType::StrictEqualUndef
                    | OperandType::NotStrictEqualUndef
                    | OperandType::TypeofCheck
            ) {
                use_suggestion = false;
            }
        }

        // For strict null/undefined checks on types that only have one of them,
        // use suggestion because optional chaining checks for BOTH.
        if use_suggestion && has_trailing_comparison {
            let mut strict_check_requires_suggestion = false;
            if !chain.is_empty() && !self.unsafe_ok() {
                let analysis = analyze_nullish_checks(chain, true);
                if analysis.has_incomplete_check() {
                    let mut all_types_match_check = true;
                    let mut has_any_nullable_type = false;
                    for op in chain {
                        let Some(e) = op.compared_expr else { continue };
                        let info = self.get_type_info(ctx, e);
                        if info.has_no_nullable_types() {
                            continue;
                        }
                        has_any_nullable_type = true;
                        if info.is_any_or_unknown() || info.has_both_null_and_undefined() {
                            all_types_match_check = false;
                            break;
                        }
                        if analysis.has_only_null_check() && (!info.has_null || info.has_undefined)
                        {
                            all_types_match_check = false;
                            break;
                        }
                        if analysis.has_only_undefined_check()
                            && (info.has_null || !info.has_undefined)
                        {
                            all_types_match_check = false;
                            break;
                        }
                    }
                    if has_any_nullable_type && all_types_match_check {
                        strict_check_requires_suggestion = true;
                    }
                }
            }
            if !strict_check_requires_suggestion {
                strict_check_requires_suggestion =
                    self.has_shorter_undefined_check_before_strict_null_comparison(ctx, chain);
            }
            if !strict_check_requires_suggestion {
                use_suggestion = false;
            }
        }

        if use_suggestion && !chain.is_empty() {
            for op in chain {
                if let Some(e) = op.compared_expr {
                    let info = self.get_type_info(ctx, e);
                    if info.is_any_or_unknown() || info.has_both_null_and_undefined() {
                        use_suggestion = false;
                        break;
                    }
                }
            }
        }

        self.report_chain_with_fixes(ctx, node, fixes, use_suggestion);
    }

    fn has_shorter_undefined_check_before_strict_null_comparison(
        &mut self,
        ctx: &Ctx,
        chain: &[Operand],
    ) -> bool {
        if chain.len() < 2 {
            return false;
        }
        let last_op = chain[chain.len() - 1];
        let Some(last_expr) = last_op.compared_expr else {
            return false;
        };
        if !is_strict_null_check_for_or_chain(&last_op) {
            return false;
        }
        let last_len = self.flatten_len(ctx, last_expr);
        for op in &chain[..chain.len() - 1] {
            let Some(e) = op.compared_expr else { continue };
            if !is_strict_undefined_check_for_or_chain(op) {
                continue;
            }
            if self.flatten_len(ctx, e) < last_len {
                return true;
            }
        }
        false
    }

    fn handle_empty_object_pattern(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let bin_expr = node.as_binary_expression();
        let right_node = bin_expr.right.get();

        let obj_lit = if ast::is_object_literal_expression(right_node) {
            Some(right_node)
        } else if ast::is_parenthesized_expression(right_node) {
            let inner_expr = right_node.as_parenthesized_expression().expression.get();
            if ast::is_object_literal_expression(inner_expr) { Some(inner_expr) } else { None }
        } else {
            None
        };

        let Some(obj_lit) = obj_lit else { return };
        if !obj_lit.as_object_literal_expression().properties.nodes().is_empty() {
            return;
        }

        if self.opts.require_nullish && !self.includes_explicit_nullish(ctx, bin_expr.left) {
            return;
        }

        let Some(parent) = node.parent() else { return };
        let access_expr = if utils::is_property_or_element_access(parent) {
            Some(parent)
        } else if ast::is_parenthesized_expression(parent) {
            parent.parent().filter(|&gp| utils::is_property_or_element_access(gp))
        } else {
            None
        };

        let Some(access_expr) = access_expr else {
            return;
        };

        let (is_optional, is_computed, prop_node) =
            if ast::is_property_access_expression(access_expr) {
                let parent_prop = access_expr.as_property_access_expression();
                (parent_prop.question_dot_token().is_some(), false, parent_prop.name)
            } else {
                let parent_elem = access_expr.as_element_access_expression();
                (parent_elem.question_dot_token().is_some(), true, parent_elem.argument_expression)
            };

        if is_optional {
            return;
        }

        let left_node = bin_expr.left;
        let mut left_text = node_text(ctx, left_node);

        let needs_parens = ast::is_await_expression(left_node)
            || ast::is_binary_expression(left_node)
            || ast::is_conditional_expression(left_node)
            || ast::is_prefix_unary_expression(left_node)
            || left_node.kind() == Kind::AsExpression
            || ast::is_void_expression(left_node)
            || ast::is_type_of_expression(left_node)
            || left_node.kind() == Kind::PostfixUnaryExpression
            || left_node.kind() == Kind::DeleteExpression;

        if needs_parens {
            left_text = format!("({left_text})");
        }

        let mut property_text = node_text(ctx, prop_node);
        if is_computed {
            property_text = format!("[{property_text}]");
        }

        let new_code = format!("{left_text}?.{property_text}");
        let (access_pos, access_end) = ctx.trim(access_expr);

        let fixes = vec![ctx.fix_replace_range(access_pos, access_end, new_code)];

        // (foo || {}).bar returns {} when foo is falsy, while foo?.bar returns undefined
        if self.unsafe_ok() {
            ctx.report_node_with_fixes(access_expr, build_prefer_optional_chain_message(), |_| {
                fixes
            });
        } else {
            ctx.report_node_with_suggestions(
                access_expr,
                build_prefer_optional_chain_message(),
                |_| vec![RuleSuggestion { message: build_optional_chain_suggest_message(), fixes }],
            );
        }
    }
}
