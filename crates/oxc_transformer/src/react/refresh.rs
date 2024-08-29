use std::{cell::Cell, iter::once};

use oxc_allocator::CloneIn;
use oxc_ast::{
    ast::*, match_expression, match_member_expression, visit::walk::walk_variable_declarator, Visit,
};
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags, SymbolId};
use oxc_span::{Atom, GetSpan, Span, SPAN};
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::{Ancestor, TraverseCtx};
use rustc_hash::FxHashMap;

use super::options::ReactRefreshOptions;

use crate::context::Ctx;

/// React Fast Refresh
///
/// Transform React functional components to integrate Fast Refresh.
///
/// References:
///
/// * <https://github.com/facebook/react/issues/16604#issuecomment-528663101>
/// * <https://github.com/facebook/react/blob/main/packages/react-refresh/src/ReactFreshBabelPlugin.js>
pub struct ReactRefresh<'a> {
    refresh_reg: Atom<'a>,
    refresh_sig: Atom<'a>,
    _emit_full_signatures: bool,
    registrations: Vec<(SymbolId, Atom<'a>)>,
    ctx: Ctx<'a>,
    signature_declarator_items: Vec<oxc_allocator::Vec<'a, VariableDeclarator<'a>>>,
    /// Used to wrap call expression with signature.
    /// (eg: hoc(() => {}) -> _s1(hoc(_s1(() => {}))))
    last_signature: Option<(BindingIdentifier<'a>, oxc_allocator::Vec<'a, Argument<'a>>)>,
    extra_statements: FxHashMap<SymbolId, oxc_allocator::Vec<'a, Statement<'a>>>,
}

impl<'a> ReactRefresh<'a> {
    pub fn new(options: &ReactRefreshOptions, ctx: Ctx<'a>) -> Self {
        // TODO: refresh_reg and refresh_sig need to support MemberExpression
        Self {
            refresh_reg: ctx.ast.atom(&options.refresh_reg),
            refresh_sig: ctx.ast.atom(&options.refresh_sig),
            _emit_full_signatures: options.emit_full_signatures,
            signature_declarator_items: Vec::new(),
            registrations: Vec::default(),
            ctx,
            last_signature: None,
            extra_statements: FxHashMap::default(),
        }
    }

    fn create_registration(
        &mut self,
        persistent_id: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> AssignmentTarget<'a> {
        let symbol_id = ctx.generate_uid_in_root_scope("c", SymbolFlags::FunctionScopedVariable);
        self.registrations.push((symbol_id, persistent_id));
        let name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));
        let ident = ctx.create_reference_id(SPAN, name, Some(symbol_id), ReferenceFlags::Write);
        let ident = ctx.ast.simple_assignment_target_from_identifier_reference(ident);
        ctx.ast.assignment_target_simple(ident)
    }

    /// Similar to the `findInnerComponents` function in `react-refresh/babel`.
    fn replace_inner_components(
        &mut self,
        inferred_name: &str,
        expr: &mut Expression<'a>,
        is_variable_declarator: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        match expr {
            Expression::Identifier(ref ident) => {
                // For case like:
                // export const Something = hoc(Foo)
                // we don't want to wrap Foo inside the call.
                // Instead we assume it's registered at definition.
                return is_componentish_name(&ident.name);
            }
            Expression::FunctionExpression(_) => {}
            Expression::ArrowFunctionExpression(arrow) => {
                // Don't transform `() => () => {}`
                if arrow
                    .get_expression()
                    .is_some_and(|expr| matches!(expr, Expression::ArrowFunctionExpression(_)))
                {
                    return false;
                }
            }
            Expression::CallExpression(ref mut call_expr) => {
                let allowed_callee = matches!(
                    call_expr.callee,
                    Expression::Identifier(_)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::StaticMemberExpression(_)
                );

                if allowed_callee {
                    let callee_span = call_expr.callee.span();

                    let Some(argument_expr) =
                        call_expr.arguments.first_mut().and_then(|e| e.as_expression_mut())
                    else {
                        return false;
                    };

                    let found_inside = self.replace_inner_components(
                        format!(
                            "{}${}",
                            inferred_name,
                            callee_span.source_text(self.ctx.source_text)
                        )
                        .as_str(),
                        argument_expr,
                        /* is_variable_declarator */ false,
                        ctx,
                    );

                    if !found_inside {
                        return false;
                    }

                    // const Foo = hoc1(hoc2(() => {}))
                    // export default memo(React.forwardRef(function() {}))
                    if is_variable_declarator {
                        return true;
                    }
                } else {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }

        *expr = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            self.create_registration(ctx.ast.atom(inferred_name), ctx),
            ctx.ast.move_expression(expr),
        );

        true
    }

    /// Create an identifier reference from a binding identifier.
    fn create_identifier_reference_from_binding_identifier(
        id: &BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        ctx.ast.expression_from_identifier_reference(ctx.create_reference_id(
            SPAN,
            id.name.clone(),
            id.symbol_id.get(),
            ReferenceFlags::Read,
        ))
    }

    /// _c = id.name;
    fn create_assignment_expression(
        &mut self,
        id: &BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let left = self.create_registration(id.name.clone(), ctx);
        let right = ctx.create_bound_reference_id(
            SPAN,
            id.name.clone(),
            id.symbol_id.get().unwrap(),
            ReferenceFlags::Read,
        );
        let right = ctx.ast.expression_from_identifier_reference(right);
        let expr = ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, right);
        ctx.ast.statement_expression(SPAN, expr)
    }

    fn create_signature_call_expression(
        &mut self,
        scope_id: ScopeId,
        body: &mut FunctionBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<(BindingIdentifier<'a>, oxc_allocator::Vec<'a, Argument<'a>>)> {
        let arguments =
            CalculateSignatureKey::new(self.ctx.source_text, scope_id, ctx).calculate(body)?;

        let symbol_id =
            ctx.generate_uid("s", ctx.current_scope_id(), SymbolFlags::FunctionScopedVariable);

        let symbol_name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));

        let binding_identifier = BindingIdentifier {
            span: SPAN,
            name: symbol_name.clone(),
            symbol_id: Cell::new(Some(symbol_id)),
        };

        let sig_identifier_reference = ctx.create_reference_id(
            SPAN,
            self.refresh_sig.clone(),
            Some(symbol_id),
            ReferenceFlags::Read,
        );

        // _s();
        let call_expression = ctx.ast.statement_expression(
            SPAN,
            ctx.ast.expression_call(
                SPAN,
                Self::create_identifier_reference_from_binding_identifier(&binding_identifier, ctx),
                Option::<TSTypeParameterInstantiation>::None,
                ctx.ast.vec(),
                false,
            ),
        );

        body.statements.insert(0, call_expression);

        // _s = refresh_sig();
        self.signature_declarator_items.last_mut().unwrap().push(ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.binding_pattern(
                ctx.ast.binding_pattern_kind_from_binding_identifier(binding_identifier.clone()),
                Option::<TSTypeAnnotation>::None,
                false,
            ),
            Some(ctx.ast.expression_call(
                SPAN,
                ctx.ast.expression_from_identifier_reference(sig_identifier_reference.clone()),
                Option::<TSTypeParameterInstantiation>::None,
                ctx.ast.vec(),
                false,
            )),
            false,
        ));

        // Following is the signature call expression, will be generated in call site.
        // _s(App, signature_key, false, function() { return [] });
        //                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ custom hooks only
        Some((binding_identifier, arguments))
    }
}

// Internal Methods for transforming
impl<'a> ReactRefresh<'a> {
    /// Process statement and return a statement(if any) to insert it after current statement.
    ///
    /// ```js
    /// const Foo = styled("div")`color: hotpink`;
    /// function Bar() {}
    /// ```
    /// to
    /// ```js
    /// const Foo = styled("div")`color: hotpink`;
    /// _c = Foo;
    /// function Bar() { }
    /// _c1 = Bar;
    /// ```
    fn process_statement(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        match statement {
            Statement::VariableDeclaration(variable) => {
                self.handle_variable_declaration(variable, ctx)
            }
            Statement::FunctionDeclaration(func) => self.handle_function_declaration(func, ctx),
            Statement::ExportNamedDeclaration(export_decl) => {
                if let Some(declaration) = &mut export_decl.declaration {
                    match declaration {
                        Declaration::FunctionDeclaration(func) => {
                            self.handle_function_declaration(func, ctx)
                        }
                        Declaration::VariableDeclaration(variable) => {
                            self.handle_variable_declaration(variable, ctx)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Statement::ExportDefaultDeclaration(ref mut stmt_decl) => {
                match &mut stmt_decl.declaration {
                    declaration @ match_expression!(ExportDefaultDeclarationKind) => {
                        let expression = declaration.to_expression_mut();
                        if !matches!(expression, Expression::CallExpression(_)) {
                            // For now, we only support possible HOC calls here.
                            // Named function declarations are handled in FunctionDeclaration.
                            // Anonymous direct exports like export default function() {}
                            // are currently ignored.
                            return None;
                        }

                        // This code path handles nested cases like:
                        // export default memo(() => {})
                        // In those cases it is more plausible people will omit names
                        // so they're worth handling despite possible false positives.
                        // More importantly, it handles the named case:
                        // export default memo(function Named() {})
                        self.replace_inner_components(
                            "%default%",
                            expression,
                            /* is_variable_declarator */ false,
                            ctx,
                        );

                        None
                    }
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        if let Some(id) = &func.id {
                            if !is_componentish_name(&id.name) {
                                return None;
                            }

                            return Some(self.create_assignment_expression(id, ctx));
                        }
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn handle_function_declaration(
        &mut self,
        func: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        let Some(id) = &func.id else {
            return None;
        };

        if !is_componentish_name(&id.name) {
            return None;
        }

        Some(self.create_assignment_expression(id, ctx))
    }

    fn handle_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        if decl.declarations.len() != 1 {
            return None;
        }

        let declarator = decl.declarations.first_mut().unwrap_or_else(|| unreachable!());
        let init = declarator.init.as_mut()?;
        let id = declarator.id.get_binding_identifier()?;

        if !is_componentish_name(&id.name) {
            return None;
        }

        match init {
            // Likely component definitions.
            Expression::ArrowFunctionExpression(arrow) => {
                // () => () => {}
                if arrow.get_expression().is_some_and(|expr| matches!(expr, Expression::ArrowFunctionExpression(_))) {
                    return None;
                }
            }
            Expression::FunctionExpression(_)
            // Maybe something like styled.div`...`
            | Expression::TaggedTemplateExpression(_) => {
                // Special case when a variable would get an inferred name:
                // let Foo = () => {}
                // let Foo = function() {}
                // let Foo = styled.div``;
                // We'll register it on next line so that
                // we don't mess up the inferred 'Foo' function name.
                // (eg: with @babel/plugin-transform-react-display-name or
                // babel-plugin-styled-components)
            }
            Expression::CallExpression(call_expr) => {
                if matches!(call_expr.callee, Expression::ImportExpression(_))
                    || call_expr.is_require_call()
                {
                    return None;
                }

                // Maybe a HOC.
                // Try to determine if this is some form of import.
                let found_inside = self.replace_inner_components(
                    &id.name,
                    init,
                    /* is_variable_declarator */ true,
                    ctx,
                );
                if !found_inside {
                    return None;
                }

                // See if this identifier is used in JSX. Then it's a component.
                // TODO:
                // https://github.com/facebook/react/blob/ba6a9e94edf0db3ad96432804f9931ce9dc89fec/packages/react-refresh/src/ReactFreshBabelPlugin.js#L161-L199
            }
            _ => {
                return None;
            }
        }

        Some(self.create_assignment_expression(id, ctx))
    }

    // --------------------------- refresh sig ---------------------------

    /// Convert arrow function expression to normal arrow function
    ///
    /// ```js
    /// () => 1
    /// ```
    /// to
    /// ```js
    /// () => { return 1 }
    /// ```
    fn transform_arrow_function_to_block(
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !arrow.expression {
            return;
        }

        arrow.expression = false;

        let Some(Statement::ExpressionStatement(statement)) = arrow.body.statements.pop() else {
            unreachable!("arrow function body is never empty")
        };

        arrow
            .body
            .statements
            .push(ctx.ast.statement_return(SPAN, Some(statement.unbox().expression)));
    }
}

// Transform
impl<'a> ReactRefresh<'a> {
    /// Mutate statements and insert new assignment statements;
    pub fn transform_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut new_statements = ctx.ast.vec_with_capacity(program.body.len());
        for mut statement in program.body.drain(..) {
            let next_statement = self.process_statement(&mut statement, ctx);
            new_statements.push(statement);
            if let Some(assignment_expression) = next_statement {
                new_statements.push(assignment_expression);
            }
        }
        program.body = new_statements;
    }

    /// Insert all registrations at the end of the program.
    ///
    /// ```
    /// _c1 = refresh_reg(Foo, ...);
    /// _c2 = refresh_reg(Foo, ...);
    /// ```
    pub fn transform_program_on_exit(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.registrations.is_empty() {
            return;
        }

        let mut variable_declarator_items = ctx.ast.vec_with_capacity(self.registrations.len());
        let mut new_statements = ctx.ast.vec_with_capacity(self.registrations.len() + 1);
        for (symbol_id, persistent_id) in self.registrations.drain(..) {
            let name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));
            let binding_identifier = BindingIdentifier {
                name: name.clone(),
                symbol_id: Cell::new(Some(symbol_id)),
                span: SPAN,
            };

            variable_declarator_items.push(
                ctx.ast.variable_declarator(
                    SPAN,
                    VariableDeclarationKind::Var,
                    ctx.ast.binding_pattern(
                        ctx.ast.binding_pattern_kind_from_binding_identifier(
                            binding_identifier.clone(),
                        ),
                        None::<TSTypeAnnotation<'a>>,
                        false,
                    ),
                    None,
                    false,
                ),
            );

            let refresh_reg_ident = ctx.create_reference_id(
                SPAN,
                self.refresh_reg.clone(),
                Some(symbol_id),
                ReferenceFlags::Read,
            );
            let callee = ctx.ast.expression_from_identifier_reference(refresh_reg_ident);
            let mut arguments = ctx.ast.vec_with_capacity(2);
            arguments.push(ctx.ast.argument_expression(
                Self::create_identifier_reference_from_binding_identifier(&binding_identifier, ctx),
            ));
            arguments.push(ctx.ast.argument_expression(
                ctx.ast.expression_string_literal(SPAN, self.ctx.ast.atom(&persistent_id)),
            ));
            new_statements.push(ctx.ast.statement_expression(
                SPAN,
                ctx.ast.expression_call(
                    SPAN,
                    callee,
                    Option::<TSTypeParameterInstantiation>::None,
                    arguments,
                    false,
                ),
            ));
        }
        program.body.push(Statement::from(ctx.ast.declaration_variable(
            SPAN,
            VariableDeclarationKind::Var,
            variable_declarator_items,
            false,
        )));
        program.body.extend(new_statements);
    }

    pub fn transform_statements(
        &mut self,
        _stmts: &mut oxc_allocator::Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.signature_declarator_items.push(ctx.ast.vec());
    }

    pub fn transform_statements_on_exit(
        &mut self,
        stmts: &mut oxc_allocator::Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: check is there any function declaration

        let mut new_stmts = ctx.ast.vec_with_capacity(stmts.len() + 1);

        let declarations = self.signature_declarator_items.pop().unwrap();
        if !declarations.is_empty() {
            new_stmts.push(Statement::from(ctx.ast.declaration_variable(
                SPAN,
                VariableDeclarationKind::Var,
                declarations,
                false,
            )));
        }
        new_stmts.extend(stmts.drain(..).flat_map(move |stmt| {
            let symbol_ids = get_symbol_id_from_function_and_declarator(&stmt);
            let extra_stmts = symbol_ids
                .into_iter()
                .filter_map(|symbol_id| self.extra_statements.remove(&symbol_id))
                .flatten()
                .collect::<Vec<_>>();
            once(stmt).chain(extra_stmts)
        }));

        *stmts = new_stmts;
    }

    /// Transform an expression to insert a signature call,
    /// and wrap it with a signature call
    ///
    /// ```js
    /// Foo(() => {})
    /// let Foo = React.forwardRef(React.memo(() => {}));
    /// ```
    /// to
    /// ```js
    /// Foo(_s1(() => {}))
    /// let Foo = s1(React.forwardRef(_s1(React.memo(_s1(() => {_s1()}, ...), ...))), ...);
    /// ```
    pub fn transform_expression_on_exit(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let signature = match expr {
            Expression::FunctionExpression(func) => self.create_signature_call_expression(
                func.scope_id.get().unwrap(),
                func.body.as_mut().unwrap(),
                ctx,
            ),
            Expression::ArrowFunctionExpression(arrow) => {
                let call_fn = self.create_signature_call_expression(
                    arrow.scope_id.get().unwrap(),
                    &mut arrow.body,
                    ctx,
                );

                // If the signature is found, we will push a new statement to the arrow function body. So it's not an expression anymore.
                if call_fn.is_some() {
                    Self::transform_arrow_function_to_block(arrow, ctx);
                }
                call_fn
            }
            // hoc(_c = function() { })
            Expression::AssignmentExpression(_) => return,
            // hoc1(hoc2(...))
            // Result: let Foo = __signature(hoc(__signature(() => {}, ...)), ...)
            Expression::CallExpression(_) => self.last_signature.take(),
            _ => None,
        };

        let Some((binding_identifier, mut arguments)) = signature else {
            return;
        };

        if !matches!(expr, Expression::CallExpression(_)) {
            if let Ancestor::VariableDeclaratorInit(declarator) = ctx.parent() {
                // Special case when a function would get an inferred name:
                // let Foo = () => {}
                // let Foo = function() {}
                // We'll add signature it on next line so that
                // we don't mess up the inferred 'Foo' function name.

                // Result: let Foo = () => {}; __signature(Foo, ...);
                let id = declarator.id().get_binding_identifier().unwrap();
                let symbol_id = id.symbol_id.get().unwrap();
                let first_argument = Argument::from(ctx.ast.expression_from_identifier_reference(
                    ctx.create_reference_id(
                        SPAN,
                        id.name.clone(),
                        Some(symbol_id),
                        ReferenceFlags::Read,
                    ),
                ));
                arguments.insert(0, first_argument);

                let statement = ctx.ast.statement_expression(
                    SPAN,
                    ctx.ast.expression_call(
                        SPAN,
                        Self::create_identifier_reference_from_binding_identifier(
                            &binding_identifier,
                            ctx,
                        ),
                        Option::<TSTypeParameterInstantiation>::None,
                        arguments,
                        false,
                    ),
                );
                self.extra_statements.entry(symbol_id).or_insert(ctx.ast.vec()).push(statement);
                return;
            }
        }

        let mut found_call_expression = false;
        for ancestor in ctx.ancestors() {
            if ancestor.is_assignment_expression() {
                continue;
            }
            if ancestor.is_call_expression() {
                found_call_expression = true;
            }
            break;
        }

        if found_call_expression {
            self.last_signature =
                Some((binding_identifier.clone(), arguments.clone_in(ctx.ast.allocator)));
        }

        arguments.insert(0, Argument::from(ctx.ast.move_expression(expr)));
        *expr = self.ctx.ast.expression_call(
            SPAN,
            Self::create_identifier_reference_from_binding_identifier(&binding_identifier, ctx),
            Option::<TSTypeParameterInstantiation>::None,
            arguments,
            false,
        );
    }

    /// Modify a function to insert a signature call,
    /// and return a statement to insert it after current statement
    ///
    /// ```js
    /// function Foo() {};
    /// ```
    /// to
    /// ```js
    /// function Foo() { _s() }; _s(Foo, ...);
    /// ```
    pub fn transform_function_on_exit(
        &mut self,
        func: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !func.is_function_declaration() {
            return;
        }

        let Some((binding_identifier, mut arguments)) = self.create_signature_call_expression(
            func.scope_id.get().unwrap(),
            func.body.as_mut().unwrap(),
            ctx,
        ) else {
            return;
        };

        let Some(id) = func.id.as_ref() else {
            return;
        };

        arguments.insert(
            0,
            Argument::from(Self::create_identifier_reference_from_binding_identifier(id, ctx)),
        );

        self.extra_statements.entry(id.symbol_id.get().unwrap()).or_insert(ctx.ast.vec()).push(
            ctx.ast.statement_expression(
                SPAN,
                ctx.ast.expression_call(
                    SPAN,
                    Self::create_identifier_reference_from_binding_identifier(
                        &binding_identifier,
                        ctx,
                    ),
                    Option::<TSTypeParameterInstantiation>::None,
                    arguments,
                    false,
                ),
            ),
        );
    }
}

// TODO: Try to remove this struct, avoid double visit
struct CalculateSignatureKey<'a, 'b> {
    key: String,
    source_text: &'a str,
    ctx: &'b mut TraverseCtx<'a>,
    callee_list: Vec<(Atom<'a>, Option<Atom<'a>>)>,
    scope_ids: Vec<ScopeId>,
    declarator_id_span: Option<Span>,
}

impl<'a, 'b> CalculateSignatureKey<'a, 'b> {
    pub fn new(source_text: &'a str, scope_id: ScopeId, ctx: &'b mut TraverseCtx<'a>) -> Self {
        Self {
            key: String::new(),
            ctx,
            source_text,
            scope_ids: vec![scope_id],
            declarator_id_span: None,
            callee_list: Vec::new(),
        }
    }

    fn current_scope_id(&self) -> ScopeId {
        *self.scope_ids.last().unwrap()
    }

    pub fn calculate(
        mut self,
        body: &FunctionBody<'a>,
    ) -> Option<oxc_allocator::Vec<'a, Argument<'a>>> {
        for statement in &body.statements {
            self.visit_statement(statement);
        }

        if self.key.is_empty() {
            return None;
        }

        // Check if a corresponding binding exists where we emit the signature.
        let mut force_reset = false;
        let mut custom_hooks_in_scope = self.ctx.ast.vec_with_capacity(self.callee_list.len());

        for (binding_name, hook_name) in &self.callee_list {
            if let Some(symbol_id) =
                self.ctx.scopes().find_binding(self.ctx.current_scope_id(), binding_name)
            {
                let ident = self.ctx.create_reference_id(
                    SPAN,
                    binding_name.clone(),
                    Some(symbol_id),
                    ReferenceFlags::Read,
                );

                let mut expr = self.ctx.ast.expression_from_identifier_reference(ident);

                if let Some(hook_name) = hook_name {
                    // binding_name.hook_name
                    expr = Expression::from(self.ctx.ast.member_expression_static(
                        SPAN,
                        expr,
                        self.ctx.ast.identifier_name(SPAN, hook_name),
                        false,
                    ));
                }

                custom_hooks_in_scope.push(self.ctx.ast.array_expression_element_expression(expr));
            } else {
                force_reset = true;
            }
        }

        let mut arguments = self.ctx.ast.vec_with_capacity(
            1 + usize::from(force_reset) + usize::from(!custom_hooks_in_scope.is_empty()),
        );
        arguments.push(self.ctx.ast.argument_expression(
            self.ctx.ast.expression_string_literal(SPAN, self.ctx.ast.atom(&self.key)),
        ));

        if force_reset || !custom_hooks_in_scope.is_empty() {
            arguments.push(
                self.ctx.ast.argument_expression(
                    self.ctx.ast.expression_boolean_literal(SPAN, force_reset),
                ),
            );
        }

        if !custom_hooks_in_scope.is_empty() {
            // function () { return custom_hooks_in_scope }
            let formal_parameters = self.ctx.ast.formal_parameters(
                SPAN,
                FormalParameterKind::FormalParameter,
                self.ctx.ast.vec(),
                Option::<BindingRestElement>::None,
            );
            let function_body = self.ctx.ast.function_body(
                SPAN,
                self.ctx.ast.vec(),
                self.ctx.ast.vec1(self.ctx.ast.statement_return(
                    SPAN,
                    Some(self.ctx.ast.expression_array(SPAN, custom_hooks_in_scope, None)),
                )),
            );
            let fn_expr = self.ctx.ast.expression_function(
                FunctionType::FunctionExpression,
                SPAN,
                None,
                false,
                false,
                false,
                Option::<TSTypeParameterDeclaration>::None,
                Option::<TSThisParameter>::None,
                formal_parameters,
                Option::<TSTypeAnnotation>::None,
                Some(function_body),
            );
            arguments.push(self.ctx.ast.argument_expression(fn_expr));
        }

        Some(arguments)
    }
}

impl<'a, 'b> Visit<'a> for CalculateSignatureKey<'a, 'b> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<oxc_semantic::ScopeId>>) {
        self.scope_ids.push(scope_id.get().unwrap());
    }

    fn leave_scope(&mut self) {
        self.scope_ids.pop();
    }

    fn visit_statements(&mut self, _stmt: &oxc_allocator::Vec<'a, Statement<'a>>) {
        // We don't need calculate any signature in nested scopes
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if matches!(declarator.init, Some(Expression::CallExpression(_))) {
            self.declarator_id_span = Some(declarator.id.span());
        }
        walk_variable_declarator(self, declarator);
        // We doesn't check the call expression is the hook,
        // So we need to reset the declarator_id_span after visiting the variable declarator.
        self.declarator_id_span = None;
    }

    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if !self.ctx.scopes().get_flags(self.current_scope_id()).is_function() {
            return;
        }

        let name = match &call_expr.callee {
            Expression::Identifier(ident) => Some(ident.name.clone()),
            Expression::StaticMemberExpression(ref member) => Some(member.property.name.clone()),
            _ => None,
        };

        let Some(hook_name) = name else {
            return;
        };

        if !is_use_hook_name(&hook_name) {
            return;
        }

        if !is_builtin_hook(&hook_name) {
            let callee = match &call_expr.callee {
                Expression::Identifier(ident) => Some((ident.name.clone(), None)),
                callee @ match_member_expression!(Expression) => {
                    let member_expr = callee.to_member_expression();
                    match member_expr.object() {
                        Expression::Identifier(ident) => {
                            Some((ident.name.clone(), Some(hook_name.clone())))
                        }
                        _ => None,
                    }
                }
                _ => None,
            };

            if let Some(callee) = callee {
                self.callee_list.push(callee);
            }
        }

        let args = &call_expr.arguments;
        let args_key = if hook_name == "useState" && args.len() > 0 {
            args[0].span().source_text(self.source_text)
        } else if hook_name == "useReducer" && args.len() > 1 {
            args[1].span().source_text(self.source_text)
        } else {
            ""
        };

        if !self.key.is_empty() {
            self.key.push_str("\\n");
        }
        self.key.push_str(&format!(
            "{hook_name}{{{}{}{args_key}{}}}",
            self.declarator_id_span.take().map_or("", |span| span.source_text(self.source_text)),
            if args_key.is_empty() { "" } else { "(" },
            if args_key.is_empty() { "" } else { ")" }
        ));
    }
}

fn is_componentish_name(name: &str) -> bool {
    name.chars().next().unwrap().is_ascii_uppercase()
}

fn is_use_hook_name(name: &str) -> bool {
    name.starts_with("use") && name.chars().nth(3).unwrap().is_ascii_uppercase()
}

#[rustfmt::skip]
fn is_builtin_hook(hook_name: &str) -> bool {
    matches!(
        hook_name,
        "useState" | "useReducer" | "useEffect" |
        "useLayoutEffect" | "useMemo" | "useCallback" |
        "useRef" | "useContext" | "useImperativeHandle" |
        "useDebugValue" | "useId" | "useDeferredValue" |
        "useTransition" | "useInsertionEffect" | "useSyncExternalStore" |
        "useFormStatus" | "useFormState" | "useActionState" |
        "useOptimistic"
    )
}

fn get_symbol_id_from_function_and_declarator(stmt: &Statement<'_>) -> Vec<SymbolId> {
    let mut symbol_ids = vec![];
    match stmt {
        Statement::FunctionDeclaration(ref func) => {
            symbol_ids.push(func.id.as_ref().unwrap().symbol_id.get().unwrap());
        }
        Statement::VariableDeclaration(ref decl) => {
            symbol_ids.extend(decl.declarations.iter().filter_map(|decl| {
                decl.id.get_binding_identifier().and_then(|id| id.symbol_id.get())
            }));
        }
        Statement::ExportNamedDeclaration(ref export_decl) => {
            if let Some(Declaration::FunctionDeclaration(func)) = &export_decl.declaration {
                symbol_ids.push(func.id.as_ref().unwrap().symbol_id.get().unwrap());
            } else if let Some(Declaration::VariableDeclaration(decl)) = &export_decl.declaration {
                symbol_ids.extend(decl.declarations.iter().filter_map(|decl| {
                    decl.id.get_binding_identifier().and_then(|id| id.symbol_id.get())
                }));
            }
        }
        Statement::ExportDefaultDeclaration(ref export_decl) => {
            if let ExportDefaultDeclarationKind::FunctionDeclaration(func) =
                &export_decl.declaration
            {
                if let Some(id) = func.id.as_ref() {
                    symbol_ids.push(id.symbol_id.get().unwrap());
                }
            }
        }
        _ => {}
    };

    symbol_ids
}
