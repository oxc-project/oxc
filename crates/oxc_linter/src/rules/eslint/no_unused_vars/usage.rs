//! This module contains logic for checking if any [`Reference`]s to a
//! [`Symbol`] are considered a usage.

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{AstNode, NodeId, Reference, ScopeId, SymbolFlags, SymbolId};
use oxc_span::{GetSpan, Span};

use super::{ignored::FoundStatus, NoUnusedVars, Symbol};

impl<'s, 'a> Symbol<'s, 'a> {
    /// 1. Imported functions will never have calls to themselves within their
    ///    own declaration since they are declared outside the current module
    /// 2. Catch variables are always parameter-like and will therefore never have
    ///    a function declaration.
    #[inline]
    const fn is_maybe_callable(&self) -> bool {
        const IMPORT: SymbolFlags = SymbolFlags::Import.union(SymbolFlags::TypeImport);
        // note: intetionally do not use `SymbolFlags::is_type` here, since that
        // can return `true` for values
        const TYPE: SymbolFlags = SymbolFlags::TypeAlias
            .union(SymbolFlags::TypeLiteral)
            .union(SymbolFlags::TypeParameter)
            .union(SymbolFlags::Interface);
        const ENUM: SymbolFlags = SymbolFlags::Enum.union(SymbolFlags::EnumMember);
        const NAMESPACE_LIKE: SymbolFlags =
            SymbolFlags::NameSpaceModule.union(SymbolFlags::ValueModule);

        !self.flags().intersects(
            IMPORT.union(TYPE).union(ENUM).union(NAMESPACE_LIKE).union(SymbolFlags::CatchVariable),
        )
    }

    /// Note: we still need to check for reassignments to const variables since
    /// eslint's original rule requires it. Const reassignments are not a syntax
    /// error in JavaScript, only TypeScript.
    #[inline]
    const fn is_possibly_reassignable(&self) -> bool {
        self.flags().intersects(SymbolFlags::Variable)
    }

    /// Check if this [`Symbol`] is definitely reassignable.
    ///
    /// Examples of non-reassignable symbols are:
    /// - function declarations
    /// - classes
    /// - enums
    /// - types (interfaces, type aliases)
    /// - const variables
    /// - imports
    ///
    /// Examples of reassinable symbols are:
    /// - `var` and `let` variable declarations
    /// - function parameters
    #[inline]
    const fn is_definitely_reassignable_variable(&self) -> bool {
        let f = self.flags();
        f.intersects(SymbolFlags::Variable)
            && !f.contains(SymbolFlags::ConstVariable.union(SymbolFlags::Function))
    }

    #[inline]
    const fn could_have_type_reference_within_own_decl(&self) -> bool {
        const TYPE_DECLS: SymbolFlags =
            SymbolFlags::TypeAlias.union(SymbolFlags::Interface).union(SymbolFlags::Class);
        self.flags().intersects(TYPE_DECLS)
    }

    /// Check if this [`Symbol`] has an [`Reference`]s that are considered a usage.
    pub fn has_usages(&self, options: &NoUnusedVars) -> bool {
        // Use symbol flags to skip the usage checks we are certain don't need
        // to be run.
        let do_reassignment_checks = self.is_possibly_reassignable();
        let do_type_self_usage_checks = self.could_have_type_reference_within_own_decl();
        let do_self_call_check = self.is_maybe_callable();
        let do_discarded_read_checks = self.is_definitely_reassignable_variable();

        for reference in self.references() {
            // Resolved references should always contain the id of the symbol
            // they are referencing. By making this an assertion instead of a
            // debug assertion, the rust compiler can optimize away None checks
            // performed down the line.
            assert!(
                reference.symbol_id().is_some(),
                "Resolved reference to symbol {:?} is missing a symbol id",
                self.id()
            );
            assert!(reference.symbol_id().is_some_and(|id| id == self.id()));

            // Write usage checks
            if reference.is_write() {
                if do_reassignment_checks
                    && (self.is_assigned_to_ignored_destructure(reference, options)
                        || self.is_used_in_for_of_loop(reference))
                {
                    return true;
                }

                // references can be both reads & writes. If this is only a
                // write, we don't need to perform any read usage checks.
                if !reference.is_read() {
                    continue;
                }
            }

            // Type usage checks
            if reference.is_type() {
                // e.g. `type Foo = Array<Foo>`
                if do_type_self_usage_checks && self.is_type_self_usage(reference) {
                    continue;
                }
                return true;
            }

            // Read usage checks

            // e.g. `let a = 0; a = a + 1`
            if do_reassignment_checks && self.is_self_reassignment(reference) {
                continue;
            }

            // e.g. reference on `a` in expression `let a = 0; let b = (a++, 0);`
            if do_discarded_read_checks && self.is_discarded_read(reference) {
                continue;
            }

            // e.g. `function foo() { foo() }`
            if do_self_call_check && self.is_self_call(reference) {
                continue;
            }

            return true;
        }

        false
    }

    /// Checks for references within for..in and for..of conditions (not
    /// bodies). These are always considered usages since their removal would
    /// introduce syntax and/or semantic errors.
    ///
    /// ## Examples
    /// ```ts
    /// // should return true
    /// var a;
    /// for (a in obj) {}
    /// for (a of iter) {}
    ///
    /// // should return false
    /// var b;
    /// for (let a in obj) { fn(b) }
    /// for (let a of iter) { fn(b) }
    /// ```
    fn is_used_in_for_of_loop(&self, reference: &Reference) -> bool {
        for parent in self.nodes().iter_parents(reference.node_id()) {
            match parent.kind() {
                AstKind::ParenthesizedExpression(_)
                | AstKind::IdentifierReference(_)
                | AstKind::SimpleAssignmentTarget(_)
                | AstKind::AssignmentTarget(_) => continue,
                AstKind::ForInStatement(ForInStatement { body, .. })
                | AstKind::ForOfStatement(ForOfStatement { body, .. }) => match body {
                    Statement::ReturnStatement(_) => return true,
                    Statement::BlockStatement(b) => {
                        return b
                            .body
                            .first()
                            .is_some_and(|s| matches!(s, Statement::ReturnStatement(_)));
                    }
                    _ => return false,
                },
                _ => return false,
            }
        }

        false
    }

    /// Does this variable have a name that is ignored by the destructuring
    /// pattern, and is also assigned inside a destructure?
    ///
    /// ```ts
    /// let a, _b;
    /// [a, _b] = [1, 2];
    /// //  ^^ this should be ignored
    ///
    /// console.log(a)
    /// ```
    fn is_assigned_to_ignored_destructure(
        &self,
        reference: &Reference,
        options: &NoUnusedVars,
    ) -> bool {
        // Return early if no destructure ignores are configured.
        if !options.should_search_destructures() {
            return false;
        }

        for parent in self.nodes().iter_parents(reference.node_id()).map(AstNode::kind) {
            match parent {
                AstKind::IdentifierReference(_)
                | AstKind::SimpleAssignmentTarget(_)
                | AstKind::AssignmentTarget(_) => continue,
                AstKind::AssignmentExpression(assignment) => {
                    return options.is_ignored_assignment_target(self, &assignment.left);
                }
                // Needs to be checked separately from AssignmentTarget due to
                // weird heritage bug for object assignment patterns.
                // when iterating over parents, after an
                // ObjectAssignmentTarget, the next parent will be the rest
                // expression instead of the top-level AssignmentTarget
                AstKind::ObjectAssignmentTarget(obj) => {
                    match options.search_obj_assignment_target(self, obj) {
                        FoundStatus::Ignored => return true,
                        FoundStatus::NotIgnored => return false,
                        FoundStatus::NotFound => continue,
                    }
                }
                AstKind::ArrayAssignmentTarget(arr) => {
                    match options.search_array_assignment_target(self, arr) {
                        FoundStatus::Ignored => return true,
                        FoundStatus::NotIgnored => return false,
                        FoundStatus::NotFound => continue,
                    }
                }
                _ => {
                    return false;
                }
            }
        }
        false
    }

    /// Checks for self-usages in type declarations.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// // should return true
    /// type Foo = Foo
    /// type Foo = Array<Foo>
    /// type Unbox<B> = B extends Box<infer R> ? Unbox<R> : B
    ///
    /// // should return false
    /// type Foo = Bar
    /// type Foo = Array<Bar>
    /// ```
    fn is_type_self_usage(&self, reference: &Reference) -> bool {
        for parent in self.iter_relevant_parents_of(reference.node_id()).map(AstNode::kind) {
            match parent {
                AstKind::TSTypeAliasDeclaration(decl) => {
                    return self == &decl.id;
                }
                // definitely not within a type alias, we can be sure this isn't
                // a self-usage. Safe CPU cycles by breaking early.
                // NOTE: we cannot short-circuit on functions since they could
                // be methods with annotations referencing the type they're in.
                // e.g.:
                // - `type Foo = { bar(): Foo }`
                // - `class Foo { static factory(): Foo { return new Foo() } }`
                AstKind::TSModuleDeclaration(_)
                | AstKind::VariableDeclaration(_)
                | AstKind::VariableDeclarator(_)
                | AstKind::ExportNamedDeclaration(_)
                | AstKind::ExportDefaultDeclaration(_)
                | AstKind::ExportAllDeclaration(_)
                | AstKind::Program(_) => {
                    return false;
                }

                AstKind::CallExpression(_) | AstKind::BinaryExpression(_) => {
                    // interfaces/type aliases cannot have value expressions
                    // within their declarations, so we know we're not in one.
                    // However, classes can.
                    if self.flags().is_class() {
                        continue;
                    }
                    return false;
                }

                // `interface LinkedList<T> { next?: LinkedList<T> }`
                AstKind::TSInterfaceDeclaration(iface) => {
                    return self.flags().is_interface() && self == &iface.id;
                }

                // `class Foo { bar(): Foo }`
                AstKind::Class(class) => {
                    return self.flags().is_class()
                        && class.id.as_ref().is_some_and(|id| self == id);
                }

                _ => continue,
            }
        }
        false
    }

    /// Checks if a read reference is only ever used to modify itself.
    ///
    /// ## Algorithm
    /// This algorithm is a little confusing, so here's how it works:
    ///
    /// A reference can be a self reassignment that is used by others or not.
    /// For example:
    /// ```ts
    /// let a = 0; a = a + 1
    /// //         ^^^^^^^^^ self reassignment, only used by itself.
    /// let a = 0, b = 0; b = a = a + 1
    /// //                    ^^^^^^^^^ self reassignment, but used by another variable.
    /// ```
    ///
    /// Initially, all references are assumed to be used by others. This allows
    /// for code like `let a = 0; a`, but bans code like `let a = 0; a++`;
    ///
    /// - We encounter a node proving that the reference is absolutely used by
    /// another variable, we return `false` immediately.
    /// - When we encounter an AST node that updates the value of the symbol this
    /// reference is for, such as an [`AssignmentExpression`] with the symbol on
    /// the LHS or a mutating [`UnaryExpression`], we mark the reference as not
    /// being used by others.
    /// - When we encounter a node where we are sure the value produced by an
    ///   expression will no longer be used, such as an [`ExpressionStatement`],
    ///   we end our search. This is because expression statements produce a
    ///   value and then discard it. In these cases, we return `true` if the
    ///   reference was not used by others, or `false` if it was.
    ///
    /// ## Examples
    /// ```
    /// let a = 0;
    /// // should return true
    /// a++;
    /// a = a + 1;
    /// a ||= 1;
    ///
    /// // should return false
    /// let b = a;
    /// if (a++) {}
    /// function f() { return a }
    /// ```
    fn is_self_reassignment(&self, reference: &Reference) -> bool {
        if reference.symbol_id().is_none() {
            debug_assert!(
                false,
                "is_self_reassignment() should only be called on resolved symbol references"
            );
            return true;
        }

        // Have we seen this reference be used to update the value of another
        // symbol, or for some other logically-relevant purpose?
        let mut is_used_by_others = true;
        let name = self.name();
        let ref_span = self.get_ref_span(reference);

        for node in self.nodes().iter_parents(reference.node_id()).skip(1) {
            match node.kind() {
                // references used in declaration of another variable are definitely
                // used by others
                AstKind::VariableDeclarator(_)
                | AstKind::JSXExpressionContainer(_)
                | AstKind::Argument(_) => {
                    // definitely used, short-circuit
                    return false;
                }
                // When symbol is being assigned a new value, we flag the reference
                // as only affecting itself until proven otherwise.
                AstKind::UpdateExpression(UpdateExpression { argument, .. })
                | AstKind::SimpleAssignmentTarget(argument) => {
                    // `a.b++` or `a[b] + 1` are not reassignment of `a`
                    if !argument.is_member_expression() {
                        is_used_by_others = false;
                    }
                }
                // RHS usage when LHS != reference's symbol is definitely used by
                // others
                AstKind::AssignmentExpression(AssignmentExpression { left, .. }) => {
                    match left {
                        AssignmentTarget::AssignmentTargetIdentifier(id) => {
                            if id.name == name {
                                is_used_by_others = false;
                            } else {
                                return false; // we can short-circuit
                            }
                        }
                        // variable is being used to index another variable, this is
                        // always a usage
                        // todo: check self index?
                        match_member_expression!(AssignmentTarget) => return false,
                        _ => {}
                    }
                }
                // `if (i++ === 0) { /* ... */ }`
                AstKind::IfStatement(IfStatement { test, .. })
                | AstKind::WhileStatement(WhileStatement { test, .. })
                | AstKind::DoWhileStatement(DoWhileStatement { test, .. })
                    if test.span().contains_inclusive(ref_span) =>
                {
                    return false;
                }

                // expression is over, save cycles by breaking
                // todo: do we need to check if variable is used as iterator in
                // loops?
                AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_) => {
                    break;
                }
                // this is needed to handle `return () => foo++`
                AstKind::ExpressionStatement(_) => {
                    if self.is_in_return_statement(node.id()) {
                        return false;
                    }
                    break;
                }
                AstKind::Function(f) if f.is_declaration() => {
                    break;
                }
                // implicit return in an arrow function
                AstKind::ArrowFunctionExpression(f)
                    if f.body.statements.len() == 1
                        && !self.get_snippet(f.body.span).starts_with('{') =>
                {
                    return false;
                }
                AstKind::ReturnStatement(_) => {
                    match self.get_nearest_function(node.id()) {
                        // We're definitely in a function (assuming valid
                        // syntax) so that means we're in an anonymous function,
                        // which is definitely not the current symbol ∴ not the
                        // current symbol ∴ not a self-reassignment
                        None => return false,
                        // Is this a return within the same function being declared?
                        Some(id) => return self.id() == id,
                    };
                }
                // function* foo() {
                //    let a = 1;
                //    a = yield a // <- still considered used b/c it's propagated to the caller
                // }
                AstKind::YieldExpression(_) => return false,
                _ => { /* continue up tree */ }
            }
        }

        !is_used_by_others
    }

    /// Check if a [`AstNode`] is within a return statement or implicit return.
    fn is_in_return_statement(&self, node_id: NodeId) -> bool {
        for parent in self.iter_relevant_parents_of(node_id).map(AstNode::kind) {
            match parent {
                AstKind::ReturnStatement(_) => return true,
                AstKind::ExpressionStatement(_) => continue,
                AstKind::Function(f) if f.is_expression() => continue,
                // note: intentionally not using
                // ArrowFunctionExpression::get_expression since it returns
                // `Some` even if
                // 1. there are more than one statements
                // 2. the expression is surrounded by braces
                AstKind::ArrowFunctionExpression(f)
                    if f.body.statements.len() == 1
                        && !self.get_snippet(f.body.span).starts_with('{') =>
                {
                    return true;
                }
                x if x.is_statement() => return false,
                _ => continue,
            }
        }
        false
    }

    /// Returns `true` for read references where we are confident the read is
    /// discarded (and therefore never used). Right now, this is only covers
    /// expressions within [`SequenceExpression`]s that are not in the last position.
    ///
    /// ```ts
    /// let a = 0; let b = (a, 0); // a is discarded
    /// let a = 1, b = 0; let c = (b = a, 0); // a is not discarded b/c it updates b
    /// ```
    ///
    /// Maybe we'll eventually handle cases like this:
    /// ```ts
    /// let a = 0;
    /// a; // not really used
    /// ```
    ///
    /// but doing so requires us to know if a read has side effects, which we
    /// can't do confidently without type information. For example, this read
    /// calls a getter that mutates state:
    ///
    /// ```ts
    /// global.x = 0;
    /// let foo = {
    ///     get bar() {
    ///         global.x += 1;
    ///         return global.x;
    ///     }
    /// };
    ///
    /// foo.bar;
    /// ```
    fn is_discarded_read(&self, reference: &Reference) -> bool {
        for (parent, grandparent) in
            self.iter_relevant_parent_and_grandparent_kinds(reference.node_id())
        {
            let ref_span = || self.get_ref_span(reference);

            match (parent, grandparent) {
                // (foo.bar = new Foo(a), f(b))
                // `a` should not be considered discarded
                // first branch happens when reference is a function call,
                // second one happens when reference is an argument to a
                // function call
                (
                    AstKind::IdentifierReference(id),
                    AstKind::CallExpression(_) | AstKind::NewExpression(_),
                ) => {
                    if id.span == ref_span() {
                        continue;
                    }
                    break;
                }
                (_, AstKind::CallExpression(_) | AstKind::NewExpression(_)) => break,
                // (AstKind::FunctionBody(_), _) => return true,
                // in `(x = a, 0)`, reference to `a` should still be considered
                // used. Note that this branch must come before the sequence
                // expression check.
                (AstKind::AssignmentExpression(assignment), _) if self != &assignment.left => break,
                (AstKind::ConditionalExpression(cond), _) => {
                    if cond.test.span().contains_inclusive(ref_span()) {
                        return false;
                    }
                }
                (parent, AstKind::SequenceExpression(seq)) => {
                    debug_assert!(
                        !seq.expressions.is_empty(),
                        "empty SequenceExpressions should be a parse error."
                    );
                    let Some(last) = seq.expressions.last() else {
                        continue;
                    };
                    // "parent" won't always have the same span as "last" even
                    // if it's in the last position since some nodes are
                    // skipped. This means an equality check cannot be used here.
                    if !last.span().contains_inclusive(parent.span()) {
                        return true;
                    }
                }
                _ => continue,
            }
        }

        false
    }

    /// Checks if a [`Reference`] is for a [`CallExpression`] or
    /// [`NewExpression`] for a method/function/class within its own declaration.
    /// These do not count as a usage.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// function foo() { foo() };
    /// const a = () => () => { a() }
    /// class Foo { bar() { return new Foo() } }
    /// ```
    fn is_self_call(&self, reference: &Reference) -> bool {
        let Some(ref_node) = self.get_ref_relevant_node(reference) else {
            return false;
        };
        if !matches!(ref_node.kind(), AstKind::CallExpression(_) | AstKind::NewExpression(_)) {
            return false;
        }

        // Do the easy/fast path if possible. If we know its a class/fn from
        // flags, that means it's declared within this file in an understandable
        // way, and we can get a container scope id for it. This isn't possible
        // for parameters, e.g. `function foo(cb) { cb = function() { cb() } }`
        if self.flags().is_function() || self.flags().is_class() {
            return self.is_self_call_simple(reference);
        }

        // check for assignment/declaration of a function expression to a variable
        if self.is_self_function_expr_assignment(ref_node) {
            return true;
        }

        false
    }

    fn is_self_function_expr_assignment(&self, ref_node: &AstNode<'a>) -> bool {
        for (parent, grandparent) in self.iter_relevant_parent_and_grandparent_kinds(ref_node.id())
        {
            match (parent, grandparent) {
                // const a = function() {}
                (AstKind::Function(f), AstKind::VariableDeclarator(decl))
                    if f.is_expression() && self == &decl.id =>
                {
                    return true;
                }
                // const a = () => {}
                (AstKind::ArrowFunctionExpression(_), AstKind::VariableDeclarator(decl))
                    if self == &decl.id =>
                {
                    return true;
                }
                // let a; a = function() {}
                (AstKind::Function(f), AstKind::AssignmentExpression(assignment))
                    if f.is_expression() && self == &assignment.left =>
                {
                    return true;
                }
                // let a; a = () => {}
                (
                    AstKind::ArrowFunctionExpression(_),
                    AstKind::AssignmentExpression(assignment),
                ) if self == &assignment.left => {
                    return true;
                }
                _ => {}
            }
        }

        false
    }

    fn is_self_call_simple(&self, reference: &Reference) -> bool {
        let decl_scope_id = self.scope_id();
        let call_scope_id = self.get_ref_scope(reference);
        let Some(container_id) = self.declaration().kind().get_container_scope_id() else {
            debug_assert!(
                false,
                "Found a function call or or new expr reference on a node flagged as a function or class, but the symbol's declaration node has no scope id. It should always be a container."
            );
            return false;
        };

        // scope ids are created in ascending order in an "E" shape
        // (depth-first, from top to bottom). if call < decl, then it will never
        // be within a scope contained by the declaration, and therefore never
        // be a self-call. Similarly, if the call is within the same scope as
        // the declaration, it will never be inside the declaration.
        if call_scope_id <= decl_scope_id {
            return false;
        }

        for scope_id in self.scopes().ancestors(call_scope_id) {
            if scope_id == container_id {
                return true;
            } else if scope_id == decl_scope_id {
                return false;
            }
        }

        unreachable!();
    }

    /// Get the [`ScopeId`] where a [`Reference`] is located.
    #[inline]
    fn get_ref_scope(&self, reference: &Reference) -> ScopeId {
        self.nodes().get_node(reference.node_id()).scope_id()
    }

    /// Get the [`Span`] covering the [`AstNode`] containing a [`Reference`].
    #[inline]
    fn get_ref_span(&self, reference: &Reference) -> Span {
        self.nodes().get_node(reference.node_id()).kind().span()
    }

    /// Get the first "relevant" parent of the node containing a [`Reference`].
    /// 1. References (should) always point to [`IdentifierReference`] nodes,
    ///    which isn't useful for checking kinds/usage, so we want the parent
    /// 2. "relevant" nodes are non "transparent". For example, parenthesis are "transparent".
    #[inline]
    fn get_ref_relevant_node(&self, reference: &Reference) -> Option<&AstNode<'a>> {
        self.iter_relevant_parents_of(reference.node_id()).next()
    }

    /// Find the [`SymbolId`] for the nearest function declaration or expression
    /// that is a parent of `node_id`.
    fn get_nearest_function(&self, node_id: NodeId) -> Option<SymbolId> {
        // set to `true` when we find an arrow function and we want to get its
        // name from the variable its assigned to.
        let mut needs_variable_identifier = false;

        for parent in self.iter_relevant_parents_of(node_id) {
            match parent.kind() {
                AstKind::Function(f) => {
                    return f.id.as_ref().and_then(|id| id.symbol_id.get());
                }
                AstKind::ArrowFunctionExpression(_) => {
                    needs_variable_identifier = true;
                    continue;
                }
                AstKind::VariableDeclarator(decl) if needs_variable_identifier => {
                    return decl.id.get_binding_identifier().and_then(|id| id.symbol_id.get());
                }
                AstKind::AssignmentTarget(target) if needs_variable_identifier => {
                    return match target {
                        AssignmentTarget::AssignmentTargetIdentifier(id) => id
                            .reference_id
                            .get()
                            .and_then(|rid| self.symbols().get_reference(rid).symbol_id()),
                        _ => None,
                    };
                }
                AstKind::Program(_) => {
                    return None;
                }
                _ => continue,
            }
        }

        None
    }
}
