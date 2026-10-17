use oxc_ast::{
    AstKind,
    ast::{FormalParameters, FunctionBody},
};
use oxc_ast_visit::{VisitJs, walk_js};
use oxc_ecmascript::IsSimpleParameterList;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::GetSpan;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::get_function_name_with_kind,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

/*
{
    function: "Use the function form of 'use strict'.",
    global: "Use the global form of 'use strict'.",
    multiple: "Multiple 'use strict' directives.",
    never: "Strict mode is not permitted.",
    unnecessary: "Unnecessary 'use strict' directive.",
    module: "'use strict' is unnecessary inside of modules.",
    implied:
        "'use strict' is unnecessary when implied strict mode is enabled.",
    unnecessaryInClasses:
        "'use strict' is unnecessary inside of classes.",
    nonSimpleParameterList:
        "'use strict' directive inside a function with non-simple parameter list throws a syntax error since ES2016.",
    wrap: "Wrap {{name}} in a function with 'use strict' directive.",
}
*/

fn use_function_form_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use the function form of 'use strict'.").with_help("TODO").with_label(span)
}

fn use_global_form_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use the global form of 'use strict'.").with_help("TODO").with_label(span)
}

fn multiple_use_strict_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Multiple 'use strict' directives.").with_help("TODO").with_label(span)
}

fn strict_mode_not_permitted_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Strict mode is not permitted.").with_help("TODO").with_label(span)
}

fn unnecessary_use_strict_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary 'use strict' directive.").with_help("TODO").with_label(span)
}

fn unnecessary_use_strict_in_module_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("'use strict' is unnecessary inside of modules.")
        .with_help("TODO")
        .with_label(span)
}

fn unnecessary_use_strict_in_classes_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("'use strict' is unnecessary inside of classes.")
        .with_help("TODO")
        .with_label(span)
}

fn non_simple_parameter_list_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("'use strict' directive inside a function with non-simple parameter list throws a syntax error since ES2016.").with_label(span)
}

fn wrap_function_with_use_strict_diagnostic(span: Span, function_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Wrap {function_name} in a function with 'use strict' directive."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum StrictOption {
    #[default]
    Safe,
    Global,
    Function,
    Never,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Strict(StrictOption);

#[derive(Debug, Clone, Copy)]
enum StrictMode {
    Global,
    Function,
    Module,
    Never,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// FIXME: Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// FIXME: Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Add at least one example of code that violates the rule.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Add at least one example of code that is allowed with the rule.
    /// ```
    Strict,
    eslint,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    fix, // TODO: describe fix capabilities. Remove or set to `none` if no fix can be done,
         // keep at 'pending' if you think one could be added but don't know how.
         // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
    config = Strict,
    version = "next",
    short_description = "FIXME: One-sentence description of the rule.",
);

impl Rule for Strict {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let strict_mode = {
            if ctx.source_type().is_module() {
                StrictMode::Module
            } else {
                match self.0 {
                    StrictOption::Safe => {
                        if ctx.source_type().is_commonjs() {
                            StrictMode::Global
                        } else {
                            StrictMode::Function
                        }
                    }
                    StrictOption::Global => StrictMode::Global,
                    StrictOption::Function => StrictMode::Function,
                    StrictOption::Never => StrictMode::Never,
                }
            }
        };

        let AstKind::Program(program) = node.kind() else { return };
        let mut visitor = StrictVisitor::new(strict_mode, ctx);
        visitor.visit_program(program);
    }
}

struct StrictVisitor<'a, 'b> {
    ctx: &'b LintContext<'a>,
    strict_mode: StrictMode,
    function_scopes: Vec<bool>,
    class_scopes: Vec<bool>,
}

impl<'a> StrictVisitor<'a, '_> {
    fn new<'b>(strict_mode: StrictMode, ctx: &'b LintContext<'a>) -> StrictVisitor<'a, 'b> {
        StrictVisitor { ctx, strict_mode, function_scopes: Vec::new(), class_scopes: Vec::new() }
    }

    fn visit_function_for_directives(
        &mut self,
        function_name: &str,
        params: &FormalParameters,
        body: Option<&FunctionBody>,
        span: Span,
    ) -> bool {
        let is_in_class = self.class_scopes.len() > 0;
        let is_parent_global = self.function_scopes.len() == 0 && self.class_scopes.len() == 0;
        let is_parent_strict = *self.function_scopes.last().unwrap_or(&false);

        let directives = body.map(|body| &body.directives);
        let is_strict = directives.map(|directives| directives.len() > 0).unwrap_or(false);

        match self.strict_mode {
            StrictMode::Function => {
                if is_strict {
                    let directives = directives.unwrap(); // safe since is_strict == true means Some(directives)

                    for (i, directive) in directives.iter().enumerate() {
                        if !directive.is_use_strict() {
                            continue;
                        }

                        if i == 0 {
                            if !params.is_simple_parameter_list() {
                                self.ctx.diagnostic_with_fix(
                                    non_simple_parameter_list_diagnostic(directive.span),
                                    |fixer| fixer.delete(directive),
                                );
                            } else if is_parent_strict {
                                self.ctx.diagnostic_with_fix(
                                    unnecessary_use_strict_diagnostic(directive.span),
                                    |fixer| fixer.delete(directive),
                                );
                            } else if is_in_class {
                                self.ctx.diagnostic_with_fix(
                                    unnecessary_use_strict_in_classes_diagnostic(directive.span),
                                    |fixer| fixer.delete(directive),
                                );
                            }
                        } else {
                            self.ctx.diagnostic_with_fix(
                                multiple_use_strict_diagnostic(directive.span),
                                |fixer| fixer.delete(directive),
                            );
                        }
                    }
                } else if is_parent_global {
                    if params.is_simple_parameter_list() {
                        self.ctx.diagnostic(use_function_form_diagnostic(span));
                    } else {
                        self.ctx.diagnostic(wrap_function_with_use_strict_diagnostic(
                            span,
                            function_name,
                        ));
                    }
                }
            }
            strict_mode => {
                if let Some(directives) = directives {
                    for directive in directives.iter() {
                        if !directive.is_use_strict() {
                            continue;
                        }

                        match strict_mode {
                            StrictMode::Global => {
                                self.ctx.diagnostic(use_global_form_diagnostic(directive.span));
                            }
                            StrictMode::Module => {
                                self.ctx.diagnostic_with_fix(
                                    unnecessary_use_strict_in_module_diagnostic(directive.span),
                                    |fixer| fixer.delete(directive),
                                );
                            }
                            StrictMode::Never => {
                                self.ctx.diagnostic(strict_mode_not_permitted_diagnostic(
                                    directive.span,
                                ));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }

        is_parent_strict || is_strict
    }
}

impl<'a> VisitJs<'a> for StrictVisitor<'a, '_> {
    fn visit_program(&mut self, it: &oxc_ast::ast::Program<'a>) {
        match self.strict_mode {
            StrictMode::Global => {
                if it.directives.len() == 0 && it.body.len() > 0 {
                    self.ctx.diagnostic(use_global_form_diagnostic(it.body[0].span()));
                }

                for (i, directive) in it.directives.iter().enumerate() {
                    if !directive.is_use_strict() {
                        continue;
                    }

                    if i > 0 {
                        self.ctx.diagnostic_with_fix(
                            multiple_use_strict_diagnostic(directive.span),
                            |fixer| fixer.delete(directive),
                        );
                    }
                }
            }
            strict_mode => {
                for directive in it.directives.iter() {
                    if !directive.is_use_strict() {
                        continue;
                    }

                    match strict_mode {
                        StrictMode::Function => {
                            self.ctx.diagnostic(use_function_form_diagnostic(directive.span));
                        }
                        StrictMode::Module => {
                            self.ctx.diagnostic_with_fix(
                                unnecessary_use_strict_in_module_diagnostic(directive.span),
                                |fixer| fixer.delete(directive),
                            );
                        }
                        StrictMode::Never => {
                            self.ctx
                                .diagnostic(strict_mode_not_permitted_diagnostic(directive.span));
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        walk_js::walk_program(self, it);
    }

    fn visit_function(&mut self, it: &oxc_ast::ast::Function<'a>, flags: oxc_semantic::ScopeFlags) {
        let parent_node = self.ctx.nodes().parent_node(it.node_id.get());
        let node = self.ctx.nodes().get_node(it.node_id.get());
        let function_name_with_kind = &get_function_name_with_kind(node, parent_node);

        let is_scope_strict = self.visit_function_for_directives(
            function_name_with_kind,
            it.params.as_ref(),
            it.body.as_deref(),
            it.span,
        );

        self.function_scopes.push(is_scope_strict);
        walk_js::walk_function(self, it, flags);
        self.function_scopes.pop();
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        let parent_node = self.ctx.nodes().parent_node(it.node_id.get());
        let node = self.ctx.nodes().get_node(it.node_id.get());
        let function_name_with_kind = &get_function_name_with_kind(node, parent_node);

        let is_scope_strict = self.visit_function_for_directives(
            function_name_with_kind,
            it.params.as_ref(),
            it.body.as_function_body(),
            it.span,
        );

        self.function_scopes.push(is_scope_strict);
        walk_js::walk_arrow_function_expression(self, it);
        self.function_scopes.pop();
    }

    fn visit_class_body(&mut self, it: &oxc_ast::ast::ClassBody<'a>) {
        self.class_scopes.push(true);
        walk_js::walk_class_body(self, it);
        self.class_scopes.pop();
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo();", Some(serde_json::json!(["never"]))),
        ("function foo() { return; }", Some(serde_json::json!(["never"]))),
        ("var foo = function() { return; };", Some(serde_json::json!(["never"]))),
        ("foo(); 'use strict';", Some(serde_json::json!(["never"]))),
        ("function foo() { bar(); 'use strict'; return; }", Some(serde_json::json!(["never"]))),
        ("var foo = function() { { 'use strict'; } return; };", Some(serde_json::json!(["never"]))),
        ("(function() { bar('use strict'); return; }());", Some(serde_json::json!(["never"]))),
        ("var fn = x => 1;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("var fn = x => { return; };", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("foo();", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() { return; }", Some(serde_json::json!(["never"]))), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("// Intentionally empty", Some(serde_json::json!(["global"]))),
        (r#""use strict"; foo();"#, Some(serde_json::json!(["global"]))),
        (
            "/* license */
            /* eslint-disable rule-to-test/strict */
            foo();",
            Some(serde_json::json!(["global"])),
        ),
        ("foo();", Some(serde_json::json!(["global"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() { return; }", Some(serde_json::json!(["global"]))), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("'use strict'; function foo() { return; }", Some(serde_json::json!(["global"]))),
        ("'use strict'; var foo = function() { return; };", Some(serde_json::json!(["global"]))),
        (
            "'use strict'; function foo() { bar(); 'use strict'; return; }",
            Some(serde_json::json!(["global"])),
        ),
        (
            "'use strict'; var foo = function() { bar(); 'use strict'; return; };",
            Some(serde_json::json!(["global"])),
        ),
        (
            "'use strict'; function foo() { return function() { bar(); 'use strict'; return; }; }",
            Some(serde_json::json!(["global"])),
        ),
        (
            "'use strict'; var foo = () => { return () => { bar(); 'use strict'; return; }; }",
            Some(serde_json::json!(["global"])),
        ), // { "ecmaVersion": 6 },
        ("function foo() { 'use strict'; return; }", Some(serde_json::json!(["function"]))),
        ("function foo() { return; }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() { return; }", Some(serde_json::json!(["function"]))), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("var foo = function() { return; }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var foo = function() { 'use strict'; return; }", Some(serde_json::json!(["function"]))),
        (
            "function foo() { 'use strict'; return; } var bar = function() { 'use strict'; bar(); };",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() { 'use strict'; function bar() { return; } bar(); };",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = () => { 'use strict'; var bar = () => 1; bar(); };",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 6 },
        ("var foo = () => { var bar = () => 1; bar(); };", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("class A { constructor() { } }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        ("class A { foo() { } }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        ("class A { foo() { function bar() { } } }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        (
            "(function() { 'use strict'; function foo(a = 0) { } }())",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 6 },
        ("function foo() { 'use strict'; return; }", Some(serde_json::json!(["safe"]))),
        ("'use strict'; function foo() { return; }", Some(serde_json::json!(["safe"]))), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("function foo() { return; }", Some(serde_json::json!(["safe"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() { return; }", Some(serde_json::json!(["safe"]))), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("function foo() { 'use strict'; return; }", None),
        ("'use strict'; function foo() { return; }", None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("function foo() { return; }", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() { return; }", None), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("'use strict'; class C { static { foo; } }", Some(serde_json::json!(["global"]))), // { "ecmaVersion": 2022 },
        ("'use strict'; class C { static { 'use strict'; } }", Some(serde_json::json!(["global"]))), // { "ecmaVersion": 2022 },
        (
            "'use strict'; class C { static { 'use strict'; 'use strict'; } }",
            Some(serde_json::json!(["global"])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { foo; } }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 2022 },
        ("class C { static { 'use strict'; } }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { 'use strict'; 'use strict'; } }",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { foo; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { 'use strict'; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { 'use strict'; 'use strict'; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { 'use strict'; } }", Some(serde_json::json!(["safe"]))), // { "ecmaVersion": 2022, "sourceType": "module" },
        ("class C { static { 'use strict'; } }", Some(serde_json::json!(["safe"]))), // { "ecmaVersion": 2022, "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("'use strict'; module.exports = function identity (value) { return value; }", None), // { "sourceType": "commonjs", },
        (
            "'use strict'; module.exports = function identity (value) { return value; }",
            Some(serde_json::json!(["safe"])),
        ), // { "sourceType": "commonjs", }
    ];

    let fail = vec![
        (r#""use strict"; foo();"#, Some(serde_json::json!(["never"]))),
        ("function foo() { 'use strict'; return; }", Some(serde_json::json!(["never"]))),
        ("var foo = function() { 'use strict'; return; };", Some(serde_json::json!(["never"]))),
        (
            "function foo() { return function() { 'use strict'; return; }; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            r#"'use strict'; function foo() { "use strict"; return; }"#,
            Some(serde_json::json!(["never"])),
        ),
        (r#""use strict"; foo();"#, Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["never"])),
        ), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 6, "sourceType": "module", "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        // ("foo();", Some(serde_json::json!(["global"]))),
        (
            "/* license */
            function foo() {}
            function bar() {}
            /* end */",
            Some(serde_json::json!(["global"])),
        ),
        ("function foo() { 'use strict'; return; }", Some(serde_json::json!(["global"]))),
        ("var foo = function() { 'use strict'; return; }", Some(serde_json::json!(["global"]))),
        ("var foo = () => { 'use strict'; return () => 1; }", Some(serde_json::json!(["global"]))), // { "ecmaVersion": 6 },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["global"])),
        ),
        (
            "'use strict'; var foo = function() { 'use strict'; return; };",
            Some(serde_json::json!(["global"])),
        ),
        ("'use strict'; 'use strict'; foo();", Some(serde_json::json!(["global"]))),
        // ("'use strict'; foo();", Some(serde_json::json!(["global"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["global"])),
        ), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["global"])),
        ), // { "ecmaVersion": 6, "sourceType": "module", "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("'use strict'; foo();", Some(serde_json::json!(["function"]))),
        (
            "'use strict'; (function() { 'use strict'; return true; }());",
            Some(serde_json::json!(["function"])),
        ),
        (
            "(function() { 'use strict'; function f() { 'use strict'; return } return true; }());",
            Some(serde_json::json!(["function"])),
        ),
        ("(function() { return true; }());", Some(serde_json::json!(["function"]))),
        ("(() => { return true; })();", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        ("(() => true)();", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        (
            "var foo = function() { foo(); 'use strict'; return; }; function bar() { foo(); 'use strict'; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "function foo() { 'use strict'; 'use strict'; return; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() { 'use strict'; 'use strict'; return; }",
            Some(serde_json::json!(["function"])),
        ),
        ("var foo = function() { 'use strict'; return; }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["function"])),
        ), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 6, "sourceType": "module", "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        (
            "function foo() { return function() { 'use strict'; return; }; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() { function bar() { 'use strict'; return; } return; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "function foo() { 'use strict'; return; } var bar = function() { return; };",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() { 'use strict'; return; }; function bar() { return; };",
            Some(serde_json::json!(["function"])),
        ),
        (
            "function foo() { 'use strict'; return function() { 'use strict'; 'use strict'; return; }; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() { 'use strict'; function bar() { 'use strict'; 'use strict'; return; } }",
            Some(serde_json::json!(["function"])),
        ),
        ("var foo = () => { return; };", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        (r#"class A { constructor() { "use strict"; } }"#, Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        (r#"class A { foo() { "use strict"; } }"#, Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        (
            r#"class A { foo() { function bar() { "use strict"; } } }"#,
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 6 },
        (r#"class A { field = () => { "use strict"; } }"#, Some(serde_json::json!(["function"]))), // { "ecmaVersion": 2022 },
        (
            r#"class A { field = function() { "use strict"; } }"#,
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 2022 },
        ("'use strict'; function foo() { return; }", Some(serde_json::json!(["safe"]))),
        ("function foo() { 'use strict'; return; }", Some(serde_json::json!(["safe"]))), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["safe"])),
        ), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            Some(serde_json::json!(["safe"])),
        ), // { "ecmaVersion": 6, "sourceType": "module", "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("'use strict'; function foo() { return; }", None),
        ("function foo() { return; }", None),
        ("function foo() { 'use strict'; return; }", None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("'use strict'; function foo() { 'use strict'; return; }", None), // { "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("'use strict'; function foo() { 'use strict'; return; }", None), // { "ecmaVersion": 6, "sourceType": "module", "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        ("function foo(a = 0) { 'use strict' }", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        (
            "(function() { 'use strict'; function foo(a = 0) { 'use strict' } }())",
            Some(serde_json::json!([])),
        ), // { "ecmaVersion": 6 },
        ("function foo(a = 0) { 'use strict' }", Some(serde_json::json!([]))), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("'use strict'; function foo(a = 0) { 'use strict' }", Some(serde_json::json!([]))), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("function foo(a = 0) { 'use strict' }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("function foo(a = 0) { 'use strict' }", Some(serde_json::json!(["global"]))), // { "ecmaVersion": 6 },
        ("'use strict'; function foo(a = 0) { 'use strict' }", Some(serde_json::json!(["global"]))), // { "ecmaVersion": 6 },
        ("function foo(a = 0) { 'use strict' }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        (
            "(function() { 'use strict'; function foo(a = 0) { 'use strict' } }())",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 6 },
        ("function foo(a = 0) { }", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        ("(function() { function foo(a = 0) { } }())", Some(serde_json::json!(["function"]))), // { "ecmaVersion": 6 },
        (
            "'use strict'; class C { static { function foo() { 
            'use strict'; } } }",
            Some(serde_json::json!(["global"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { function foo() { 
            'use strict'; } } }",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { function foo() {
            'use strict'; } } }",
            Some(serde_json::json!(["safe"])),
        ), // { "ecmaVersion": 2022, "sourceType": "module" },
        (
            "class C { static { function foo() {
            'use strict'; } } }",
            Some(serde_json::json!(["safe"])),
        ), // { "ecmaVersion": 2022, "parserOptions": { "ecmaFeatures": { "impliedStrict": true } }, },
        (
            "function foo() {'use strict'; class C { static { function foo() { 
            'use strict'; } } } }",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { function foo() { 
            'use strict'; } } }",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { function foo() { 
            'use strict';
            'use strict'; } } }",
            Some(serde_json::json!(["function"])),
        ), // { "ecmaVersion": 2022 },
        (
            "module.exports = function identity (value) { return value; }",
            Some(serde_json::json!(["safe"])),
        ), // { "sourceType": "commonjs", },
        ("module.exports = function identity (value) { return value; }", None), // { "sourceType": "commonjs", }
    ];

    let fix = vec![
        (r#""use strict"; foo();"#, " foo();", Some(serde_json::json!(["never"]))),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "'use strict'; 'use strict'; foo();",
            "'use strict';  foo();",
            Some(serde_json::json!(["global"])),
        ),
        ("'use strict'; foo();", " foo();", Some(serde_json::json!(["global"]))),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["global"])),
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["global"])),
        ),
        (
            "(function() { 'use strict'; function f() { 'use strict'; return } return true; }());",
            "(function() { 'use strict'; function f() {  return } return true; }());",
            Some(serde_json::json!(["function"])),
        ),
        (
            "function foo() { 'use strict'; 'use strict'; return; }",
            "function foo() { 'use strict';  return; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() { 'use strict'; 'use strict'; return; }",
            "var foo = function() { 'use strict';  return; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() {  'use strict'; return; }",
            "var foo = function() {   return; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "function foo() { 'use strict'; return function() { 'use strict'; 'use strict'; return; }; }",
            "function foo() { 'use strict'; return function() {   return; }; }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "var foo = function() { 'use strict'; function bar() { 'use strict'; 'use strict'; return; } }",
            "var foo = function() { 'use strict'; function bar() {   return; } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            r#"class A { constructor() { "use strict"; } }"#,
            "class A { constructor() {  } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            r#"class A { foo() { "use strict"; } }"#,
            "class A { foo() {  } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            r#"class A { foo() { function bar() { "use strict"; } } }"#,
            "class A { foo() { function bar() {  } } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            r#"class A { field = () => { "use strict"; } }"#,
            "class A { field = () => {  } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            r#"class A { field = function() { "use strict"; } }"#,
            "class A { field = function() {  } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["safe"])),
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            Some(serde_json::json!(["safe"])),
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            None,
        ),
        (
            "'use strict'; function foo() { 'use strict'; return; }",
            " function foo() {  return; }",
            None,
        ),
        (
            "class C { static { function foo() {
            'use strict'; } } }",
            "class C { static { function foo() {
             } } }",
            Some(serde_json::json!(["safe"])),
        ),
        (
            "class C { static { function foo() {
            'use strict'; } } }",
            "class C { static { function foo() {
             } } }",
            Some(serde_json::json!(["safe"])),
        ),
        (
            "function foo() {'use strict'; class C { static { function foo() { 
            'use strict'; } } } }",
            "function foo() {'use strict'; class C { static { function foo() { 
             } } } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "class C { static { function foo() { 
            'use strict'; } } }",
            "class C { static { function foo() { 
             } } }",
            Some(serde_json::json!(["function"])),
        ),
        (
            "class C { static { function foo() { 
            'use strict';
            'use strict'; } } }",
            "class C { static { function foo() { 
            
             } } }",
            Some(serde_json::json!(["function"])),
        ),
    ];

    Tester::new(Strict::NAME, Strict::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
