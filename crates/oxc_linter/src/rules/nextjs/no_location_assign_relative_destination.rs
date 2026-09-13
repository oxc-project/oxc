use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, Expression, IdentifierReference, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{GlobalContext, ToBoolean, ToJsString};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_location_assign_relative_destination_diagnostic(
    span: Span,
    expression: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not use `{expression}` to navigate to internal Next.js pages."
    ))
        .with_help(
            "Use `redirect()` during rendering, or `useRouter().push()` in a Client Component event handler.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLocationAssignRelativeDestination;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents assignments to `location.href` and calls to `location.assign()`
    /// with relative URLs.
    ///
    /// ### Why is this bad?
    ///
    /// Next.js cannot apply client-side routing optimizations when browser location
    /// APIs navigate directly to an internal page. Use Next.js navigation APIs to
    /// avoid a full page reload.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// location.href = "/dashboard";
    /// window.location.assign("/profile");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// location.href = "https://example.com";
    /// window.location.assign(externalUrl);
    /// ```
    NoLocationAssignRelativeDestination,
    nextjs,
    correctness,
    version = "next",
    short_description = "Prevents browser location APIs from navigating to internal Next.js pages.",
);

impl Rule for NoLocationAssignRelativeDestination {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => {
                let Some(callee) = call.callee.as_member_expression() else {
                    return;
                };
                if !is_named_property(callee, "assign") {
                    return;
                }

                let Some(root) = get_location_root_identifier(callee.object()) else {
                    return;
                };
                if !ctx.is_reference_to_global_variable(root) {
                    return;
                }

                let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression())
                else {
                    return;
                };
                if !has_relative_url_prefix(argument, ctx) {
                    return;
                }

                let expression = format!("{}()", ctx.source_range(callee.span()));
                ctx.diagnostic(no_location_assign_relative_destination_diagnostic(
                    call.span,
                    &expression,
                ));
            }
            AstKind::AssignmentExpression(assignment) => {
                let Some(target) = assignment.left.as_member_expression() else {
                    return;
                };
                if !is_named_property(target, "href") {
                    return;
                }

                let Some(root) = get_location_root_identifier(target.object()) else {
                    return;
                };
                if !ctx.is_reference_to_global_variable(root)
                    || !has_relative_url_prefix(&assignment.right, ctx)
                {
                    return;
                }

                ctx.diagnostic(no_location_assign_relative_destination_diagnostic(
                    assignment.span,
                    ctx.source_range(target.span()),
                ));
            }
            _ => {}
        }
    }
}

const GLOBAL_LOCATION_PREFIXES: [&str; 4] = ["window", "globalThis", "document", "self"];

fn is_named_property(member: &MemberExpression<'_>, name: &str) -> bool {
    match member {
        MemberExpression::StaticMemberExpression(member) => member.property.name == name,
        MemberExpression::ComputedMemberExpression(member) => {
            matches!(&member.expression, Expression::StringLiteral(literal) if literal.value == name)
        }
        MemberExpression::PrivateFieldExpression(_) => false,
    }
}

fn get_location_root_identifier<'a>(
    object: &'a Expression<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    let object = object.get_inner_expression();
    if let Expression::Identifier(identifier) = object
        && identifier.name == "location"
    {
        return Some(identifier);
    }

    let location = object.as_member_expression()?;
    if !is_named_property(location, "location") {
        return None;
    }

    let root = location.object().get_inner_expression().get_identifier_reference()?;
    GLOBAL_LOCATION_PREFIXES.contains(&root.name.as_str()).then_some(root)
}

fn has_relative_url_prefix<'a>(expression: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    static_string_prefix(expression, ctx, 0).is_some_and(|value| !is_absolute_url(&value))
}

struct RuleGlobalContext<'c, 'a>(&'c LintContext<'a>);

impl<'a> GlobalContext<'a> for RuleGlobalContext<'_, 'a> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool {
        self.0.is_reference_to_global_variable(reference)
    }
}

fn static_string_prefix<'a>(
    expression: &Expression<'a>,
    ctx: &LintContext<'a>,
    depth: u8,
) -> Option<Cow<'a, str>> {
    if depth >= 16 {
        return None;
    }

    if let Some(value) = static_string_value(expression, ctx, depth) {
        return Some(value);
    }

    match expression.get_inner_expression() {
        Expression::TemplateLiteral(template) => {
            let quasi = template.quasis.first()?;
            let value = quasi.value.cooked.as_ref().unwrap_or(&quasi.value.raw);
            Some(Cow::Borrowed(value.as_str()))
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            static_string_prefix(&binary.left, ctx, depth + 1)
        }
        Expression::Identifier(identifier) => last_write_expression(identifier, ctx)
            .and_then(|value| static_string_prefix(value, ctx, depth + 1)),
        _ => None,
    }
}

fn static_string_value<'a>(
    expression: &Expression<'a>,
    ctx: &LintContext<'a>,
    depth: u8,
) -> Option<Cow<'a, str>> {
    if depth >= 16 {
        return None;
    }

    let expression = expression.get_inner_expression();
    match expression {
        Expression::Identifier(identifier) => last_write_expression(identifier, ctx)
            .and_then(|value| static_string_value(value, ctx, depth + 1)),
        Expression::TemplateLiteral(template) => {
            let first = template.quasis.first()?.value.cooked.as_ref()?;
            if template.expressions.is_empty() {
                return Some(Cow::Borrowed(first.as_str()));
            }

            let mut value = first.as_str().to_owned();
            for (substitution, quasi) in
                template.expressions.iter().zip(template.quasis.iter().skip(1))
            {
                value.push_str(&static_string_value(substitution, ctx, depth + 1)?);
                value.push_str(quasi.value.cooked.as_ref()?.as_str());
            }
            Some(Cow::Owned(value))
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            if !is_definitely_string(&binary.left, ctx, depth + 1)
                && !is_definitely_string(&binary.right, ctx, depth + 1)
            {
                return None;
            }
            let left = static_string_value(&binary.left, ctx, depth + 1)?;
            let right = static_string_value(&binary.right, ctx, depth + 1)?;
            let mut value = left.into_owned();
            value.push_str(&right);
            Some(Cow::Owned(value))
        }
        Expression::ConditionalExpression(conditional) => {
            let branch = if static_truthiness(&conditional.test, ctx, depth + 1)? {
                &conditional.consequent
            } else {
                &conditional.alternate
            };
            static_string_value(branch, ctx, depth + 1)
        }
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().and_then(|value| static_string_value(value, ctx, depth + 1))
        }
        Expression::CallExpression(call) => {
            let callee = call.callee.get_identifier_reference()?;
            if callee.name != "String" || !ctx.is_reference_to_global_variable(callee) {
                return None;
            }

            call.arguments.first().map_or(Some(Cow::Borrowed("")), |argument| {
                static_string_value(argument.as_expression()?, ctx, depth + 1)
            })
        }
        _ => expression.to_js_string(&RuleGlobalContext(ctx)),
    }
}

fn is_definitely_string<'a>(expression: &Expression<'a>, ctx: &LintContext<'a>, depth: u8) -> bool {
    if depth >= 16 {
        return false;
    }

    match expression.get_inner_expression() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::Identifier(identifier) => last_write_expression(identifier, ctx)
            .is_some_and(|value| is_definitely_string(value, ctx, depth + 1)),
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            is_definitely_string(&binary.left, ctx, depth + 1)
                || is_definitely_string(&binary.right, ctx, depth + 1)
        }
        Expression::ConditionalExpression(conditional) => {
            let Some(test) = static_truthiness(&conditional.test, ctx, depth + 1) else {
                return false;
            };
            let branch = if test { &conditional.consequent } else { &conditional.alternate };
            is_definitely_string(branch, ctx, depth + 1)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(|value| is_definitely_string(value, ctx, depth + 1)),
        Expression::CallExpression(call) => {
            call.callee.get_identifier_reference().is_some_and(|callee| {
                callee.name == "String" && ctx.is_reference_to_global_variable(callee)
            })
        }
        _ => false,
    }
}

fn static_truthiness<'a>(
    expression: &Expression<'a>,
    ctx: &LintContext<'a>,
    depth: u8,
) -> Option<bool> {
    if depth >= 16 {
        return None;
    }

    let expression = expression.get_inner_expression();
    match expression {
        Expression::Identifier(identifier) => last_write_expression(identifier, ctx)
            .and_then(|value| static_truthiness(value, ctx, depth + 1)),
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().and_then(|value| static_truthiness(value, ctx, depth + 1))
        }
        _ => expression.to_boolean(&RuleGlobalContext(ctx)),
    }
}

fn last_write_expression<'a>(
    identifier: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let symbol_id = ctx.scoping().get_reference(identifier.reference_id()).symbol_id()?;
    let declaration = ctx.symbol_declaration(symbol_id);
    let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
        return None;
    };

    let mut latest = declarator.init.as_ref().map(|initial| (declarator.span.start, initial));
    for reference in ctx.symbol_references(symbol_id) {
        if !reference.is_write() {
            continue;
        }

        let reference_span = ctx.nodes().get_node(reference.node_id()).kind().span();
        if reference_span.start >= identifier.span.start {
            continue;
        }

        let AstKind::AssignmentExpression(assignment) =
            ctx.nodes().parent_kind(reference.node_id())
        else {
            continue;
        };
        let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left else {
            continue;
        };
        if ctx.scoping().get_reference(target.reference_id()).symbol_id() != Some(symbol_id) {
            continue;
        }

        if latest.is_none_or(|(position, _)| reference_span.start > position) {
            latest = Some((reference_span.start, &assignment.right));
        }
    }

    latest.map(|(_, expression)| expression)
}

fn is_absolute_url(value: &str) -> bool {
    if value.starts_with("//") {
        return true;
    }

    let mut chars = value.chars();
    if !chars.next().is_some_and(|first| first.is_ascii_alphabetic()) {
        return false;
    }

    for character in chars {
        if character == ':' {
            return true;
        }
        if !(character.is_ascii_alphanumeric() || matches!(character, '+' | '.' | '-')) {
            return false;
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // no-location-assign-relative-destination
        "location.href = 'https://example.com'",
        "location.href = 'https://example.com/path?q=1'",
        "window.location.href = 'https://example.com'",
        "globalThis.location.href = 'https://example.com'",
        "location.assign('https://example.com')",
        "window.location.assign('https://example.com')",
        "globalThis.location.assign('https://example.com')",
        "location.href = '//example.com/path'",
        "location.assign('//cdn.example.com/file.js')",
        "location.href = 'ftp://files.example.com'",
        "location.href = 'mailto:user@example.com'",
        "location.href = 'CUSTOM+SCHEME:value'",
        "location.href = 'https' + '://example.com/path'",
        "location.href = `https://example.com/${path}`",
        "location.href = `${'https://example.com'}`",
        "location.href = true ? 'https://example.com' : '/dashboard'",
        "location.href = String('https://example.com')",
        "location.assign(`https://example.com/${path}`)",
        "location.href = someVariable",
        "location.assign(someVariable)",
        "window.location.href = computedUrl()",
        "window.location.assign(computedUrl())",
        "location.assign()",
        "location.assign(...urls)",
        "location[`assign`]('/foo')",
        "location[`href`] = '/foo'",
        "
                  const url = 'https://example.com';
                  location.href = url;
                  location.assign(url);
                ",
        "
                  const url = 'https://example.com/' + someVariable;
                  location.href = url;
                  location.assign(url);
                ",
        "
                  const url = `https://example.com/${someVariable}`;
                  location.href = url;
                  location.assign(url);
                ",
        "foo.location.href = '/path'",
        "foo.location.assign('/path')",
        "
                  let url = '/dashboard';
                  url = 'https://example.com';
                  location.href = url;
                ",
        "
                  const location = { href: '' };
                  location.href = '/foo'
                ",
        "function handler(location) { location.href = '/foo'; location.assign('/foo') }",
        "
                  const window = { location: { href: '' } };
                  window.location.href = '/foo'
                ",
        "
                  function handler(globalThis) {
                    globalThis.location.assign('/foo')
                  }
                ",
        "function handler(document) { document.location.href = '/foo' }",
        "function handler(self) { self.location.assign('/foo') }",
        "function handler(String) { location.href = String('/foo') }",
        "const protocol = 'https'; location.href = `${protocol}://example.com`",
        "
                  import { location } from './my-module';
                  location.href = '/foo'
                ",
    ];

    let fail = vec![
        // no-location-assign-relative-destination
        "location.href = '/foo'",
        "location['href'] = '/foo'",
        "location.href = `/users/${id}`",
        "window.location.href = '/foo'",
        "window.location.href = '/dashboard'",
        "window.location['href'] = '/foo'",
        "window.location['href'] = '/dashboard'",
        "globalThis.location.href = '/foo'",
        "globalThis.location.href = '/dashboard'",
        "document.location.href = '/foo'",
        "self.location.href = '/foo'",
        "window['location'].href = '/foo'",
        "location.assign('/foo')",
        "location.assign('/dashboard')",
        "location['assign']('/foo')",
        "location['assign']('/dashboard')",
        "location.assign(`/users/${id}/profile`)",
        "window.location.assign('/foo')",
        "window.location.assign('/dashboard')",
        "window.location['assign']('/foo')",
        "globalThis.location.assign('/foo')",
        "globalThis.location.assign('/dashboard')",
        "document.location.assign('/foo')",
        "self.location.assign('/foo')",
        "globalThis['location']['assign']('/foo')",
        "location.href = './page'",
        "location.href = '../page'",
        "location.assign('?tab=settings')",
        "location.assign('#section')",
        "location.href = true ? '/dashboard' : 'https://example.com'",
        "location.href = (0, '/dashboard')",
        "location.href = String('/dashboard')",
        "location.href = String()",
        "location.href = (true + false) + ':'",
        "const useInternal = true; const url = useInternal ? '/dashboard' : 'https://example.com'; location.href = url",
        "
                      const url = '/dashboard';
                      location.href = url;",
        "
                      const url = '/dashboard/' + someVariable;
                      location.href = url;",
        "
                      const url = `/dashboard/${someVariable}`;
                      location.href = url;",
        "
                      let url = 'https://example.com';
                      url = '/other-path';
                      location.href = url;",
        "
                      let url = '/dashboard';
                      location.href = url;
                      url = 'https://example.com';",
        "
                      function handleClick() {
                        window.location.href = '/dashboard'
                      }",
        "
                      function handleClick() {
                        location.assign('/dashboard')
                      }",
    ];

    Tester::new(
        NoLocationAssignRelativeDestination::NAME,
        NoLocationAssignRelativeDestination::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
