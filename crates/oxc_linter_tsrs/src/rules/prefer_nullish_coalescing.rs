// Port of internal/rules/prefer_nullish_coalescing (prefer_nullish_coalescing.go, options.go).

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleFix, RuleMessage, RuleSuggestion, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

fn build_no_strict_null_check_message() -> RuleMessage {
    RuleMessage::new(
        "noStrictNullCheck",
        "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    )
}

fn build_prefer_nullish_over_or_message(description: &str, equals: &str) -> RuleMessage {
    RuleMessage::new(
        "preferNullishOverOr",
        format!(
            "Prefer using nullish coalescing operator (`??{equals}`) instead of a logical {description} (`||{equals}`), as it is a safer operator."
        ),
    )
}

fn build_prefer_nullish_over_ternary_message() -> RuleMessage {
    RuleMessage::new(
        "preferNullishOverTernary",
        "Prefer using nullish coalescing operator (`??`) instead of a ternary expression, as it is simpler to read.",
    )
}

fn build_prefer_nullish_over_assignment_message() -> RuleMessage {
    RuleMessage::new(
        "preferNullishOverAssignment",
        "Prefer using nullish coalescing operator (`??=`) instead of an assignment expression, as it is simpler to read.",
    )
}

fn build_suggest_nullish_coalescing_message() -> RuleMessage {
    RuleMessage::new("suggestNullishCoalescing", "Change to the nullish coalescing operator.")
}

/// The operator used in nullish checks (Go NullishCheckOperator; None is "").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NullishCheckOperator {
    Empty,
    Not,
    Equal,
    StrictEqual,
    NotEqual,
    NotStrictEq,
}

fn is_logical_or_operator(node: P<Node>) -> bool {
    if !ast::is_binary_expression(node) {
        return false;
    }
    let op = node.as_binary_expression().operator_token.kind();
    op == Kind::BarBarToken || op == Kind::BarBarEqualsToken
}

fn is_member_access_like(node: P<Node>) -> bool {
    // Skip parentheses to handle deeply nested patterns like ((((foo.a))))
    let node = ast::skip_parentheses(node);
    match node.kind() {
        Kind::Identifier | Kind::PropertyAccessExpression | Kind::ElementAccessExpression => true,
        _ => ast::is_optional_chain(node),
    }
}

fn is_node_equal(a: Option<P<Node>>, b: Option<P<Node>>) -> bool {
    let (Some(a), Some(b)) = (a, b) else {
        return a.is_none() && b.is_none();
    };
    if a.kind() != b.kind() {
        return false;
    }
    match a.kind() {
        Kind::Identifier => a.text() == b.text(),
        Kind::PropertyAccessExpression => {
            let a_prop = a.as_property_access_expression();
            let b_prop = b.as_property_access_expression();
            is_node_equal(Some(a_prop.name()), Some(b_prop.name()))
                && is_node_equal(Some(a_prop.expression), Some(b_prop.expression))
        }
        Kind::ElementAccessExpression => {
            let a_elem = a.as_element_access_expression();
            let b_elem = b.as_element_access_expression();
            is_node_equal(Some(a_elem.argument_expression), Some(b_elem.argument_expression))
                && is_node_equal(Some(a_elem.expression), Some(b_elem.expression))
        }
        Kind::NullKeyword => true,
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => a.text() == b.text(),
        Kind::NumericLiteral | Kind::BigIntLiteral => a.text() == b.text(),
        Kind::ThisKeyword => true,
        _ => false,
    }
}

/// The property name of a PropertyAccessExpression or ElementAccessExpression; empty if it cannot be
/// determined statically.
fn get_property_name_from_access(node: P<Node>) -> &'static str {
    if ast::is_property_access_expression(node) {
        return node.as_property_access_expression().name().text();
    }
    if ast::is_element_access_expression(node) {
        let arg = node.as_element_access_expression().argument_expression;
        // Handle string literal like x['a'] and no substitution template literal like x[`a`]
        if arg.kind() == Kind::StringLiteral || arg.kind() == Kind::NoSubstitutionTemplateLiteral {
            return arg.text();
        }
        return "";
    }
    ""
}

fn get_object_expression_from_access(node: P<Node>) -> P<Node> {
    if ast::is_property_access_expression(node) {
        return node.as_property_access_expression().expression;
    }
    node.as_element_access_expression().expression
}

/// Whether two nodes have the same member access sequence.
fn are_nodes_similar_member_access(mut a: P<Node>, mut b: P<Node>) -> bool {
    // Unwrap parenthesized expressions
    while a.kind() == Kind::ParenthesizedExpression {
        a = a.expression().unwrap();
    }
    while b.kind() == Kind::ParenthesizedExpression {
        b = b.expression().unwrap();
    }
    // Unwrap non-null expressions (like x!)
    if a.kind() == Kind::NonNullExpression {
        a = a.expression().unwrap();
    }
    if b.kind() == Kind::NonNullExpression {
        b = b.expression().unwrap();
    }

    let a_is_prop_access =
        ast::is_property_access_expression(a) || ast::is_element_access_expression(a);
    let b_is_prop_access =
        ast::is_property_access_expression(b) || ast::is_element_access_expression(b);

    if a_is_prop_access && b_is_prop_access {
        let a_name = get_property_name_from_access(a);
        let b_name = get_property_name_from_access(b);

        // If either name couldn't be determined statically, fall back to strict comparison
        if a_name.is_empty() || b_name.is_empty() {
            // Both must be the same kind for non-static comparison
            if a.kind() != b.kind() {
                return false;
            }
            if ast::is_element_access_expression(a) && ast::is_element_access_expression(b) {
                let a_elem = a.as_element_access_expression();
                let b_elem = b.as_element_access_expression();
                if !are_nodes_similar_member_access(a_elem.expression, b_elem.expression) {
                    return false;
                }
                return is_node_equal(
                    Some(a_elem.argument_expression),
                    Some(b_elem.argument_expression),
                );
            }
        }

        if a_name != b_name {
            return false;
        }

        return are_nodes_similar_member_access(
            get_object_expression_from_access(a),
            get_object_expression_from_access(b),
        );
    }

    is_node_equal(Some(a), Some(b))
}

fn is_conditional_test(node: P<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };

    if ast::is_logical_expression(parent) {
        return is_conditional_test(parent);
    }

    // Handle parenthesized expressions - traverse up through them
    if parent.kind() == Kind::ParenthesizedExpression {
        return is_conditional_test(parent);
    }

    if ast::is_conditional_expression(parent) {
        let cond = parent.as_conditional_expression();
        if cond.when_true == node || cond.when_false == node {
            return is_conditional_test(parent);
        }
    }

    if parent.kind() == Kind::BinaryExpression
        && parent.as_binary_expression().operator_token.kind() == Kind::CommaToken
        && parent.as_binary_expression().right.get() == node
    {
        return is_conditional_test(parent);
    }

    if ast::is_prefix_unary_expression(parent)
        && parent.as_prefix_unary_expression().operator == Kind::ExclamationToken
    {
        return is_conditional_test(parent);
    }

    match parent.kind() {
        Kind::ConditionalExpression => parent.as_conditional_expression().condition == node,
        Kind::DoStatement => parent.as_do_statement().expression == node,
        Kind::IfStatement => parent.as_if_statement().expression == node,
        Kind::ForStatement => parent.as_for_statement().condition == Some(node),
        Kind::WhileStatement => parent.as_while_statement().expression == node,
        _ => false,
    }
}

fn is_boolean_constructor_context(ctx: &mut Ctx, node: P<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };

    // Handle parenthesized expressions - traverse up through them
    if parent.kind() == Kind::ParenthesizedExpression {
        return is_boolean_constructor_context(ctx, parent);
    }

    if ast::is_logical_expression(parent) {
        return is_boolean_constructor_context(ctx, parent);
    }

    if ast::is_conditional_expression(parent) {
        let cond = parent.as_conditional_expression();
        if cond.when_true == node || cond.when_false == node {
            return is_boolean_constructor_context(ctx, parent);
        }
    }

    if parent.kind() == Kind::BinaryExpression
        && parent.as_binary_expression().operator_token.kind() == Kind::CommaToken
        && parent.as_binary_expression().right.get() == node
    {
        return is_boolean_constructor_context(ctx, parent);
    }

    // Check if it's a call to Boolean()
    if ast::is_call_expression(parent) {
        let callee = parent.expression().unwrap();
        if ast::is_identifier(callee) && callee.text() == "Boolean" {
            // The Boolean being called - check if it's the global Boolean
            if let Some(symbol) = ctx.checker.get_symbol_at_location_exported(callee) {
                // Check if this is the global Boolean (no user-defined declarations in source files)
                for &decl in symbol.declarations() {
                    if let Some(sf) = ast::get_source_file_of_node(decl) {
                        if !sf.is_declaration_file() {
                            return false; // User-defined Boolean
                        }
                    }
                }
            }
            return true;
        }
    }

    false
}

pub struct PreferNullishCoalescing {
    ignore_boolean_coercion: bool,
    ignore_conditional_tests: bool,
    ignore_if_statements: bool,
    ignore_mixed_logical_expressions: bool,
    ignore_ternary_tests: bool,
    /// Pre-computed from ignorePrimitives.
    ignorable_flags: TypeFlags,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    // ignorePrimitives is a utils.BoolOr[IgnorePrimitivesOptions].
    let mut ignorable_flags = TypeFlags::empty();
    match m.get("ignorePrimitives") {
        Some(serde_json::Value::Bool(true)) => {
            // If true, ignore all primitive types
            ignorable_flags = TypeFlags::BigIntLike
                | TypeFlags::BooleanLike
                | TypeFlags::NumberLike
                | TypeFlags::StringLike;
        }
        Some(serde_json::Value::Object(p)) => {
            if opt_bool(p, "bigint", false) {
                ignorable_flags |= TypeFlags::BigIntLike;
            }
            if opt_bool(p, "boolean", false) {
                ignorable_flags |= TypeFlags::BooleanLike;
            }
            if opt_bool(p, "number", false) {
                ignorable_flags |= TypeFlags::NumberLike;
            }
            if opt_bool(p, "string", false) {
                ignorable_flags |= TypeFlags::StringLike;
            }
        }
        _ => {}
    }
    Ok(Box::new(PreferNullishCoalescing {
        ignore_boolean_coercion: opt_bool(&m, "ignoreBooleanCoercion", false),
        ignore_conditional_tests: opt_bool(&m, "ignoreConditionalTests", true),
        ignore_if_statements: opt_bool(&m, "ignoreIfStatements", false),
        ignore_mixed_logical_expressions: opt_bool(&m, "ignoreMixedLogicalExpressions", false),
        ignore_ternary_tests: opt_bool(&m, "ignoreTernaryTests", false),
        ignorable_flags,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::ConditionalExpression),
    Listener::Enter(Kind::IfStatement),
];

impl Rule for PreferNullishCoalescing {
    fn name(&self) -> &'static str {
        "prefer-nullish-coalescing"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let options = ctx.program.options();
        if !utils::is_strict_compiler_option_enabled(&options, options.strict_null_checks) {
            ctx.report_range(0, 0, build_no_strict_null_check_message());
        }
        Box::new(Visitor { opts: self })
    }
}

struct Visitor {
    opts: &'static PreferNullishCoalescing,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::BinaryExpression) => {
                let op = node.as_binary_expression().operator_token.kind();
                if op == Kind::BarBarToken {
                    self.check_and_fix_with_prefer_nullish_over_or(ctx, node, "or", "");
                } else if op == Kind::BarBarEqualsToken {
                    self.check_and_fix_with_prefer_nullish_over_or(ctx, node, "assignment", "=");
                }
            }
            Listener::Enter(Kind::ConditionalExpression) => {
                self.check_conditional_expression(ctx, node)
            }
            Listener::Enter(Kind::IfStatement) => self.check_if_statement(ctx, node),
            _ => {}
        }
    }
}

/// Whether a type includes null or undefined; also true for any/unknown since they can include
/// null/undefined.
fn is_nullable_type(t: P<Type>) -> bool {
    if utils::is_type_flag_set(t, TypeFlags::Any | TypeFlags::Unknown) {
        return true;
    }
    utils::union_type_parts(t).into_iter().any(|part| part.flags().intersects(TypeFlags::Nullable))
}

/// Go getOperatorAndNodesInsideTestExpression; None stands for the ("", nil) result.
fn get_operator_and_nodes_inside_test_expression(
    node: P<Node>,
) -> Option<(NullishCheckOperator, Vec<P<Node>>)> {
    let test = if ast::is_conditional_expression(node) {
        node.as_conditional_expression().condition
    } else if ast::is_if_statement(node) {
        node.as_if_statement().expression
    } else {
        return None;
    };
    let test = ast::skip_parentheses(test);

    // Check for simple truthiness or negation check
    if is_member_access_like(test) {
        return Some((NullishCheckOperator::Empty, Vec::new()));
    }
    if ast::is_prefix_unary_expression(test)
        && test.as_prefix_unary_expression().operator == Kind::ExclamationToken
    {
        let arg = ast::skip_parentheses(test.as_prefix_unary_expression().operand);
        if is_member_access_like(arg) {
            return Some((NullishCheckOperator::Not, Vec::new()));
        }
        return None;
    }

    // Check for binary comparison
    if ast::is_binary_expression(test) {
        let bin = test.as_binary_expression();
        let left = ast::skip_parentheses(bin.left);
        let right = ast::skip_parentheses(bin.right.get());
        match bin.operator_token.kind() {
            Kind::EqualsEqualsToken => {
                return Some((NullishCheckOperator::Equal, vec![left, right]));
            }
            Kind::ExclamationEqualsToken => {
                return Some((NullishCheckOperator::NotEqual, vec![left, right]));
            }
            Kind::EqualsEqualsEqualsToken => {
                return Some((NullishCheckOperator::StrictEqual, vec![left, right]));
            }
            Kind::ExclamationEqualsEqualsToken => {
                return Some((NullishCheckOperator::NotStrictEq, vec![left, right]));
            }
            Kind::BarBarToken | Kind::AmpersandAmpersandToken => {
                // Compound check like (a === null || a === undefined)
                let bin_left = bin.left;
                let bin_right = bin.right.get();
                if ast::is_binary_expression(bin_left) && ast::is_binary_expression(bin_right) {
                    let left_bin = bin_left.as_binary_expression();
                    let right_bin = bin_right.as_binary_expression();

                    // Check if one side is a simple nullish comparison (null === null or undefined === undefined)
                    let left_is_nullish_comparison = utils::is_null_literal_or_undefined_identifier(
                        Some(ast::skip_parentheses(left_bin.left)),
                    )
                        && utils::is_null_literal_or_undefined_identifier(Some(
                            ast::skip_parentheses(left_bin.right.get()),
                        ));
                    let right_is_nullish_comparison = utils::is_null_literal_or_undefined_identifier(
                        Some(ast::skip_parentheses(right_bin.left)),
                    )
                        && utils::is_null_literal_or_undefined_identifier(Some(
                            ast::skip_parentheses(right_bin.right.get()),
                        ));
                    if left_is_nullish_comparison || right_is_nullish_comparison {
                        return None;
                    }

                    let nodes = vec![
                        ast::skip_parentheses(left_bin.left),
                        ast::skip_parentheses(left_bin.right.get()),
                        ast::skip_parentheses(right_bin.left),
                        ast::skip_parentheses(right_bin.right.get()),
                    ];
                    let lop = left_bin.operator_token.kind();
                    let rop = right_bin.operator_token.kind();

                    if bin.operator_token.kind() == Kind::BarBarToken {
                        if lop == Kind::EqualsEqualsEqualsToken
                            && rop == Kind::EqualsEqualsEqualsToken
                        {
                            return Some((NullishCheckOperator::StrictEqual, nodes));
                        }
                        if (lop == Kind::EqualsEqualsEqualsToken || lop == Kind::EqualsEqualsToken)
                            && (rop == Kind::EqualsEqualsEqualsToken
                                || rop == Kind::EqualsEqualsToken)
                        {
                            return Some((NullishCheckOperator::Equal, nodes));
                        }
                    } else {
                        if lop == Kind::ExclamationEqualsEqualsToken
                            && rop == Kind::ExclamationEqualsEqualsToken
                        {
                            return Some((NullishCheckOperator::NotStrictEq, nodes));
                        }
                        if (lop == Kind::ExclamationEqualsEqualsToken
                            || lop == Kind::ExclamationEqualsToken)
                            && (rop == Kind::ExclamationEqualsEqualsToken
                                || rop == Kind::ExclamationEqualsToken)
                        {
                            return Some((NullishCheckOperator::NotEqual, nodes));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    None
}

/// The (non-nullish, nullish) branch nodes of a conditional expression.
fn get_branch_nodes(node: P<Node>, operator: NullishCheckOperator) -> (P<Node>, P<Node>) {
    let cond = node.as_conditional_expression();
    if matches!(
        operator,
        NullishCheckOperator::Empty
            | NullishCheckOperator::NotEqual
            | NullishCheckOperator::NotStrictEq
    ) {
        return (cond.when_true, cond.when_false);
    }
    (cond.when_false, cond.when_true)
}

/// Whether node is part of a mixed logical expression (with &&).
fn is_mixed_logical_expression(node: P<Node>) -> bool {
    let mut seen: FxHashSet<P<Node>> = FxHashSet::default();
    let bin = node.as_binary_expression();
    let mut queue: std::collections::VecDeque<Option<P<Node>>> =
        [node.parent(), Some(bin.left), Some(bin.right.get())].into();

    while let Some(current) = queue.pop_front() {
        let Some(current) = current else { continue };
        if !seen.insert(current) {
            continue;
        }
        if ast::is_logical_expression(current) {
            // Skip parentheses to get to the actual binary expression
            let unwrapped = ast::skip_parentheses(current);
            if !ast::is_binary_expression(unwrapped) {
                continue;
            }
            let b = unwrapped.as_binary_expression();
            let op = b.operator_token.kind();
            if op == Kind::AmpersandAmpersandToken {
                return true;
            }
            if op == Kind::BarBarToken
                || (op == Kind::EqualsToken && ast::is_logical_expression(b.left))
            {
                queue.push_back(current.parent());
                queue.push_back(Some(b.left));
                queue.push_back(Some(b.right.get()));
            }
        }
    }
    false
}

fn node_text(ctx: &Ctx, node: P<Node>) -> &'static str {
    &ctx.text()[node.pos() as usize..node.end() as usize]
}

impl Visitor {
    /// Whether a type tested for truthiness is eligible for conversion to a nullishness check, taking
    /// into account the rule's configuration.
    fn is_type_eligible_for_prefer_nullish(&self, t: P<Type>) -> bool {
        if !is_nullable_type(t) {
            return false;
        }
        let ignorable_flags = self.opts.ignorable_flags;
        if ignorable_flags.is_empty() {
            // Any types are eligible for conversion
            return true;
        }
        // If the type is any or unknown we can't make any assumptions
        if utils::is_type_flag_set(t, TypeFlags::Any | TypeFlags::Unknown) {
            return false;
        }
        // Check if any type constituents match the ignorable flags
        for part in utils::union_type_parts(t) {
            for intersection_part in utils::intersection_type_parts(part) {
                if utils::is_type_flag_set(intersection_part, ignorable_flags) {
                    return false;
                }
            }
        }
        true
    }

    /// Whether a control flow construct that uses the truthiness of a test expression is eligible for
    /// conversion.
    fn is_truthiness_check_eligible_for_prefer_nullish(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        test_node: P<Node>,
    ) -> bool {
        let test_type = ctx.checker.get_type_at_location(test_node);
        if !self.is_type_eligible_for_prefer_nullish(test_type) {
            return false;
        }
        if self.opts.ignore_conditional_tests && is_conditional_test(node) {
            return false;
        }
        if self.opts.ignore_boolean_coercion && is_boolean_constructor_context(ctx, node) {
            // For conditional expressions inside Boolean calls, still check
            if !(ast::is_conditional_expression(node)
                && node.parent().is_some_and(ast::is_call_expression))
            {
                return false;
            }
        }
        true
    }

    /// Whether a nullish check with only null or only undefined should be skipped based on the type of
    /// the expression. allow_not_equal_operator controls whether != is treated like == (ternary vs
    /// if-statement handling).
    fn should_skip_due_to_partial_nullish_check(
        &self,
        ctx: &mut Ctx,
        has_null_check: bool,
        has_undefined_check: bool,
        operator: NullishCheckOperator,
        target_node: P<Node>,
        allow_not_equal_operator: bool,
    ) -> bool {
        // If we check for both null and undefined, it's always fixable
        if has_null_check && has_undefined_check {
            return false;
        }
        // If we check for neither null nor undefined, skip
        if !has_null_check && !has_undefined_check {
            return true;
        }
        // Only one check: for == and != operators, loose equality handles both null and undefined
        if operator == NullishCheckOperator::Equal {
            return false;
        }
        if allow_not_equal_operator && operator == NullishCheckOperator::NotEqual {
            return false;
        }

        let t = ctx.checker.get_type_at_location(target_node);
        // Skip if the type is any or unknown
        if t.flags().intersects(TypeFlags::Any | TypeFlags::Unknown) {
            return true;
        }

        let mut has_null_type = false;
        let mut has_undefined_type = false;
        for part in utils::union_type_parts(t) {
            if part.flags().intersects(TypeFlags::Null) {
                has_null_type = true;
            }
            if part.flags().intersects(TypeFlags::Undefined) {
                has_undefined_type = true;
            }
        }
        // If checking for undefined but type includes null, skip (incomplete nullish check)
        if has_undefined_check && has_null_type {
            return true;
        }
        // If checking for null but type includes undefined, skip (incomplete nullish check)
        if has_null_check && has_undefined_type {
            return true;
        }
        false
    }

    /// Handles || and ||= operators.
    fn check_and_fix_with_prefer_nullish_over_or(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        description: &str,
        equals: &str,
    ) {
        let bin = node.as_binary_expression();
        if !self.is_truthiness_check_eligible_for_prefer_nullish(ctx, node, bin.left) {
            return;
        }
        if self.opts.ignore_mixed_logical_expressions && is_mixed_logical_expression(node) {
            return;
        }

        ctx.report_node_with_suggestions(
            bin.operator_token,
            build_prefer_nullish_over_or_message(description, equals),
            |ctx| {
                let mut fixes: Vec<RuleFix> = Vec::new();
                let mut left_operand_starts_inside_left_expression = false;
                let left = bin.left;
                let right = bin.right.get();

                // If parent is a logical or expression (skipping parentheses), wrap with
                // parentheses, but don't add parentheses if already wrapped in parentheses.
                let mut parent_node = node.parent();
                while let Some(p) = parent_node {
                    if !ast::is_parenthesized_expression(p) {
                        break;
                    }
                    parent_node = p.parent();
                }
                if parent_node.is_some_and(ast::is_logical_expression)
                    && !node.parent().is_some_and(ast::is_parenthesized_expression)
                {
                    // Only apply special logical expression handling when left is directly a
                    // binary expression. If it's wrapped in parentheses, the parentheses already
                    // provide visual grouping, so we insert before left instead
                    // (oxc-project/tsgolint#604).
                    if ast::is_binary_expression(left)
                        && ast::is_logical_expression(left)
                        && !is_logical_or_operator(left.as_binary_expression().left)
                    {
                        fixes.push(
                            ctx.fix_insert_before(left.as_binary_expression().right.get(), "("),
                        );
                        left_operand_starts_inside_left_expression = true;
                    } else {
                        fixes.push(ctx.fix_insert_before(left, "("));
                    }
                    fixes.push(ctx.fix_insert_after(right, ")"));
                }

                let is_or = bin.operator_token.kind() == Kind::BarBarToken;
                if is_or {
                    if !left_operand_starts_inside_left_expression
                        && ast::is_logical_expression(left)
                        && !ast::is_parenthesized_expression(left)
                    {
                        fixes.push(ctx.fix_insert_before(left, "("));
                        fixes.push(ctx.fix_insert_after(left, ")"));
                    }
                    if ast::is_logical_expression(right) && !ast::is_parenthesized_expression(right)
                    {
                        fixes.push(ctx.fix_insert_before(right, "("));
                        fixes.push(ctx.fix_insert_after(right, ")"));
                    }
                }

                // Replace || with ?? or ||= with ??=
                let new_operator = if is_or { "??" } else { "??=" };
                fixes.push(ctx.fix_replace(bin.operator_token, new_operator));

                vec![RuleSuggestion { message: build_suggest_nullish_coalescing_message(), fixes }]
            },
        );
    }

    fn check_conditional_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        if self.opts.ignore_ternary_tests {
            return;
        }
        let Some((operator, nodes_inside_test)) =
            get_operator_and_nodes_inside_test_expression(node)
        else {
            return;
        };

        let (non_nullish_branch, nullish_branch) = get_branch_nodes(node, operator);
        // Skip parentheses for comparison
        let non_nullish_branch_unwrapped = ast::skip_parentheses(non_nullish_branch);

        let mut nullish_coalescing_left_node: Option<P<Node>> = None;
        let mut has_truthiness_check = false;

        if nodes_inside_test.is_empty() {
            // Simple truthiness check
            has_truthiness_check = true;
            let test_node = ast::skip_parentheses(node.as_conditional_expression().condition);
            let left = if ast::is_prefix_unary_expression(test_node)
                && test_node.as_prefix_unary_expression().operator == Kind::ExclamationToken
            {
                ast::skip_parentheses(test_node.as_prefix_unary_expression().operand)
            } else {
                test_node
            };
            nullish_coalescing_left_node = Some(left);
            if !are_nodes_similar_member_access(left, non_nullish_branch_unwrapped) {
                return;
            }
        } else {
            // Check that the test only contains null, undefined and the identifier
            let mut has_null_check = false;
            let mut has_undefined_check = false;
            for &test_node in &nodes_inside_test {
                if utils::is_null_literal(Some(test_node)) {
                    has_null_check = true;
                } else if utils::is_undefined_identifier(Some(test_node)) {
                    has_undefined_check = true;
                } else if are_nodes_similar_member_access(test_node, non_nullish_branch_unwrapped) {
                    if nullish_coalescing_left_node.is_none() {
                        nullish_coalescing_left_node = Some(test_node);
                    }
                } else {
                    return;
                }
            }
            let Some(left) = nullish_coalescing_left_node else {
                return;
            };
            // Check if fixable
            if self.should_skip_due_to_partial_nullish_check(
                ctx,
                has_null_check,
                has_undefined_check,
                operator,
                left,
                true,
            ) {
                return;
            }
        }
        let left_node = nullish_coalescing_left_node.unwrap();

        if has_truthiness_check
            && !self.is_truthiness_check_eligible_for_prefer_nullish(ctx, node, left_node)
        {
            return;
        }

        ctx.report_node_with_suggestions(
            node,
            build_prefer_nullish_over_ternary_message(),
            |ctx| {
                let left_text = node_text(ctx, left_node);
                // Trim whitespace before processing
                let mut right_text = node_text(ctx, nullish_branch).trim().to_string();
                // Wrap right side if needed
                if !utils::is_strong_precedence_node(nullish_branch)
                    && !ast::is_parenthesized_expression(nullish_branch)
                {
                    right_text = format!("({right_text})");
                }
                let new_text = format!("{} ?? {}", left_text.trim(), right_text);
                vec![RuleSuggestion {
                    message: build_suggest_nullish_coalescing_message(),
                    fixes: vec![ctx.fix_replace(node, new_text)],
                }]
            },
        );
    }

    fn check_if_statement(&self, ctx: &mut Ctx, node: P<Node>) {
        if self.opts.ignore_if_statements {
            return;
        }
        let if_stmt = node.as_if_statement();
        if if_stmt.else_statement.is_some() {
            return;
        }

        // Get the assignment expression from the consequent
        let then_statement = if_stmt.then_statement;
        let mut assignment_expr = None;
        if ast::is_block(then_statement) {
            let statements = then_statement.statements();
            if statements.len() == 1 && ast::is_expression_statement(statements[0]) {
                assignment_expr = statements[0].expression();
            }
        } else if ast::is_expression_statement(then_statement) {
            assignment_expr = then_statement.expression();
        }
        let Some(assignment_expr) = assignment_expr else {
            return;
        };

        // Skip parentheses around the assignment expression to handle cases like
        // ((((foo.a)))) = value;
        let assignment_expr_unwrapped = ast::skip_parentheses(assignment_expr);
        if !ast::is_assignment_expression(assignment_expr_unwrapped, false) {
            return;
        }
        let bin = assignment_expr_unwrapped.as_binary_expression();
        if !is_member_access_like(bin.left) {
            return;
        }
        let nullish_coalescing_left_node = bin.left;
        let nullish_coalescing_right_node = bin.right.get();

        let Some((operator, nodes_inside_test)) =
            get_operator_and_nodes_inside_test_expression(node)
        else {
            return;
        };
        // Only handle negation or equality checks
        if !matches!(
            operator,
            NullishCheckOperator::Not
                | NullishCheckOperator::Equal
                | NullishCheckOperator::StrictEqual
        ) {
            return;
        }

        // Verify the test is checking the same variable being assigned
        if nodes_inside_test.is_empty() {
            // Simple negation check
            let mut test_node = ast::skip_parentheses(if_stmt.expression);
            if ast::is_prefix_unary_expression(test_node) {
                test_node = ast::skip_parentheses(test_node.as_prefix_unary_expression().operand);
            }
            if !are_nodes_similar_member_access(test_node, nullish_coalescing_left_node) {
                return;
            }
            if !self.is_truthiness_check_eligible_for_prefer_nullish(ctx, node, test_node) {
                return;
            }
        } else {
            // Check null/undefined comparison
            let mut has_null_check = false;
            let mut has_undefined_check = false;
            let mut found_matching_node = false;
            for &test_node in &nodes_inside_test {
                if utils::is_null_literal(Some(test_node)) {
                    has_null_check = true;
                } else if utils::is_undefined_identifier(Some(test_node)) {
                    has_undefined_check = true;
                } else if are_nodes_similar_member_access(test_node, nullish_coalescing_left_node) {
                    found_matching_node = true;
                } else {
                    return;
                }
            }
            if !found_matching_node {
                return;
            }
            // Check if fixable
            if self.should_skip_due_to_partial_nullish_check(
                ctx,
                has_null_check,
                has_undefined_check,
                operator,
                nullish_coalescing_left_node,
                false,
            ) {
                return;
            }
        }

        ctx.report_node_with_suggestions(
            node,
            build_prefer_nullish_over_assignment_message(),
            |ctx| {
                // Strip all outer parentheses from the left node to get the inner expression
                let left_node_unwrapped = ast::skip_parentheses(nullish_coalescing_left_node);
                let left_text = node_text(ctx, left_node_unwrapped);
                let right_text = node_text(ctx, nullish_coalescing_right_node);
                let mut left_text_trimmed = left_text.trim().to_string();
                // Only wrap in parentheses if the original expression was parenthesized
                if ast::is_parenthesized_expression(nullish_coalescing_left_node) {
                    left_text_trimmed = format!("({left_text_trimmed})");
                }
                let new_text = format!("{} ??= {};", left_text_trimmed, right_text.trim());
                vec![RuleSuggestion {
                    message: build_suggest_nullish_coalescing_message(),
                    fixes: vec![ctx.fix_replace(node, new_text)],
                }]
            },
        );
    }
}
