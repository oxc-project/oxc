use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, DoWhileStatement, ForInStatement, ForOfStatement, ForStatement,
        Function, IdentifierReference, Statement, WhileStatement,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolFlags};

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
    pedantic,
    version = "1.33.0",
);

impl Rule for NoLoopFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ForStatement(statement) => Self::check_loop_body(node, &statement.body, ctx),
            AstKind::ForInStatement(statement) => Self::check_loop_body(node, &statement.body, ctx),
            AstKind::ForOfStatement(statement) => Self::check_loop_body(node, &statement.body, ctx),
            AstKind::WhileStatement(statement) => Self::check_loop_body(node, &statement.body, ctx),
            AstKind::DoWhileStatement(statement) => {
                Self::check_loop_body(node, &statement.body, ctx);
            }
            _ => {}
        }
    }
}

impl NoLoopFunc {
    fn check_loop_body<'a>(loop_node: &AstNode<'a>, body: &Statement<'a>, ctx: &LintContext<'a>) {
        let mut collector = LoopFunctionCollector::default();
        collector.visit_statement(body);

        for function_node_id in collector.function_node_ids {
            let function_node = ctx.nodes().get_node(function_node_id);
            Self::check_function_in_loop(function_node, loop_node, ctx);
        }
    }

    fn check_function_in_loop<'a>(
        node: &AstNode<'a>,
        loop_node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let (func_span, is_async_or_generator) = match node.kind() {
            AstKind::Function(func) => {
                // Skip if not inside a statement (i.e., method definitions, etc.)
                if !func.is_expression() && !func.is_declaration() {
                    return;
                }

                (func.span, func.r#async || func.generator)
            }
            AstKind::ArrowFunctionExpression(arrow) => (arrow.span, arrow.r#async),
            _ => return,
        };

        // Skip synchronous IIFEs (Immediately Invoked Function Expressions) only if they
        // don't contain nested functions. Nested functions inside an IIFE could escape
        // (be returned, stored, etc.) with captured variables that change across iterations.
        // Async IIFEs and generator IIFEs are never safe:
        // - Async: execution suspends at await points, may resume after loop iteration
        // - Generator: returns iterator, code runs when iterated (possibly after loop)
        if !is_async_or_generator
            && Self::is_safe_iife(node, ctx)
            && !Self::contains_nested_functions(node, ctx)
        {
            return;
        }

        // Check if any referenced variables are unsafe
        if Self::has_unsafe_references(node, loop_node, ctx) {
            ctx.diagnostic(no_loop_func_diagnostic(func_span));
        }
    }

    /// Check if the function is a safe IIFE (Immediately Invoked Function Expression).
    /// A safe IIFE is one that:
    /// 1. Is immediately invoked (has a CallExpression parent where this function is the callee)
    /// 2. Does not reference itself (named function expressions that reference their own name
    ///    could escape by storing themselves somewhere)
    ///
    /// This is safe because the function executes immediately within each iteration,
    /// so the closure captures the current value and uses it right away.
    fn is_safe_iife<'a>(func_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        let nodes = ctx.nodes();

        let mut current = nodes.parent_node(func_node.id());
        while matches!(current.kind(), AstKind::ParenthesizedExpression(_)) {
            current = nodes.parent_node(current.id());
        }

        let AstKind::CallExpression(call_expr) = current.kind() else {
            return false;
        };

        let func_span = func_node.span();
        if !call_expr.callee.span().contains_inclusive(func_span) {
            return false;
        }

        // Check if the function has a name that is referenced inside itself.
        // Named function expressions like `(function f() { arr.push(f); })()` can escape
        // by storing themselves somewhere, making them unsafe.
        if let AstKind::Function(func) = func_node.kind()
            && let Some(id) = &func.id
        {
            for reference in ctx.scoping().get_resolved_references(id.symbol_id()) {
                let ref_node = nodes.get_node(reference.node_id());
                // If the reference is inside the function, the function could escape
                if func_span.contains_inclusive(ref_node.span()) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a function contains nested functions that could escape.
    /// A nested function "escapes" if:
    /// - It's not immediately invoked (not an IIFE), OR
    /// - It's async or a generator (even if IIFE, execution doesn't complete immediately), OR
    /// - It's a named IIFE that references itself (could store itself somewhere)
    fn contains_nested_functions<'a>(func_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        let mut finder = NestedFunctionFinder::new(func_node.id(), ctx);
        Self::visit_function_node(func_node, &mut finder);
        finder.found
    }

    /// Check if the function has any unsafe references to variables
    fn has_unsafe_references<'a>(
        func_node: &AstNode<'a>,
        loop_node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let mut finder = UnsafeReferenceFinder::new(func_node.span(), loop_node, ctx);
        Self::visit_function_node(func_node, &mut finder);
        finder.found
    }

    fn visit_function_node<'a, V: Visit<'a>>(func_node: &AstNode<'a>, visitor: &mut V) {
        match func_node.kind() {
            AstKind::Function(function) => visitor.visit_function(function, ScopeFlags::Function),
            AstKind::ArrowFunctionExpression(arrow) => {
                visitor.visit_arrow_function_expression(arrow);
            }
            _ => {}
        }
    }

    /// Determine if a variable reference is unsafe within a loop context
    fn is_unsafe_reference<'a>(
        symbol_id: SymbolId,
        loop_node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
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
    fn is_var_unsafe_in_loop<'a>(
        symbol_id: SymbolId,
        symbol_decl_node: &AstNode<'a>,
        loop_node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let loop_span = loop_node.span();
        let decl_span = symbol_decl_node.span();

        // If declared inside the loop (including loop header), it's unsafe
        // because all iterations share the same binding
        if loop_span.contains_inclusive(decl_span) {
            return true;
        }

        // Check if the variable is declared in a for-in/for-of header of an OUTER loop.
        // For `for (var x of xs)`, x gets a new value each iteration, which is not tracked
        // as a write reference. We need to check if the function is nested inside such a loop.
        if Self::is_var_in_outer_forof_loop(symbol_decl_node, loop_node, ctx) {
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

    /// Check if a var variable is declared in an outer for-in/for-of loop header.
    /// This is needed because the iteration assignment in for-in/for-of is not tracked
    /// as a write reference, but it still makes the closure unsafe.
    fn is_var_in_outer_forof_loop<'a>(
        symbol_decl_node: &AstNode<'a>,
        current_loop: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let nodes = ctx.nodes();
        let decl_span = symbol_decl_node.span();

        // Walk up from the current loop to find outer loops
        let mut current = nodes.parent_node(current_loop.id());
        loop {
            match current.kind() {
                AstKind::ForInStatement(stmt)
                    // Check if the variable is declared in the left part
                    if stmt.left.span().contains_inclusive(decl_span) => {
                        return true;
                    }
                AstKind::ForOfStatement(stmt)
                    // Check if the variable is declared in the left part
                    if stmt.left.span().contains_inclusive(decl_span) => {
                        return true;
                    }
                AstKind::ForStatement(stmt) => {
                    // For regular for loops, check if declared in init and modified in update
                    if let Some(init) = &stmt.init
                        && init.span().contains_inclusive(decl_span)
                        && stmt.update.is_some()
                    {
                        return true;
                    }
                }
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::Program(_) => {
                    return false;
                }
                _ => {}
            }
            current = nodes.parent_node(current.id());
        }
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

/// Collects functions in the current loop body. Nested loop bodies are skipped because
/// nested loops get their own rule invocation, but their headers still belong to this loop body.
#[derive(Default)]
struct LoopFunctionCollector {
    function_node_ids: Vec<NodeId>,
}

impl<'a> Visit<'a> for LoopFunctionCollector {
    fn visit_function(&mut self, function: &Function<'a>, _flags: ScopeFlags) {
        if function.is_expression() || function.is_declaration() {
            self.function_node_ids.push(function.node_id());
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.function_node_ids.push(arrow.node_id());
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        if let Some(init) = &statement.init {
            self.visit_for_statement_init(init);
        }
        if let Some(test) = &statement.test {
            self.visit_expression(test);
        }
        if let Some(update) = &statement.update {
            self.visit_expression(update);
        }
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        self.visit_for_statement_left(&statement.left);
        self.visit_expression(&statement.right);
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.visit_for_statement_left(&statement.left);
        self.visit_expression(&statement.right);
    }

    fn visit_while_statement(&mut self, statement: &WhileStatement<'a>) {
        self.visit_expression(&statement.test);
    }

    fn visit_do_while_statement(&mut self, statement: &DoWhileStatement<'a>) {
        self.visit_expression(&statement.test);
    }
}

/// Finds nested functions inside of a given function node.
struct NestedFunctionFinder<'a, 'ctx> {
    ctx: &'ctx LintContext<'a>,
    root_node_id: NodeId,
    found: bool,
}

impl<'a, 'ctx> NestedFunctionFinder<'a, 'ctx> {
    fn new(root_node_id: NodeId, ctx: &'ctx LintContext<'a>) -> Self {
        Self { ctx, root_node_id, found: false }
    }

    fn should_walk_function(&mut self, node_id: NodeId, is_async_or_generator: bool) -> bool {
        if self.found {
            return false;
        }

        if node_id == self.root_node_id {
            return true;
        }

        if is_async_or_generator {
            self.found = true;
            return false;
        }

        let node = self.ctx.nodes().get_node(node_id);
        if NoLoopFunc::is_safe_iife(node, self.ctx) {
            true
        } else {
            self.found = true;
            false
        }
    }
}

impl<'a> Visit<'a> for NestedFunctionFinder<'a, '_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if self.should_walk_function(function.node_id(), function.r#async || function.generator) {
            walk::walk_function(self, function, flags);
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.should_walk_function(arrow.node_id(), arrow.r#async) {
            walk::walk_arrow_function_expression(self, arrow);
        }
    }
}

/// Finds unsafe variable references inside of a given function node. A reference is unsafe if it resolves
/// to a variable that is declared outside the loop body and is modified across iterations.
struct UnsafeReferenceFinder<'a, 'ctx> {
    ctx: &'ctx LintContext<'a>,
    loop_node: &'ctx AstNode<'a>,
    func_span: Span,
    seen_symbols: FxHashSet<SymbolId>,
    found: bool,
}

impl<'a, 'ctx> UnsafeReferenceFinder<'a, 'ctx> {
    fn new(func_span: Span, loop_node: &'ctx AstNode<'a>, ctx: &'ctx LintContext<'a>) -> Self {
        Self { ctx, loop_node, func_span, seen_symbols: FxHashSet::default(), found: false }
    }
}

impl<'a> Visit<'a> for UnsafeReferenceFinder<'a, '_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.found {
            return;
        }

        let reference = self.ctx.scoping().get_reference(identifier.reference_id());
        let Some(symbol_id) = reference.symbol_id() else {
            return;
        };

        if !self.seen_symbols.insert(symbol_id) {
            return;
        }

        let scoping = self.ctx.scoping();
        let flags = scoping.symbol_flags(symbol_id);

        if flags.is_type() && !flags.is_value() {
            return;
        }

        let symbol_decl_node = self.ctx.nodes().get_node(scoping.symbol_declaration(symbol_id));
        if self.func_span.contains_inclusive(symbol_decl_node.span()) {
            return;
        }

        if NoLoopFunc::is_unsafe_reference(symbol_id, self.loop_node, self.ctx) {
            self.found = true;
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
        "for (using i of foo) { (function() { i; }) }",
        "for (await using i of foo) { (function() { i; }) }",
        "for (var i = 0; i < 10; ++i) { using foo = bar(i); (function() { foo; }) }",
        "for (var i = 0; i < 10; ++i) { await using foo = bar(i); (function() { foo; }) }",
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
        "let result = {};
            for (const score in scores) {
              const letters = scores[score];
              letters.split('').forEach(letter => {
                result[letter] = score;
              });
            }
            result.__default = 6;",
"while (true) {
                (function() { a; });
            }
            let a;",
        "while(i) { (function() { i; }) }",
        "do { (function() { i; }) } while (i)",
        "var i; while(i) { (function() { i; }) }",
        "var i; do { (function() { i; }) } while (i)",
        "for (var i=0; i<l; i++) { (function() { undeclared; }) }",
        "for (let i=0; i<l; i++) { (function() { undeclared; }) }",
        "for (var i in {}) { i = 7; (function() { undeclared; }) }",
        "for (let i in {}) { i = 7; (function() { undeclared; }) }",
        "for (const i of {}) { (function() { undeclared; }) }",
        "for (let i = 0; i < 10; ++i) { for (let x in xs.filter(x => x != undeclared)) {  } }",
"
                        let current = getStart();
                        while (current) {
                        (() => {
                            current;
                            current.a;
                            current.b;
                            current.c;
                            current.d;
                        })();

                        current = current.upper;
                        }
                        ",
"for (var i=0; (function() { i; })(), i<l; i++) { }",
"for (var i=0; i<l; (function() { i; })(), i++) { }",
"for (var i = 0; i < 10; ++i) { (()=>{ i;})() }",
"for (var i = 0; i < 10; ++i) { (function a(){i;})() }",
"
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            arr.push((f => f)((() => i)()));
                        }
                        ",
"
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            arr.push((() => {
                                return (() => i)();
                            })());
                        }
                        ",
"
                        const foo = bar;

                        for (var i = 0; i < 5; i++) {
                            arr.push(() => foo);
                        }

                        foo = baz; // This is a runtime error, but not concern of this rule. For this rule, variable 'foo' is constant.
                        ",
"
                        using foo = bar;

                        for (var i = 0; i < 5; i++) {
                            arr.push(() => foo);
                        }

                        foo = baz; // This is a runtime error, but not concern of this rule. For this rule, variable 'foo' is constant.
                        ",
"
                        await using foo = bar;

                        for (var i = 0; i < 5; i++) {
                            arr.push(() => foo);
                        }

                        foo = baz; // This is a runtime error, but not concern of this rule. For this rule, variable 'foo' is constant.
                        ",
"
              for (let i = 0; i < 10; i++) {
                function foo() {
                  console.log('A');
                }
              }
                  ",
"
              let someArray: MyType[] = [];
              for (let i = 0; i < 10; i += 1) {
                someArray = someArray.filter((item: MyType) => !!item);
              }
                    ",
"
              type MyType = 1;
              let someArray: MyType[] = [];
              for (let i = 0; i < 10; i += 1) {
                someArray = someArray.filter((item: MyType) => !!item);
              }
                  ",
"
                // UnconfiguredGlobalType is not defined anywhere or configured in globals
                for (var i = 0; i < 10; i++) {
                  const process = (item: UnconfiguredGlobalType) => {
                    // This is valid because the type reference is considered safe
                    // even though UnconfiguredGlobalType is not configured
                    return item.id;
                  };
                }
                ",
"
                for (var i = 0; i < 10; i++) {
                  // ConfiguredType is in globals, UnconfiguredType is not
                  // Both should be considered safe as they are type references
                  const process = (configItem: ConfiguredType, unconfigItem: UnconfiguredType) => {
                    return {
                      config: configItem.value,
                      unconfig: unconfigItem.value
                    };
                  };
                }
                  ",
        // Function in the for-update slot is not in the loop body.
        "for (var i = 0; i < l; i++, (function () { i; })()) { }",
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
        "let a; for (let i=0; i<l; i++) { (function() { (function() { a; }); }); a = 1; }",
        "let a; for (let i in {}) { a = 1; function foo() { (function() { a; }); } }",
        "let a; for (let i of {}) { (() => { (function() { a; }); }); } a = 1;",
        "for (var i = 0; i < 10; ++i) { for (let x in xs.filter(x => x != i)) {  } }",
        "for (let x of xs) { let a; for (let y of ys) { a = 1; (function() { a; }); } }",
        "for (var x of xs) { for (let y of ys) { (function() { x; }); } }",
        "for (var x of xs) { (function() { x; }); }",
        "var a; for (let x of xs) { a = 1; (function() { a; }); }",
        "var a; for (let x of xs) { (function() { a; }); a = 1; }",
        "let a; function foo() { a = 10; } for (let x of xs) { (function() { a; }); } foo();",
        "let a; function foo() { a = 10; for (let x of xs) { (function() { a; }); } } foo();",
        "let a; for (var i=0; i<l; i++) { (function* (){i;})() }",
        "let a; for (var i=0; i<l; i++) { (async function (){i;})() }",
        "
                        let current = getStart();
                        const arr = [];
                        while (current) {
                            (function f() {
                                current;
                                arr.push(f);
                            })();

                            current = current.upper;
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            (function fun () {
                                if (arr.includes(fun)) return i;
                                else arr.push(fun);
                            })();
                        }
                        ",
        "
                        let current = getStart();
                        const arr = [];
                        while (current) {
                            const p = (async () => {
                                await someDelay();
                                current;
                            })();

                            arr.push(p);
                            current = current.upper;
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            arr.push((f => f)(
                                () => i
                            ));
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            arr.push((() => {
                                return () => i;
                            })());
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            arr.push((() => {
                                return () => { return i };
                            })());
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            arr.push((() => {
                                return () => {
                                    return () => i
                                };
                            })());
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i++) {
                            arr.push((() => {
                                return () =>
                                    (() => i)();
                            })());
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i ++) {
                            (() => {
                                arr.push((async () => {
                                    await 1;
                                    return i;
                                })());
                            })();
                        }
                        ",
        "
                        var arr = [];

                        for (var i = 0; i < 5; i ++) {
                            (() => {
                                (function f() {
                                    if (!arr.includes(f)) {
                                        arr.push(f);
                                    }
                                    return i;
                                })();
                            })();

                        }
                        ",
        r#"
                        var arr1 = [], arr2 = [];

                        for (var [i, j] of ["a", "b", "c"].entries()) {
                            (() => {
                                arr1.push((() => i)());
                                arr2.push(() => j);
                            })();
                        }
                        "#,
        "
                        var arr = [];

                        for (var i = 0; i < 5; i ++) {
                            ((f) => {
                                arr.push(f);
                            })(() => {
                                return (() => i)();
                            });

                        }
                        ",
        "
                        for (var i = 0; i < 5; i++) {
                            (async () => {
                                () => i;
                            })();
                        }
                        ",
        r#"
                        for (var i = 0; i < 10; i++) {
                            items.push({
                                id: i,
                                name: "Item " + i
                            });

                            const process = function (callback){
                                callback({ id: i, name: "Item " + i });
                            };
                        }
                        "#,
        "
              for (var i = 0; i < 10; i++) {
                function foo() {
                  console.log(i);
                }
              }
                        ",
        "
              for (var i = 0; i < 10; i++) {
                const handler = (event: Event) => {
                  console.log(i);
                };
              }
                        ",
        r#"
              interface Item {
                id: number;
                name: string;
              }

              const items: Item[] = [];
              for (var i = 0; i < 10; i++) {
                items.push({
                  id: i,
                  name: "Item " + i
                });

                const process = function(callback: (item: Item) => void): void {
                  callback({ id: i, name: "Item " + i });
                };
              }
                        "#,
        "
              type Processor<T> = (item: T) => void;

              for (var i = 0; i < 10; i++) {
                const processor: Processor<number> = (item) => {
                  return item + i;
                };
              }
                        ",
        "
                  for (var i = 0; i < 10; i++) {
                    // UnconfiguredGlobalType is not defined anywhere
                    // But the function still references i which makes it unsafe
                    const process = (item: UnconfiguredGlobalType) => {
                      console.log(i, item.value);
                    };
                  }
                  ",
        "
            var arr = [];
            for (var i = 0; i < 5; i++) {
                (() => {
                    (function f() {
                        arr.push(f);
                        return i;
                    })();
                })();
            }
            ",
        // Multiple sibling functions in one loop body — all must be reported.
        "for (var i = 0; i < 5; i++) {
            (function () { return i; });
            (function () { return i + 1; });
            (() => i);
        }",
        // Function nested inside switch/if/try inside a loop body — descendant walk
        // must reach it.
        "for (var i = 0; i < 5; i++) {
            if (cond) {
                try {
                    arr.push(function () { return i; });
                } catch (e) {}
            } else {
                switch (k) {
                    case 1:
                        arr.push(() => i);
                }
            }
        }",
        // do-while body comes before the test in source; multiple statements in body.
        "var i = 0; do {
            arr.push(function () { return i; });
            arr.push(() => i);
        } while (i++ < 5);",
    ];

    Tester::new(NoLoopFunc::NAME, NoLoopFunc::PLUGIN, pass, fail).test_and_snapshot();
}
