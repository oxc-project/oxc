use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, TaggedTemplateExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        EscapeConfig, EscapeSchema, Sanitization, SinkValue, callee_code_name, object_code_name,
    },
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
    let sink = |properties: Vec<usize>, object_matches: Option<Vec<Regex>>| SinkCheck {
        properties,
        object_matches,
        escape: EscapeConfig::default(),
    };
    // `document` as a regex, as upstream: it matches `documentish`, `window.document`, ...
    let document = || Some(vec![Regex::new("(?i)document").unwrap()]);
    FxHashMap::from_iter([
        (CompactStr::new("insertAdjacentHTML"), sink(vec![1], None)),
        (CompactStr::new("import"), sink(vec![0], None)),
        (CompactStr::new("createContextualFragment"), sink(vec![0], None)),
        (CompactStr::new("write"), sink(vec![0], document())),
        (CompactStr::new("writeln"), sink(vec![0], document())),
        (CompactStr::new("setHTMLUnsafe"), sink(vec![0], None)),
    ])
}

#[derive(Debug, Default, Clone)]
pub struct Method(Box<MethodConfig>);

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MethodOptionsSchema {
    /// Escaping functions which mark a value as safe, for every sink.
    escape: Option<EscapeSchema>,
    /// Drops the built-in sink list, so only sinks given in the second options
    /// object are checked.
    default_disable: Option<bool>,
    /// Regexes the object of a call must match, for every sink.
    object_matches: Option<Vec<String>>,
    /// Argument indices which are parsed as HTML, for every sink.
    properties: Option<Vec<usize>>,
    /// Whether values coming from local `let`/`const` variables are traced back
    /// to their initializers. Defaults to `true`.
    variable_tracing: Option<bool>,
}

/// The per-sink options of the second options object.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SinkSchema {
    escape: Option<EscapeSchema>,
    object_matches: Option<Vec<String>>,
    properties: Option<Vec<usize>>,
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
    Method,
    no_unsanitized,
    restriction,
    config = MethodOptionsSchema,
    version = "1.78.0",
    short_description = "Disallow unsanitized arguments to DOM methods that parse HTML.",
);

impl Rule for Method {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let parent = value.get(0).cloned().unwrap_or(Value::Null);
        let parent: MethodOptionsSchema = if parent.is_null() {
            MethodOptionsSchema::default()
        } else {
            serde_json::from_value(parent)?
        };
        let children: FxHashMap<String, SinkSchema> = match value.get(1) {
            Some(Value::Object(_)) => serde_json::from_value(value[1].clone())?,
            _ => FxHashMap::default(),
        };

        let default_disable = parent.default_disable.unwrap_or(false);
        let mut checks = if default_disable { FxHashMap::default() } else { default_checks() };

        // The first options object applies to every sink, ...
        for check in checks.values_mut() {
            apply_parent(check, &parent);
        }
        // ... the second one adds or overrides individual sinks.
        for (name, child) in &children {
            let name = CompactStr::from(name.as_str());
            let mut check = checks.remove(&name).unwrap_or_else(|| {
                let escape = if default_disable {
                    EscapeConfig { tagged_templates: Vec::new(), methods: Vec::new() }
                } else {
                    EscapeConfig::default()
                };
                let mut check = SinkCheck { properties: Vec::new(), object_matches: None, escape };
                apply_parent(&mut check, &parent);
                check
            });
            if let Some(escape) = &child.escape {
                check.escape.apply(escape);
            }
            if let Some(properties) = &child.properties {
                check.properties.clone_from(properties);
            }
            if let Some(object_matches) = &child.object_matches {
                check.object_matches = Some(compile_regexes(object_matches));
            }
            checks.insert(name, check);
        }

        Ok(Self(Box::new(MethodConfig {
            checks,
            variable_tracing: parent.variable_tracing.unwrap_or(true),
        })))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => self.check_call(call, ctx),
            AstKind::TaggedTemplateExpression(tagged) => self.check_tagged_template(tagged, ctx),
            AstKind::ImportExpression(import) => {
                self.check(
                    "import",
                    None,
                    &[SinkValue::Expression(&import.source)],
                    "import",
                    import.span,
                    ctx,
                );
            }
            _ => {}
        }
    }
}

impl Method {
    fn check_call<'a>(&self, call: &CallExpression<'a>, ctx: &LintContext<'a>) {
        if call.arguments.is_empty() {
            return;
        }
        // Tagged templates are visited on their own.
        let Some(callee) = effective_callee(&call.callee) else {
            return;
        };
        let arguments: Vec<SinkValue> = call
            .arguments
            .iter()
            .map(|argument| match argument {
                Argument::SpreadElement(_) => SinkValue::Unsupported,
                argument => {
                    argument.as_expression().map_or(SinkValue::Unsupported, SinkValue::Expression)
                }
            })
            .collect();
        self.check_callee(callee, &arguments, call.span, ctx);
    }

    /// A tagged template behaves like a call with the quasis as first argument,
    /// followed by the interpolated expressions.
    fn check_tagged_template<'a>(
        &self,
        tagged: &TaggedTemplateExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(callee) = effective_callee(&tagged.tag) else {
            return;
        };
        let mut arguments = vec![SinkValue::Quasis];
        arguments.extend(tagged.quasi.expressions.iter().map(SinkValue::Expression));
        self.check_callee(callee, &arguments, tagged.span, ctx);
    }

    fn check_callee<'a>(
        &self,
        callee: &Expression<'a>,
        arguments: &[SinkValue<'a, '_>],
        span: Span,
        ctx: &LintContext<'a>,
    ) {
        let (method_name, object) = match callee {
            Expression::Identifier(identifier) => (identifier.name.as_str(), None),
            Expression::StaticMemberExpression(member) => {
                // `foo.import(bar)` is a plain method call, not a dynamic import.
                if member.property.name == "import" {
                    return;
                }
                (member.property.name.as_str(), Some(&member.object))
            }
            _ => return,
        };
        let code_name = callee_code_name(callee, ctx).unwrap_or_else(|| method_name.to_string());
        self.check(method_name, object, arguments, &code_name, span, ctx);
    }

    fn check<'a>(
        &self,
        method_name: &str,
        object: Option<&Expression<'a>>,
        arguments: &[SinkValue<'a, '_>],
        code_name: &str,
        span: Span,
        ctx: &LintContext<'a>,
    ) {
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
            if !sanitization.is_allowed(*argument, ctx) {
                ctx.diagnostic(unsafe_call_diagnostic(code_name, *index, span));
            }
        }
    }
}

fn apply_parent(check: &mut SinkCheck, parent: &MethodOptionsSchema) {
    if let Some(escape) = &parent.escape {
        check.escape.apply(escape);
    }
    if let Some(properties) = &parent.properties {
        check.properties.clone_from(properties);
    }
    if let Some(object_matches) = &parent.object_matches {
        check.object_matches = Some(compile_regexes(object_matches));
    }
}

fn compile_regexes(patterns: &[String]) -> Vec<Regex> {
    patterns.iter().filter_map(|pattern| Regex::new(&format!("(?i){pattern}")).ok()).collect()
}

/// Resolves what is actually being called, following the upstream
/// `checkCallExpression` walk.
///
/// Returns `None` for callees the rule intentionally does not reason about,
/// such as calls, conditionals or function expressions.
fn effective_callee<'a, 'b>(callee: &'b Expression<'a>) -> Option<&'b Expression<'a>> {
    match callee {
        Expression::Identifier(_) | Expression::StaticMemberExpression(_) => Some(callee),
        Expression::ParenthesizedExpression(paren) => effective_callee(&paren.expression),
        Expression::TSNonNullExpression(non_null) => effective_callee(&non_null.expression),
        // The value of a sequence is its last expression.
        Expression::SequenceExpression(sequence) => effective_callee(sequence.expressions.last()?),
        // `(a.b = c.d)()` calls whatever was assigned.
        Expression::AssignmentExpression(assignment) => effective_callee(&assignment.right),
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
    ];

    Tester::new(Method::NAME, Method::PLUGIN, pass, fail).test_and_snapshot();

    let ts_pass = vec![
        ("node.insertAdjacentHTML('beforebegin', (5 as string));", None),
        ("node!.insertAdjacentHTML('beforebegin', 'raw string');", None),
        ("node!().insertAdjacentHTML('beforebegin', 'raw string');", None),
    ];

    let ts_fail = vec![
        ("node!().insertAdjacentHTML('beforebegin', htmlString);", None),
        ("node!.insertAdjacentHTML('beforebegin', htmlString);", None),
        ("(x as HTMLElement).insertAdjacentHTML('beforebegin', htmlString)", None),
    ];

    Tester::new(Method::NAME, Method::PLUGIN, ts_pass, ts_fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("typescript")
        .test_and_snapshot();
}
