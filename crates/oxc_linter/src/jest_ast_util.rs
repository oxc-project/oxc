use std::{borrow::Cow, cmp::Ordering};

use oxc_ast::{
    ast::{
        Argument, CallExpression, Expression, IdentifierName, IdentifierReference,
        ImportDeclarationSpecifier, MemberExpression, ModuleDeclaration, TemplateLiteral,
    },
    AstKind,
};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{Atom, Span};

use crate::context::LintContext;

pub fn is_type_of_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    kinds: &[JestFnKind],
) -> bool {
    let jest_fn_call = parse_jest_fn_call(call_expr, node, ctx);
    if let Some(jest_fn_call) = jest_fn_call {
        let kind = jest_fn_call.kind();
        if kinds.contains(&kind) {
            return true;
        }
    }

    false
}

pub fn parse_general_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<ParsedGeneralJestFnCall<'a>> {
    let jest_fn_call = parse_jest_fn_call(call_expr, node, ctx)?;

    if let ParsedJestFnCall::GeneralJestFnCall(jest_fn_call) = jest_fn_call {
        return Some(jest_fn_call);
    }
    None
}

pub fn parse_expect_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<ParsedExpectFnCall<'a>> {
    let jest_fn_call = parse_jest_fn_call(call_expr, node, ctx)?;

    if let ParsedJestFnCall::ExpectFnCall(jest_fn_call) = jest_fn_call {
        return Some(jest_fn_call);
    }
    None
}

pub fn parse_jest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<ParsedJestFnCall<'a>> {
    let callee = &call_expr.callee;

    // If bailed out, we're not jest function
    let resolved = resolve_to_jest_fn(call_expr, ctx)?;

    // only the top level Call expression callee's parent is None, it's not necessary to set it to None, but
    // I didn't know how to pass Expression to it.
    let chain = get_node_chain(callee, None);
    let all_member_expr_except_last = chain
        .iter()
        .rev()
        .skip(1)
        .all(|member| matches!(member.parent, Some(Expression::MemberExpression(_))));

    // Ensure that we're at the "top" of the function call chain otherwise when
    // parsing e.g. x().y.z(), we'll incorrectly find & parse "x()" even though
    // the full chain is not a valid jest function call chain
    if ctx.nodes().parent_node(node.id()).is_some_and(|parent_node| {
        matches!(parent_node.kind(), AstKind::CallExpression(_) | AstKind::MemberExpression(_))
    }) {
        return None;
    }

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
        let iter = chain.into_iter().skip(1);
        let rest = iter;

        // every member node must have a member expression as their parent
        // in order to be part of the call chain we're parsing
        for member in rest {
            members.push(member);
        }

        if matches!(kind, JestFnKind::Expect) {
            return parse_jest_expect_fn_call(call_expr, members, name);
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
    call_expr: &'a CallExpression<'a>,
    members: Vec<KnownMemberExpressionProperty<'a>>,
    name: &'a str,
) -> Option<ParsedJestFnCall<'a>> {
    // check if the `member` is being called, which means it is the matcher
    let has_matcher = match &call_expr.callee {
        Expression::MemberExpression(member_expr) => member_expr.static_property_name().is_some(),
        _ => false,
    };

    let Some((modifiers, matcher)) = find_modifiers_and_matcher(&members, has_matcher) else {
        return None;
    };

    return Some(ParsedJestFnCall::ExpectFnCall(ParsedExpectFnCall {
        kind: JestFnKind::Expect,
        members,
        name: Cow::Borrowed(name),
        args: &call_expr.arguments,
        matcher_index: matcher,
        modifier_indices: modifiers,
    }));
}

fn find_modifiers_and_matcher(
    members: &[KnownMemberExpressionProperty],
    has_matcher: bool,
) -> Option<(Vec<usize>, Option<usize>)> {
    let mut matcher = None;
    let mut modifiers: Vec<usize> = vec![];

    // matcher is the end of the entire "expect" call chain
    if has_matcher {
        matcher = Some(members.len() - 1);
    }

    for (index, member) in members.iter().enumerate() {
        // the last member is the matcher, so we can stop here
        if index == members.len() - 1 {
            break;
        }

        // the first modifier can be any of the three modifiers
        if modifiers.is_empty() {
            if !member.is_name_in_modifiers(&[
                ModifierName::Not,
                ModifierName::Resolves,
                ModifierName::Rejects,
            ]) {
                return None;
            }
        } else if modifiers.len() == 1 {
            // the second modifier can only be "not"
            if !member.is_name_in_modifiers(&[ModifierName::Not]) {
                return None;
            }
            // and the first modifier has to be either "resolves" or "rejects"
            if !members[modifiers[0]]
                .is_name_in_modifiers(&[ModifierName::Resolves, ModifierName::Rejects])
            {
                return None;
            }
        } else {
            // the third modifier can only be "resolves" or "rejects"
            if !member.is_name_in_modifiers(&[ModifierName::Resolves, ModifierName::Rejects]) {
                return None;
            }
        }

        modifiers.push(index);
    }

    Some((modifiers, matcher))
}

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
    ctx: &LintContext<'a>,
) -> Option<ResolvedJestFn<'a>> {
    let ident = resolve_first_ident(&call_expr.callee)?;
    if ctx.semantic().is_reference_to_global_variable(ident) {
        // If the identifier is not a jest function, bail out
        if JestFnKind::from(&ident.name) == JestFnKind::Unknown {
            return None;
        }

        return Some(ResolvedJestFn {
            local: &ident.name,
            kind: JestFnFrom::Global,
            original: None,
        });
    }

    let node_id = get_import_decl_node_id(ident, ctx)?;
    let node = ctx.nodes().get_node(node_id);
    let AstKind::ModuleDeclaration(module_decl) = node.kind() else {
        return None;
    };
    let ModuleDeclaration::ImportDeclaration(import_decl) = module_decl else {
        return None;
    };

    if import_decl.source.value == "@jest/globals" {
        let original = import_decl.specifiers.iter().find_map(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                Some(import_specifier.imported.name())
            }
            _ => None,
        });

        return Some(ResolvedJestFn { local: &ident.name, kind: JestFnFrom::Import, original });
    }
    None
}

fn get_import_decl_node_id(ident: &IdentifierReference, ctx: &LintContext) -> Option<AstNodeId> {
    let symbol_table = ctx.semantic().symbols();
    let reference_id = ident.reference_id.get()?;
    let reference = symbol_table.get_reference(reference_id);
    // import binding is not a write reference
    if reference.is_write() {
        return None;
    }
    let symbol_id = reference.symbol_id()?;
    if symbol_table.get_flag(symbol_id).is_import_binding() {
        return Some(symbol_table.get_declaration(symbol_id));
    }

    None
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JestFnKind {
    Expect,
    General(JestGeneralFnKind),
    Unknown,
}

impl JestFnKind {
    pub fn from(name: &str) -> Self {
        match name {
            "expect" => Self::Expect,
            "jest" => Self::General(JestGeneralFnKind::Jest),
            "describe" | "fdescribe" | "xdescribe" => Self::General(JestGeneralFnKind::Describe),
            "fit" | "it" | "test" | "xit" | "xtest" => Self::General(JestGeneralFnKind::Test),
            "beforeAll" | "beforeEach" | "afterAll" | "afterEach" => {
                Self::General(JestGeneralFnKind::Hook)
            }
            _ => Self::Unknown,
        }
    }

    pub fn to_general(self) -> Option<JestGeneralFnKind> {
        match self {
            Self::General(kind) => Some(kind),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JestGeneralFnKind {
    Hook,
    Describe,
    Test,
    Jest,
}

pub enum ParsedJestFnCall<'a> {
    GeneralJestFnCall(ParsedGeneralJestFnCall<'a>),
    #[allow(unused)]
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
}

pub struct ParsedExpectFnCall<'a> {
    pub kind: JestFnKind,
    pub members: Vec<KnownMemberExpressionProperty<'a>>,
    pub name: Cow<'a, str>,
    pub args: &'a oxc_allocator::Vec<'a, Argument<'a>>,
    // In `expect(1).not.resolved.toBe()`, "not", "resolved" will be modifier
    // it save a group of modifier index from members
    #[allow(unused)]
    modifier_indices: Vec<usize>,
    // In `expect(1).toBe(2)`, "toBe" will be matcher
    // it save the matcher index from members
    matcher_index: Option<usize>,
}

impl<'a> ParsedExpectFnCall<'a> {
    pub fn matcher(&self) -> Option<&KnownMemberExpressionProperty<'a>> {
        let matcher_index = self.matcher_index?;
        self.members.get(matcher_index)
    }
}

struct ResolvedJestFn<'a> {
    pub local: &'a Atom,
    pub original: Option<&'a Atom>,
    #[allow(unused)]
    kind: JestFnFrom,
}

pub enum JestFnFrom {
    Global,
    Import,
}

pub struct KnownMemberExpressionProperty<'a> {
    pub element: MemberExpressionElement<'a>,
    pub parent: Option<&'a Expression<'a>>,
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
        let Some((span, _)) = member_expr.static_property_info() else { return None };
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

/// join name of the expression. e.g.
/// `expect(foo).toBe(bar)`  -> "expect.toBe"
/// `new Foo().bar` -> "Foo.bar"
pub fn get_node_name<'a>(expr: &'a Expression<'a>) -> String {
    let chain = get_node_name_vec(expr);
    chain.join(".")
}

pub fn get_node_name_vec<'a>(expr: &'a Expression<'a>) -> Vec<Cow<'a, str>> {
    let mut chain: Vec<Cow<'a, str>> = Vec::new();

    match expr {
        Expression::Identifier(ident) => chain.push(Cow::Borrowed(ident.name.as_str())),
        Expression::StringLiteral(string_literal) => {
            chain.push(Cow::Borrowed(&string_literal.value));
        }
        Expression::TemplateLiteral(template_literal) if is_pure_string(template_literal) => {
            chain.push(Cow::Borrowed(template_literal.quasi().unwrap()));
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            chain.extend(get_node_name_vec(&tagged_expr.tag));
        }
        Expression::CallExpression(call_expr) => chain.extend(get_node_name_vec(&call_expr.callee)),
        Expression::MemberExpression(member_expr) => {
            chain.extend(get_node_name_vec(member_expr.object()));
            if let Some(name) = member_expr.static_property_name() {
                chain.push(Cow::Borrowed(name));
            }
        }
        Expression::NewExpression(new_expr) => {
            chain.extend(get_node_name_vec(&new_expr.callee));
        }
        _ => {}
    };

    chain
}

/// Port from [eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest/blob/a058f22f94774eeea7980ea2d1f24c6808bf3e2c/src/rules/utils/parseJestFnCall.ts#L36-L51)
fn get_node_chain<'a>(
    expr: &'a Expression<'a>,
    parent: Option<&'a Expression<'a>>,
) -> Vec<KnownMemberExpressionProperty<'a>> {
    let mut chain = Vec::new();

    match expr {
        Expression::MemberExpression(member_expr) => {
            chain.extend(get_node_chain(member_expr.object(), Some(expr)));
            if let Some((span, element)) = MemberExpressionElement::from_member_expr(member_expr) {
                chain.push(KnownMemberExpressionProperty { element, parent: Some(expr), span });
            }
        }
        Expression::Identifier(ident) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent,
                span: ident.span,
            });
        }
        Expression::CallExpression(call_expr) => {
            let sub_chain = get_node_chain(&call_expr.callee, Some(expr));
            chain.extend(sub_chain);
        }
        Expression::TaggedTemplateExpression(tagged_expr) => {
            let sub_chain = get_node_chain(&tagged_expr.tag, Some(expr));
            chain.extend(sub_chain);
        }
        Expression::StringLiteral(string_literal) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent,
                span: string_literal.span,
            });
        }
        Expression::TemplateLiteral(template_literal) if is_pure_string(template_literal) => {
            chain.push(KnownMemberExpressionProperty {
                element: MemberExpressionElement::Expression(expr),
                parent,
                span: template_literal.span,
            });
        }
        _ => {}
    };

    chain
}

fn is_pure_string(template_literal: &TemplateLiteral) -> bool {
    template_literal.expressions.is_empty() && template_literal.quasis.len() == 1
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
