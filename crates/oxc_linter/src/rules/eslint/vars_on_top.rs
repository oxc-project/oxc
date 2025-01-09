use oxc_ast::ast::{Declaration, Expression, Program, Statement, VariableDeclarationKind};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn vars_on_top_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("All 'var' declarations must be at the top of the function scope.")
        .with_help("Consider moving this to the top of the functions scope or using let or const to declare this variable.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct VarsOnTop;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that all `var` declarations are placed at the top of their containing scope.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, `var` declarations are hoisted to the top of their containing scope. Placing `var` declarations at the top explicitly improves code readability and maintainability by making the scope of variables clear.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function doSomething() {
    ///     if (true) {
    ///         var first = true;
    ///     }
    ///     var second;
    /// }
    ///
    /// function doSomethingElse() {
    ///     for (var i = 0; i < 10; i++) {}
    /// }
    ///
    /// f();
    /// var a;
    ///
    /// class C {
    ///     static {
    ///         if (something) {
    ///             var a = true;
    ///         }
    ///     }
    ///     static {
    ///         f();
    ///         var a;
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function doSomething() {
    ///     var first;
    ///     var second;
    ///     if (true) {
    ///         first = true;
    ///     }
    /// }
    ///
    /// function doSomethingElse() {
    ///     var i;
    ///     for (i = 0; i < 10; i++) {}
    /// }
    ///
    /// var a;
    /// f();
    ///
    /// class C {
    ///     static {
    ///         var a;
    ///         if (something) {
    ///             a = true;
    ///         }
    ///     }
    ///     static {
    ///         var a;
    ///         f();
    ///     }
    /// }
    /// ```
    VarsOnTop,
    eslint,
    style,
);

impl Rule for VarsOnTop {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(declaration) = node.kind() else {
            return;
        };
        if declaration.kind != VariableDeclarationKind::Var {
            return;
        }
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        match parent.kind() {
            AstKind::ExportNamedDeclaration(_) => {
                if let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) {
                    if let AstKind::Program(grand_parent) = grand_parent.kind() {
                        global_var_check(parent, grand_parent, ctx);
                    }
                }
            }
            AstKind::Program(parent) => {
                global_var_check(node, parent, ctx);
            }
            _ => block_scope_var_check(node, ctx),
        }
    }
}

fn looks_like_directive(node: &Statement) -> bool {
    matches!(
        node,
        Statement::ExpressionStatement(expr_stmt) if matches!(
            &expr_stmt.expression,
            Expression::StringLiteral(_)
        )
    )
}

fn looks_like_import(node: &Statement) -> bool {
    matches!(node, Statement::ImportDeclaration(_))
}

fn is_variable_declaration(node: &Statement) -> bool {
    if matches!(node, Statement::VariableDeclaration(_)) {
        return true;
    }

    if let Statement::ExportNamedDeclaration(export) = node {
        return matches!(export.declaration, Some(Declaration::VariableDeclaration(_)));
    }

    false
}

fn is_var_on_top(node: &AstNode, statements: &[Statement], ctx: &LintContext) -> bool {
    let mut i = 0;
    let len = statements.len();
    let parent = ctx.nodes().parent_node(node.id());

    if let Some(parent) = parent {
        if !matches!(parent.kind(), AstKind::StaticBlock(_)) {
            while i < len {
                if !looks_like_directive(&statements[i]) && !looks_like_import(&statements[i]) {
                    break;
                }
                i += 1;
            }
        }
    }

    let node_span = node.span();
    while i < len {
        if !is_variable_declaration(&statements[i]) {
            return false;
        }
        let stmt_span = statements[i].span();

        if stmt_span == node_span {
            return true;
        }
        i += 1;
    }

    false
}

fn global_var_check(node: &AstNode, parent: &Program, ctx: &LintContext) {
    if !is_var_on_top(node, &parent.body, ctx) {
        ctx.diagnostic(vars_on_top_diagnostic(node.span()));
    }
}

fn block_scope_var_check(node: &AstNode, ctx: &LintContext) {
    if let Some(parent) = ctx.nodes().parent_node(node.id()) {
        match parent.kind() {
            AstKind::BlockStatement(block) => {
                if check_var_on_top_in_function_scope(node, &block.body, parent, ctx) {
                    return;
                }
            }
            AstKind::FunctionBody(block) => {
                if check_var_on_top_in_function_scope(node, &block.statements, parent, ctx) {
                    return;
                }
            }
            AstKind::StaticBlock(block) => {
                if is_var_on_top(node, &block.body, ctx) {
                    return;
                }
            }
            _ => {}
        }
    }
    ctx.diagnostic(vars_on_top_diagnostic(node.span()));
}

fn check_var_on_top_in_function_scope(
    node: &AstNode,
    statements: &[Statement],
    parent: &AstNode,
    ctx: &LintContext,
) -> bool {
    if let Some(grandparent) = ctx.nodes().parent_node(parent.id()) {
        if matches!(
            grandparent.kind(),
            AstKind::Function(_) | AstKind::FunctionBody(_) | AstKind::ArrowFunctionExpression(_)
        ) && is_var_on_top(node, statements, ctx)
        {
            return true;
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var first = 0;
			function foo() {
			    first = 2;
			}
			",
"function foo() {
			}
			",
"function foo() {
			   var first;
			   if (true) {
			       first = true;
			   } else {
			       first = 1;
			   }
			}
			",
"function foo() {
			   var first;
			   var second = 1;
			   var third;
			   var fourth = 1, fifth, sixth = third;
			   var seventh;
			   if (true) {
			       third = true;
			   }
			   first = second;
			}
			",
"function foo() {
			   var i;
			   for (i = 0; i < 10; i++) {
			       alert(i);
			   }
			}
			",
"function foo() {
			   var outer;
			   function inner() {
			       var inner = 1;
			       var outer = inner;
			   }
			   outer = 1;
			}
			",
"function foo() {
			   var first;
			   //Hello
			   var second = 1;
			   first = second;
			}
			",
"function foo() {
			   var first;
			   /*
			       Hello Clarice
			   */
			   var second = 1;
			   first = second;
			}
			",
"function foo() {
			   var first;
			   var second = 1;
			   function bar(){
			       var first;
			       first = 5;
			   }
			   first = second;
			}
			",
"function foo() {
			   var first;
			   var second = 1;
			   function bar(){
			       var third;
			       third = 5;
			   }
			   first = second;
			}
			",
"function foo() {
			   var first;
			   var bar = function(){
			       var third;
			       third = 5;
			   }
			   first = 5;
			}
			",
"function foo() {
			   var first;
			   first.onclick(function(){
			       var third;
			       third = 5;
			   });
			   first = 5;
			}
			",
"function foo() {
			   var i = 0;
			   for (let j = 0; j < 10; j++) {
			       alert(j);
			   }
			   i = i + 1;
			}", // {                "ecmaVersion": 6            },
"'use strict'; var x; f();",
"'use strict'; 'directive'; var x; var y; f();",
"function f() { 'use strict'; var x; f(); }",
"function f() { 'use strict'; 'directive'; var x; var y; f(); }",
"import React from 'react'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"'use strict'; import React from 'react'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"import React from 'react'; 'use strict'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"import * as foo from 'mod.js'; 'use strict'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"import { square, diag } from 'lib'; 'use strict'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"import { default as foo } from 'lib'; 'use strict'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"import 'src/mylib'; 'use strict'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"import theDefault, { named1, named2 } from 'src/mylib'; 'use strict'; var y; function f() { 'use strict'; var x; var y; f(); }", // { "ecmaVersion": 6, "sourceType": "module" },
"export var x;
			var y;
			var z;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
"var x;
			export var y;
			var z;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
"var x;
			var y;
			export var z;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
"class C {
			    static {
			        var x;
			    }
			}", // {                "ecmaVersion": 2022            },
"class C {
			    static {
			        var x;
			        foo();
			    }
			}", // {                "ecmaVersion": 2022            },
"class C {
			    static {
			        var x;
			        var y;
			    }
			}", // {                "ecmaVersion": 2022            },
"class C {
			    static {
			        var x;
			        var y;
			        foo();
			    }
			}", // {                "ecmaVersion": 2022            },
"class C {
			    static {
			        let x;
			        var y;
			    }
			}", // {                "ecmaVersion": 2022            },
"class C {
			    static {
			        foo();
			        let x;
			    }
			}", // {                "ecmaVersion": 2022            }
    ];

    let fail = vec![
        "var first = 0;
			function foo() {
			    first = 2;
			    second = 2;
			}
			var second = 0;",
        "function foo() {
			   var first;
			   first = 1;
			   first = 2;
			   first = 3;
			   first = 4;
			   var second = 1;
			   second = 2;
			   first = second;
			}",
        "function foo() {
			   var first;
			   if (true) {
			       var second = true;
			   }
			   first = second;
			}",
        "function foo() {
			   for (var i = 0; i < 10; i++) {
			       alert(i);
			   }
			}",
        "function foo() {
			   var first = 10;
			   var i;
			   for (i = 0; i < first; i ++) {
			       var second = i;
			   }
			}",
        "function foo() {
			   var first = 10;
			   var i;
			   switch (first) {
			       case 10:
			           var hello = 1;
			           break;
			   }
			}",
        "function foo() {
			   var first = 10;
			   var i;
			   try {
			       var hello = 1;
			   } catch (e) {
			       alert('error');
			   }
			}",
        "function foo() {
			   var first = 10;
			   var i;
			   try {
			       asdf;
			   } catch (e) {
			       var hello = 1;
			   }
			}",
        "function foo() {
			   var first = 10;
			   while (first) {
			       var hello = 1;
			   }
			}",
        "function foo() {
			   var first = 10;
			   do {
			       var hello = 1;
			   } while (first == 10);
			}",
        "function foo() {
			   var first = [1,2,3];
			   for (var item in first) {
			       item++;
			   }
			}",
        "function foo() {
			   var first = [1,2,3];
			   var item;
			   for (item in first) {
			       var hello = item;
			   }
			}",
        "var foo = () => {
			   var first = [1,2,3];
			   var item;
			   for (item in first) {
			       var hello = item;
			   }
			}", // { "ecmaVersion": 6 },
        "'use strict'; 0; var x; f();",
        "'use strict'; var x; 'directive'; var y; f();",
        "function f() { 'use strict'; 0; var x; f(); }",
        "function f() { 'use strict'; var x; 'directive';  var y; f(); }",
        "export function f() {}
			var x;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
        "var x;
			export function f() {}
			var y;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
        "import {foo} from 'foo';
			export {foo};
			var test = 1;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
        "export {foo} from 'foo';
			var test = 1;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
        "export * from 'foo';
			var test = 1;", // {                "ecmaVersion": 6,                "sourceType": "module"            },
        "class C {
			    static {
			        foo();
			        var x;
			    }
			}", // {                "ecmaVersion": 2022            },
        "class C {
			    static {
			        'use strict';
			        var x;
			    }
			}", // {                "ecmaVersion": 2022            },
        "class C {
			    static {
			        var x;
			        foo();
			        var y;
			    }
			}", // {                "ecmaVersion": 2022            },
        "class C {
			    static {
			        if (foo) {
			            var x;
			        }
			    }
			}", // {                "ecmaVersion": 2022            },
        "class C {
			    static {
			        if (foo)
			            var x;
			    }
			}", // {                "ecmaVersion": 2022            }
    ];

    Tester::new(VarsOnTop::NAME, VarsOnTop::PLUGIN, pass, fail).test_and_snapshot();
}
