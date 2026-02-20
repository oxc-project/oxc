use lazy_regex::Regex;
use oxc_ast::{AstKind, ast::FormalParameters};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::Rule};

fn handle_callback_err_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected error to be handled.")
        .with_help("Handle the error or rename the parameter if it's not an error.")
        .with_label(span)
}

#[derive(Debug, Clone)]
enum ErrorPattern {
    Plain(String),
    Regex(Regex),
}

impl Default for ErrorPattern {
    fn default() -> Self {
        Self::Plain("err".to_string())
    }
}

impl ErrorPattern {
    fn matches(&self, name: &str) -> bool {
        match self {
            Self::Plain(s) => name == s,
            Self::Regex(r) => r.is_match(name),
        }
    }
}

#[derive(Debug, Clone, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HandleCallbackErrConfig {
    /// Name of the first callback parameter to treat as the error parameter.
    ///
    /// This can be either:
    /// - an exact name (e.g. `"err"`, `"error"`)
    /// - a regexp pattern (e.g. `"^(err|error)$"`)
    ///
    /// If the configured name of the error variable begins with a `^` it is considered to be a regexp pattern.
    ///
    /// Default: `"err"`.
    pattern: String,
}

#[derive(Debug, Default, Clone)]
pub struct HandleCallbackErr(Box<ErrorPattern>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule expects that when you're using the callback pattern in Node.js you'll handle the error.
    ///
    /// ### Why is this bad?
    ///
    /// In Node.js, a common pattern for dealing with asynchronous behavior is called the callback pattern.
    /// This pattern expects an Error object or null as the first argument of the callback.
    /// Forgetting to handle these errors can lead to some really strange behavior in your application.
    ///
    /// ```js
    /// function loadData (err, data) {
    ///     doSomething(); // forgot to handle error
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"err"` parameter name:
    /// ```js
    /// function loadData (err, data) {
    ///     doSomething();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"err"`` parameter name:
    /// ```js
    /// function loadData (err, data) {
    ///     if (err) {
    ///         console.log(err.stack);
    ///     }
    ///     doSomething();
    /// }
    ///
    /// function generateError (err) {
    ///     if (err) {}
    /// }
    /// ```
    ///
    /// Examples of correct code for this rule with a sample `"error"` parameter name:
    ///```js
    /// function loadData (error, data) {
    ///    if (error) {
    ///       console.log(error.stack);
    ///    }
    ///    doSomething();
    /// }
    ///```
    HandleCallbackErr,
    node,
    restriction,
    config = HandleCallbackErrConfig,
);

impl Rule for HandleCallbackErr {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let pattern = value
            .get(0)
            .and_then(serde_json::Value::as_str)
            .map(|s| {
                if s.starts_with('^') {
                    Regex::new(s)
                        .map_or_else(|_| ErrorPattern::Plain(s.to_string()), ErrorPattern::Regex)
                } else {
                    ErrorPattern::Plain(s.to_string())
                }
            })
            .unwrap_or_default();

        Ok(Self(Box::new(pattern)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) => {
                check_for_error(&func.params, &self.0, ctx);
            }
            AstKind::ArrowFunctionExpression(arrow_func) => {
                check_for_error(&arrow_func.params, &self.0, ctx);
            }
            _ => {}
        }
    }
}

fn check_for_error(params: &FormalParameters, pattern: &ErrorPattern, ctx: &LintContext) {
    let Some(first_param) = params.items.first() else {
        return;
    };

    let Some(ident) = first_param.pattern.get_binding_identifier() else {
        return;
    };

    if pattern.matches(&ident.name) && ctx.symbol_references(ident.symbol_id()).count() == 0 {
        ctx.diagnostic(handle_callback_err_diagnostic(ident.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function test(error) {}", None),
        ("function test(err) {console.log(err);}", None),
        ("function test(err, data) {if(err){ data = 'ERROR';}}", None),
        ("var test = function(err) {console.log(err);};", None),
        ("var test = function(err) {if(err){/* do nothing */}};", None),
        ("var test = function(err) {if(!err){doSomethingHere();}else{};}", None),
        ("var test = function(err, data) {if(!err) { good(); } else { bad(); }}", None),
        ("try { } catch(err) {}", None),
        (
            "getData(function(err, data) {if (err) {}getMoreDataWith(data, function(err, moreData) {if (err) {}getEvenMoreDataWith(moreData, function(err, allOfTheThings) {if (err) {}});});});",
            None,
        ),
        ("var test = function(err) {if(! err){doSomethingHere();}};", None),
        (
            "function test(err, data) {if (data) {doSomething(function(err) {console.error(err);});} else if (err) {console.log(err);}}",
            None,
        ),
        (
            "function handler(err, data) {if (data) {doSomethingWith(data);} else if (err) {console.log(err);}}",
            None,
        ),
        (
            "function handler(err) {logThisAction(function(err) {if (err) {}}); console.log(err);}",
            None,
        ),
        ("function userHandler(err) {process.nextTick(function() {if (err) {}})}", None),
        (
            "function help() { function userHandler(err) {function tester() { err; process.nextTick(function() { err; }); } } }",
            None,
        ),
        ("function help(done) { var err = new Error('error'); done(); }", None),
        ("var test = err => err;", None),
        ("var test = err => !err;", None),
        ("var test = err => err.message;", None),
        (
            "var test = function(error) {if(error){/* do nothing */}};",
            Some(serde_json::json!(["error"])),
        ),
        (
            "var test = (error) => {if(error){/* do nothing */}};",
            Some(serde_json::json!(["error"])),
        ),
        (
            "var test = function(error) {if(! error){doSomethingHere();}};",
            Some(serde_json::json!(["error"])),
        ),
        (
            "var test = function(err) { console.log(err); };",
            Some(serde_json::json!(["^(err|error)$"])),
        ),
        (
            "var test = function(error) { console.log(error); };",
            Some(serde_json::json!(["^(err|error)$"])),
        ),
        (
            "var test = function(anyError) { console.log(anyError); };",
            Some(serde_json::json!(["^.+Error$"])),
        ),
        (
            "var test = function(any_error) { console.log(anyError); };",
            Some(serde_json::json!(["^.+Error$"])),
        ),
        (
            "var test = function(any_error) { console.log(any_error); };",
            Some(serde_json::json!(["^.+(e|E)rror$"])),
        ),
    ];

    let fail = vec![
        ("function test(err) {}", None),
        ("function test(err, data) {}", None),
        ("function test(err) {errorLookingWord();}", None),
        ("function test(err) {try{} catch(err) {}}", None),
        ("function test(err, callback) { foo(function(err, callback) {}); }", None),
        ("var test = (err) => {};", None),
        ("var test = function(err) {};", None),
        ("var test = function test(err, data) {};", None),
        ("var test = function test(err) {/* if(err){} */};", None),
        ("function test(err) {doSomethingHere(function(err){console.log(err);})}", None),
        ("function test(error) {}", Some(serde_json::json!(["error"]))),
        (
            "getData(function(err, data) {getMoreDataWith(data, function(err, moreData) {if (err) {}getEvenMoreDataWith(moreData, function(err, allOfTheThings) {if (err) {}});}); });",
            None,
        ),
        (
            "getData(function(err, data) {getMoreDataWith(data, function(err, moreData) {getEvenMoreDataWith(moreData, function(err, allOfTheThings) {if (err) {}});}); });",
            None,
        ),
        (
            "function userHandler(err) {logThisAction(function(err) {if (err) { console.log(err); } })}",
            None,
        ),
        (
            "function help() { function userHandler(err) {function tester(err) { err; process.nextTick(function() { err; }); } } }",
            None,
        ),
        (
            "var test = function(anyError) { console.log(otherError); };",
            Some(serde_json::json!(["^.+Error$"])),
        ),
        ("var test = function(anyError) { };", Some(serde_json::json!(["^.+Error$"]))),
        (
            "var test = function(err) { console.log(error); };",
            Some(serde_json::json!(["^(err|error)$"])),
        ),
    ];

    Tester::new(HandleCallbackErr::NAME, HandleCallbackErr::PLUGIN, pass, fail).test_and_snapshot();
}
