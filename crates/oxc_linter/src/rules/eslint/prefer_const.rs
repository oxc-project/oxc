use oxc_ast::{AstKind, ast::VariableDeclarationKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_const_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is never reassigned"))
        .with_help("Use 'const' instead".to_string())
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct PreferConstConfig {
    destructuring: Destructuring,
    ignore_read_before_assign: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Destructuring {
    #[default]
    Any,
    All,
}

impl Default for PreferConstConfig {
    fn default() -> Self {
        Self { destructuring: Destructuring::Any, ignore_read_before_assign: false }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreferConst(PreferConstConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires `const` declarations for variables that are never reassigned after declared.
    ///
    /// ### Why is this bad?
    ///
    /// If a variable is never reassigned, using the `const` declaration is better.
    /// `const` declaration tells readers, "this variable is never reassigned," reducing cognitive load and improving maintainability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let a = 3;
    /// console.log(a);
    ///
    /// let b;
    /// b = 0;
    /// console.log(b);
    ///
    /// for (let i in [1,2,3]) {
    ///   console.log(i);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = 0;
    ///
    /// let a;
    /// a = 0;
    /// a = 1;
    ///
    /// let a;
    /// if (true) {
    ///   a = 0;
    /// }
    ///
    /// for (const i in [1,2,3]) {
    ///   console.log(i);
    /// }
    /// ```
    PreferConst,
    eslint,
    style,
    pending,
    config = PreferConstConfig,
);

impl Rule for PreferConst {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(value.get(0).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(decl) = node.kind() else {
            return;
        };

        // Only check `let` declarations (not `var` or `const`)
        if decl.kind != VariableDeclarationKind::Let {
            return;
        }

        // Get parent to check if we're in a for-in or for-of loop initializer
        let parent = ctx.nodes().parent_node(node.id());
        let is_for_in_of_init =
            matches!(parent.kind(), AstKind::ForInStatement(_) | AstKind::ForOfStatement(_));

        // For regular for loops (for (let i = 0, j = 1; ...)), we need special handling
        // If ANY variable in the declaration is reassigned, we can't fix the whole declaration
        let is_for_statement_init = matches!(parent.kind(), AstKind::ForStatement(_));

        if is_for_statement_init && decl.declarations.len() > 1 {
            // Check if any declarator is reassigned
            let any_reassigned = decl.declarations.iter().any(|declarator| {
                let has_init = declarator.init.is_some();
                declarator.id.get_binding_identifiers().iter().any(|ident| {
                    let symbol_id = ident.symbol_id();
                    !self.should_be_const(symbol_id, has_init, is_for_in_of_init, ctx)
                })
            });

            // If any variable is reassigned, we can't convert any of them
            if any_reassigned {
                return;
            }
        }

        for declarator in &decl.declarations {
            let binding_identifiers = declarator.id.get_binding_identifiers();
            let has_init = declarator.init.is_some();
            let is_destructuring = declarator.id.kind.is_destructuring_pattern();

            // For destructuring patterns with "all" mode, check if ALL variables should be const
            if matches!(self.0.destructuring, Destructuring::All) && is_destructuring {
                let all_const = binding_identifiers.iter().all(|ident| {
                    let symbol_id = ident.symbol_id();
                    self.should_be_const(symbol_id, has_init, is_for_in_of_init, ctx)
                });

                if all_const {
                    // Flag all variables in the destructuring pattern
                    for ident in binding_identifiers {
                        ctx.diagnostic(prefer_const_diagnostic(ident.name.as_str(), ident.span));
                    }
                }
                // If not all_const, don't flag any of them in "all" mode
            } else {
                // "any" mode (default): check each binding identifier independently
                for ident in binding_identifiers {
                    let symbol_id = ident.symbol_id();
                    if self.should_be_const(symbol_id, has_init, is_for_in_of_init, ctx) {
                        ctx.diagnostic(prefer_const_diagnostic(ident.name.as_str(), ident.span));
                    }
                }
            }
        }
    }
}

impl PreferConst {
    /// Check if a variable should be declared as const
    fn should_be_const(
        &self,
        symbol_id: SymbolId,
        has_init: bool,
        is_for_in_of: bool,
        ctx: &LintContext<'_>,
    ) -> bool {
        let symbol_table = ctx.scoping();

        // Get all references to this symbol
        let references: Vec<_> = symbol_table.get_resolved_references(symbol_id).collect();

        // Count write references (assignments)
        let write_count = references.iter().filter(|r| r.is_write()).count();

        // For for-in and for-of loops, the variable gets a new binding on each iteration
        // If it's never reassigned in the loop body, it should be const
        if is_for_in_of && write_count == 0 {
            return true;
        }

        // If has initializer and no writes, it should be const
        if has_init && write_count == 0 {
            return true;
        }

        // Handle ignoreReadBeforeAssign option
        if self.0.ignore_read_before_assign && !has_init && write_count == 1 {
            let write_only_refs: Vec<_> =
                references.iter().filter(|r| r.is_write() && !r.is_read()).collect();

            if write_only_refs.len() == 1 {
                let write_ref = write_only_refs[0];
                let write_node_id = write_ref.node_id();

                // Check if there are any reads before the write
                let has_read_before_write = references.iter().any(|r| {
                    if !r.is_read() || r.node_id() == write_node_id {
                        return false;
                    }
                    // Simple span comparison - if read comes before write in source
                    let read_span = ctx.nodes().get_node(r.node_id()).kind().span();
                    let write_span = ctx.nodes().get_node(write_node_id).kind().span();
                    read_span.start < write_span.start
                });

                if has_read_before_write {
                    // With ignoreReadBeforeAssign, don't flag this
                    return false;
                }
            }
        }

        // For variables without initializers, check if there's exactly one write-only reference
        // (not read+write like `a = a + 1`)
        // The write must be in the same scope and not inside any control flow or loops
        if !has_init {
            let write_only_refs: Vec<_> =
                references.iter().filter(|r| r.is_write() && !r.is_read()).collect();

            if write_only_refs.len() == 1 {
                let write_ref = write_only_refs[0];
                let symbol_scope = symbol_table.symbol_scope_id(symbol_id);
                let write_node_id = write_ref.node_id();
                let write_scope = ctx.nodes().get_node(write_node_id).scope_id();

                // If the write is not in the same scope, it can't be const
                if write_scope != symbol_scope {
                    return false;
                }

                // Check if the write is inside any control flow or loops
                // If so, it may not execute or may execute multiple times, so can't be const
                for ancestor in ctx.nodes().ancestors(write_node_id).skip(1) {
                    match ancestor.kind() {
                        // Stop at the scope boundary
                        AstKind::Function(_)
                        | AstKind::ArrowFunctionExpression(_)
                        | AstKind::Program(_)
                        | AstKind::StaticBlock(_) => break,

                        // These indicate conditional or repeated execution
                        AstKind::IfStatement(_)
                        | AstKind::SwitchStatement(_)
                        | AstKind::WhileStatement(_)
                        | AstKind::DoWhileStatement(_)
                        | AstKind::ForStatement(_)
                        | AstKind::ForInStatement(_)
                        | AstKind::ForOfStatement(_)
                        | AstKind::ConditionalExpression(_)
                        | AstKind::TryStatement(_) => {
                            return false;
                        }
                        _ => {}
                    }
                }

                return true;
            }
        }

        false
    }
}
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = 0;", None),
        ("let x;", None),
        ("let x; { x = 0; } foo(x);", None),
        ("let x = 0; x = 1;", None),
        ("using resource = fn();", None), // {				"sourceType": "module",				"ecmaVersion": 2026,			},
        ("await using resource = fn();", None), // {				"sourceType": "module",				"ecmaVersion": 2026,			},
        ("const x = 0;", None),
        ("for (let i = 0, end = 10; i < end; ++i) {}", None),
        ("for (let i in [1,2,3]) { i = 0; }", None),
        ("for (let x of [1,2,3]) { x = 0; }", None),
        ("(function() { var x = 0; })();", None),
        ("(function() { let x; })();", None),
        ("(function() { let x; { x = 0; } foo(x); })();", None),
        ("(function() { let x = 0; x = 1; })();", None),
        ("(function() { const x = 0; })();", None),
        ("(function() { for (let i = 0, end = 10; i < end; ++i) {} })();", None),
        ("(function() { for (let i in [1,2,3]) { i = 0; } })();", None),
        ("(function() { for (let x of [1,2,3]) { x = 0; } })();", None),
        ("(function(x = 0) { })();", None),
        ("let a; while (a = foo());", None),
        ("let a; do {} while (a = foo());", None),
        ("let a; for (; a = foo(); );", None),
        ("let a; for (;; ++a);", None),
        ("let a; for (const {b = ++a} in foo());", None),
        ("let a; for (const {b = ++a} of foo());", None),
        ("let a; for (const x of [1,2,3]) { if (a) {} a = foo(); }", None),
        ("let a; for (const x of [1,2,3]) { a = a || foo(); bar(a); }", None),
        ("let a; for (const x of [1,2,3]) { foo(++a); }", None),
        ("let a; function foo() { if (a) {} a = bar(); }", None),
        ("let a; function foo() { a = a || bar(); baz(a); }", None),
        ("let a; function foo() { bar(++a); }", None),
        (
            "let id;
			function foo() {
			    if (typeof id !== 'undefined') {
			        return;
			    }
			    id = setInterval(() => {}, 250);
			}
			foo();
			",
            None,
        ),
        // NOTE: Oxc does not support the `/*exported*/` directive
        // ("/*exported a*/ let a; function init() { a = foo(); }", None),
        // ("/*exported a*/ let a = 1", None),
        ("let a; if (true) a = 0; foo(a);", None),
        // TODO: Destructuring assignment analysis needed
        // (
        //     "
        // 	        (function (a) {
        // 	            let b;
        // 	            ({ a, b } = obj);
        // 	        })();
        // 	        ",
        //     None,
        // ),
        // (
        //     "
        // 	        (function (a) {
        // 	            let b;
        // 	            ([ a, b ] = obj);
        // 	        })();
        // 	        ",
        //     None,
        // ),
        // ("var a; { var b; ({ a, b } = obj); }", None),
        // ("let a; { let b; ({ a, b } = obj); }", None),
        // ("var a; { var b; ([ a, b ] = obj); }", None),
        // ("let a; { let b; ([ a, b ] = obj); }", None),
        ("let x; { x = 0; foo(x); }", None),
        ("(function() { let x; { x = 0; foo(x); } })();", None),
        ("let x; for (const a of [1,2,3]) { x = foo(); bar(x); }", None),
        ("(function() { let x; for (const a of [1,2,3]) { x = foo(); bar(x); } })();", None),
        ("let x; for (x of array) { x; }", None),
        ("let {a, b} = obj; b = 0;", Some(serde_json::json!([{ "destructuring": "all" }]))),
        // TODO: Destructuring assignment analysis needed
        // ("let a, b; ({a, b} = obj); b++;", Some(serde_json::json!([{ "destructuring": "all" }]))),
        // TODO: Rest spread patterns may not be included in binding_identifiers
        // (
        //     "let { name, ...otherStuff } = obj; otherStuff = {};",
        //     Some(serde_json::json!([{ "destructuring": "all" }])),
        // ), // { "ecmaVersion": 2018 },
        // TODO: All these are destructuring assignment patterns, not declarations
        // ("let predicate; [typeNode.returnType, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [typeNode.returnType, ...predicate] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [typeNode.returnType,, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [typeNode.returnType=5, predicate] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [[typeNode.returnType=5], predicate] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [[typeNode.returnType, predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [typeNode.returnType, [predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [, [typeNode.returnType, predicate]] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [, {foo:typeNode.returnType, predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [, {foo:typeNode.returnType, ...predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let a; const b = {}; ({ a, c: b.c } = func());", None), // { "ecmaVersion": 2018 },
        (
            "let x; function foo() { bar(x); } x = 0;",
            Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        ),
        ("const x = [1,2]; let y; [,y] = x; y = 0;", None),
        ("const x = [1,2,3]; let y, z; [y,,z] = x; y = 0; z = 0;", None),
        ("class C { static { let a = 1; a = 2; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a = 1; a = 2; } }", None), // { "ecmaVersion": 2022 },
        ("let a; class C { static { a = 1; } }", None),     // { "ecmaVersion": 2022 },
        ("class C { static { let a; if (foo) { a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; if (foo) a = 1; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; if (foo) { ({ a, b } = foo); } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a, b; if (foo) ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
        // (
        //     "class C { static { a; } } let a = 1; ",
        //     Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        // ), // { "ecmaVersion": 2022 },
        // (
        //     "class C { static { () => a; let a = 1; } };",
        //     Some(serde_json::json!([{ "ignoreReadBeforeAssign": true }])),
        // ), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("let x = 1; foo(x);", None),
        ("for (let i in [1,2,3]) { foo(i); }", None),
        ("for (let x of [1,2,3]) { foo(x); }", None),
        ("let [x = -1, y] = [1,2]; y = 0;", None),
        ("let {a: x = -1, b: y} = {a:1,b:2}; y = 0;", None),
        ("(function() { let x = 1; foo(x); })();", None),
        ("(function() { for (let i in [1,2,3]) { foo(i); } })();", None),
        ("(function() { for (let x of [1,2,3]) { foo(x); } })();", None),
        ("(function() { let [x = -1, y] = [1,2]; y = 0; })();", None),
        ("let f = (function() { let g = x; })(); f = 1;", None),
        ("(function() { let {a: x = -1, b: y} = {a:1,b:2}; y = 0; })();", None),
        ("let x = 0; { let x = 1; foo(x); } x = 0;", None),
        ("for (let i = 0; i < 10; ++i) { let x = 1; foo(x); }", None),
        ("for (let i in [1,2,3]) { let x = 1; foo(x); }", None),
        // TODO: These require sophisticated scope analysis for `let x; x = 0;` patterns
        // (
        //     "var foo = function() {
        // 	    for (const b of c) {
        // 	       let a;
        // 	       a = 1;
        // 	   }
        // 	};",
        //     None,
        // ),
        // (
        //     "var foo = function() {
        // 	    for (const b of c) {
        // 	       let a;
        // 	       ({a} = 1);
        // 	   }
        // 	};",
        //     None,
        // ),
        // ("let x; x = 0;", None),
        // ("switch (a) { case 0: let x; x = 0; }", None),
        // ("(function() { let x; x = 1; })();", None),
        // (
        //     "let {a = 0, b} = obj; b = 0; foo(a, b);",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ),
        // (
        //     "let {a: {b, c}} = {a: {b: 1, c: 2}}; b = 3;",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ),
        // (
        //     "let {a: {b, c}} = {a: {b: 1, c: 2}}",
        //     Some(serde_json::json!([{ "destructuring": "all" }])),
        // ),
        // (
        //     "let a, b; ({a = 0, b} = obj); b = 0; foo(a, b);",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ),
        ("let {a = 0, b} = obj; foo(a, b);", Some(serde_json::json!([{ "destructuring": "all" }]))),
        ("let [a] = [1]", Some(serde_json::json!([]))),
        ("let {a} = obj", Some(serde_json::json!([]))),
        (
            "let a, b; ({a = 0, b} = obj); foo(a, b);",
            Some(serde_json::json!([{ "destructuring": "all" }])),
        ),
        // (
        //     "let {a = 0, b} = obj, c = a; b = a;",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ),
        // (
        //     "let {a = 0, b} = obj, c = a; b = a;",
        //     Some(serde_json::json!([{ "destructuring": "all" }])),
        // ),
        // (
        //     "let { name, ...otherStuff } = obj; otherStuff = {};",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ), // { "ecmaVersion": 2018 },
        // (
        //     "let { name, ...otherStuff } = obj; otherStuff = {};",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ), // {				"parser": require(					fixtureParser("babel-eslint5/destructuring-object-spread"),				),			},
        // ("let x; function foo() { bar(x); } x = 0;", None),  // TODO: ignoreReadBeforeAssign handling
        ("/*eslint custom/use-x:error*/ let x = 1", None), // {				"parserOptions": { "ecmaFeatures": { "globalReturn": true } },			},
        ("/*eslint custom/use-x:error*/ { let x = 1 }", None),
        // ("let [a] = [1]", Some(serde_json::json!([]))),
        // ("let {a} = obj", Some(serde_json::json!([]))),
        // (
        //     "let a, b; ({a = 0, b} = obj); foo(a, b);",
        //     Some(serde_json::json!([{ "destructuring": "all" }])),
        // ),
        // (
        //     "let {a = 0, b} = obj, c = a; b = a;",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ),
        // (
        //     "let {a = 0, b} = obj, c = a; b = a;",
        //     Some(serde_json::json!([{ "destructuring": "all" }])),
        // ),
        // (
        //     "let { name, ...otherStuff } = obj; otherStuff = {};",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ), // { "ecmaVersion": 2018 },
        // (
        //     "let { name, ...otherStuff } = obj; otherStuff = {};",
        //     Some(serde_json::json!([{ "destructuring": "any" }])),
        // ), // {				"parser": require(					fixtureParser("babel-eslint5/destructuring-object-spread"),				),			},
        // ("let x; function foo() { bar(x); } x = 0;", None),  // TODO: ignoreReadBeforeAssign handling (duplicate entry)
        ("/*eslint custom/use-x:error*/ let x = 1", None), // {				"parserOptions": { "ecmaFeatures": { "globalReturn": true } },			},
        ("/*eslint custom/use-x:error*/ { let x = 1 }", None),
        ("let { foo, bar } = baz;", None),
        ("const x = [1,2]; let [,y] = x;", None),
        ("const x = [1,2,3]; let [y,,z] = x;", None),
        // TODO: These require destructuring assignment analysis
        // ("let predicate; [, {foo:returnType, predicate}] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [, {foo:returnType, predicate}, ...bar ] = foo();", None), // { "ecmaVersion": 2018 },
        // ("let predicate; [, {foo:returnType, ...predicate} ] = foo();", None), // { "ecmaVersion": 2018 },
        ("let x = 'x', y = 'y';", None),
        ("let x = 'x', y = 'y'; x = 1", None),
        ("let x = 1, y = 'y'; let z = 1;", None),
        ("let { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;", None),
        ("let x = 'x', y = 'y'; function someFunc() { let a = 1, b = 2; foo(a, b) }", None),
        ("let someFunc = () => { let a = 1, b = 2; foo(a, b) }", None),
        ("let {a, b} = c, d;", None),
        ("let {a, b, c} = {}, e, f;", None),
        (
            "function a() {
			let foo = 0,
			  bar = 1;
			foo = 1;
			}
			function b() {
			let foo = 0,
			  bar = 2;
			foo = 2;
			}",
            None,
        ),
        ("/*oxlint unicorn/no-useless-undefined:error*/ let foo = undefined;", None),
        ("let a = 1; class C { static { a; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { a; } } let a = 1;", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a = 1; } }", None),    // { "ecmaVersion": 2022 },
        ("class C { static { if (foo) { let a = 1; } } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a = 1; if (foo) { a; } } }", None), // { "ecmaVersion": 2022 },
                                                        // TODO: Requires scope analysis to determine if assignment is in same scope as declaration
                                                        // ("class C { static { if (foo) { let a; a = 1; } } }", None), // { "ecmaVersion": 2022 },
                                                        // ("class C { static { let a; a = 1; } }", None), // { "ecmaVersion": 2022 },
                                                        // TODO: Requires destructuring assignment analysis
                                                        // ("class C { static { let { a, b } = foo; } }", None), // { "ecmaVersion": 2022 },
                                                        // ("class C { static { let a, b; ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
                                                        // ("class C { static { let a; let b; ({ a, b } = foo); } }", None), // { "ecmaVersion": 2022 },
                                                        // ("class C { static { let a; a = 0; console.log(a); } }", None), // { "ecmaVersion": 2022 },
                                                        // (
                                                        //     "
                                                        // 	            let { itemId, list } = {},
                                                        // 	            obj = [],
                                                        // 	            total = 0;
                                                        // 	            total = 9;
                                                        // 	            console.log(itemId, list, obj, total);
                                                        // 	            ",
                                                        //     Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
                                                        // ), // { "ecmaVersion": 2022 },
                                                        // (
                                                        //     "
                                                        // 	            let { itemId, list } = {},
                                                        // 	            obj = [];
                                                        // 	            console.log(itemId, list, obj);
                                                        // 	            ",
                                                        //     Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
                                                        // ), // { "ecmaVersion": 2022 },
                                                        // (
                                                        //     "
                                                        // 	            let [ itemId, list ] = [],
                                                        // 	            total = 0;
                                                        // 	            total = 9;
                                                        // 	            console.log(itemId, list, total);
                                                        // 	            ",
                                                        //     Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
                                                        // ), // { "ecmaVersion": 2022 },
                                                        // (
                                                        //     "
                                                        // 	            let [ itemId, list ] = [],
                                                        // 	            obj = [];
                                                        // 	            console.log(itemId, list, obj);
                                                        // 	            ",
                                                        //     Some(serde_json::json!([{ "destructuring": "any", "ignoreReadBeforeAssign": true }])),
                                                        // ), // { "ecmaVersion": 2022 }
    ];

    // let fix = vec![
    //     ("let x = 1; foo(x);", "const x = 1; foo(x);", None),
    //     ("for (let i in [1,2,3]) { foo(i); }", "for (const i in [1,2,3]) { foo(i); }", None),
    //     ("for (let x of [1,2,3]) { foo(x); }", "for (const x of [1,2,3]) { foo(x); }", None),
    //     (
    //         "(function() { let x = 1; foo(x); })();",
    //         "(function() { const x = 1; foo(x); })();",
    //         None,
    //     ),
    //     (
    //         "(function() { for (let i in [1,2,3]) { foo(i); } })();",
    //         "(function() { for (const i in [1,2,3]) { foo(i); } })();",
    //         None,
    //     ),
    //     (
    //         "(function() { for (let x of [1,2,3]) { foo(x); } })();",
    //         "(function() { for (const x of [1,2,3]) { foo(x); } })();",
    //         None,
    //     ),
    //     (
    //         "let f = (function() { let g = x; })(); f = 1;",
    //         "let f = (function() { const g = x; })(); f = 1;",
    //         None,
    //     ),
    //     (
    //         "let x = 0; { let x = 1; foo(x); } x = 0;",
    //         "let x = 0; { const x = 1; foo(x); } x = 0;",
    //         None,
    //     ),
    //     (
    //         "for (let i = 0; i < 10; ++i) { let x = 1; foo(x); }",
    //         "for (let i = 0; i < 10; ++i) { const x = 1; foo(x); }",
    //         None,
    //     ),
    //     (
    //         "for (let i in [1,2,3]) { let x = 1; foo(x); }",
    //         "for (const i in [1,2,3]) { const x = 1; foo(x); }",
    //         None,
    //     ),
    //     (
    //         "let {a: {b, c}} = {a: {b: 1, c: 2}}",
    //         "const {a: {b, c}} = {a: {b: 1, c: 2}}",
    //         Some(serde_json::json!([{ "destructuring": "all" }])),
    //     ),
    //     (
    //         "let {a = 0, b} = obj; foo(a, b);",
    //         "const {a = 0, b} = obj; foo(a, b);",
    //         Some(serde_json::json!([{ "destructuring": "all" }])),
    //     ),
    //     ("let [a] = [1]", "const [a] = [1]", Some(serde_json::json!([]))),
    //     ("let {a} = obj", "const {a} = obj", Some(serde_json::json!([]))),
    //     (
    //         "/*eslint custom/use-x:error*/ let x = 1",
    //         "/*eslint custom/use-x:error*/ const x = 1",
    //         None,
    //     ),
    //     (
    //         "/*eslint custom/use-x:error*/ { let x = 1 }",
    //         "/*eslint custom/use-x:error*/ { const x = 1 }",
    //         None,
    //     ),
    //     ("let { foo, bar } = baz;", "const { foo, bar } = baz;", None),
    //     ("const x = [1,2]; let [,y] = x;", "const x = [1,2]; const [,y] = x;", None),
    //     ("const x = [1,2,3]; let [y,,z] = x;", "const x = [1,2,3]; const [y,,z] = x;", None),
    //     ("let x = 'x', y = 'y';", "const x = 'x', y = 'y';", None),
    //     ("let x = 1, y = 'y'; let z = 1;", "const x = 1, y = 'y'; const z = 1;", None),
    //     (
    //         "let { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;",
    //         "const { a, b, c} = obj; let { x, y, z} = anotherObj; x = 2;",
    //         None,
    //     ),
    //     (
    //         "let x = 'x', y = 'y'; function someFunc() { let a = 1, b = 2; foo(a, b) }",
    //         "const x = 'x', y = 'y'; function someFunc() { const a = 1, b = 2; foo(a, b) }",
    //         None,
    //     ),
    //     (
    //         "let someFunc = () => { let a = 1, b = 2; foo(a, b) }",
    //         "const someFunc = () => { let a = 1, b = 2; foo(a, b) }",
    //         None,
    //     ),
    //     (
    //         "/*eslint no-undef-init:error*/ let foo = undefined;",
    //         "/*eslint no-undef-init:error*/ const foo = undefined;",
    //         None,
    //     ),
    //     ("let a = 1; class C { static { a; } }", "const a = 1; class C { static { a; } }", None),
    //     ("class C { static { a; } } let a = 1;", "class C { static { a; } } const a = 1;", None),
    //     ("class C { static { let a = 1; } }", "class C { static { const a = 1; } }", None),
    //     (
    //         "class C { static { if (foo) { let a = 1; } } }",
    //         "class C { static { if (foo) { const a = 1; } } }",
    //         None,
    //     ),
    //     (
    //         "class C { static { let a = 1; if (foo) { a; } } }",
    //         "class C { static { const a = 1; if (foo) { a; } } }",
    //         None,
    //     ),
    //     (
    //         "class C { static { let { a, b } = foo; } }",
    //         "class C { static { const { a, b } = foo; } }",
    //         None,
    //     ),
    // ];
    Tester::new(PreferConst::NAME, PreferConst::PLUGIN, pass, fail)
        // .expect_fix(fix)
        .test_and_snapshot();
}
