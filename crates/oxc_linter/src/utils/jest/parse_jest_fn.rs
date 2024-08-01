use std::{borrow::Cow, cmp::Ordering};

use oxc_ast::{
    ast::{
        match_member_expression, Argument, CallExpression, Expression, IdentifierName,
        IdentifierReference, MemberExpression,
    },
    AstKind,
};
use oxc_semantic::AstNode;
use oxc_span::Span;
use phf::phf_set;

use crate::{
    context::LintContext,
    utils::jest::{is_pure_string, JestFnKind, JestGeneralFnKind, PossibleJestNode},
};

pub fn parse_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) -> Option<ParsedJestFnCall<'a>> {
    let original = possible_jest_node.original;
    let node = possible_jest_node.node;
    let callee = &call_expr.callee;
    // If bailed out, we're not jest function
    let resolved = resolve_to_jest_fn(call_expr, original)?;

    let params = NodeChainParams {
        expr: callee,
        parent: None, // TODO: not really know how to convert type of call_expr to Expression, set to `None` temporarily.
        parent_kind: Some(KnownMemberExpressionParentKind::Call),
        grandparent_kind: None,
    };
    let chain = get_node_chain(&params);
    let all_member_expr_except_last =
        chain.iter().rev().skip(1).all(|member| {
            matches!(member.parent_kind, Some(KnownMemberExpressionParentKind::Member))
        });

    if let Some(last) = chain.last() {
        // If we're an `each()`, ensure we're the outer CallExpression (i.e `.each()()`)
        if last.is_name_equal("each")
            && !matches!(
                callee,
                Expression::CallExpression(_) | Expression::TaggedTemplateExpression(_)
            )
        {
            return None;
        }

        if matches!(callee, Expression::TaggedTemplateExpression(_)) && last.is_name_unequal("each")
        {
            return None;
        }

        let name = resolved.original.unwrap_or(resolved.local);
        let kind = JestFnKind::from(name);
        let mut members = Vec::new();
        let mut iter = chain.into_iter();
        let head = iter.next()?;
        let rest = iter;

        // every member node must have a member expression as their parent
        // in order to be part of the call chain we're parsing
        for member in rest {
            members.push(member);
        }

        if matches!(kind, JestFnKind::Expect) {
            let options = ExpectFnCallOptions {
                call_expr,
                members,
                name,
                local: resolved.local,
                head,
                node,
                ctx,
            };
            return parse_jest_expect_fn_call(options);
        }

        // Ensure that we're at the "top" of the function call chain otherwise when
        // parsing e.g. x().y.z(), we'll incorrectly find & parse "x()" even though
        // the full chain is not a valid jest function call chain
        if ctx.nodes().parent_node(node.id()).is_some_and(|parent_node| {
            matches!(parent_node.kind(), AstKind::CallExpression(_) | AstKind::MemberExpression(_))
        }) {
            return None;
        }

        if matches!(kind, JestFnKind::General(JestGeneralFnKind::Jest)) {
            return parse_jest_jest_fn_call(members, name, resolved.local);
        }

        // Check every link in the chain except the last is a member expression
        if !all_member_expr_except_last {
            return None;
        }

        let mut call_chains = Vec::from([Cow::Borrowed(name)]);
        call_chains.extend(members.iter().filter_map(KnownMemberExpressionProperty::name));

        if !is_valid_jest_call(&call_chains) && !is_valid_vitest_call(&call_chains) {
            return None;
        }

        return Some(ParsedJestFnCall::GeneralJestFnCall(ParsedGeneralJestFnCall {
            kind,
            members,
            name: Cow::Borrowed(name),
            local: Cow::Borrowed(resolved.local),
        }));
    }

    None
}

fn parse_jest_expect_fn_call<'a>(
    options: ExpectFnCallOptions<'a, '_>,
) -> Option<ParsedJestFnCall<'a>> {
    let ExpectFnCallOptions { call_expr, members, name, local, head, node, ctx } = options;
    let (modifiers, matcher, mut expect_error) = match find_modifiers_and_matcher(&members) {
        Ok((modifier, matcher)) => (modifier, matcher, None),
        Err(e) => (vec![], None, Some(e)),
    };

    // if the `expect` call chain is not valid, only report on the topmost node
    // since all members in the chain are likely to get flagged for some reason
    if expect_error.is_some() && !is_top_most_call_expr(node, ctx) {
        return None;
    }

    if matches!(expect_error, Some(ExpectError::MatcherNotFound)) {
        let parent = ctx.nodes().parent_node(node.id())?;
        if matches!(parent.kind(), AstKind::MemberExpression(_)) {
            expect_error = Some(ExpectError::MatcherNotCalled);
        }
    }

    return Some(ParsedJestFnCall::ExpectFnCall(ParsedExpectFnCall {
        kind: JestFnKind::Expect,
        head,
        members,
        name: Cow::Borrowed(name),
        local: Cow::Borrowed(local),
        args: &call_expr.arguments,
        matcher_index: matcher,
        modifier_indices: modifiers,
        expect_error,
    }));
}

type ModifiersAndMatcherIndex = (Vec<usize>, Option<usize>);

#[derive(PartialEq, Eq)]
pub enum ModifierName {
    Not,
    Rejects,
    Resolves,
}

impl ModifierName {
    pub fn from(name: &str) -> Option<Self> {
        match name {
            "not" => Some(Self::Not),
            "rejects" => Some(Self::Rejects),
            "resolves" => Some(Self::Resolves),
            _ => None,
        }
    }
}

fn find_modifiers_and_matcher(
    members: &[KnownMemberExpressionProperty],
) -> Result<ModifiersAndMatcherIndex, ExpectError> {
    let mut modifiers: Vec<usize> = vec![];

    for (index, member) in members.iter().enumerate() {
        // check if the member is being called, which means it is the matcher
        // (and also the end of the entire "expect" call chain)
        if matches!(member.parent_kind, Some(KnownMemberExpressionParentKind::Member))
            && matches!(member.grandparent_kind, Some(KnownMemberExpressionParentKind::Call))
        {
            let matcher = Some(index);
            return Ok((modifiers, matcher));
        }

        // the first modifier can be any of the three modifiers
        if modifiers.is_empty() {
            if !member.is_name_in_modifiers(&[
                ModifierName::Not,
                ModifierName::Resolves,
                ModifierName::Rejects,
            ]) {
                return Err(ExpectError::ModifierUnknown);
            }
        } else if modifiers.len() == 1 {
            // the second modifier can only be "not"
            if !member.is_name_in_modifiers(&[ModifierName::Not]) {
                return Err(ExpectError::ModifierUnknown);
            }
            // and the first modifier has to be either "resolves" or "rejects"
            if !members[modifiers[0]]
                .is_name_in_modifiers(&[ModifierName::Resolves, ModifierName::Rejects])
            {
                return Err(ExpectError::ModifierUnknown);
            }
        } else {
            return Err(ExpectError::ModifierUnknown);
        }

        modifiers.push(index);
    }

    Err(ExpectError::MatcherNotFound)
}

fn is_top_most_call_expr<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let mut node = node;

    loop {
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return true;
        };

        match parent.kind() {
            AstKind::CallExpression(_) => return false,
            AstKind::MemberExpression(_) => node = parent,
            _ => {
                return true;
            }
        }
    }
}

fn parse_jest_jest_fn_call<'a>(
    members: Vec<KnownMemberExpressionProperty<'a>>,
    name: &'a str,
    local: &'a str,
) -> Option<ParsedJestFnCall<'a>> {
    if !name.to_ascii_lowercase().eq_ignore_ascii_case("jest") {
        return None;
    }

    return Some(ParsedJestFnCall::GeneralJestFnCall(ParsedGeneralJestFnCall {
        kind: JestFnKind::General(JestGeneralFnKind::Jest),
        members,
        name: Cow::Borrowed(name),
        local: Cow::Borrowed(local),
    }));
}

#[derive(Clone, Copy, Debug)]
pub enum ExpectError {
    ModifierUnknown,
    MatcherNotFound,
    MatcherNotCalled,
}

pub struct ExpectFnCallOptions<'a, 'b> {
    pub call_expr: &'a CallExpression<'a>,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    pub name: &'a str,
    pub local: &'a str,
    pub head: KnownMemberExpressionProperty<'a>,
    pub node: &'b AstNode<'a>,
    pub ctx: &'b LintContext<'a>,
}

// If find a match in `VALID_JEST_FN_CALL_CHAINS`, return true.
fn is_valid_jest_call(members: &[Cow<str>]) -> bool {
    VALID_JEST_FN_CALL_CHAINS
        .binary_search_by(|chain| {
            chain
                .iter()
                .zip(members.iter())
                .find_map(|(&chain, member)| {
                    let ordering = chain.cmp(member.as_ref());
                    if ordering != Ordering::Equal {
                        return Some(ordering);
                    }
                    None
                })
                .unwrap_or(Ordering::Equal)
        })
        .is_ok()
}

fn is_valid_vitest_call(members: &[Cow<str>]) -> bool {
    let vitest_fn = members.join(".");
    VALID_VITEST_FN_CALL_CHAINS.contains(&vitest_fn)
}

fn resolve_to_jest_fn<'a>(
    call_expr: &'a CallExpression<'a>,
    original: Option<&'a str>,
) -> Option<ResolvedJestFn<'a>> {
    let ident = resolve_first_ident(&call_expr.callee)?;
    Some(ResolvedJestFn { local: ident.name.as_str(), original })
}

fn resolve_first_ident<'a>(expr: &'a Expression<'a>) -> Option<&'a IdentifierReference<'a>> {
    match expr {
        Expression::Identifier(ident) => Some(ident),
        match_member_expression!(Expression) => {
            resolve_first_ident(expr.to_member_expression().object())
        }
        Expression::CallExpression(call_expr) => resolve_first_ident(&call_expr.callee),
        Expression::TaggedTemplateExpression(tagged_expr) => resolve_first_ident(&tagged_expr.tag),
        _ => None,
    }
}

pub enum ParsedJestFnCall<'a> {
    GeneralJestFnCall(ParsedGeneralJestFnCall<'a>),
    ExpectFnCall(ParsedExpectFnCall<'a>),
}

impl<'a> ParsedJestFnCall<'a> {
    pub fn kind(&self) -> JestFnKind {
        match self {
            Self::GeneralJestFnCall(call) => call.kind,
            Self::ExpectFnCall(call) => call.kind,
        }
    }
}

pub struct ParsedGeneralJestFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    pub name: Cow<'a, str>,
    #[allow(unused)]
    pub local: Cow<'a, str>,
}

#[derive(Debug)]
pub struct ParsedExpectFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    #[allow(unused)]
    pub name: Cow<'a, str>,
    pub local: Cow<'a, str>,
    pub head: KnownMemberExpressionProperty<'a>,
    pub args: &'a oxc_allocator::Vec<'a, Argument<'a>>,
    // In `expect(1).not.resolved.toBe()`, "not", "resolved" will be modifier
    // it save a group of modifier index from members
    pub modifier_indices: Vec<usize>,
    // In `expect(1).toBe(2)`, "toBe" will be matcher
    // it save the matcher index from members
    pub matcher_index: Option<usize>,
    pub expect_error: Option<ExpectError>,
}

impl<'a> ParsedExpectFnCall<'a> {
    pub fn matcher(&self) -> Option<&KnownMemberExpressionProperty<'a>> {
        let matcher_index = self.matcher_index?;
        self.members.get(matcher_index)
    }

    pub fn modifiers(&self) -> Vec<&KnownMemberExpressionProperty<'a>> {
        self.modifier_indices.iter().filter_map(|i| self.members.get(*i)).collect::<Vec<_>>()
    }
}

struct ResolvedJestFn<'a> {
    pub local: &'a str,
    pub original: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
pub enum KnownMemberExpressionParentKind {
    Member,
    Call,
    TaggedTemplate,
}

#[derive(Debug)]
pub struct KnownMemberExpressionProperty<'a> {
    pub element: MemberExpressionElement<'a>,
    pub parent: Option<&'a Expression<'a>>,
    pub parent_kind: Option<KnownMemberExpressionParentKind>,
    pub grandparent_kind: Option<KnownMemberExpressionParentKind>,
    pub span: Span,
}

impl<'a> KnownMemberExpressionProperty<'a> {
    pub fn name(&self) -> Option<Cow<'a, str>> {
        match &self.element {
            MemberExpressionElement::Expression(expr) => match expr {
                Expression::Identifier(ident) => Some(Cow::Borrowed(ident.name.as_str())),
                Expression::StringLiteral(string_literal) => {
                    Some(Cow::Borrowed(string_literal.value.as_str()))
                }
                Expression::TemplateLiteral(template_literal) => Some(Cow::Borrowed(
                    template_literal.quasi().expect("get string content").as_str(),
                )),
                _ => None,
            },
            MemberExpressionElement::IdentName(ident_name) => {
                Some(Cow::Borrowed(ident_name.name.as_str()))
            }
        }
    }

    pub fn is_name_equal(&self, name: &str) -> bool {
        self.name().map_or(false, |n| n == name)
    }

    pub fn is_name_unequal(&self, name: &str) -> bool {
        !self.is_name_equal(name)
    }

    pub fn is_name_in_modifiers(&self, modifiers: &[ModifierName]) -> bool {
        self.name().map_or(false, |name| {
            if let Some(modifier_name) = ModifierName::from(name.as_ref()) {
                return modifiers.contains(&modifier_name);
            }
            false
        })
    }
}

#[derive(Debug)]
pub enum MemberExpressionElement<'a> {
    Expression(&'a Expression<'a>),
    IdentName(&'a IdentifierName<'a>),
}

impl<'a> MemberExpressionElement<'a> {
    pub fn from_member_expr(
        member_expr: &'a MemberExpression<'a>,
    ) -> Option<(Span, MemberExpressionElement<'a>)> {
        let (span, _) = member_expr.static_property_info()?;
        match member_expr {
            MemberExpression::ComputedMemberExpression(expr) => {
                Some((span, Self::Expression(&expr.expression)))
            }
            MemberExpression::StaticMemberExpression(expr) => {
                Some((span, Self::IdentName(&expr.property)))
            }
            // Jest fn chains don't have private fields, just ignore it.
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    pub fn is_string_literal(&self) -> bool {
        matches!(
            self,
            Self::Expression(Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
        )
    }
}

struct NodeChainParams<'a> {
    expr: &'a Expression<'a>,
    parent: Option<&'a Expression<'a>>,
    parent_kind: Option<KnownMemberExpressionParentKind>,
    grandparent_kind: Option<KnownMemberExpressionParentKind>,
}

/// Port from [eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest/blob/a058f22f94774eeea7980ea2d1f24c6808bf3e2c/src/rules/utils/parseJestFnCall.ts#L36-L51)
fn get_node_chain<'a>(params: &NodeChainParams<'a>) -> Vec<KnownMemberExpressionProperty<'a>> {
    let mut chain = Vec::new();
    let NodeChainParams { expr, parent, parent_kind, grandparent_kind } = params;

    match expr {
        match_member_expression!(Expression) => {
            let member_expr = expr.to_member_expression();
            let params = NodeChainParams {
                expr: member_expr.object(),
                parent: Some(expr),
                parent_kind: Some(KnownMemberExpressionParentKind::Member),
                grandparent_kind: *parent_kind,
            };

            chain.extend(get_node_chain(&params));
            if let Some((span, element)) = MemberExpressionElement::from_member_expr(member_expr) {
                chain.push(KnownMemberExpressionProperty {
                    element,
                    parent: Some(expr),
                    parent_kind: Some(KnownMemberExpressionParentKind::Member),
                    grandparent_kind: *parent_kind,
                    span,
                });
            }
        }
        Expression::Identifier(ident) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent: *parent,
                parent_kind: *parent_kind,
                grandparent_kind: *grandparent_kind,
                span: ident.span,
            });
        }
        Expression::CallExpression(call_expr) => {
            let params = NodeChainParams {
                expr: &call_expr.callee,
                parent: Some(expr),
                parent_kind: Some(KnownMemberExpressionParentKind::Call),
                grandparent_kind: *parent_kind,
            };
            let sub_chain = get_node_chain(&params);
            chain.extend(sub_chain);
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            let params = NodeChainParams {
                expr: &tagged_expr.tag,
                parent: Some(expr),
                parent_kind: Some(KnownMemberExpressionParentKind::TaggedTemplate),
                grandparent_kind: *parent_kind,
            };

            let sub_chain = get_node_chain(&params);
            chain.extend(sub_chain);
        }
        Expression::StringLiteral(string_literal) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent: *parent,
                parent_kind: *parent_kind,
                grandparent_kind: *grandparent_kind,
                span: string_literal.span,
            });
        }
        Expression::TemplateLiteral(template_literal) if is_pure_string(template_literal) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent: *parent,
                parent_kind: *parent_kind,
                grandparent_kind: *grandparent_kind,
                span: template_literal.span,
            });
        }
        _ => {}
    };

    chain
}

// sorted list for binary search.
const VALID_JEST_FN_CALL_CHAINS: [[&str; 4]; 51] = [
    ["afterAll", "", "", ""],
    ["afterEach", "", "", ""],
    ["beforeAll", "", "", ""],
    ["beforeEach", "", "", ""],
    ["describe", "", "", ""],
    ["describe", "each", "", ""],
    ["describe", "only", "", ""],
    ["describe", "only", "each", ""],
    ["describe", "skip", "", ""],
    ["describe", "skip", "each", ""],
    ["fdescribe", "", "", ""],
    ["fdescribe", "each", "", ""],
    ["fit", "", "", ""],
    ["fit", "each", "", ""],
    ["fit", "failing", "", ""],
    ["it", "", "", ""],
    ["it", "concurrent", "", ""],
    ["it", "concurrent", "each", ""],
    ["it", "concurrent", "only", "each"],
    ["it", "concurrent", "skip", "each"],
    ["it", "each", "", ""],
    ["it", "failing", "", ""],
    ["it", "only", "", ""],
    ["it", "only", "each", ""],
    ["it", "only", "failing", ""],
    ["it", "skip", "", ""],
    ["it", "skip", "each", ""],
    ["it", "skip", "failing", ""],
    ["it", "todo", "", ""],
    ["test", "", "", ""],
    ["test", "concurrent", "", ""],
    ["test", "concurrent", "each", ""],
    ["test", "concurrent", "only", "each"],
    ["test", "concurrent", "skip", "each"],
    ["test", "each", "", ""],
    ["test", "failing", "", ""],
    ["test", "only", "", ""],
    ["test", "only", "each", ""],
    ["test", "only", "failing", ""],
    ["test", "skip", "", ""],
    ["test", "skip", "each", ""],
    ["test", "skip", "failing", ""],
    ["test", "todo", "", ""],
    ["xdescribe", "", "", ""],
    ["xdescribe", "each", "", ""],
    ["xit", "", "", ""],
    ["xit", "each", "", ""],
    ["xit", "failing", "", ""],
    ["xtest", "", "", ""],
    ["xtest", "each", "", ""],
    ["xtest", "failing", "", ""],
];

const VALID_VITEST_FN_CALL_CHAINS: phf::Set<&'static str> = phf_set! {
    "beforeEach",
    "beforeAll",
    "afterEach",
    "afterAll",
    "it",
    "it.skip",
    "it.only",
    "it.concurrent",
    "it.sequential",
    "it.todo",
    "it.fails",
    "it.extend",
    "it.skipIf",
    "it.runIf",
    "it.each",
    "it.skip.only",
    "it.skip.concurrent",
    "it.skip.sequential",
    "it.skip.todo",
    "it.skip.fails",
    "it.only.skip",
    "it.only.concurrent",
    "it.only.sequential",
    "it.only.todo",
    "it.only.fails",
    "it.concurrent.skip",
    "it.concurrent.only",
    "it.concurrent.sequential",
    "it.concurrent.todo",
    "it.concurrent.fails",
    "it.sequential.skip",
    "it.sequential.only",
    "it.sequential.concurrent",
    "it.sequential.todo",
    "it.sequential.fails",
    "it.todo.skip",
    "it.todo.only",
    "it.todo.concurrent",
    "it.todo.sequential",
    "it.todo.fails",
    "it.fails.skip",
    "it.fails.only",
    "it.fails.concurrent",
    "it.fails.sequential",
    "it.fails.todo",
    "it.extend.skip",
    "it.extend.only",
    "it.extend.concurrent",
    "it.extend.sequential",
    "it.extend.todo",
    "it.extend.fails",
    "it.skipIf.skip",
    "it.skipIf.only",
    "it.skipIf.concurrent",
    "it.skipIf.sequential",
    "it.skipIf.todo",
    "it.skipIf.fails",
    "it.runIf.skip",
    "it.runIf.only",
    "it.runIf.concurrent",
    "it.runIf.sequential",
    "it.runIf.todo",
    "it.runIf.fails",
    "it.skip.each",
    "it.only.each",
    "it.concurrent.each",
    "it.sequential.each",
    "it.todo.each",
    "it.fails.each",
    "it.extend.skipIf",
    "it.extend.runIf",
    "it.extend.each",
    "it.skipIf.each",
    "it.runIf.each",
    "it.skip.only.concurrent",
    "it.skip.only.sequential",
    "it.skip.only.todo",
    "it.skip.only.fails",
    "it.skip.concurrent.only",
    "it.skip.concurrent.sequential",
    "it.skip.concurrent.todo",
    "it.skip.concurrent.fails",
    "it.skip.sequential.only",
    "it.skip.sequential.concurrent",
    "it.skip.sequential.todo",
    "it.skip.sequential.fails",
    "it.skip.todo.only",
    "it.skip.todo.concurrent",
    "it.skip.todo.sequential",
    "it.skip.todo.fails",
    "it.skip.fails.only",
    "it.skip.fails.concurrent",
    "it.skip.fails.sequential",
    "it.skip.fails.todo",
    "it.only.skip.concurrent",
    "it.only.skip.sequential",
    "it.only.skip.todo",
    "it.only.skip.fails",
    "it.only.concurrent.skip",
    "it.only.concurrent.sequential",
    "it.only.concurrent.todo",
    "it.only.concurrent.fails",
    "it.only.sequential.skip",
    "it.only.sequential.concurrent",
    "it.only.sequential.todo",
    "it.only.sequential.fails",
    "it.only.todo.skip",
    "it.only.todo.concurrent",
    "it.only.todo.sequential",
    "it.only.todo.fails",
    "it.only.fails.skip",
    "it.only.fails.concurrent",
    "it.only.fails.sequential",
    "it.only.fails.todo",
    "it.concurrent.skip.only",
    "it.concurrent.skip.sequential",
    "it.concurrent.skip.todo",
    "it.concurrent.skip.fails",
    "it.concurrent.only.skip",
    "it.concurrent.only.sequential",
    "it.concurrent.only.todo",
    "it.concurrent.only.fails",
    "it.concurrent.sequential.skip",
    "it.concurrent.sequential.only",
    "it.concurrent.sequential.todo",
    "it.concurrent.sequential.fails",
    "it.concurrent.todo.skip",
    "it.concurrent.todo.only",
    "it.concurrent.todo.sequential",
    "it.concurrent.todo.fails",
    "it.concurrent.fails.skip",
    "it.concurrent.fails.only",
    "it.concurrent.fails.sequential",
    "it.concurrent.fails.todo",
    "it.sequential.skip.only",
    "it.sequential.skip.concurrent",
    "it.sequential.skip.todo",
    "it.sequential.skip.fails",
    "it.sequential.only.skip",
    "it.sequential.only.concurrent",
    "it.sequential.only.todo",
    "it.sequential.only.fails",
    "it.sequential.concurrent.skip",
    "it.sequential.concurrent.only",
    "it.sequential.concurrent.todo",
    "it.sequential.concurrent.fails",
    "it.sequential.todo.skip",
    "it.sequential.todo.only",
    "it.sequential.todo.concurrent",
    "it.sequential.todo.fails",
    "it.sequential.fails.skip",
    "it.sequential.fails.only",
    "it.sequential.fails.concurrent",
    "it.sequential.fails.todo",
    "it.todo.skip.only",
    "it.todo.skip.concurrent",
    "it.todo.skip.sequential",
    "it.todo.skip.fails",
    "it.todo.only.skip",
    "it.todo.only.concurrent",
    "it.todo.only.sequential",
    "it.todo.only.fails",
    "it.todo.concurrent.skip",
    "it.todo.concurrent.only",
    "it.todo.concurrent.sequential",
    "it.todo.concurrent.fails",
    "it.todo.sequential.skip",
    "it.todo.sequential.only",
    "it.todo.sequential.concurrent",
    "it.todo.sequential.fails",
    "it.todo.fails.skip",
    "it.todo.fails.only",
    "it.todo.fails.concurrent",
    "it.todo.fails.sequential",
    "it.fails.skip.only",
    "it.fails.skip.concurrent",
    "it.fails.skip.sequential",
    "it.fails.skip.todo",
    "it.fails.only.skip",
    "it.fails.only.concurrent",
    "it.fails.only.sequential",
    "it.fails.only.todo",
    "it.fails.concurrent.skip",
    "it.fails.concurrent.only",
    "it.fails.concurrent.sequential",
    "it.fails.concurrent.todo",
    "it.fails.sequential.skip",
    "it.fails.sequential.only",
    "it.fails.sequential.concurrent",
    "it.fails.sequential.todo",
    "it.fails.todo.skip",
    "it.fails.todo.only",
    "it.fails.todo.concurrent",
    "it.fails.todo.sequential",
    "it.extend.skip.only",
    "it.extend.skip.concurrent",
    "it.extend.skip.sequential",
    "it.extend.skip.todo",
    "it.extend.skip.fails",
    "it.extend.only.skip",
    "it.extend.only.concurrent",
    "it.extend.only.sequential",
    "it.extend.only.todo",
    "it.extend.only.fails",
    "it.extend.concurrent.skip",
    "it.extend.concurrent.only",
    "it.extend.concurrent.sequential",
    "it.extend.concurrent.todo",
    "it.extend.concurrent.fails",
    "it.extend.sequential.skip",
    "it.extend.sequential.only",
    "it.extend.sequential.concurrent",
    "it.extend.sequential.todo",
    "it.extend.sequential.fails",
    "it.extend.todo.skip",
    "it.extend.todo.only",
    "it.extend.todo.concurrent",
    "it.extend.todo.sequential",
    "it.extend.todo.fails",
    "it.extend.fails.skip",
    "it.extend.fails.only",
    "it.extend.fails.concurrent",
    "it.extend.fails.sequential",
    "it.extend.fails.todo",
    "it.skipIf.skip.only",
    "it.skipIf.skip.concurrent",
    "it.skipIf.skip.sequential",
    "it.skipIf.skip.todo",
    "it.skipIf.skip.fails",
    "it.skipIf.only.skip",
    "it.skipIf.only.concurrent",
    "it.skipIf.only.sequential",
    "it.skipIf.only.todo",
    "it.skipIf.only.fails",
    "it.skipIf.concurrent.skip",
    "it.skipIf.concurrent.only",
    "it.skipIf.concurrent.sequential",
    "it.skipIf.concurrent.todo",
    "it.skipIf.concurrent.fails",
    "it.skipIf.sequential.skip",
    "it.skipIf.sequential.only",
    "it.skipIf.sequential.concurrent",
    "it.skipIf.sequential.todo",
    "it.skipIf.sequential.fails",
    "it.skipIf.todo.skip",
    "it.skipIf.todo.only",
    "it.skipIf.todo.concurrent",
    "it.skipIf.todo.sequential",
    "it.skipIf.todo.fails",
    "it.skipIf.fails.skip",
    "it.skipIf.fails.only",
    "it.skipIf.fails.concurrent",
    "it.skipIf.fails.sequential",
    "it.skipIf.fails.todo",
    "it.runIf.skip.only",
    "it.runIf.skip.concurrent",
    "it.runIf.skip.sequential",
    "it.runIf.skip.todo",
    "it.runIf.skip.fails",
    "it.runIf.only.skip",
    "it.runIf.only.concurrent",
    "it.runIf.only.sequential",
    "it.runIf.only.todo",
    "it.runIf.only.fails",
    "it.runIf.concurrent.skip",
    "it.runIf.concurrent.only",
    "it.runIf.concurrent.sequential",
    "it.runIf.concurrent.todo",
    "it.runIf.concurrent.fails",
    "it.runIf.sequential.skip",
    "it.runIf.sequential.only",
    "it.runIf.sequential.concurrent",
    "it.runIf.sequential.todo",
    "it.runIf.sequential.fails",
    "it.runIf.todo.skip",
    "it.runIf.todo.only",
    "it.runIf.todo.concurrent",
    "it.runIf.todo.sequential",
    "it.runIf.todo.fails",
    "it.runIf.fails.skip",
    "it.runIf.fails.only",
    "it.runIf.fails.concurrent",
    "it.runIf.fails.sequential",
    "it.runIf.fails.todo",
    "it.skip.only.each",
    "it.skip.concurrent.each",
    "it.skip.sequential.each",
    "it.skip.todo.each",
    "it.skip.fails.each",
    "it.only.skip.each",
    "it.only.concurrent.each",
    "it.only.sequential.each",
    "it.only.todo.each",
    "it.only.fails.each",
    "it.concurrent.skip.each",
    "it.concurrent.only.each",
    "it.concurrent.sequential.each",
    "it.concurrent.todo.each",
    "it.concurrent.fails.each",
    "it.sequential.skip.each",
    "it.sequential.only.each",
    "it.sequential.concurrent.each",
    "it.sequential.todo.each",
    "it.sequential.fails.each",
    "it.todo.skip.each",
    "it.todo.only.each",
    "it.todo.concurrent.each",
    "it.todo.sequential.each",
    "it.todo.fails.each",
    "it.fails.skip.each",
    "it.fails.only.each",
    "it.fails.concurrent.each",
    "it.fails.sequential.each",
    "it.fails.todo.each",
    "it.extend.skipIf.skip",
    "it.extend.skipIf.only",
    "it.extend.skipIf.concurrent",
    "it.extend.skipIf.sequential",
    "it.extend.skipIf.todo",
    "it.extend.skipIf.fails",
    "it.extend.runIf.skip",
    "it.extend.runIf.only",
    "it.extend.runIf.concurrent",
    "it.extend.runIf.sequential",
    "it.extend.runIf.todo",
    "it.extend.runIf.fails",
    "it.extend.skip.each",
    "it.extend.only.each",
    "it.extend.concurrent.each",
    "it.extend.sequential.each",
    "it.extend.todo.each",
    "it.extend.fails.each",
    "it.skipIf.skip.each",
    "it.skipIf.only.each",
    "it.skipIf.concurrent.each",
    "it.skipIf.sequential.each",
    "it.skipIf.todo.each",
    "it.skipIf.fails.each",
    "it.runIf.skip.each",
    "it.runIf.only.each",
    "it.runIf.concurrent.each",
    "it.runIf.sequential.each",
    "it.runIf.todo.each",
    "it.runIf.fails.each",
    "it.extend.skipIf.each",
    "it.extend.runIf.each",
    "test",
    "test.skip",
    "test.only",
    "test.concurrent",
    "test.sequential",
    "test.todo",
    "test.fails",
    "test.extend",
    "test.skipIf",
    "test.runIf",
    "test.each",
    "test.skip.only",
    "test.skip.concurrent",
    "test.skip.sequential",
    "test.skip.todo",
    "test.skip.fails",
    "test.only.skip",
    "test.only.concurrent",
    "test.only.sequential",
    "test.only.todo",
    "test.only.fails",
    "test.concurrent.skip",
    "test.concurrent.only",
    "test.concurrent.sequential",
    "test.concurrent.todo",
    "test.concurrent.fails",
    "test.sequential.skip",
    "test.sequential.only",
    "test.sequential.concurrent",
    "test.sequential.todo",
    "test.sequential.fails",
    "test.todo.skip",
    "test.todo.only",
    "test.todo.concurrent",
    "test.todo.sequential",
    "test.todo.fails",
    "test.fails.skip",
    "test.fails.only",
    "test.fails.concurrent",
    "test.fails.sequential",
    "test.fails.todo",
    "test.extend.skip",
    "test.extend.only",
    "test.extend.concurrent",
    "test.extend.sequential",
    "test.extend.todo",
    "test.extend.fails",
    "test.skipIf.skip",
    "test.skipIf.only",
    "test.skipIf.concurrent",
    "test.skipIf.sequential",
    "test.skipIf.todo",
    "test.skipIf.fails",
    "test.runIf.skip",
    "test.runIf.only",
    "test.runIf.concurrent",
    "test.runIf.sequential",
    "test.runIf.todo",
    "test.runIf.fails",
    "test.skip.each",
    "test.only.each",
    "test.concurrent.each",
    "test.sequential.each",
    "test.todo.each",
    "test.fails.each",
    "test.extend.skipIf",
    "test.extend.runIf",
    "test.extend.each",
    "test.skipIf.each",
    "test.runIf.each",
    "test.skip.only.concurrent",
    "test.skip.only.sequential",
    "test.skip.only.todo",
    "test.skip.only.fails",
    "test.skip.concurrent.only",
    "test.skip.concurrent.sequential",
    "test.skip.concurrent.todo",
    "test.skip.concurrent.fails",
    "test.skip.sequential.only",
    "test.skip.sequential.concurrent",
    "test.skip.sequential.todo",
    "test.skip.sequential.fails",
    "test.skip.todo.only",
    "test.skip.todo.concurrent",
    "test.skip.todo.sequential",
    "test.skip.todo.fails",
    "test.skip.fails.only",
    "test.skip.fails.concurrent",
    "test.skip.fails.sequential",
    "test.skip.fails.todo",
    "test.only.skip.concurrent",
    "test.only.skip.sequential",
    "test.only.skip.todo",
    "test.only.skip.fails",
    "test.only.concurrent.skip",
    "test.only.concurrent.sequential",
    "test.only.concurrent.todo",
    "test.only.concurrent.fails",
    "test.only.sequential.skip",
    "test.only.sequential.concurrent",
    "test.only.sequential.todo",
    "test.only.sequential.fails",
    "test.only.todo.skip",
    "test.only.todo.concurrent",
    "test.only.todo.sequential",
    "test.only.todo.fails",
    "test.only.fails.skip",
    "test.only.fails.concurrent",
    "test.only.fails.sequential",
    "test.only.fails.todo",
    "test.concurrent.skip.only",
    "test.concurrent.skip.sequential",
    "test.concurrent.skip.todo",
    "test.concurrent.skip.fails",
    "test.concurrent.only.skip",
    "test.concurrent.only.sequential",
    "test.concurrent.only.todo",
    "test.concurrent.only.fails",
    "test.concurrent.sequential.skip",
    "test.concurrent.sequential.only",
    "test.concurrent.sequential.todo",
    "test.concurrent.sequential.fails",
    "test.concurrent.todo.skip",
    "test.concurrent.todo.only",
    "test.concurrent.todo.sequential",
    "test.concurrent.todo.fails",
    "test.concurrent.fails.skip",
    "test.concurrent.fails.only",
    "test.concurrent.fails.sequential",
    "test.concurrent.fails.todo",
    "test.sequential.skip.only",
    "test.sequential.skip.concurrent",
    "test.sequential.skip.todo",
    "test.sequential.skip.fails",
    "test.sequential.only.skip",
    "test.sequential.only.concurrent",
    "test.sequential.only.todo",
    "test.sequential.only.fails",
    "test.sequential.concurrent.skip",
    "test.sequential.concurrent.only",
    "test.sequential.concurrent.todo",
    "test.sequential.concurrent.fails",
    "test.sequential.todo.skip",
    "test.sequential.todo.only",
    "test.sequential.todo.concurrent",
    "test.sequential.todo.fails",
    "test.sequential.fails.skip",
    "test.sequential.fails.only",
    "test.sequential.fails.concurrent",
    "test.sequential.fails.todo",
    "test.todo.skip.only",
    "test.todo.skip.concurrent",
    "test.todo.skip.sequential",
    "test.todo.skip.fails",
    "test.todo.only.skip",
    "test.todo.only.concurrent",
    "test.todo.only.sequential",
    "test.todo.only.fails",
    "test.todo.concurrent.skip",
    "test.todo.concurrent.only",
    "test.todo.concurrent.sequential",
    "test.todo.concurrent.fails",
    "test.todo.sequential.skip",
    "test.todo.sequential.only",
    "test.todo.sequential.concurrent",
    "test.todo.sequential.fails",
    "test.todo.fails.skip",
    "test.todo.fails.only",
    "test.todo.fails.concurrent",
    "test.todo.fails.sequential",
    "test.fails.skip.only",
    "test.fails.skip.concurrent",
    "test.fails.skip.sequential",
    "test.fails.skip.todo",
    "test.fails.only.skip",
    "test.fails.only.concurrent",
    "test.fails.only.sequential",
    "test.fails.only.todo",
    "test.fails.concurrent.skip",
    "test.fails.concurrent.only",
    "test.fails.concurrent.sequential",
    "test.fails.concurrent.todo",
    "test.fails.sequential.skip",
    "test.fails.sequential.only",
    "test.fails.sequential.concurrent",
    "test.fails.sequential.todo",
    "test.fails.todo.skip",
    "test.fails.todo.only",
    "test.fails.todo.concurrent",
    "test.fails.todo.sequential",
    "test.extend.skip.only",
    "test.extend.skip.concurrent",
    "test.extend.skip.sequential",
    "test.extend.skip.todo",
    "test.extend.skip.fails",
    "test.extend.only.skip",
    "test.extend.only.concurrent",
    "test.extend.only.sequential",
    "test.extend.only.todo",
    "test.extend.only.fails",
    "test.extend.concurrent.skip",
    "test.extend.concurrent.only",
    "test.extend.concurrent.sequential",
    "test.extend.concurrent.todo",
    "test.extend.concurrent.fails",
    "test.extend.sequential.skip",
    "test.extend.sequential.only",
    "test.extend.sequential.concurrent",
    "test.extend.sequential.todo",
    "test.extend.sequential.fails",
    "test.extend.todo.skip",
    "test.extend.todo.only",
    "test.extend.todo.concurrent",
    "test.extend.todo.sequential",
    "test.extend.todo.fails",
    "test.extend.fails.skip",
    "test.extend.fails.only",
    "test.extend.fails.concurrent",
    "test.extend.fails.sequential",
    "test.extend.fails.todo",
    "test.skipIf.skip.only",
    "test.skipIf.skip.concurrent",
    "test.skipIf.skip.sequential",
    "test.skipIf.skip.todo",
    "test.skipIf.skip.fails",
    "test.skipIf.only.skip",
    "test.skipIf.only.concurrent",
    "test.skipIf.only.sequential",
    "test.skipIf.only.todo",
    "test.skipIf.only.fails",
    "test.skipIf.concurrent.skip",
    "test.skipIf.concurrent.only",
    "test.skipIf.concurrent.sequential",
    "test.skipIf.concurrent.todo",
    "test.skipIf.concurrent.fails",
    "test.skipIf.sequential.skip",
    "test.skipIf.sequential.only",
    "test.skipIf.sequential.concurrent",
    "test.skipIf.sequential.todo",
    "test.skipIf.sequential.fails",
    "test.skipIf.todo.skip",
    "test.skipIf.todo.only",
    "test.skipIf.todo.concurrent",
    "test.skipIf.todo.sequential",
    "test.skipIf.todo.fails",
    "test.skipIf.fails.skip",
    "test.skipIf.fails.only",
    "test.skipIf.fails.concurrent",
    "test.skipIf.fails.sequential",
    "test.skipIf.fails.todo",
    "test.runIf.skip.only",
    "test.runIf.skip.concurrent",
    "test.runIf.skip.sequential",
    "test.runIf.skip.todo",
    "test.runIf.skip.fails",
    "test.runIf.only.skip",
    "test.runIf.only.concurrent",
    "test.runIf.only.sequential",
    "test.runIf.only.todo",
    "test.runIf.only.fails",
    "test.runIf.concurrent.skip",
    "test.runIf.concurrent.only",
    "test.runIf.concurrent.sequential",
    "test.runIf.concurrent.todo",
    "test.runIf.concurrent.fails",
    "test.runIf.sequential.skip",
    "test.runIf.sequential.only",
    "test.runIf.sequential.concurrent",
    "test.runIf.sequential.todo",
    "test.runIf.sequential.fails",
    "test.runIf.todo.skip",
    "test.runIf.todo.only",
    "test.runIf.todo.concurrent",
    "test.runIf.todo.sequential",
    "test.runIf.todo.fails",
    "test.runIf.fails.skip",
    "test.runIf.fails.only",
    "test.runIf.fails.concurrent",
    "test.runIf.fails.sequential",
    "test.runIf.fails.todo",
    "test.skip.only.each",
    "test.skip.concurrent.each",
    "test.skip.sequential.each",
    "test.skip.todo.each",
    "test.skip.fails.each",
    "test.only.skip.each",
    "test.only.concurrent.each",
    "test.only.sequential.each",
    "test.only.todo.each",
    "test.only.fails.each",
    "test.concurrent.skip.each",
    "test.concurrent.only.each",
    "test.concurrent.sequential.each",
    "test.concurrent.todo.each",
    "test.concurrent.fails.each",
    "test.sequential.skip.each",
    "test.sequential.only.each",
    "test.sequential.concurrent.each",
    "test.sequential.todo.each",
    "test.sequential.fails.each",
    "test.todo.skip.each",
    "test.todo.only.each",
    "test.todo.concurrent.each",
    "test.todo.sequential.each",
    "test.todo.fails.each",
    "test.fails.skip.each",
    "test.fails.only.each",
    "test.fails.concurrent.each",
    "test.fails.sequential.each",
    "test.fails.todo.each",
    "test.extend.skipIf.skip",
    "test.extend.skipIf.only",
    "test.extend.skipIf.concurrent",
    "test.extend.skipIf.sequential",
    "test.extend.skipIf.todo",
    "test.extend.skipIf.fails",
    "test.extend.runIf.skip",
    "test.extend.runIf.only",
    "test.extend.runIf.concurrent",
    "test.extend.runIf.sequential",
    "test.extend.runIf.todo",
    "test.extend.runIf.fails",
    "test.extend.skip.each",
    "test.extend.only.each",
    "test.extend.concurrent.each",
    "test.extend.sequential.each",
    "test.extend.todo.each",
    "test.extend.fails.each",
    "test.skipIf.skip.each",
    "test.skipIf.only.each",
    "test.skipIf.concurrent.each",
    "test.skipIf.sequential.each",
    "test.skipIf.todo.each",
    "test.skipIf.fails.each",
    "test.runIf.skip.each",
    "test.runIf.only.each",
    "test.runIf.concurrent.each",
    "test.runIf.sequential.each",
    "test.runIf.todo.each",
    "test.runIf.fails.each",
    "test.extend.skipIf.each",
    "test.extend.runIf.each",
    "bench",
    "bench.skip",
    "bench.only",
    "bench.todo",
    "bench.skipIf",
    "bench.runIf",
    "bench.skip.only",
    "bench.skip.todo",
    "bench.only.skip",
    "bench.only.todo",
    "bench.todo.skip",
    "bench.todo.only",
    "bench.skipIf.skip",
    "bench.skipIf.only",
    "bench.skipIf.todo",
    "bench.runIf.skip",
    "bench.runIf.only",
    "bench.runIf.todo",
    "bench.skip.only.todo",
    "bench.skip.todo.only",
    "bench.only.skip.todo",
    "bench.only.todo.skip",
    "bench.todo.skip.only",
    "bench.todo.only.skip",
    "bench.skipIf.skip.only",
    "bench.skipIf.skip.todo",
    "bench.skipIf.only.skip",
    "bench.skipIf.only.todo",
    "bench.skipIf.todo.skip",
    "bench.skipIf.todo.only",
    "bench.runIf.skip.only",
    "bench.runIf.skip.todo",
    "bench.runIf.only.skip",
    "bench.runIf.only.todo",
    "bench.runIf.todo.skip",
    "bench.runIf.todo.only",
    "describe",
    "describe.skip",
    "describe.only",
    "describe.concurrent",
    "describe.sequential",
    "describe.shuffle",
    "describe.todo",
    "describe.skipIf",
    "describe.runIf",
    "describe.each",
    "describe.skip.only",
    "describe.skip.concurrent",
    "describe.skip.sequential",
    "describe.skip.shuffle",
    "describe.skip.todo",
    "describe.only.skip",
    "describe.only.concurrent",
    "describe.only.sequential",
    "describe.only.shuffle",
    "describe.only.todo",
    "describe.concurrent.skip",
    "describe.concurrent.only",
    "describe.concurrent.sequential",
    "describe.concurrent.shuffle",
    "describe.concurrent.todo",
    "describe.sequential.skip",
    "describe.sequential.only",
    "describe.sequential.concurrent",
    "describe.sequential.shuffle",
    "describe.sequential.todo",
    "describe.shuffle.skip",
    "describe.shuffle.only",
    "describe.shuffle.concurrent",
    "describe.shuffle.sequential",
    "describe.shuffle.todo",
    "describe.todo.skip",
    "describe.todo.only",
    "describe.todo.concurrent",
    "describe.todo.sequential",
    "describe.todo.shuffle",
    "describe.skipIf.skip",
    "describe.skipIf.only",
    "describe.skipIf.concurrent",
    "describe.skipIf.sequential",
    "describe.skipIf.shuffle",
    "describe.skipIf.todo",
    "describe.runIf.skip",
    "describe.runIf.only",
    "describe.runIf.concurrent",
    "describe.runIf.sequential",
    "describe.runIf.shuffle",
    "describe.runIf.todo",
    "describe.skip.each",
    "describe.only.each",
    "describe.concurrent.each",
    "describe.sequential.each",
    "describe.shuffle.each",
    "describe.todo.each",
    "describe.skipIf.each",
    "describe.runIf.each",
    "describe.skip.only.concurrent",
    "describe.skip.only.sequential",
    "describe.skip.only.shuffle",
    "describe.skip.only.todo",
    "describe.skip.concurrent.only",
    "describe.skip.concurrent.sequential",
    "describe.skip.concurrent.shuffle",
    "describe.skip.concurrent.todo",
    "describe.skip.sequential.only",
    "describe.skip.sequential.concurrent",
    "describe.skip.sequential.shuffle",
    "describe.skip.sequential.todo",
    "describe.skip.shuffle.only",
    "describe.skip.shuffle.concurrent",
    "describe.skip.shuffle.sequential",
    "describe.skip.shuffle.todo",
    "describe.skip.todo.only",
    "describe.skip.todo.concurrent",
    "describe.skip.todo.sequential",
    "describe.skip.todo.shuffle",
    "describe.only.skip.concurrent",
    "describe.only.skip.sequential",
    "describe.only.skip.shuffle",
    "describe.only.skip.todo",
    "describe.only.concurrent.skip",
    "describe.only.concurrent.sequential",
    "describe.only.concurrent.shuffle",
    "describe.only.concurrent.todo",
    "describe.only.sequential.skip",
    "describe.only.sequential.concurrent",
    "describe.only.sequential.shuffle",
    "describe.only.sequential.todo",
    "describe.only.shuffle.skip",
    "describe.only.shuffle.concurrent",
    "describe.only.shuffle.sequential",
    "describe.only.shuffle.todo",
    "describe.only.todo.skip",
    "describe.only.todo.concurrent",
    "describe.only.todo.sequential",
    "describe.only.todo.shuffle",
    "describe.concurrent.skip.only",
    "describe.concurrent.skip.sequential",
    "describe.concurrent.skip.shuffle",
    "describe.concurrent.skip.todo",
    "describe.concurrent.only.skip",
    "describe.concurrent.only.sequential",
    "describe.concurrent.only.shuffle",
    "describe.concurrent.only.todo",
    "describe.concurrent.sequential.skip",
    "describe.concurrent.sequential.only",
    "describe.concurrent.sequential.shuffle",
    "describe.concurrent.sequential.todo",
    "describe.concurrent.shuffle.skip",
    "describe.concurrent.shuffle.only",
    "describe.concurrent.shuffle.sequential",
    "describe.concurrent.shuffle.todo",
    "describe.concurrent.todo.skip",
    "describe.concurrent.todo.only",
    "describe.concurrent.todo.sequential",
    "describe.concurrent.todo.shuffle",
    "describe.sequential.skip.only",
    "describe.sequential.skip.concurrent",
    "describe.sequential.skip.shuffle",
    "describe.sequential.skip.todo",
    "describe.sequential.only.skip",
    "describe.sequential.only.concurrent",
    "describe.sequential.only.shuffle",
    "describe.sequential.only.todo",
    "describe.sequential.concurrent.skip",
    "describe.sequential.concurrent.only",
    "describe.sequential.concurrent.shuffle",
    "describe.sequential.concurrent.todo",
    "describe.sequential.shuffle.skip",
    "describe.sequential.shuffle.only",
    "describe.sequential.shuffle.concurrent",
    "describe.sequential.shuffle.todo",
    "describe.sequential.todo.skip",
    "describe.sequential.todo.only",
    "describe.sequential.todo.concurrent",
    "describe.sequential.todo.shuffle",
    "describe.shuffle.skip.only",
    "describe.shuffle.skip.concurrent",
    "describe.shuffle.skip.sequential",
    "describe.shuffle.skip.todo",
    "describe.shuffle.only.skip",
    "describe.shuffle.only.concurrent",
    "describe.shuffle.only.sequential",
    "describe.shuffle.only.todo",
    "describe.shuffle.concurrent.skip",
    "describe.shuffle.concurrent.only",
    "describe.shuffle.concurrent.sequential",
    "describe.shuffle.concurrent.todo",
    "describe.shuffle.sequential.skip",
    "describe.shuffle.sequential.only",
    "describe.shuffle.sequential.concurrent",
    "describe.shuffle.sequential.todo",
    "describe.shuffle.todo.skip",
    "describe.shuffle.todo.only",
    "describe.shuffle.todo.concurrent",
    "describe.shuffle.todo.sequential",
    "describe.todo.skip.only",
    "describe.todo.skip.concurrent",
    "describe.todo.skip.sequential",
    "describe.todo.skip.shuffle",
    "describe.todo.only.skip",
    "describe.todo.only.concurrent",
    "describe.todo.only.sequential",
    "describe.todo.only.shuffle",
    "describe.todo.concurrent.skip",
    "describe.todo.concurrent.only",
    "describe.todo.concurrent.sequential",
    "describe.todo.concurrent.shuffle",
    "describe.todo.sequential.skip",
    "describe.todo.sequential.only",
    "describe.todo.sequential.concurrent",
    "describe.todo.sequential.shuffle",
    "describe.todo.shuffle.skip",
    "describe.todo.shuffle.only",
    "describe.todo.shuffle.concurrent",
    "describe.todo.shuffle.sequential",
    "describe.skipIf.skip.only",
    "describe.skipIf.skip.concurrent",
    "describe.skipIf.skip.sequential",
    "describe.skipIf.skip.shuffle",
    "describe.skipIf.skip.todo",
    "describe.skipIf.only.skip",
    "describe.skipIf.only.concurrent",
    "describe.skipIf.only.sequential",
    "describe.skipIf.only.shuffle",
    "describe.skipIf.only.todo",
    "describe.skipIf.concurrent.skip",
    "describe.skipIf.concurrent.only",
    "describe.skipIf.concurrent.sequential",
    "describe.skipIf.concurrent.shuffle",
    "describe.skipIf.concurrent.todo",
    "describe.skipIf.sequential.skip",
    "describe.skipIf.sequential.only",
    "describe.skipIf.sequential.concurrent",
    "describe.skipIf.sequential.shuffle",
    "describe.skipIf.sequential.todo",
    "describe.skipIf.shuffle.skip",
    "describe.skipIf.shuffle.only",
    "describe.skipIf.shuffle.concurrent",
    "describe.skipIf.shuffle.sequential",
    "describe.skipIf.shuffle.todo",
    "describe.skipIf.todo.skip",
    "describe.skipIf.todo.only",
    "describe.skipIf.todo.concurrent",
    "describe.skipIf.todo.sequential",
    "describe.skipIf.todo.shuffle",
    "describe.runIf.skip.only",
    "describe.runIf.skip.concurrent",
    "describe.runIf.skip.sequential",
    "describe.runIf.skip.shuffle",
    "describe.runIf.skip.todo",
    "describe.runIf.only.skip",
    "describe.runIf.only.concurrent",
    "describe.runIf.only.sequential",
    "describe.runIf.only.shuffle",
    "describe.runIf.only.todo",
    "describe.runIf.concurrent.skip",
    "describe.runIf.concurrent.only",
    "describe.runIf.concurrent.sequential",
    "describe.runIf.concurrent.shuffle",
    "describe.runIf.concurrent.todo",
    "describe.runIf.sequential.skip",
    "describe.runIf.sequential.only",
    "describe.runIf.sequential.concurrent",
    "describe.runIf.sequential.shuffle",
    "describe.runIf.sequential.todo",
    "describe.runIf.shuffle.skip",
    "describe.runIf.shuffle.only",
    "describe.runIf.shuffle.concurrent",
    "describe.runIf.shuffle.sequential",
    "describe.runIf.shuffle.todo",
    "describe.runIf.todo.skip",
    "describe.runIf.todo.only",
    "describe.runIf.todo.concurrent",
    "describe.runIf.todo.sequential",
    "describe.runIf.todo.shuffle",
    "describe.skip.only.each",
    "describe.skip.concurrent.each",
    "describe.skip.sequential.each",
    "describe.skip.shuffle.each",
    "describe.skip.todo.each",
    "describe.only.skip.each",
    "describe.only.concurrent.each",
    "describe.only.sequential.each",
    "describe.only.shuffle.each",
    "describe.only.todo.each",
    "describe.concurrent.skip.each",
    "describe.concurrent.only.each",
    "describe.concurrent.sequential.each",
    "describe.concurrent.shuffle.each",
    "describe.concurrent.todo.each",
    "describe.sequential.skip.each",
    "describe.sequential.only.each",
    "describe.sequential.concurrent.each",
    "describe.sequential.shuffle.each",
    "describe.sequential.todo.each",
    "describe.shuffle.skip.each",
    "describe.shuffle.only.each",
    "describe.shuffle.concurrent.each",
    "describe.shuffle.sequential.each",
    "describe.shuffle.todo.each",
    "describe.todo.skip.each",
    "describe.todo.only.each",
    "describe.todo.concurrent.each",
    "describe.todo.sequential.each",
    "describe.todo.shuffle.each",
    "describe.skipIf.skip.each",
    "describe.skipIf.only.each",
    "describe.skipIf.concurrent.each",
    "describe.skipIf.sequential.each",
    "describe.skipIf.shuffle.each",
    "describe.skipIf.todo.each",
    "describe.runIf.skip.each",
    "describe.runIf.only.each",
    "describe.runIf.concurrent.each",
    "describe.runIf.sequential.each",
    "describe.runIf.shuffle.each",
    "describe.runIf.todo.each",
    "suite",
    "suite.skip",
    "suite.only",
    "suite.concurrent",
    "suite.sequential",
    "suite.shuffle",
    "suite.todo",
    "suite.skipIf",
    "suite.runIf",
    "suite.each",
    "suite.skip.only",
    "suite.skip.concurrent",
    "suite.skip.sequential",
    "suite.skip.shuffle",
    "suite.skip.todo",
    "suite.only.skip",
    "suite.only.concurrent",
    "suite.only.sequential",
    "suite.only.shuffle",
    "suite.only.todo",
    "suite.concurrent.skip",
    "suite.concurrent.only",
    "suite.concurrent.sequential",
    "suite.concurrent.shuffle",
    "suite.concurrent.todo",
    "suite.sequential.skip",
    "suite.sequential.only",
    "suite.sequential.concurrent",
    "suite.sequential.shuffle",
    "suite.sequential.todo",
    "suite.shuffle.skip",
    "suite.shuffle.only",
    "suite.shuffle.concurrent",
    "suite.shuffle.sequential",
    "suite.shuffle.todo",
    "suite.todo.skip",
    "suite.todo.only",
    "suite.todo.concurrent",
    "suite.todo.sequential",
    "suite.todo.shuffle",
    "suite.skipIf.skip",
    "suite.skipIf.only",
    "suite.skipIf.concurrent",
    "suite.skipIf.sequential",
    "suite.skipIf.shuffle",
    "suite.skipIf.todo",
    "suite.runIf.skip",
    "suite.runIf.only",
    "suite.runIf.concurrent",
    "suite.runIf.sequential",
    "suite.runIf.shuffle",
    "suite.runIf.todo",
    "suite.skip.each",
    "suite.only.each",
    "suite.concurrent.each",
    "suite.sequential.each",
    "suite.shuffle.each",
    "suite.todo.each",
    "suite.skipIf.each",
    "suite.runIf.each",
    "suite.skip.only.concurrent",
    "suite.skip.only.sequential",
    "suite.skip.only.shuffle",
    "suite.skip.only.todo",
    "suite.skip.concurrent.only",
    "suite.skip.concurrent.sequential",
    "suite.skip.concurrent.shuffle",
    "suite.skip.concurrent.todo",
    "suite.skip.sequential.only",
    "suite.skip.sequential.concurrent",
    "suite.skip.sequential.shuffle",
    "suite.skip.sequential.todo",
    "suite.skip.shuffle.only",
    "suite.skip.shuffle.concurrent",
    "suite.skip.shuffle.sequential",
    "suite.skip.shuffle.todo",
    "suite.skip.todo.only",
    "suite.skip.todo.concurrent",
    "suite.skip.todo.sequential",
    "suite.skip.todo.shuffle",
    "suite.only.skip.concurrent",
    "suite.only.skip.sequential",
    "suite.only.skip.shuffle",
    "suite.only.skip.todo",
    "suite.only.concurrent.skip",
    "suite.only.concurrent.sequential",
    "suite.only.concurrent.shuffle",
    "suite.only.concurrent.todo",
    "suite.only.sequential.skip",
    "suite.only.sequential.concurrent",
    "suite.only.sequential.shuffle",
    "suite.only.sequential.todo",
    "suite.only.shuffle.skip",
    "suite.only.shuffle.concurrent",
    "suite.only.shuffle.sequential",
    "suite.only.shuffle.todo",
    "suite.only.todo.skip",
    "suite.only.todo.concurrent",
    "suite.only.todo.sequential",
    "suite.only.todo.shuffle",
    "suite.concurrent.skip.only",
    "suite.concurrent.skip.sequential",
    "suite.concurrent.skip.shuffle",
    "suite.concurrent.skip.todo",
    "suite.concurrent.only.skip",
    "suite.concurrent.only.sequential",
    "suite.concurrent.only.shuffle",
    "suite.concurrent.only.todo",
    "suite.concurrent.sequential.skip",
    "suite.concurrent.sequential.only",
    "suite.concurrent.sequential.shuffle",
    "suite.concurrent.sequential.todo",
    "suite.concurrent.shuffle.skip",
    "suite.concurrent.shuffle.only",
    "suite.concurrent.shuffle.sequential",
    "suite.concurrent.shuffle.todo",
    "suite.concurrent.todo.skip",
    "suite.concurrent.todo.only",
    "suite.concurrent.todo.sequential",
    "suite.concurrent.todo.shuffle",
    "suite.sequential.skip.only",
    "suite.sequential.skip.concurrent",
    "suite.sequential.skip.shuffle",
    "suite.sequential.skip.todo",
    "suite.sequential.only.skip",
    "suite.sequential.only.concurrent",
    "suite.sequential.only.shuffle",
    "suite.sequential.only.todo",
    "suite.sequential.concurrent.skip",
    "suite.sequential.concurrent.only",
    "suite.sequential.concurrent.shuffle",
    "suite.sequential.concurrent.todo",
    "suite.sequential.shuffle.skip",
    "suite.sequential.shuffle.only",
    "suite.sequential.shuffle.concurrent",
    "suite.sequential.shuffle.todo",
    "suite.sequential.todo.skip",
    "suite.sequential.todo.only",
    "suite.sequential.todo.concurrent",
    "suite.sequential.todo.shuffle",
    "suite.shuffle.skip.only",
    "suite.shuffle.skip.concurrent",
    "suite.shuffle.skip.sequential",
    "suite.shuffle.skip.todo",
    "suite.shuffle.only.skip",
    "suite.shuffle.only.concurrent",
    "suite.shuffle.only.sequential",
    "suite.shuffle.only.todo",
    "suite.shuffle.concurrent.skip",
    "suite.shuffle.concurrent.only",
    "suite.shuffle.concurrent.sequential",
    "suite.shuffle.concurrent.todo",
    "suite.shuffle.sequential.skip",
    "suite.shuffle.sequential.only",
    "suite.shuffle.sequential.concurrent",
    "suite.shuffle.sequential.todo",
    "suite.shuffle.todo.skip",
    "suite.shuffle.todo.only",
    "suite.shuffle.todo.concurrent",
    "suite.shuffle.todo.sequential",
    "suite.todo.skip.only",
    "suite.todo.skip.concurrent",
    "suite.todo.skip.sequential",
    "suite.todo.skip.shuffle",
    "suite.todo.only.skip",
    "suite.todo.only.concurrent",
    "suite.todo.only.sequential",
    "suite.todo.only.shuffle",
    "suite.todo.concurrent.skip",
    "suite.todo.concurrent.only",
    "suite.todo.concurrent.sequential",
    "suite.todo.concurrent.shuffle",
    "suite.todo.sequential.skip",
    "suite.todo.sequential.only",
    "suite.todo.sequential.concurrent",
    "suite.todo.sequential.shuffle",
    "suite.todo.shuffle.skip",
    "suite.todo.shuffle.only",
    "suite.todo.shuffle.concurrent",
    "suite.todo.shuffle.sequential",
    "suite.skipIf.skip.only",
    "suite.skipIf.skip.concurrent",
    "suite.skipIf.skip.sequential",
    "suite.skipIf.skip.shuffle",
    "suite.skipIf.skip.todo",
    "suite.skipIf.only.skip",
    "suite.skipIf.only.concurrent",
    "suite.skipIf.only.sequential",
    "suite.skipIf.only.shuffle",
    "suite.skipIf.only.todo",
    "suite.skipIf.concurrent.skip",
    "suite.skipIf.concurrent.only",
    "suite.skipIf.concurrent.sequential",
    "suite.skipIf.concurrent.shuffle",
    "suite.skipIf.concurrent.todo",
    "suite.skipIf.sequential.skip",
    "suite.skipIf.sequential.only",
    "suite.skipIf.sequential.concurrent",
    "suite.skipIf.sequential.shuffle",
    "suite.skipIf.sequential.todo",
    "suite.skipIf.shuffle.skip",
    "suite.skipIf.shuffle.only",
    "suite.skipIf.shuffle.concurrent",
    "suite.skipIf.shuffle.sequential",
    "suite.skipIf.shuffle.todo",
    "suite.skipIf.todo.skip",
    "suite.skipIf.todo.only",
    "suite.skipIf.todo.concurrent",
    "suite.skipIf.todo.sequential",
    "suite.skipIf.todo.shuffle",
    "suite.runIf.skip.only",
    "suite.runIf.skip.concurrent",
    "suite.runIf.skip.sequential",
    "suite.runIf.skip.shuffle",
    "suite.runIf.skip.todo",
    "suite.runIf.only.skip",
    "suite.runIf.only.concurrent",
    "suite.runIf.only.sequential",
    "suite.runIf.only.shuffle",
    "suite.runIf.only.todo",
    "suite.runIf.concurrent.skip",
    "suite.runIf.concurrent.only",
    "suite.runIf.concurrent.sequential",
    "suite.runIf.concurrent.shuffle",
    "suite.runIf.concurrent.todo",
    "suite.runIf.sequential.skip",
    "suite.runIf.sequential.only",
    "suite.runIf.sequential.concurrent",
    "suite.runIf.sequential.shuffle",
    "suite.runIf.sequential.todo",
    "suite.runIf.shuffle.skip",
    "suite.runIf.shuffle.only",
    "suite.runIf.shuffle.concurrent",
    "suite.runIf.shuffle.sequential",
    "suite.runIf.shuffle.todo",
    "suite.runIf.todo.skip",
    "suite.runIf.todo.only",
    "suite.runIf.todo.concurrent",
    "suite.runIf.todo.sequential",
    "suite.runIf.todo.shuffle",
    "suite.skip.only.each",
    "suite.skip.concurrent.each",
    "suite.skip.sequential.each",
    "suite.skip.shuffle.each",
    "suite.skip.todo.each",
    "suite.only.skip.each",
    "suite.only.concurrent.each",
    "suite.only.sequential.each",
    "suite.only.shuffle.each",
    "suite.only.todo.each",
    "suite.concurrent.skip.each",
    "suite.concurrent.only.each",
    "suite.concurrent.sequential.each",
    "suite.concurrent.shuffle.each",
    "suite.concurrent.todo.each",
    "suite.sequential.skip.each",
    "suite.sequential.only.each",
    "suite.sequential.concurrent.each",
    "suite.sequential.shuffle.each",
    "suite.sequential.todo.each",
    "suite.shuffle.skip.each",
    "suite.shuffle.only.each",
    "suite.shuffle.concurrent.each",
    "suite.shuffle.sequential.each",
    "suite.shuffle.todo.each",
    "suite.todo.skip.each",
    "suite.todo.only.each",
    "suite.todo.concurrent.each",
    "suite.todo.sequential.each",
    "suite.todo.shuffle.each",
    "suite.skipIf.skip.each",
    "suite.skipIf.only.each",
    "suite.skipIf.concurrent.each",
    "suite.skipIf.sequential.each",
    "suite.skipIf.shuffle.each",
    "suite.skipIf.todo.each",
    "suite.runIf.skip.each",
    "suite.runIf.only.each",
    "suite.runIf.concurrent.each",
    "suite.runIf.sequential.each",
    "suite.runIf.shuffle.each",
    "suite.runIf.todo.each",
    "xtest",
    "xtest.each",
    "xit",
    "xit.each",
    "fit",
    "xdescribe",
    "xdescribe.each",
    "fdescribe"
};
