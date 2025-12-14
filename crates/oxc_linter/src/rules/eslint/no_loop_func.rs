use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ScopeId, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::symbol::SymbolFlags;

use crate::{context::LintContext, rule::Rule};

fn no_loop_func_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Function declared in a loop contains unsafe references to variable(s)")
        .with_help("Variables declared with 'var' are function-scoped, not block-scoped. Consider using 'let' or 'const' for block-scoped variables, or move the function outside the loop.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLoopFunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows function declarations and expressions inside loop statements
    /// when they reference variables declared in the outer scope that may change
    /// across iterations.
    ///
    /// ### Why is this bad?
    ///
    /// Writing functions within loops tends to result in errors due to the way
    /// closures work in JavaScript. Functions capture variables by reference,
    /// not by value. When using `var`, which is function-scoped, all iterations
    /// share the same variable binding, leading to unexpected behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (var i = 0; i < 10; i++) {
    ///     funcs[i] = function() {
    ///         return i;
    ///     };
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// for (let i = 0; i < 10; i++) {
    ///     funcs[i] = function() {
    ///         return i;
    ///     };
    /// }
    /// ```
    NoLoopFunc,
    eslint,
    pedantic
);

impl Rule for NoLoopFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Check for function expressions, arrow functions, and function declarations
        let func_span = match node.kind() {
            AstKind::Function(func) => {
                // Skip if not inside a statement (i.e., method definitions, etc.)
                if !func.is_expression() && !func.is_declaration() {
                    return;
                }
                func.span
            }
            AstKind::ArrowFunctionExpression(arrow) => arrow.span,
            _ => return,
        };

        // Find the containing loop, if any
        let Some(loop_node) = Self::get_containing_loop(node, ctx) else {
            return;
        };

        // Check if any referenced variables are unsafe
        if Self::has_unsafe_references(node, loop_node, ctx) {
            ctx.diagnostic(no_loop_func_diagnostic(func_span));
        }
    }
}

impl NoLoopFunc {
    /// Find the containing loop statement, stopping at function boundaries.
    /// Only returns a loop if the function is inside the loop's body (not init/test/update).
    fn get_containing_loop<'a, 'b>(
        node: &AstNode<'a>,
        ctx: &'b LintContext<'a>,
    ) -> Option<&'b AstNode<'a>> {
        let nodes = ctx.nodes();
        let func_span = node.span();
        let mut current = nodes.parent_node(node.id());

        loop {
            match current.kind() {
                // Found a loop - check if function is in the body (not init/test/update)
                AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_) => {
                    let body_span = Self::get_loop_body_span(current);
                    if body_span.contains_inclusive(func_span) {
                        return Some(current);
                    }
                }
                // Stop at function boundaries or program root
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::Program(_) => {
                    return None;
                }
                _ => {}
            }
            current = nodes.parent_node(current.id());
        }
    }

    /// Check if the function has any unsafe references to variables
    fn has_unsafe_references<'a>(
        func_node: &AstNode<'a>,
        loop_node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let scoping = ctx.scoping();
        let nodes = ctx.nodes();
        let func_scope_id = func_node.scope_id();

        // Iterate through all symbols and check their references
        for symbol_id in scoping.symbol_ids() {
            let flags = scoping.symbol_flags(symbol_id);

            // Skip type-only symbols (TypeScript types, interfaces, etc.)
            if flags.is_type() && !flags.is_value() {
                continue;
            }

            // Get the scope where the symbol is declared
            let symbol_scope_id = scoping.symbol_scope_id(symbol_id);

            // Skip if the symbol is declared inside the function (local variable)
            if Self::is_scope_inside_function(symbol_scope_id, func_scope_id, ctx) {
                continue;
            }

            // Check if any reference to this symbol is from inside the function
            let mut is_referenced_in_function = false;
            for reference in scoping.get_resolved_references(symbol_id) {
                let ref_node = nodes.get_node(reference.node_id());
                let ref_scope_id = ref_node.scope_id();
                if Self::is_scope_inside_function(ref_scope_id, func_scope_id, ctx) {
                    is_referenced_in_function = true;
                    break;
                }
            }

            if !is_referenced_in_function {
                continue;
            }

            // Now check if this reference is unsafe
            if Self::is_unsafe_reference(symbol_id, loop_node, ctx) {
                return true;
            }
        }

        false
    }

    /// Check if a scope is inside a function (including the function's own scope)
    fn is_scope_inside_function(
        scope_id: ScopeId,
        func_scope_id: ScopeId,
        ctx: &LintContext,
    ) -> bool {
        let scoping = ctx.scoping();
        let mut current = Some(scope_id);
        while let Some(id) = current {
            if id == func_scope_id {
                return true;
            }
            current = scoping.scope_parent_id(id);
        }
        false
    }

    /// Determine if a variable reference is unsafe within a loop context
    fn is_unsafe_reference(symbol_id: SymbolId, loop_node: &AstNode, ctx: &LintContext) -> bool {
        let scoping = ctx.scoping();
        let nodes = ctx.nodes();
        let flags = scoping.symbol_flags(symbol_id);

        // const, using, await using are always safe (immutable bindings)
        if flags.is_const_variable() {
            return false;
        }

        // Import bindings are always safe (immutable)
        if flags.is_import() {
            return false;
        }

        // Get the declaration node for the symbol
        let symbol_decl_node_id = scoping.symbol_declaration(symbol_id);
        let symbol_decl_node = nodes.get_node(symbol_decl_node_id);

        // Check if the variable is declared with var (function-scoped)
        if flags.is_function_scoped_declaration() {
            // var declarations are unsafe if the variable is referenced in the loop
            // and declared in the loop or any ancestor scope
            return Self::is_var_unsafe_in_loop(symbol_id, symbol_decl_node, loop_node, ctx);
        }

        // For let declarations, check if:
        // 1. Declared outside the loop body (not fresh binding per iteration)
        // 2. Modified somewhere in the code
        if flags.intersects(SymbolFlags::BlockScopedVariable) && !flags.is_const_variable() {
            return Self::is_let_unsafe_in_loop(symbol_id, symbol_decl_node, loop_node, ctx);
        }

        false
    }

    /// Check if a var-declared variable is unsafe in a loop
    fn is_var_unsafe_in_loop(
        symbol_id: SymbolId,
        symbol_decl_node: &AstNode,
        loop_node: &AstNode,
        ctx: &LintContext,
    ) -> bool {
        let loop_span = loop_node.span();
        let decl_span = symbol_decl_node.span();

        // If declared inside the loop (including loop header), it's unsafe
        // because all iterations share the same binding
        if loop_span.contains_inclusive(decl_span) {
            return true;
        }

        // If declared outside but modified inside the loop, it's unsafe
        // because the captured value changes across iterations
        let scoping = ctx.scoping();
        for reference in scoping.get_resolved_references(symbol_id) {
            if reference.is_write() {
                let ref_span = ctx.semantic().reference_span(reference);
                if loop_span.contains_inclusive(ref_span) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a let-declared variable is unsafe in a loop
    fn is_let_unsafe_in_loop(
        symbol_id: SymbolId,
        symbol_decl_node: &AstNode,
        loop_node: &AstNode,
        ctx: &LintContext,
    ) -> bool {
        let loop_body_span = Self::get_loop_body_span(loop_node);
        let decl_span = symbol_decl_node.span();

        // If let is declared inside the loop body, each iteration gets a fresh binding - safe
        if loop_body_span.contains_inclusive(decl_span) {
            return false;
        }

        // For `for` loops, check if let is declared in the loop header (init expression)
        // `for (let i = 0; ...)` creates fresh bindings per iteration - safe
        if Self::is_in_for_loop_header(symbol_decl_node, loop_node) {
            return false;
        }

        // If declared outside the loop body, check for modifications
        let scoping = ctx.scoping();
        for reference in scoping.get_resolved_references(symbol_id) {
            if reference.is_write() {
                return true;
            }
        }

        false
    }

    /// Check if a variable declaration is in a for loop's header (init expression)
    fn is_in_for_loop_header(symbol_decl_node: &AstNode, loop_node: &AstNode) -> bool {
        let decl_span = symbol_decl_node.span();
        match loop_node.kind() {
            AstKind::ForStatement(stmt) => {
                // Check if declaration is in the init part
                if let Some(init) = &stmt.init {
                    return init.span().contains_inclusive(decl_span);
                }
                false
            }
            AstKind::ForInStatement(stmt) => {
                // For-in: the left part is the iteration variable
                stmt.left.span().contains_inclusive(decl_span)
            }
            AstKind::ForOfStatement(stmt) => {
                // For-of: the left part is the iteration variable
                stmt.left.span().contains_inclusive(decl_span)
            }
            _ => false,
        }
    }

    /// Get the span of the loop body
    fn get_loop_body_span(loop_node: &AstNode) -> Span {
        match loop_node.kind() {
            AstKind::ForStatement(stmt) => stmt.body.span(),
            AstKind::ForInStatement(stmt) => stmt.body.span(),
            AstKind::ForOfStatement(stmt) => stmt.body.span(),
            AstKind::WhileStatement(stmt) => stmt.body.span(),
            AstKind::DoWhileStatement(stmt) => stmt.body.span(),
            _ => loop_node.span(),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "string = 'function a() {}';",
        "for (var i=0; i<l; i++) { } var a = function() { i; };",
        "for (var i=0, a=function() { i; }; i<l; i++) { }",
        "for (var x in xs.filter(function(x) { return x != upper; })) { }",
        "for (var x of xs.filter(function(x) { return x != upper; })) { }",
        "for (var i=0; i<l; i++) { (function() {}) }",
        "for (var i in {}) { (function() {}) }",
        "for (var i of {}) { (function() {}) }",
        "for (let i=0; i<l; i++) { (function() { i; }) }",
        "for (let i in {}) { i = 7; (function() { i; }) }",
        "for (const i of {}) { (function() { i; }) }",
        "for (let i = 0; i < 10; ++i) { for (let x in xs.filter(x => x != i)) {  } }",
        "let a = 0; for (let i=0; i<l; i++) { (function() { a; }); }",
        "let a = 0; for (let i in {}) { (function() { a; }); }",
        "let a = 0; for (let i of {}) { (function() { a; }); }",
        "let a = 0; for (let i=0; i<l; i++) { (function() { (function() { a; }); }); }",
        "let a = 0; for (let i in {}) { function foo() { (function() { a; }); } }",
        "let a = 0; for (let i of {}) { (() => { (function() { a; }); }); }",
        "var a = 0; for (let i=0; i<l; i++) { (function() { a; }); }",
        "var a = 0; for (let i in {}) { (function() { a; }); }",
        "var a = 0; for (let i of {}) { (function() { a; }); }",
        "while (true) { (function() { a; }); } let a;",
        "while(i) { (function() { i; }) }",
        "do { (function() { i; }) } while (i)",
        "var i; while(i) { (function() { i; }) }",
        "var i; do { (function() { i; }) } while (i)",
    ];

    let fail = vec![
        "for (var i=0; i<l; i++) { (function() { i; }) }",
        "for (var i=0; i<l; i++) { for (var j=0; j<m; j++) { (function() { i+j; }) } }",
        "for (var i in {}) { (function() { i; }) }",
        "for (var i of {}) { (function() { i; }) }",
        "for (var i=0; i < l; i++) { (() => { i; }) }",
        "for (var i=0; i < l; i++) { var a = function() { i; } }",
        "for (var i=0; i < l; i++) { function a() { i; }; a(); }",
        "let a; for (let i=0; i<l; i++) { a = 1; (function() { a; });}",
        "let a; for (let i in {}) { (function() { a; }); a = 1; }",
        "let a; for (let i of {}) { (function() { a; }); } a = 1; ",
        "for (var x of xs) { (function() { x; }); }",
        "var a; for (let x of xs) { a = 1; (function() { a; }); }",
        "var a; for (let x of xs) { (function() { a; }); a = 1; }",
    ];

    Tester::new(NoLoopFunc::NAME, NoLoopFunc::PLUGIN, pass, fail).test_and_snapshot();
}
