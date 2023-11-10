use std::{borrow::Cow, cmp::Ordering};

use oxc_ast::{
    ast::{
        Argument, CallExpression, Expression, IdentifierName, IdentifierReference, MemberExpression,
    },
    AstKind,
};
use oxc_semantic::AstNode;
use oxc_span::{Atom, Span};

use crate::context::LintContext;

use crate::utils::jest::{is_pure_string, JestFnKind, JestGeneralFnKind};

pub fn parse_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    original: Option<&'a Atom>,
    ctx: &LintContext<'a>,
) -> Option<ParsedJestFnCall<'a>> {
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

        let name = resolved.original.unwrap_or(resolved.local).as_str();
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
            let options = ExpectFnCallOptions { call_expr, members, name, head, node, ctx };
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
            return parse_jest_jest_fn_call(members, name);
        }

        // Check every link in the chain except the last is a member expression
        if !all_member_expr_except_last {
            return None;
        }

        let mut call_chains = Vec::from([Cow::Borrowed(name)]);
        call_chains.extend(members.iter().filter_map(KnownMemberExpressionProperty::name));
        if !is_valid_jest_call(&call_chains) {
            return None;
        }

        return Some(ParsedJestFnCall::GeneralJestFnCall(ParsedGeneralJestFnCall {
            kind,
            members,
            name: Cow::Borrowed(name),
        }));
    }

    None
}

fn parse_jest_expect_fn_call<'a>(
    options: ExpectFnCallOptions<'a, '_>,
) -> Option<ParsedJestFnCall<'a>> {
    let ExpectFnCallOptions { call_expr, members, name, head, node, ctx } = options;
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
        let Some(parent) = ctx.nodes().parent_node(node.id()) else { return true };

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
) -> Option<ParsedJestFnCall<'a>> {
    if !name.to_ascii_lowercase().eq_ignore_ascii_case("jest") {
        return None;
    }

    return Some(ParsedJestFnCall::GeneralJestFnCall(ParsedGeneralJestFnCall {
        kind: JestFnKind::General(JestGeneralFnKind::Jest),
        members,
        name: Cow::Borrowed(name),
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

fn resolve_to_jest_fn<'a>(
    call_expr: &'a CallExpression<'a>,
    original: Option<&'a Atom>,
) -> Option<ResolvedJestFn<'a>> {
    let ident = resolve_first_ident(&call_expr.callee)?;
    Some(ResolvedJestFn { local: &ident.name, original })
}

fn resolve_first_ident<'a>(expr: &'a Expression) -> Option<&'a IdentifierReference> {
    match expr {
        Expression::Identifier(ident) => Some(ident),
        Expression::MemberExpression(member_expr) => resolve_first_ident(member_expr.object()),
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
    #[allow(unused)]
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
}

pub struct ParsedExpectFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    pub name: Cow<'a, str>,
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
    #[allow(unused)]
    pub fn matcher(&self) -> Option<&KnownMemberExpressionProperty<'a>> {
        let matcher_index = self.matcher_index?;
        self.members.get(matcher_index)
    }
    pub fn modifiers(&self) -> Vec<&KnownMemberExpressionProperty<'a>> {
        self.modifier_indices.iter().filter_map(|i| self.members.get(*i)).collect::<Vec<_>>()
    }
}

struct ResolvedJestFn<'a> {
    pub local: &'a Atom,
    pub original: Option<&'a Atom>,
}

#[derive(Clone, Copy, Debug)]
pub enum KnownMemberExpressionParentKind {
    Member,
    Call,
    TaggedTemplate,
}

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

pub enum MemberExpressionElement<'a> {
    Expression(&'a Expression<'a>),
    IdentName(&'a IdentifierName),
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
    #[allow(unused)]
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
        Expression::MemberExpression(member_expr) => {
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
