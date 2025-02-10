use oxc_ast::{
    ast::{BindingPatternKind, Expression, FormalParameter, FormalParameters},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, rule::Rule, AstNode};

fn param_names_diagnostic(span: Span, pattern: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Promise constructor parameters must be named to match `{pattern}`"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ParamNames(Box<ParamNamesConfig>);

#[derive(Debug, Default, Clone)]
pub struct ParamNamesConfig {
    resolve_pattern: Option<Regex>,
    reject_pattern: Option<Regex>,
}

impl std::ops::Deref for ParamNames {
    type Target = ParamNamesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

enum ParamType {
    Resolve,
    Reject,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce standard parameter names for Promise constructors.
    ///
    /// ### Why is this bad?
    ///
    /// Ensures that new Promise() is instantiated with the parameter names resolve, reject to
    /// avoid confusion with order such as reject, resolve. The Promise constructor uses the
    /// RevealingConstructor pattern. Using the same parameter names as the language specification
    /// makes code more uniform and easier to understand.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// new Promise(function (reject, resolve) { /* ... */ }) // incorrect order
    /// new Promise(function (ok, fail) { /* ... */ }) // non-standard parameter names
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// new Promise(function(resolve, reject) {})
    /// ```
    ParamNames,
    promise,
    style,
);

impl Rule for ParamNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut cfg = ParamNamesConfig::default();

        if let Some(config) = value.get(0) {
            if let Some(val) = config.get("resolvePattern").and_then(serde_json::Value::as_str) {
                cfg.resolve_pattern = Regex::new(val).ok();
            }
            if let Some(val) = config.get("rejectPattern").and_then(serde_json::Value::as_str) {
                cfg.reject_pattern = Regex::new(val).ok();
            }
        }

        Self(Box::new(cfg))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        if !new_expr.callee.is_specific_id("Promise") || new_expr.arguments.len() != 1 {
            return;
        }

        for argument in &new_expr.arguments {
            let Some(arg_expr) = argument.as_expression() else {
                continue;
            };
            match arg_expr {
                Expression::ArrowFunctionExpression(arrow_expr) => {
                    self.check_parameter_names(&arrow_expr.params, ctx);
                }
                Expression::FunctionExpression(func_expr) => {
                    self.check_parameter_names(&func_expr.params, ctx);
                }
                _ => continue,
            }
        }
    }
}

impl ParamNames {
    fn check_parameter_names(&self, params: &FormalParameters, ctx: &LintContext) {
        if params.items.is_empty() {
            return;
        }

        self.check_parameter(&params.items[0], &ParamType::Resolve, ctx);

        if params.items.len() > 1 {
            self.check_parameter(&params.items[1], &ParamType::Reject, ctx);
        }
    }

    fn check_parameter(&self, param: &FormalParameter, param_type: &ParamType, ctx: &LintContext) {
        let BindingPatternKind::BindingIdentifier(param_ident) = &param.pattern.kind else {
            return;
        };

        let param_pattern = if matches!(param_type, ParamType::Reject) {
            &self.reject_pattern
        } else {
            &self.resolve_pattern
        };

        match param_pattern {
            Some(pattern) => {
                if !pattern.is_match(param_ident.name.as_str()) {
                    ctx.diagnostic(param_names_diagnostic(param_ident.span, pattern.as_str()));
                }
            }
            None => {
                if matches!(param_type, ParamType::Resolve)
                    && !matches!(param_ident.name.as_str(), "_resolve" | "resolve")
                {
                    ctx.diagnostic(param_names_diagnostic(param_ident.span, "^_?resolve$"));
                } else if matches!(param_type, ParamType::Reject)
                    && !matches!(param_ident.name.as_str(), "_reject" | "reject")
                {
                    ctx.diagnostic(param_names_diagnostic(param_ident.span, "^_?reject$"));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("new Promise(function(resolve, reject) {})", None),
        ("new Promise(function(resolve, _reject) {})", None),
        ("new Promise(function(_resolve, reject) {})", None),
        ("new Promise(function(_resolve, _reject) {})", None),
        ("new Promise(function(resolve) {})", None),
        ("new Promise(function(_resolve) {})", None),
        ("new Promise(resolve => {})", None),
        ("new Promise((resolve, reject) => {})", None),
        ("new Promise(() => {})", None),
        ("new NonPromise()", None),
        (
            "new Promise((yes, no) => {})",
            Some(serde_json::json!([{ "resolvePattern": "^yes$", "rejectPattern": "^no$" }])),
        ),
    ];

    let fail = vec![
        ("new Promise(function(reject, resolve) {})", None),
        ("new Promise(function(resolve, rej) {})", None),
        ("new Promise(yes => {})", None),
        ("new Promise((yes, no) => {})", None),
        (
            "new Promise(function(resolve, reject) { config(); })",
            Some(serde_json::json!([{ "resolvePattern": "^yes$", "rejectPattern": "^no$" }])),
        ),
    ];

    Tester::new(ParamNames::NAME, ParamNames::PLUGIN, pass, fail).test_and_snapshot();
}
