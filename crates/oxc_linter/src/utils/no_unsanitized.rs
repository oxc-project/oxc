//! Shared sanitization analysis for the `no-unsanitized` plugin.
//!
//! Ported from the `RuleHelper` of
//! <https://github.com/mozilla/eslint-plugin-no-unsanitized>.

use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, VariableDeclarationKind},
};
use oxc_semantic::SymbolId;
use oxc_span::GetSpan;
use oxc_str::CompactStr;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::context::LintContext;

/// Escaping functions whose output is considered safe HTML.
#[derive(Debug, Clone)]
pub struct EscapeConfig {
    pub tagged_templates: Vec<CompactStr>,
    pub methods: Vec<CompactStr>,
}

impl Default for EscapeConfig {
    fn default() -> Self {
        Self {
            tagged_templates: vec![
                CompactStr::new("Sanitizer.escapeHTML"),
                CompactStr::new("escapeHTML"),
            ],
            methods: vec![
                CompactStr::new("Sanitizer.unwrapSafeHTML"),
                CompactStr::new("unwrapSafeHTML"),
            ],
        }
    }
}

impl EscapeConfig {
    /// Applies the `escape` part of a user configuration on top of `self`.
    pub fn apply(&mut self, schema: &EscapeSchema) {
        if let Some(tagged_templates) = &schema.tagged_templates {
            self.tagged_templates =
                tagged_templates.iter().map(|s| CompactStr::from(s.as_str())).collect();
        }
        if let Some(methods) = &schema.methods {
            self.methods = methods.iter().map(|s| CompactStr::from(s.as_str())).collect();
        }
    }
}

/// The `escape` option of both `no-unsanitized` rules.
#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EscapeSchema {
    /// Tagged template functions which return safe HTML.
    /// Defaults to `["Sanitizer.escapeHTML", "escapeHTML"]`.
    pub tagged_templates: Option<Vec<String>>,
    /// Methods which return safe HTML.
    /// Defaults to `["Sanitizer.unwrapSafeHTML", "unwrapSafeHTML"]`.
    pub methods: Option<Vec<String>>,
}

/// A value flowing into an HTML sink.
#[derive(Debug, Clone, Copy)]
pub enum SinkValue<'a, 'b> {
    Expression(&'b Expression<'a>),
    /// The quasis of a tagged template, which are raw text and therefore always safe.
    Quasis,
    /// A value the rule cannot reason about, e.g. a spread element.
    Unsupported,
}

/// Decides whether a value is provably safe to pass into an HTML sink.
pub struct Sanitization<'e> {
    pub escape: &'e EscapeConfig,
    pub variable_tracing: bool,
}

impl Sanitization<'_> {
    pub fn is_allowed<'a>(&self, value: SinkValue<'a, '_>, ctx: &LintContext<'a>) -> bool {
        match value {
            SinkValue::Quasis => true,
            SinkValue::Unsupported => false,
            SinkValue::Expression(expression) => {
                let mut seen = FxHashSet::default();
                self.is_allowed_expression(expression, ctx, &mut seen)
            }
        }
    }

    fn is_allowed_expression<'a>(
        &self,
        expression: &Expression<'a>,
        ctx: &LintContext<'a>,
        seen: &mut FxHashSet<SymbolId>,
    ) -> bool {
        match expression {
            // A literal cannot carry attacker controlled markup, only malice.
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_) => true,
            // Only the `${...}` parts need checking, a quasi is raw text.
            // A template literal without interpolations is therefore always safe.
            Expression::TemplateLiteral(template) => template
                .expressions
                .iter()
                .all(|expression| self.is_allowed_expression(expression, ctx, seen)),
            Expression::TaggedTemplateExpression(tagged) => {
                is_allowed_callee(&tagged.tag, &self.escape.tagged_templates, ctx)
            }
            Expression::CallExpression(call) => {
                is_allowed_callee(&call.callee, &self.escape.methods, ctx)
            }
            Expression::BinaryExpression(binary) => {
                self.is_allowed_expression(&binary.left, ctx, seen)
                    && self.is_allowed_expression(&binary.right, ctx, seen)
            }
            Expression::ParenthesizedExpression(paren) => {
                self.is_allowed_expression(&paren.expression, ctx, seen)
            }
            Expression::TSAsExpression(_)
            | Expression::TSSatisfiesExpression(_)
            | Expression::TSNonNullExpression(_)
            | Expression::TSTypeAssertion(_)
            | Expression::TSInstantiationExpression(_) => {
                self.is_allowed_expression(expression.get_inner_expression(), ctx, seen)
            }
            Expression::Identifier(identifier) => self.is_allowed_identifier(identifier, ctx, seen),
            _ => false,
        }
    }

    /// Traces an identifier back to its declaration.
    ///
    /// Only `let`/`const` bindings whose initializer and every write reference are
    /// themselves allowed expressions are considered safe.
    fn is_allowed_identifier<'a>(
        &self,
        identifier: &IdentifierReference<'a>,
        ctx: &LintContext<'a>,
        seen: &mut FxHashSet<SymbolId>,
    ) -> bool {
        if !self.variable_tracing {
            return false;
        }
        // Unresolved references (globals, implicit assignments) cannot be traced.
        let Some(symbol_id) = ctx.scoping().get_reference(identifier.reference_id()).symbol_id()
        else {
            return false;
        };
        // Guard against cycles such as `let a = ''; a = `${a}<p>`;`.
        if !seen.insert(symbol_id) {
            return false;
        }

        let allowed = self.is_allowed_symbol(symbol_id, ctx, seen);
        seen.remove(&symbol_id);
        allowed
    }

    fn is_allowed_symbol(
        &self,
        symbol_id: SymbolId,
        ctx: &LintContext<'_>,
        seen: &mut FxHashSet<SymbolId>,
    ) -> bool {
        let scoping = ctx.scoping();
        let declaration = ctx.nodes().get_node(scoping.symbol_declaration(symbol_id));
        let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
            // Function parameters, imports, classes, ... are not traceable.
            return false;
        };
        // `var` can be overwritten in ways that are not visible here.
        if !matches!(declarator.kind, VariableDeclarationKind::Let | VariableDeclarationKind::Const)
        {
            return false;
        }
        if let Some(init) = &declarator.init
            && !self.is_allowed_expression(init, ctx, seen)
        {
            return false;
        }

        scoping.get_resolved_references(symbol_id).filter(|reference| reference.is_write()).all(
            |reference| {
                write_expression(ctx, reference.node_id())
                    .is_some_and(|expression| self.is_allowed_expression(expression, ctx, seen))
            },
        )
    }
}

/// The right-hand side of the assignment a write reference belongs to.
fn write_expression<'a, 'b>(
    ctx: &'b LintContext<'a>,
    node_id: oxc_semantic::NodeId,
) -> Option<&'b Expression<'a>> {
    match ctx.nodes().parent_kind(node_id) {
        AstKind::AssignmentExpression(assignment) => Some(&assignment.right),
        _ => None,
    }
}

fn is_allowed_callee<'a>(
    callee: &Expression<'a>,
    allowed: &[CompactStr],
    ctx: &LintContext<'a>,
) -> bool {
    let Some(name) = callee_code_name(callee, ctx) else {
        return false;
    };
    allowed.iter().any(|candidate| candidate.as_str() == name)
}

/// `foo` for `foo()`, `obj.foo` for `obj.foo()`, using source text for
/// non-identifier objects, matching the upstream `getCodeName` helper.
pub fn callee_code_name<'a>(callee: &Expression<'a>, ctx: &LintContext<'a>) -> Option<String> {
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::StaticMemberExpression(member) => {
            Some(format!("{}.{}", object_code_name(&member.object, ctx), member.property.name))
        }
        _ => None,
    }
}

/// Name of the object a method is called on, as used for `objectMatches`.
pub fn object_code_name<'a>(object: &Expression<'a>, ctx: &LintContext<'a>) -> String {
    match object {
        Expression::Identifier(identifier) => identifier.name.to_string(),
        object => ctx.source_range(object.span()).to_string(),
    }
}
