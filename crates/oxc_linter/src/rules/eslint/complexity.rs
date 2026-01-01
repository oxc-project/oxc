use oxc_ast::AstKind;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::AssignmentOperator;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;

use crate::{
    AstNode,
    ast_util::get_function_name_with_kind,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn complexity_diagnostic(span: Span, name: &str, complexity: usize, max: usize) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn(format!(
        "{name} has a complexity of {complexity}. Maximum allowed is {max}."
    ))
    .with_label(span)
}

const THRESHOLD_DEFAULT: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[schemars(rename_all = "camelCase")]
pub struct ComplexityConfig {
    /// Maximum amount of cyclomatic complexity
    max: usize,
    /// The cyclomatic complexity variant to use
    variant: Variant,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self { max: THRESHOLD_DEFAULT, variant: Variant::Classic }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[schemars(untagged, rename_all = "camelCase")]
pub enum Variant {
    /// Classic means McCabe cyclomatic complexity
    Classic,
    /// Modified means classic cyclomatic complexity but a switch statement increases
    /// complexity by 1 irrespective of the number of `case` statements
    Modified,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Complexity(Box<ComplexityConfig>);

impl Deref for Complexity {
    type Target = ComplexityConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a maximum cyclomatic complexity in a program, which is the number
    /// of linearly independent paths in a program.
    ///
    /// ### Why is this bad?
    ///
    /// Having high code complexity reduces code readability. This rule
    /// aims to make the code easier to follow by reducing the number of branches
    /// in the program.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with `{ "max": 2 }`
    /// ```js
    /// function foo() {
    ///   if (foo1) {
    ///     return x1; // 1st path
    ///   } else if (foo2) {
    ///     return x2; // 2nd path
    ///   } else {
    ///     return x3; // 3rd path
    ///   }
    /// }
    ///
    /// function bar() {
    ///   // there are 2 paths - when bar1 is falsy, and when bar1 is truthy, in which bar1 = bar1 && bar2;
    ///   bar1 &&= bar2;
    ///   // there are 2 paths - when bar3 is truthy, and when bar3 is falsy, in which bar3 = 4;
    ///   bar3 ||= 4;
    /// }
    ///
    /// // there are 2 paths - when baz1 is defined, and when baz1 is undefined and is assigned 'a'
    /// function baz(baz1 = 'a') {
    ///   const { baz2 = 'b' } = baz3; // there are 2 additional paths - when baz2 is defined and when baz2 is not
    /// }
    ///
    /// function d() {
    ///   d1 = d2?.d3?.(); // optional chaining creates 2 paths each - when object is defined and when it is not
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "max": 2 }`
    /// ```js
    /// // This example is taken directly from ESLint documentation
    /// function foo() { // this function has complexity = 1
    ///   class C {
    ///     x = a + b; // this initializer has complexity = 1
    ///     y = c || d; // this initializer has complexity = 2
    ///     z = e && f; // this initializer has complexity = 2
    ///
    ///     static p = g || h; // this initializer has complexity = 2
    ///     static q = i ? j : k; // this initializer has complexity = 2
    ///
    ///     static { // this static block has complexity = 2
    ///       if (foo) {
    ///         baz = bar;
    ///       }
    ///     }
    ///
    ///     static { // this static block has complexity = 2
    ///       qux = baz || quux;
    ///     }
    ///   }
    /// }
    /// ```
    Complexity,
    eslint,
    style,
    config = ComplexityConfig,
);

impl Rule for Complexity {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        if let Some(max) = value
            .get(0)
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Ok(Self(Box::new(ComplexityConfig { max, variant: Variant::Classic })))
        } else {
            Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
                .unwrap_or_default()
                .into_inner())
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let mut visitor = ComplexityVisitor::new(self.variant);
        let (span, diagnostic_type) = match node.kind() {
            AstKind::Function(func) => {
                visitor.visit_function(func, ScopeFlags::Function);
                (func.span, DiagnosticType::Function)
            }
            AstKind::ArrowFunctionExpression(func) => {
                visitor.visit_arrow_function_expression(func);
                (func.span, DiagnosticType::Function)
            }
            AstKind::StaticBlock(block) => {
                visitor.visit_static_block(block);
                (block.span, DiagnosticType::ClassStaticBlock)
            }
            AstKind::PropertyDefinition(prop_def) => {
                if let Some(expr) = &prop_def.value {
                    visitor.visit_property_definition(prop_def);
                    (expr.span(), DiagnosticType::ClassPropertyInitializer)
                } else {
                    return;
                }
            }
            _ => {
                return;
            }
        };

        if visitor.complexity > self.max {
            let name = match diagnostic_type {
                DiagnosticType::ClassStaticBlock => "class static block",
                DiagnosticType::ClassPropertyInitializer => "class field initializer",
                DiagnosticType::Function => {
                    let parent_node = ctx.nodes().parent_node(node.id());
                    &get_function_name_with_kind(node, parent_node)
                }
            };

            ctx.diagnostic(complexity_diagnostic(span, name, visitor.complexity, self.max));
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DiagnosticType {
    ClassPropertyInitializer,
    ClassStaticBlock,
    Function,
}

struct ComplexityVisitor {
    variant: Variant,
    complexity: usize,
    has_entered_complexity_evaluation: bool,
}

impl ComplexityVisitor {
    fn new(variant: Variant) -> Self {
        Self { variant, complexity: 1, has_entered_complexity_evaluation: false }
    }
}

impl Visit<'_> for ComplexityVisitor {
    fn visit_catch_clause(&mut self, it: &oxc_ast::ast::CatchClause<'_>) {
        self.complexity += 1;
        walk::walk_catch_clause(self, it);
    }

    fn visit_conditional_expression(&mut self, it: &oxc_ast::ast::ConditionalExpression<'_>) {
        self.complexity += 1;
        walk::walk_conditional_expression(self, it);
    }

    fn visit_logical_expression(&mut self, it: &oxc_ast::ast::LogicalExpression<'_>) {
        self.complexity += 1;
        walk::walk_logical_expression(self, it);
    }

    fn visit_for_statement(&mut self, it: &oxc_ast::ast::ForStatement<'_>) {
        self.complexity += 1;
        walk::walk_for_statement(self, it);
    }

    fn visit_for_in_statement(&mut self, it: &oxc_ast::ast::ForInStatement<'_>) {
        self.complexity += 1;
        walk::walk_for_in_statement(self, it);
    }

    fn visit_for_of_statement(&mut self, it: &oxc_ast::ast::ForOfStatement<'_>) {
        self.complexity += 1;
        walk::walk_for_of_statement(self, it);
    }

    fn visit_if_statement(&mut self, it: &oxc_ast::ast::IfStatement<'_>) {
        self.complexity += 1;
        walk::walk_if_statement(self, it);
    }

    fn visit_while_statement(&mut self, it: &oxc_ast::ast::WhileStatement<'_>) {
        self.complexity += 1;
        walk::walk_while_statement(self, it);
    }

    fn visit_do_while_statement(&mut self, it: &oxc_ast::ast::DoWhileStatement<'_>) {
        self.complexity += 1;
        walk::walk_do_while_statement(self, it);
    }

    fn visit_assignment_pattern(&mut self, it: &oxc_ast::ast::AssignmentPattern<'_>) {
        self.complexity += 1;
        walk::walk_assignment_pattern(self, it);
    }

    fn visit_formal_parameter(&mut self, it: &oxc_ast::ast::FormalParameter<'_>) {
        if let Some(_) = &it.initializer {
            self.complexity += 1;
        }
        walk::walk_formal_parameter(self, it);
    }

    fn visit_switch_case(&mut self, it: &oxc_ast::ast::SwitchCase<'_>) {
        if self.variant == Variant::Classic && it.test.is_some() {
            self.complexity += 1;
        }
        walk::walk_switch_case(self, it);
    }

    fn visit_switch_statement(&mut self, it: &oxc_ast::ast::SwitchStatement<'_>) {
        if self.variant == Variant::Modified {
            self.complexity += 1;
        }
        walk::walk_switch_statement(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &oxc_ast::ast::AssignmentExpression<'_>) {
        if matches!(
            it.operator,
            AssignmentOperator::LogicalAnd
                | AssignmentOperator::LogicalOr
                | AssignmentOperator::LogicalNullish
        ) {
            self.complexity += 1;
        }
        walk::walk_assignment_expression(self, it);
    }

    fn visit_member_expression(&mut self, it: &oxc_ast::ast::MemberExpression<'_>) {
        if it.optional() {
            self.complexity += 1;
        }
        walk::walk_member_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &oxc_ast::ast::CallExpression<'_>) {
        if it.optional {
            self.complexity += 1;
        }
        walk::walk_call_expression(self, it);
    }

    fn visit_function(&mut self, it: &oxc_ast::ast::Function<'_>, flags: oxc_semantic::ScopeFlags) {
        if !self.has_entered_complexity_evaluation {
            self.has_entered_complexity_evaluation = true;
            walk::walk_function(self, it, flags);
        }
        // Do not enter function if we already started evaluating complexity
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'_>) {
        if !self.has_entered_complexity_evaluation {
            self.has_entered_complexity_evaluation = true;
            walk::walk_arrow_function_expression(self, it);
        }
        // Do not enter function if we already started evaluating complexity
    }

    fn visit_static_block(&mut self, it: &oxc_ast::ast::StaticBlock<'_>) {
        if !self.has_entered_complexity_evaluation {
            self.has_entered_complexity_evaluation = true;
            walk::walk_static_block(self, it);
        }
        // Do not enter static block if we already started evaluating complexity
    }

    fn visit_property_definition(&mut self, it: &oxc_ast::ast::PropertyDefinition<'_>) {
        if !self.has_entered_complexity_evaluation {
            if let Some(value) = &it.value {
                self.has_entered_complexity_evaluation = true;
                self.visit_expression(value);
            } else {
                // Do not visit any other node if there is no value expression to
                // evaluate - only visit all other nodes if it is part of another
                // function / static block evaluation
                return;
            }
        } else {
            // Visit all other nodes except for value expression if part of another
            // function / static block's complexity evaluation
            let kind = oxc_ast::AstKind::PropertyDefinition(self.alloc(it));
            self.enter_node(kind);
            self.visit_span(&it.span);
            self.visit_decorators(&it.decorators);
            self.visit_property_key(&it.key);
            if let Some(type_annotation) = &it.type_annotation {
                self.visit_ts_type_annotation(type_annotation);
            }
            if let Some(value) = &it.value {
                // Do not enter value expression if we already started evaluating complexity
            }
            self.leave_node(kind);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function a(x) {}", None),
        ("function b(x) {}", Some(serde_json::json!([1]))),
        ("function a(x) {if (true) {return x;}}", Some(serde_json::json!([2]))),
        ("function a(x) {if (true) {return x;} else {return x+1;}}", Some(serde_json::json!([2]))),
        (
            "function a(x) {if (true) {return x;} else if (false) {return x+1;} else {return 4;}}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {x ++;} return x;}",
            Some(serde_json::json!([2])),
        ),
        ("function a(obj) {for(var i in obj) {obj[i] = 3;}}", Some(serde_json::json!([2]))),
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {if(i % 2 === 0) {x ++;}} return x;}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(obj) {if(obj){ for(var x in obj) {try {x.getThis();} catch (e) {x.getThat();}}} else {return false;}}",
            Some(serde_json::json!([4])),
        ),
        (
            "function a(x) {try {x.getThis();} catch (e) {x.getThat();}}",
            Some(serde_json::json!([2])),
        ),
        ("function a(x) {return x === 4 ? 3 : 5;}", Some(serde_json::json!([2]))),
        ("function a(x) {return x === 4 ? 3 : (x === 3 ? 2 : 1);}", Some(serde_json::json!([3]))),
        ("function a(x) {return x || 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x && 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x ?? 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x ||= 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x &&= 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x ??= 4;}", Some(serde_json::json!([2]))),
        ("function a(x) {x = 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x |= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x &= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x += 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x >>= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x >>>= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x == 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x === 4;}", Some(serde_json::json!([1]))),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([4])),
        ),
        ("function a(x) {while(true) {'foo';}}", Some(serde_json::json!([2]))),
        ("function a(x) {do {'foo';} while (true)}", Some(serde_json::json!([2]))),
        ("if (foo) { bar(); }", Some(serde_json::json!([3]))),
        ("var a = (x) => {do {'foo';} while (true)}", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([{ "max": 2, "variant": "modified" }])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([{ "max": 3, "variant": "modified" }])),
        ),
        ("function foo() { class C { x = a || b; y = c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static x = a || b; static y = c || d; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { x = a || b; y = c || d; } e || f; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { a || b; class C { x = c || d; y = e || f; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("function foo() { class C { [x || y] = a || b; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; y() { c || d; } z = e || f; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x() { a || b; } y = c || d; z() { e || f; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = (() => { a || b }) || (() => { c || d }) }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => { a || b }; y = () => { c || d } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || (() => { b || c }); }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = class { y = a || b; z = c || d; }; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || class { y = b || c; z = d || e; }; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x; y = a; static z; static q = b; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static { a || b; } static { c || d; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("function foo() { a || b; class C { static { c || d; } } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo() { class C { static { a || b; } } c || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static { a || b; } } class D { static { c || d; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; } static { c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static { c || d; } static { e || f; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { () => a || b; c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; () => c || d; } static { c || d; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a } }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { static { a } static { b } }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } } class D { static { c || d; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; } static c = d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static a = b || c; static { c || d; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; } c = d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { a = b || c; static { d || e; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; } }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        ("class C { static { if (a || b) c = d || e; } }", Some(serde_json::json!([4]))), // { "ecmaVersion": 2022 },
        ("function b(x) {}", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { b?.c; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b = '') {}", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { const { c = '' } = b; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { const [ c = '' ] = b; }", Some(serde_json::json!([{ "max": 2 }]))),
    ];

    let fail = vec![
        ("function a(x) {}", Some(serde_json::json!([0]))),
        (
            "function foo(x) {if (x > 10) {return 'x is greater than 10';} else if (x > 5) {return 'x is greater than 5';} else {return 'x is less than 5';}}",
            Some(serde_json::json!([2])),
        ),
        ("var func = function () {}", Some(serde_json::json!([0]))),
        ("var obj = { a(x) {} }", Some(serde_json::json!([0]))), // { "ecmaVersion": 6 },
        ("class Test { a(x) {} }", Some(serde_json::json!([0]))), // { "ecmaVersion": 6 },
        ("var a = (x) => {if (true) {return x;}}", Some(serde_json::json!([1]))), // { "ecmaVersion": 6 },
        ("function a(x) {if (true) {return x;}}", Some(serde_json::json!([1]))),
        ("function a(x) {if (true) {return x;} else {return x+1;}}", Some(serde_json::json!([1]))),
        (
            "function a(x) {if (true) {return x;} else if (false) {return x+1;} else {return 4;}}",
            Some(serde_json::json!([2])),
        ),
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {x ++;} return x;}",
            Some(serde_json::json!([1])),
        ),
        ("function a(obj) {for(var i in obj) {obj[i] = 3;}}", Some(serde_json::json!([1]))),
        ("function a(obj) {for(var i of obj) {obj[i] = 3;}}", Some(serde_json::json!([1]))), // { "ecmaVersion": 6 },
        (
            "function a(x) {for(var i = 0; i < 5; i ++) {if(i % 2 === 0) {x ++;}} return x;}",
            Some(serde_json::json!([2])),
        ),
        (
            "function a(obj) {if(obj){ for(var x in obj) {try {x.getThis();} catch (e) {x.getThat();}}} else {return false;}}",
            Some(serde_json::json!([3])),
        ),
        (
            "function a(x) {try {x.getThis();} catch (e) {x.getThat();}}",
            Some(serde_json::json!([1])),
        ),
        ("function a(x) {return x === 4 ? 3 : 5;}", Some(serde_json::json!([1]))),
        ("function a(x) {return x === 4 ? 3 : (x === 3 ? 2 : 1);}", Some(serde_json::json!([2]))),
        ("function a(x) {return x || 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x && 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x ?? 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x ||= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x &&= 4;}", Some(serde_json::json!([1]))),
        ("function a(x) {x ??= 4;}", Some(serde_json::json!([1]))),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([2])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([3])),
        ),
        ("function a(x) {while(true) {'foo';}}", Some(serde_json::json!([1]))),
        ("function a(x) {do {'foo';} while (true)}", Some(serde_json::json!([1]))),
        (
            "function a(x) {(function() {while(true){'foo';}})(); (function() {while(true){'bar';}})();}",
            Some(serde_json::json!([1])),
        ),
        (
            "function a(x) {(function() {while(true){'foo';}})(); (function() {'bar';})();}",
            Some(serde_json::json!([1])),
        ),
        ("var obj = { a(x) { return x ? 0 : 1; } };", Some(serde_json::json!([1]))), // { "ecmaVersion": 6 },
        ("var obj = { a: function b(x) { return x ? 0 : 1; } };", Some(serde_json::json!([1]))),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: 3;}}",
            Some(serde_json::json!([{ "max": 1, "variant": "modified" }])),
        ),
        (
            "function a(x) {switch(x){case 1: 1; break; case 2: 2; break; default: if(x == 'foo') {5;};}}",
            Some(serde_json::json!([{ "max": 2, "variant": "modified" }])),
        ),
        ("function foo () { a || b; class C { x; } c || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { x = c; } d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { [x || y]; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { [x || y] = c; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { class C { [x || y]; } a || b; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { class C { [x || y] = a; } b || c; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function foo () { class C { [x || y]; [z || q]; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo () { class C { [x || y] = a; [z || q] = b; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo () { a || b; class C { x = c || d; } e || f; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { x(){ a || b; } y = c || d || e; z() { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; y() { c || d || e; } z = f || g; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x; y() { c || d || e; } z; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("(class { x = a || b; })", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { static x = a || b; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("(class { x = a ? b : c; })", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b || c; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b; y = b || c || d; z = e || f; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = a || b || c; y = d || e; z = f || g || h; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = (() => a || b || c) || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; y = d || e; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; y = d || e || f; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "class C { x = function () { a || b }; y = function () { c || d }; }",
            Some(serde_json::json!([1])),
        ), // { "ecmaVersion": 2022 },
        ("class C { x = class { [y || z]; }; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { x = class { [y || z] = a; }; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { x = class { y = a || b; }; }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("function foo () { a || b; class C { static {} } c || d; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo () { a || b; class C { static { c || d; } } e || f; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; }  }", Some(serde_json::json!([1]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b || c; }  }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; }  }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; e || f; }  }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        ("class C { static { a || b; c || d; { e || f; } }  }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        ("class C { static { if (a || b) c = d || e; } }", Some(serde_json::json!([3]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { if (a || b) c = (d => e || f)() || (g => h || i)(); } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { x(){ a || b; } static { c || d || e; } z() { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { x = a || b; static { c || d || e; } y = f || g; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static x = a || b; static { c || d || e; } static y = f || g; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static(){ c || d || e; } static { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static static(){ c || d || e; } static { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b; } static x = c || d || e; static { f || g; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b || c || d; } static { e || f || g; } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b || c; } static { d || e || f || g; } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { a || b || c || d; } static { e || f || g || h; } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 2022 },
        ("class C { x = () => a || b || c; y = f || g || h; }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        ("function a(x) {}", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "const obj = { b: (a) => a?.b?.c, c: function (a) { return a?.b?.c; } };",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        ("function a(b) { b?.c; }", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { b?.['c']; }", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { b?.c; d || e; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.d; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.['c']?.['d']; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.['d']; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c.d?.e; }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.(); }", Some(serde_json::json!([{ "max": 2 }]))),
        ("function a(b) { b?.c?.()?.(); }", Some(serde_json::json!([{ "max": 3 }]))),
        ("function a(b = '') {}", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { const { c = '' } = b; }", Some(serde_json::json!([{ "max": 1 }]))),
        ("function a(b) { const [ c = '' ] = b; }", Some(serde_json::json!([{ "max": 1 }]))),
        (
            "function a(b) { const [ { c: d = '' } = {} ] = b; }",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
    ];

    Tester::new(Complexity::NAME, Complexity::PLUGIN, pass, fail).test_and_snapshot();
}
