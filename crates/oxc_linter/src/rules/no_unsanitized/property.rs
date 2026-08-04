use oxc_ast::{
    AstKind,
    ast::{
        AssignmentOperator, AssignmentTarget, Expression, IdentifierReference,
        VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::Rule};

fn unsafe_assignment_diagnostic(property: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unsafe assignment to {property}"))
        .with_help(
            "Assign a string literal, or wrap the value in an escaping function such as `Sanitizer.escapeHTML`.",
        )
        .with_label(span)
}

/// Escaping functions which mark a value as safe.
#[derive(Debug, Clone)]
struct Escape {
    tagged_templates: Vec<CompactStr>,
    methods: Vec<CompactStr>,
}

impl Default for Escape {
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

#[derive(Debug, Clone)]
pub struct PropertyConfig {
    /// Property names which are treated as HTML sinks.
    properties: Vec<CompactStr>,
    escape: Escape,
    variable_tracing: bool,
}

impl Default for PropertyConfig {
    fn default() -> Self {
        Self {
            properties: vec![CompactStr::new("innerHTML"), CompactStr::new("outerHTML")],
            escape: Escape::default(),
            variable_tracing: true,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Property(Box<PropertyConfig>);

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EscapeSchema {
    /// Tagged template functions which return safe HTML.
    /// Defaults to `["Sanitizer.escapeHTML", "escapeHTML"]`.
    tagged_templates: Option<Vec<String>>,
    /// Methods which return safe HTML.
    /// Defaults to `["Sanitizer.unwrapSafeHTML", "unwrapSafeHTML"]`.
    methods: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PropertyOptionsSchema {
    /// Escaping functions which mark a value as safe.
    escape: Option<EscapeSchema>,
    /// Whether values assigned from local `let`/`const` variables are traced back
    /// to their initializers. Defaults to `true`.
    variable_tracing: Option<bool>,
    /// Property names treated as HTML sinks, replacing the defaults
    /// `["innerHTML", "outerHTML"]`.
    properties: Option<Vec<String>>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows assigning values that are not provably safe to DOM properties
    /// which parse their input as HTML, such as `innerHTML` and `outerHTML`.
    ///
    /// This is a port of the `property` rule of
    /// [`eslint-plugin-no-unsanitized`](https://github.com/mozilla/eslint-plugin-no-unsanitized).
    ///
    /// ### Why is this bad?
    ///
    /// Assigning a dynamic value to an HTML sink is the most common source of DOM
    /// based cross-site scripting. Only string literals, template literals whose
    /// interpolations are themselves safe, and the output of a known escaping
    /// function can be assumed not to introduce markup an attacker controls.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// node.innerHTML = htmlString;
    /// node.innerHTML = "<span>" + userInput + "</span>";
    /// node.innerHTML = `<span>${userInput}</span>`;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// node.innerHTML = "<span>static</span>";
    /// node.innerHTML = ``;
    /// node.innerHTML = Sanitizer.escapeHTML`<span>${userInput}</span>`;
    /// node.innerHTML = Sanitizer.unwrapSafeHTML(safeHtml);
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "no-unsanitized/property": [
    ///     "error",
    ///     {
    ///       "escape": {
    ///         "taggedTemplates": ["safeHTML"],
    ///         "methods": ["DOMPurify.sanitize"]
    ///       },
    ///       "properties": ["innerHTML", "outerHTML", "srcdoc"],
    ///       "variableTracing": true
    ///     }
    ///   ]
    /// }
    /// ```
    Property,
    no_unsanitized,
    restriction,
    config = PropertyOptionsSchema,
    version = "1.78.0",
    short_description = "Disallow unsanitized assignment to DOM properties that parse HTML.",
);

impl Rule for Property {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let mut config = PropertyConfig::default();

        let options = value.get(0).cloned().unwrap_or(serde_json::Value::Null);
        if options.is_null() {
            return Ok(Self(Box::new(config)));
        }
        let options: PropertyOptionsSchema = serde_json::from_value(options)?;

        if let Some(escape) = options.escape {
            if let Some(tagged_templates) = escape.tagged_templates {
                config.escape.tagged_templates =
                    tagged_templates.iter().map(|s| CompactStr::from(s.as_str())).collect();
            }
            if let Some(methods) = escape.methods {
                config.escape.methods =
                    methods.iter().map(|s| CompactStr::from(s.as_str())).collect();
            }
        }
        if let Some(variable_tracing) = options.variable_tracing {
            config.variable_tracing = variable_tracing;
        }
        if let Some(properties) = options.properties {
            config.properties = properties.iter().map(|s| CompactStr::from(s.as_str())).collect();
        }

        Ok(Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assignment) = node.kind() else {
            return;
        };
        if !is_checked_operator(assignment.operator) {
            return;
        }
        let Some(property) = assigned_property_name(&assignment.left) else {
            return;
        };
        if !self.0.properties.iter().any(|p| p == property) {
            return;
        }

        let mut seen = FxHashSet::default();
        if !self.is_allowed_expression(&assignment.right, ctx, &mut seen) {
            ctx.diagnostic(unsafe_assignment_diagnostic(property, assignment.span));
        }
    }
}

impl Property {
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
                is_allowed_callee(&tagged.tag, &self.0.escape.tagged_templates, ctx)
            }
            Expression::CallExpression(call) => {
                is_allowed_callee(&call.callee, &self.0.escape.methods, ctx)
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
        if !self.0.variable_tracing {
            return false;
        }
        let scoping = ctx.scoping();
        // Unresolved references (globals, implicit assignments) cannot be traced.
        let Some(symbol_id) = scoping.get_reference(identifier.reference_id()).symbol_id() else {
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

/// Operators which need checking.
///
/// Arithmetic and bitwise compound assignments such as `x.innerHTML *= 2` cannot
/// introduce markup and are ignored, matching the upstream rule.
fn is_checked_operator(operator: AssignmentOperator) -> bool {
    matches!(
        operator,
        AssignmentOperator::Assign
            | AssignmentOperator::Addition
            | AssignmentOperator::LogicalOr
            | AssignmentOperator::LogicalAnd
            | AssignmentOperator::LogicalNullish
    )
}

/// Name of the statically known property being assigned to, e.g. `innerHTML` for
/// `node.innerHTML = x`. Computed accesses are not resolved, as upstream.
fn assigned_property_name<'a>(target: &AssignmentTarget<'a>) -> Option<&'a str> {
    let simple = target.as_simple_assignment_target()?;
    let member = match simple.get_expression() {
        // `(a.b as T) = c` and friends
        Some(expression) => expression.get_inner_expression().get_member_expr(),
        None => simple.as_member_expression(),
    }?;
    match member {
        oxc_ast::ast::MemberExpression::StaticMemberExpression(member) => {
            Some(member.property.name.as_str())
        }
        _ => None,
    }
}

/// `foo` for `foo()`, `obj.foo` for `obj.foo()`, using source text for
/// non-identifier objects, matching the upstream `getCodeName` helper.
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

fn callee_code_name<'a>(callee: &Expression<'a>, ctx: &LintContext<'a>) -> Option<String> {
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::StaticMemberExpression(member) => {
            let object = match &member.object {
                Expression::Identifier(identifier) => identifier.name.to_string(),
                object => ctx.source_range(object.span()).to_string(),
            };
            Some(format!("{object}.{}", member.property.name))
        }
        _ => None,
    }
}

#[test]
#[expect(clippy::literal_string_with_formatting_args)] // test cases contain `${...}` in JS strings
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        // literals and template literals without interpolation
        ("a.innerHTML = '';", None),
        ("a.innerHTML *= 'test';", None),
        ("c.innerHTML = ``;", None),
        ("a.innerHTML += '';", None),
        ("b.innerHTML += \"\";", None),
        ("c.innerHTML += ``;", None),
        ("x.innerHTML = `foo`+`bar`;", None),
        ("y.innerHTML = '<span>' + 5 + '</span>';", None),
        ("u.innerHTML = `<span>${'lulz'}</span>`;", None),
        ("v.innerHTML = `<span>${'lulz'}</span>${55}`;", None),
        ("w.innerHTML = `<span>${'lulz'+'meh'}</span>`;", None),
        // escaping functions
        ("g.innerHTML = Sanitizer.escapeHTML``;", None),
        ("h.innerHTML = Sanitizer.escapeHTML`foo`;", None),
        ("i.innerHTML = Sanitizer.escapeHTML`foo${bar}baz`;", None),
        ("g.innerHTML += Sanitizer.escapeHTML``;", None),
        ("h.innerHTML += Sanitizer.escapeHTML`foo`;", None),
        ("i.innerHTML += Sanitizer.escapeHTML`foo${bar}baz`;", None),
        ("i.innerHTML += Sanitizer.unwrapSafeHTML(htmlSnippet)", None),
        ("i.outerHTML += Sanitizer.unwrapSafeHTML(htmlSnippet)", None),
        ("this.imeList.innerHTML = Sanitizer.unwrapSafeHTML(...listHtml);", None),
        // not a sink
        ("document.toString = evil;", None),
        ("document.writeln(Sanitizer.escapeHTML`<em>${evil}</em>`);", None),
        // configured escapers
        (
            "w.innerHTML = templateEscaper`<em>${evil}</em>`;",
            Some(json!([{"escape": {"taggedTemplates": ["templateEscaper"]}}])),
        ),
        (
            "w.innerHTML = DOMPurify.sanitize('<em>${evil}</em>');",
            Some(json!([{"escape": {"methods": ["DOMPurify.sanitize"]}}])),
        ),
        // variable tracing
        ("let c; c = 123; a.innerHTML = `${c}`;", None),
        ("let c; a.innerHTML = `${c}`;", None),
        ("let literalFromElsewhere = '<b>safe</b>'; y.innerHTML = literalFromElsewhere;", None),
        (
            "const literalFromElsewhereWithInnerExpr = '<b>safe</b>'+'yo'; y.innerHTML = literalFromElsewhereWithInnerExpr;",
            None,
        ),
        (
            "let multiStepVarSearch = '<b>safe</b>'+'yo'; const copy = multiStepVarSearch; y.innerHTML = copy;",
            None,
        ),
        ("let copies = '<b>safe</b>'; copies = 'stillOK'; y.innerHTML = copies;", None),
        (
            "let copies = '<b>safe</b>'; if (monday) { copies = 'stillOK'; }; y.innerHTML = copies;",
            None,
        ),
        (
            "let msg = '<b>safe</b>'; let altMsg = 'also cool';  if (monday) { msg = altMsg; }; y.innerHTML = msg;",
            None,
        ),
    ];

    let fail = vec![
        ("m.innerHTML = htmlString;", None),
        ("a.innerHTML += htmlString;", None),
        ("a.innerHTML += template.toHtml();", None),
        ("m.outerHTML = htmlString;", None),
        ("t.innerHTML = `<span>${name}</span>`;", None),
        ("t.innerHTML = `<span>${'foobar'}</span>${evil}`;", None),
        ("node.innerHTML = '<span>'+ htmlInput;", None),
        ("node.innerHTML = '<span>'+ htmlInput + '</span>';", None),
        ("title.innerHTML = _('WB_LT_TIPS_S_SEARCH', {value0:engine});", None),
        ("x.innerHTML = Sanitizer.escapeHTML(evil)", None),
        ("x.innerHTML = Sanitizer.escapeHTML(`evil`)", None),
        ("y.innerHTML = ((arrow_function)=>null)`some HTML`", None),
        ("this.imeList.innerHTML = Sanitizer.unrapSafeHTML(...listHtml);", None),
        ("g.innerHTML = potentiallyUnsafe;", None),
        ("function foo() { return this().innerHTML = evil; };", None),
        ("describe.each`table${m.innerHTML = htmlString}`(name, fn, timeout)", None),
        ("a.innerHTML = somefn()()", None),
        ("a.innerHTML = (cond ? maybe_safe : or_evil)()", None),
        ("yoink.innerHTML &&= bar;", None),
        ("yoink.innerHTML ||= bar;", None),
        ("yoink.innerHTML ??= bar;", None),
        // variable tracing
        ("copy = '<b>safe</b>'; copy = evil; y.innerHTML = copy;", None),
        ("let copies = '<b>safe</b>'; copies = suddenlyUnsafe; y.innerHTML = copies;", None),
        (
            "let copies = '<b>safe</b>'; if (monday) { copies = badness }; y.innerHTML = copies;",
            None,
        ),
        (
            "let copies = '<b>safe</b>'; (() => { copies = badness; })(); y.innerHTML = copies;",
            None,
        ),
        (
            "let obj = { prop: '<b>safe</b>' }; doSomething(obj); let copies = obj.prop; y.innerHTML = copies;",
            None,
        ),
        ("let c; if (cond) { c = '<b>safe</b>'; } else { c = evil; } a.innerHTML = `${c}`;", None),
        ("let c; c = 'apparently-safe'; functionCall(c); n.innerHTML = c.property;", None),
        ("let text = ''; text = `${text}<p>`; scratchDiv.innerHTML = text;", None),
        (
            "let msg = '<b>safe</b>'; let altMsg = 'also cool';  if (monday) { msg = altMsg; }; y.innerHTML = msg;",
            Some(json!([{"variableTracing": false}])),
        ),
        // configurable sinks
        ("frame.srcdoc = evil;", Some(json!([{"properties": ["srcdoc"]}]))),
    ];

    Tester::new(Property::NAME, Property::PLUGIN, pass, fail).test_and_snapshot();

    let ts_pass = vec![
        ("(options as HTMLElement).innerHTML = '<s>safe</s>';", None),
        ("(<HTMLElement>items[i](args)).innerHTML = 'rawstring';", None),
        ("lol.innerHTML = (5 as string);", None),
        (
            "node!.innerHTML = DOMPurify.sanitize(evil);",
            Some(json!([{"escape": {"methods": ["DOMPurify.sanitize"]}}])),
        ),
    ];

    let ts_fail = vec![
        ("x!().innerHTML = htmlString", None),
        ("(x as HTMLElement).innerHTML = htmlString", None),
        ("lol.innerHTML = (foo as string);", None),
    ];

    Tester::new(Property::NAME, Property::PLUGIN, ts_pass, ts_fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("typescript")
        .test_and_snapshot();
}
