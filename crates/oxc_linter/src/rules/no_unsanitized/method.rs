use lazy_regex::Regex;
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{
    AstKind,
    ast::{Argument, AssignmentOperator, AssignmentTarget, Expression, TemplateLiteral},
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
use serde::{Deserialize, de::Error as _};
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{EscapeConfig, EscapeSchema, Sanitization, SinkValue, object_code_name},
};

fn unsafe_call_diagnostic(callee: &str, argument: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unsafe call to {callee} for argument {argument}"))
        .with_help(
            "Pass a string literal, or wrap the value in an escaping function such as `Sanitizer.escapeHTML`.",
        )
        .with_label(span)
}

/// One HTML sink: which arguments are parsed as HTML, and on which objects.
#[derive(Debug, Clone)]
struct SinkCheck {
    /// Indices of the arguments that end up being parsed as HTML.
    properties: Vec<usize>,
    /// Regexes the name of the object must match, `None` matches everything.
    object_matches: Option<Vec<Regex>>,
    escape: EscapeConfig,
}

/// A built-in sink, before user configuration is applied.
struct DefaultSink {
    name: &'static str,
    properties: &'static [usize],
    object_matches: Option<&'static [&'static str]>,
}

/// `document` is a regex upstream, so it also matches `documentish`, `window.document`, ...
const DEFAULT_SINKS: [DefaultSink; 6] = [
    DefaultSink { name: "insertAdjacentHTML", properties: &[1], object_matches: None },
    DefaultSink { name: "import", properties: &[0], object_matches: None },
    DefaultSink { name: "createContextualFragment", properties: &[0], object_matches: None },
    DefaultSink { name: "write", properties: &[0], object_matches: Some(&["document"]) },
    DefaultSink { name: "writeln", properties: &[0], object_matches: Some(&["document"]) },
    DefaultSink { name: "setHTMLUnsafe", properties: &[0], object_matches: None },
];

#[derive(Debug, Clone)]
pub struct MethodConfig {
    checks: FxHashMap<CompactStr, SinkCheck>,
    variable_tracing: bool,
}

impl Default for MethodConfig {
    fn default() -> Self {
        Self { checks: default_checks(), variable_tracing: true }
    }
}

fn default_checks() -> FxHashMap<CompactStr, SinkCheck> {
    DEFAULT_SINKS
        .iter()
        .map(|sink| {
            let object_matches = sink
                .object_matches
                .map(|patterns| patterns.iter().map(|p| compile_regex(p).unwrap()).collect());
            (
                CompactStr::new(sink.name),
                SinkCheck {
                    properties: sink.properties.to_vec(),
                    object_matches,
                    escape: EscapeConfig::default(),
                },
            )
        })
        .collect()
}

#[derive(Debug, Default, Clone)]
pub struct Method(Box<MethodConfig>);

/// The first options object, applying to every sink.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MethodGlobalOptions {
    /// Escaping functions which mark a value as safe.
    escape: Option<EscapeSchema>,
    /// Drops the built-in sink list, so only sinks given in the second options
    /// object are checked. Built-in sinks named there keep their argument indices.
    default_disable: Option<bool>,
    /// Regexes the name of the object a method is called on must match.
    ///
    /// These are Rust regexes, which do not support backreferences or lookaround.
    object_matches: Option<Vec<String>>,
    /// Argument indices which are parsed as HTML.
    properties: Option<Vec<usize>>,
    /// Whether values coming from local `let`/`const` variables are traced back
    /// to their initializers. Defaults to `true`.
    variable_tracing: Option<bool>,
}

/// The options of a single sink in the second options object.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MethodSinkOptions {
    /// Escaping functions accepted for this sink, replacing the ones from the
    /// first options object.
    escape: Option<EscapeSchema>,
    /// Regexes the name of the object must match, replacing the ones from the
    /// first options object.
    object_matches: Option<Vec<String>>,
    /// Argument indices which are parsed as HTML.
    properties: Option<Vec<usize>>,
}

/// `[globalOptions, { methodName: sinkOptions }]`, as upstream.
#[derive(Debug)]
#[expect(unused)] // only for schemars
pub struct MethodOptionsSchema(MethodGlobalOptions, FxHashMap<String, MethodSinkOptions>);

impl JsonSchema for MethodOptionsSchema {
    fn schema_name() -> String {
        "MethodOptionsSchema".to_string()
    }

    fn is_referenceable() -> bool {
        false
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        let global = r#gen.subschema_for::<MethodGlobalOptions>();
        let sinks = r#gen.subschema_for::<FxHashMap<String, MethodSinkOptions>>();

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
    /// Disallows passing values that are not provably safe to DOM methods which
    /// parse an argument as HTML, such as `insertAdjacentHTML`, `document.write`
    /// or `Range.createContextualFragment`. Dynamic `import()` is checked as well,
    /// since its argument is a code location.
    ///
    /// This is a port of the `method` rule of
    /// [`eslint-plugin-no-unsanitized`](https://github.com/mozilla/eslint-plugin-no-unsanitized).
    ///
    /// ### Why is this bad?
    ///
    /// Passing a dynamic value to a method that parses HTML is a common source of
    /// DOM based cross-site scripting. Only string literals, template literals whose
    /// interpolations are themselves safe, and the output of a known escaping
    /// function can be assumed not to introduce markup an attacker controls.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// node.insertAdjacentHTML("beforebegin", htmlString);
    /// document.write("<span>" + userInput + "</span>");
    /// range.createContextualFragment(userInput);
    /// import(userInput);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// node.insertAdjacentHTML("beforebegin", "<b>static</b>");
    /// node.insertAdjacentHTML("beforebegin", Sanitizer.escapeHTML`<b>${userInput}</b>`);
    /// document.write("static");
    /// import("lodash");
    /// ```
    ///
    /// ### Options
    ///
    /// The first object configures all sinks, the second one adds or overrides
    /// individual sinks:
    ///
    /// ```json
    /// {
    ///   "no-unsanitized/method": [
    ///     "error",
    ///     {
    ///       "escape": { "methods": ["DOMPurify.sanitize"] }
    ///     },
    ///     {
    ///       "setHTML": { "properties": [0] }
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// `objectMatches` patterns are matched case-insensitively as Rust regexes,
    /// which unlike JavaScript regexes support no backreferences and no lookaround.
    /// A pattern that fails to compile is reported as a configuration error rather
    /// than silently disabling the sink.
    ///
    /// ### Known limitations
    ///
    /// Calls whose method name is only known at runtime (`node[whichMethod](evil)`)
    /// and arguments hidden behind a spread (`node.insertAdjacentHTML(...args)`)
    /// are not reported, as upstream.
    Method,
    no_unsanitized,
    restriction,
    config = MethodOptionsSchema,
    version = "1.78.0",
    short_description = "Disallow unsanitized arguments to DOM methods that parse HTML.",
);

impl Rule for Method {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let global = match value.get(0) {
            Some(global) if !global.is_null() => {
                serde_json::from_value::<MethodGlobalOptions>(global.clone())?
            }
            _ => MethodGlobalOptions::default(),
        };
        let sinks: FxHashMap<String, MethodSinkOptions> = match value.get(1) {
            Some(Value::Object(_)) => serde_json::from_value(value[1].clone())?,
            _ => FxHashMap::default(),
        };

        let global_escape = global.escape.as_ref().map(EscapeConfig::from);
        let global_object_matches =
            global.object_matches.as_ref().map(|patterns| compile_regexes(patterns)).transpose()?;

        // Without `defaultDisable` every built-in sink is checked, with it only the
        // ones named in the second options object -- but those keep the built-in
        // argument indices.
        let default_disable = global.default_disable.unwrap_or(false);
        let names: Vec<&str> = if default_disable {
            sinks.keys().map(String::as_str).collect()
        } else {
            DEFAULT_SINKS
                .iter()
                .map(|sink| sink.name)
                .chain(
                    sinks
                        .keys()
                        .map(String::as_str)
                        .filter(|name| !DEFAULT_SINKS.iter().any(|sink| sink.name == *name)),
                )
                .collect()
        };

        let mut checks = FxHashMap::default();
        for name in names {
            let default = DEFAULT_SINKS.iter().find(|sink| sink.name == name);
            let sink = sinks.get(name);

            let properties = sink
                .and_then(|sink| sink.properties.clone())
                .or_else(|| global.properties.clone())
                .or_else(|| default.map(|default| default.properties.to_vec()))
                .unwrap_or_default();

            let object_matches = match sink.and_then(|sink| sink.object_matches.as_ref()) {
                Some(patterns) => Some(compile_regexes(patterns)?),
                None => match &global_object_matches {
                    Some(regexes) => Some(regexes.clone()),
                    None => default
                        .and_then(|default| default.object_matches)
                        .map(|patterns| {
                            patterns.iter().map(|pattern| compile_regex(pattern)).collect()
                        })
                        .transpose()?,
                },
            };

            // An `escape` object replaces the one it overrides as a whole.
            let escape = sink
                .and_then(|sink| sink.escape.as_ref())
                .map(EscapeConfig::from)
                .or_else(|| global_escape.clone())
                .unwrap_or_default();

            checks.insert(CompactStr::from(name), SinkCheck { properties, object_matches, escape });
        }

        Ok(Self(Box::new(MethodConfig {
            checks,
            variable_tracing: global.variable_tracing.unwrap_or(true),
        })))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => {
                if call.arguments.is_empty() {
                    return;
                }
                let Some(callee) = effective_callee(&call.callee) else {
                    return;
                };
                self.check(callee, &SinkArguments::Call(&call.arguments), call.span, ctx);
            }
            AstKind::TaggedTemplateExpression(tagged) => {
                let Some(callee) = effective_callee(&tagged.tag) else {
                    return;
                };
                self.check(callee, &SinkArguments::Template(&tagged.quasi), tagged.span, ctx);
            }
            AstKind::ImportExpression(import) => {
                self.check_sink(
                    "import",
                    None,
                    &SinkArguments::Single(&import.source),
                    import.span,
                    ctx,
                );
            }
            _ => {}
        }
    }
}

impl Method {
    fn check<'a>(
        &self,
        callee: Callee<'a, '_>,
        arguments: &SinkArguments<'a, '_>,
        span: Span,
        ctx: &LintContext<'a>,
    ) {
        let Some((method_name, object)) = callee.parts() else {
            return;
        };
        // `foo.import(bar)` is a plain method call, not a dynamic import.
        if method_name == "import" && object.is_some() {
            return;
        }
        self.check_sink(method_name, object, arguments, span, ctx);
    }

    fn check_sink<'a>(
        &self,
        method_name: &str,
        object: Option<&Expression<'a>>,
        arguments: &SinkArguments<'a, '_>,
        span: Span,
        ctx: &LintContext<'a>,
    ) {
        // Cheapest first: most calls are not sinks at all.
        let Some(check) = self.0.checks.get(method_name) else {
            return;
        };
        if let Some(object_matches) = &check.object_matches {
            let Some(object) = object else {
                // A bare function call cannot match an object name.
                return;
            };
            let object_name = object_code_name(object, ctx);
            if !object_matches.iter().any(|regex| regex.is_match(&object_name)) {
                return;
            }
        }

        let sanitization =
            Sanitization { escape: &check.escape, variable_tracing: self.0.variable_tracing };
        for index in &check.properties {
            // Fewer arguments than configured, e.g. because of a spread element.
            let Some(argument) = arguments.get(*index) else {
                continue;
            };
            if !sanitization.is_allowed(argument, ctx) {
                let code_name = match object {
                    Some(object) => format!("{}.{method_name}", object_code_name(object, ctx)),
                    None => method_name.to_string(),
                };
                ctx.diagnostic(unsafe_call_diagnostic(&code_name, *index, span));
            }
        }
    }
}

/// The arguments of a sink, without collecting them into a new list.
enum SinkArguments<'a, 'b> {
    Call(&'b ArenaVec<'a, Argument<'a>>),
    /// A tagged template behaves like a call with the quasis as first argument,
    /// followed by the interpolated expressions.
    Template(&'b TemplateLiteral<'a>),
    /// The single argument of a dynamic `import()`.
    Single(&'b Expression<'a>),
}

impl<'a, 'b> SinkArguments<'a, 'b> {
    fn get(&self, index: usize) -> Option<SinkValue<'a, 'b>> {
        match self {
            Self::Call(arguments) => match arguments.get(index)? {
                Argument::SpreadElement(_) => Some(SinkValue::Unsupported),
                argument => Some(
                    argument.as_expression().map_or(SinkValue::Unsupported, SinkValue::Expression),
                ),
            },
            Self::Template(template) => match index.checked_sub(1) {
                None => Some(SinkValue::Quasis),
                Some(index) => template.expressions.get(index).map(SinkValue::Expression),
            },
            Self::Single(expression) => (index == 0).then_some(SinkValue::Expression(expression)),
        }
    }
}

/// What a call expression actually calls.
#[derive(Clone, Copy)]
enum Callee<'a, 'b> {
    Expression(&'b Expression<'a>),
    /// The left-hand side of a logical assignment, whose value may well be the
    /// sink that was already there: `(node.insertAdjacentHTML ||= fallback)(...)`.
    AssignmentTarget(&'b AssignmentTarget<'a>),
}

impl<'a, 'b> Callee<'a, 'b> {
    /// Method name and the object it is called on, if any.
    fn parts(self) -> Option<(&'a str, Option<&'b Expression<'a>>)> {
        let member = match self {
            Self::Expression(Expression::Identifier(identifier)) => {
                return Some((identifier.name.as_str(), None));
            }
            Self::Expression(expression) => expression.as_member_expression()?,
            Self::AssignmentTarget(target) => {
                let simple = target.as_simple_assignment_target()?;
                match simple.get_expression() {
                    Some(expression) => expression.get_inner_expression().get_member_expr()?,
                    None => simple.as_member_expression()?,
                }
            }
        };
        Some((member.static_property_name()?, Some(member.object())))
    }
}

fn compile_regexes(patterns: &[String]) -> Result<Vec<Regex>, serde_json::Error> {
    patterns.iter().map(|pattern| compile_regex(pattern)).collect()
}

fn compile_regex(pattern: &str) -> Result<Regex, serde_json::Error> {
    Regex::new(&format!("(?i){pattern}")).map_err(|err| {
        serde_json::Error::custom(format!("invalid `objectMatches` regex `{pattern}`: {err}"))
    })
}

/// Resolves what is actually being called, following the upstream
/// `checkCallExpression` walk.
///
/// Returns `None` for callees the rule intentionally does not reason about.
// The ignored callee shapes are listed one by one on purpose, so that they are
// documented rather than hidden behind the catch-all arm.
#[expect(clippy::match_same_arms)]
fn effective_callee<'a, 'b>(callee: &'b Expression<'a>) -> Option<Callee<'a, 'b>> {
    match callee {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_) => Some(Callee::Expression(callee)),
        Expression::ParenthesizedExpression(paren) => effective_callee(&paren.expression),
        Expression::TSNonNullExpression(_)
        | Expression::TSAsExpression(_)
        | Expression::TSSatisfiesExpression(_)
        | Expression::TSTypeAssertion(_)
        | Expression::TSInstantiationExpression(_) => {
            effective_callee(callee.get_inner_expression())
        }
        // The value of a sequence is its last expression.
        Expression::SequenceExpression(sequence) => effective_callee(sequence.expressions.last()?),
        Expression::AssignmentExpression(assignment) => match assignment.operator {
            // `(a.b = c.d)()` calls whatever was assigned.
            AssignmentOperator::Assign => effective_callee(&assignment.right),
            // A logical assignment may evaluate to the previous value of the
            // left-hand side, so that is what gets called.
            AssignmentOperator::LogicalOr
            | AssignmentOperator::LogicalAnd
            | AssignmentOperator::LogicalNullish => {
                Some(Callee::AssignmentTarget(&assignment.left))
            }
            // Arithmetic and bitwise compound assignments cannot produce a callable sink.
            _ => None,
        },
        // Known callee shapes whose target the rule cannot resolve, listed
        // explicitly so that new expression kinds are not silently ignored.
        Expression::CallExpression(_)
        | Expression::NewExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::ThisExpression(_)
        | Expression::Super(_)
        | Expression::ChainExpression(_)
        | Expression::ImportExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::YieldExpression(_)
        | Expression::PrivateFieldExpression(_) => None,
        _ => None,
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("n.insertAdjacentHTML('afterend', 'meh');", None),
        ("n.insertAdjacentHTML('afterend', `<br>`);", None),
        ("n.insertAdjacentHTML('afterend', Sanitizer.escapeHTML`${title}`);", None),
        ("document.write('lulz');", None),
        ("document.write();", None),
        ("document.writeln(Sanitizer.escapeHTML`<em>${evil}</em>`);", None),
        ("otherNodeWeDontCheckFor.writeln(evil);", None),
        ("document.toString(evil);", None),
        ("document.write(escaper(x))", Some(json!([{"escape": {"methods": ["escaper"]}}]))),
        (
            "document.write(evilest)",
            Some(
                json!([{"objectMatches": ["document", "documentFun"]}, {"write": {"objectMatches": ["thing"]}}]),
            ),
        ),
        ("document.write(evil)", Some(json!([{"defaultDisable": true}]))),
        ("  _tests.shift()();", None),
        ("(Async.checkAppReady = function() { return true; })();", None),
        ("let endTime = (mapEnd || (e => e.delta))(this._data[this._data.length - 1]);", None),
        ("(text.endsWith('\\n') ? document.write : document.writeln)(text)", None),
        ("function foo() { return this().bar(); };", None),
        ("new Function()();", None),
        ("range.createContextualFragment('<p class=\"greeting\">Hello!</p>');", None),
        (
            "range.createContextualFragment(Sanitizer.escapeHTML`<em>${evil}</em>`);",
            Some(json!([{"escape": {"methods": ["escaper"]}}])),
        ),
        (
            "range.createContextualFragment(escaper('<em>'+evil+'</em>'));",
            Some(json!([{"escape": {"methods": ["escaper"]}}])),
        ),
        ("import('lodash')", None),
        (
            "range.createContextualFragment(templateEscaper`<em>${evil}</em>`);",
            Some(json!([{"escape": {"taggedTemplates": ["templateEscaper"]}}])),
        ),
        (
            "n.insertAdjacentHTML('afterend', DOMPurify.sanitize(evil));",
            Some(json!([{"escape": {"methods": ["DOMPurify.sanitize"]}}])),
        ),
        (
            "n.insertAdjacentHTML('afterend', DOMPurify.sanitize(evil, options));",
            Some(json!([{"escape": {"methods": ["DOMPurify.sanitize"]}}])),
        ),
        (
            "n.insertAdjacentHTML('afterend', DOMPurify.sanitize(evil, {ALLOWED_TAGS: ['b']}));",
            Some(json!([{"escape": {"methods": ["DOMPurify.sanitize"]}}])),
        ),
        ("describe.each`table`(name, fn, timeout)", None),
        ("document.write`text`", None),
        ("document.write`text ${'static string'}`", None),
        ("custom`text ${variable}`", Some(json!([{}, {"custom": {"properties": [0]}}]))),
        ("custom`text ${'string'}`", Some(json!([{}, {"custom": {"properties": [1]}}]))),
        ("document.write`text ${variable}`", None),
        ("let a = (0,1,2,34);", None),
        ("(async function()  { (await somePromise)(); })", None),
        ("async () => (await TheRuleDoesntKnowWhatIsBeingReturnedHere())('afterend', blah);", None),
        ("(e = this.n[n.i])(i, r)", None),
        ("(e = node.insertAdjacentHTML('beforebegin', '<s>safe</s>'))()", None),
        ("foo.import(bar)", None),
        ("let c; n.insertAdjacentHTML('beforebegin', c)", None),
        ("x.setHTML(evil)", None),
        ("x.setHTML(evil, { sanitizer: s })", None),
        ("x.setHTML(evil, { sanitizer: new Sanitizer()})", None),
        ("(info.current = type)(child_ctx)", None),
        ("(info.current = n.insertAdjacentHTML)('beforebegin', 'innocent')", None),
        ("let l = ['afterend', 'harmless']; foo.insertAdjacentHTML(...l);", None),
        ("foo.insertAdjacentHTML(wrongParamCount);", None),
        ("foo.setHTMLUnsafe('static string')", None),
        // computed member access with a dynamic key: documented false negative
        ("document[whichMethod](evil)", None),
        // a spread argument shifts the checked index out of range, as upstream
        ("foo.insertAdjacentHTML(...args);", None),
        // a logical assignment callee is treated as a call to the existing sink
        ("(node.insertAdjacentHTML ||= fallback)('beforebegin', '<b>static</b>')", None),
        // an arithmetic compound assignment cannot produce a callable sink
        ("(node.insertAdjacentHTML += fallback)('beforebegin', evil)", None),
        // `defaultDisable` with a re-enabled built-in sink keeps its argument index,
        // so the harmless first argument stays unchecked
        (
            "document.write('<b>static</b>')",
            Some(json!([{"defaultDisable": true}, {"write": {"objectMatches": ["document"]}}])),
        ),
        // a per-sink `escape` replaces the global one as a whole, so `methods`
        // falls back to the built-in escapers again
        (
            "n.insertAdjacentHTML('afterend', Sanitizer.unwrapSafeHTML(x));",
            Some(json!([
                {"escape": {"methods": ["DOMPurify.sanitize"]}},
                {"insertAdjacentHTML": {"escape": {"taggedTemplates": ["safeHTML"]}}}
            ])),
        ),
        // per-sink configuration of a method that is not a built-in sink
        ("html('<b>static</b>')", Some(json!([{}, {"html": {"properties": [0]}}]))),
    ];

    let fail = vec![
        ("node.insertAdjacentHTML('beforebegin', htmlString);", None),
        ("node.insertAdjacentHTML('beforebegin', template.getHTML());", None),
        ("document.write('<span>'+ htmlInput + '</span>');", None),
        ("documentish.write('<span>'+ htmlInput + '</span>');", None),
        ("documentIframe.write('<span>'+ htmlInput + '</span>');", None),
        ("document.writeln(evil);", None),
        ("window.document.writeln(bad);", None),
        ("function foo() { return this().insertAdjacentHTML(foo, bar); };", None),
        (
            "document.write(evil); b.thing(x); b.other(me);",
            Some(json!([{"defaultDisable": true}, {"other": {"properties": [0]}}])),
        ),
        ("getDocument(myID).write(evil)", None),
        ("range.createContextualFragment(badness)", None),
        ("import(foo)", None),
        ("(0, node.insertAdjacentHTML)('beforebegin', evil);", None),
        (
            "n.insertAdjacentHTML('afterend', templateEscaper(evil, options));",
            Some(json!([{"escape": {"taggedTemplates": ["templateEscaper"]}}])),
        ),
        (
            "n.insertAdjacentHTML('afterend', sanitize`<em>${evil}</em>`);",
            Some(json!([{"escape": {"methods": ["sanitize"]}}])),
        ),
        (
            "document.writeln(Sanitizer.escapeHTML`<em>${evil}</em>`);",
            Some(json!([
                {"defaultDisable": true},
                {"writeln": {"objectMatches": ["document"], "properties": [0], "escape": {"methods": [], "taggedTemplates": []}}}
            ])),
        ),
        (
            "describe.each`table${node.insertAdjacentHTML('beforebegin', htmlString)}`(name, fn, timeout)",
            None,
        ),
        ("describe.each`table${document.writeln(evil)}`(name, fn, timeout)", None),
        ("node.insertAdjacentHTML`text ${variable}`", None),
        ("custom`text ${variable}`", Some(json!([{}, {"custom": {"properties": [1]}}]))),
        (
            "custom`text ${variable} ${variable2}`",
            Some(json!([{}, {"custom": {"properties": [2]}}])),
        ),
        ("async () => await foo.insertAdjacentHTML('afterend', blah);", None),
        ("async () => (await foo.insertAdjacentHTML('afterend', blah))();", None),
        ("async () => (await foo)().insertAdjacentHTML('afterend', blah);", None),
        ("(e = node.insertAdjacentHTML)('beforebegin', evil)", None),
        ("(e = node.insertAdjacentHTML('beforebegin', evil))()", None),
        ("var copies = '<b>safe</b>'; n.insertAdjacentHTML('beforebegin', copies);", None),
        ("let copies = evil; n.insertAdjacentHTML('beforebegin', copies);", None),
        (
            "let copies = '<b>safe</b>'; copies = suddenlyUnsafe; n.insertAdjacentHTML('beforebegin', copies);",
            None,
        ),
        (
            "function test(evil) { let copies = '<b>safe</b>'; copies = evil; n.insertAdjacentHTML('beforebegin', copies); }",
            None,
        ),
        (
            "const fn = function (evil) { let copies = '<b>safe</b>'; copies = evil; n.insertAdjacentHTML('beforebegin', copies); }",
            None,
        ),
        (
            "const fn = (evil) => { let copies = '<b>safe</b>'; copies = evil; n.insertAdjacentHTML('beforebegin', copies); }",
            None,
        ),
        (
            "let c; if (cond) { c = '<b>safe</b>'; } else { c = evil; } n.insertAdjacentHTML('beforebegin', `${c}`);",
            None,
        ),
        ("(info.current = n.insertAdjacentHTML)('beforebegin', c)", None),
        ("foo.setHTMLUnsafe(badness)", None),
        // computed member access with a static key
        ("document['write'](evil)", None),
        // logical assignment callee: the call may hit the sink that was there before
        ("(node.insertAdjacentHTML ||= fallback)('beforebegin', evil)", None),
        ("(node.insertAdjacentHTML &&= fallback)('beforebegin', evil)", None),
        ("(node.insertAdjacentHTML ??= fallback)('beforebegin', evil)", None),
        // `defaultDisable` with a re-enabled built-in sink keeps its argument index
        (
            "document.write(evil)",
            Some(json!([{"defaultDisable": true}, {"write": {"objectMatches": ["document"]}}])),
        ),
        // the per-sink `escape` object replaces the global one as a whole
        (
            "n.insertAdjacentHTML('afterend', DOMPurify.sanitize(evil));",
            Some(json!([
                {"escape": {"methods": ["DOMPurify.sanitize"]}},
                {"insertAdjacentHTML": {"escape": {"taggedTemplates": ["safeHTML"]}}}
            ])),
        ),
        // per-sink configuration of a method that is not a built-in sink
        ("html(evil)", Some(json!([{}, {"html": {"properties": [0]}}]))),
    ];

    Tester::new(Method::NAME, Method::PLUGIN, pass, fail).test_and_snapshot();

    let ts_pass = vec![
        ("node.insertAdjacentHTML('beforebegin', (5 as string));", None),
        ("node!.insertAdjacentHTML('beforebegin', 'raw string');", None),
        ("node!().insertAdjacentHTML('beforebegin', 'raw string');", None),
    ];

    let ts_fail = vec![
        ("(node.insertAdjacentHTML as InsertFn)('beforebegin', htmlString);", None),
        ("node!().insertAdjacentHTML('beforebegin', htmlString);", None),
        ("node!.insertAdjacentHTML('beforebegin', htmlString);", None),
        ("(x as HTMLElement).insertAdjacentHTML('beforebegin', htmlString)", None),
    ];

    Tester::new(Method::NAME, Method::PLUGIN, ts_pass, ts_fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("typescript")
        .test_and_snapshot();
}

#[test]
fn invalid_object_matches_regex_is_a_configuration_error() {
    let error = Method::from_configuration(serde_json::json!([{"objectMatches": ["("]}]))
        .expect_err("an invalid regex must be rejected instead of disabling the sink");
    assert!(error.to_string().contains("invalid `objectMatches` regex"));

    let error =
        Method::from_configuration(serde_json::json!([{}, {"write": {"objectMatches": ["*"]}}]))
            .expect_err("an invalid regex must be rejected instead of disabling the sink");
    assert!(error.to_string().contains("invalid `objectMatches` regex"));
}
