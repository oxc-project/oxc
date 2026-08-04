use oxc_ast::{
    AstKind,
    ast::{AssignmentOperator, AssignmentTarget},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{ArrayValidation, InstanceType, Schema, SchemaObject, SingleOrVec},
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{EscapeConfig, EscapeSchema, Sanitization, SinkValue},
};

fn unsafe_assignment_diagnostic(property: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unsafe assignment to {property}"))
        .with_help(
            "Assign a string literal, or wrap the value in an escaping function such as `Sanitizer.escapeHTML`.",
        )
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct PropertyConfig {
    /// Property names treated as HTML sinks, with the escaping functions accepted
    /// for each of them.
    checks: FxHashMap<CompactStr, EscapeConfig>,
    variable_tracing: bool,
}

impl Default for PropertyConfig {
    fn default() -> Self {
        Self { checks: default_checks(), variable_tracing: true }
    }
}

fn default_checks() -> FxHashMap<CompactStr, EscapeConfig> {
    FxHashMap::from_iter([
        (CompactStr::new("innerHTML"), EscapeConfig::default()),
        (CompactStr::new("outerHTML"), EscapeConfig::default()),
    ])
}

#[derive(Debug, Default, Clone)]
pub struct Property(Box<PropertyConfig>);

/// The first options object, applying to every property.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PropertyGlobalOptions {
    /// Escaping functions which mark a value as safe.
    escape: Option<EscapeSchema>,
    /// Whether values assigned from local `let`/`const` variables are traced back
    /// to their initializers. Defaults to `true`.
    variable_tracing: Option<bool>,
}

/// The options of a single property in the second options object.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PropertySinkOptions {
    /// Escaping functions accepted for this property, replacing the ones from the
    /// first options object.
    escape: Option<EscapeSchema>,
}

/// `[globalOptions, { propertyName: sinkOptions }]`, as upstream.
#[derive(Debug)]
#[expect(unused)] // only for schemars
pub struct PropertyOptionsSchema(PropertyGlobalOptions, FxHashMap<String, PropertySinkOptions>);

impl JsonSchema for PropertyOptionsSchema {
    fn schema_name() -> String {
        "PropertyOptionsSchema".to_string()
    }

    fn is_referenceable() -> bool {
        false
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        let global = r#gen.subschema_for::<PropertyGlobalOptions>();
        let sinks = r#gen.subschema_for::<FxHashMap<String, PropertySinkOptions>>();

        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                items: Some(SingleOrVec::Vec(vec![global, sinks])),
                max_items: Some(2),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
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
    /// The first object configures every property, the second one adds properties
    /// to the built-in `innerHTML` and `outerHTML` or overrides their escapers:
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
    ///       "variableTracing": true
    ///     },
    ///     {
    ///       "srcdoc": {}
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// ### Known limitations
    ///
    /// Assignments whose property name is only known at runtime
    /// (`node[whichProperty] = evil`) are not reported, as upstream.
    Property,
    no_unsanitized,
    restriction,
    config = PropertyOptionsSchema,
    version = "1.78.0",
    short_description = "Disallow unsanitized assignment to DOM properties that parse HTML.",
);

impl Rule for Property {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let global = match value.get(0) {
            Some(global) if !global.is_null() => {
                serde_json::from_value::<PropertyGlobalOptions>(global.clone())?
            }
            _ => PropertyGlobalOptions::default(),
        };
        let sinks: FxHashMap<String, PropertySinkOptions> = match value.get(1) {
            Some(Value::Object(_)) => serde_json::from_value(value[1].clone())?,
            _ => FxHashMap::default(),
        };

        let global_escape = global.escape.as_ref().map(EscapeConfig::from);
        let mut checks = default_checks();
        if let Some(global_escape) = &global_escape {
            for escape in checks.values_mut() {
                *escape = global_escape.clone();
            }
        }
        // The second object adds properties to the defaults, or overrides their escapers.
        for (property, sink) in &sinks {
            let escape = sink
                .escape
                .as_ref()
                .map(EscapeConfig::from)
                .or_else(|| global_escape.clone())
                .unwrap_or_default();
            checks.insert(CompactStr::from(property.as_str()), escape);
        }

        Ok(Self(Box::new(PropertyConfig {
            checks,
            variable_tracing: global.variable_tracing.unwrap_or(true),
        })))
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
        let Some(escape) = self.0.checks.get(property) else {
            return;
        };

        let sanitization = Sanitization { escape, variable_tracing: self.0.variable_tracing };
        if !sanitization.is_allowed(SinkValue::Expression(&assignment.right), ctx) {
            ctx.diagnostic(unsafe_assignment_diagnostic(property, assignment.span));
        }
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
/// `node.innerHTML = x` and for `node["innerHTML"] = x`.
///
/// A computed access with a dynamic key (`node[name] = x`) cannot be resolved and
/// is not reported, as upstream.
fn assigned_property_name<'a>(target: &AssignmentTarget<'a>) -> Option<&'a str> {
    let simple = target.as_simple_assignment_target()?;
    let member = match simple.get_expression() {
        // `(a.b as T) = c` and friends
        Some(expression) => expression.get_inner_expression().get_member_expr(),
        None => simple.as_member_expression(),
    }?;
    member.static_property_name()
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
        // computed member access with a static key
        ("node['innerHTML'] = '<b>static</b>';", None),
        // computed member with a dynamic key is a documented false negative
        ("node[whichProperty] = evil;", None),
        // per-property escaper, replacing the global one
        (
            "el.innerHTML = safeHTML`<b>${evil}</b>`;",
            Some(json!([{}, {"innerHTML": {"escape": {"taggedTemplates": ["safeHTML"]}}}])),
        ),
        // a property added through the second object keeps the global escapers
        (
            "frame.srcdoc = DOMPurify.sanitize(evil);",
            Some(json!([{"escape": {"methods": ["DOMPurify.sanitize"]}}, {"srcdoc": {}}])),
        ),
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
        ("node['innerHTML'] = htmlString;", None),
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
        // per-property configuration in the second options object
        ("frame.srcdoc = evil;", Some(json!([{}, {"srcdoc": {}}]))),
        (
            "el.innerHTML = safeHTML`<b>${evil}</b>`;",
            Some(
                json!([{"escape": {"taggedTemplates": ["otherEscaper"]}}, {"innerHTML": {"escape": {"taggedTemplates": ["alsoNot"]}}}]),
            ),
        ),
        // a per-property `escape` replaces the global one as a whole, so
        // `methods` falls back to the built-in escapers again
        (
            "el.innerHTML = DOMPurify.sanitize(evil);",
            Some(
                json!([{"escape": {"methods": ["DOMPurify.sanitize"]}}, {"innerHTML": {"escape": {"taggedTemplates": ["safeHTML"]}}}]),
            ),
        ),
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
