//! Element and class decorator collection.

use std::mem;

use rustc_hash::FxHashMap;

use oxc_allocator::{Address, ArenaBox, ArenaVec, CloneIn, GetAddress, GetAllocator, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::{
    number::NumberBase,
    reference::ReferenceFlags,
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::{
        computed_key::{create_computed_key_temp_var, key_needs_temp_var},
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    state::TransformState,
    utils::ast_builder::create_assignment,
};

use super::{
    ClassDecoratorState, ClassDecoratorTables, DecoratorElementKind, PublicMethodKey,
    StandardDecoratorState,
};

impl<'a> StandardDecoratorState<'a> {
    /// Give an unbraced loop body a scope for per-evaluation decorator state.
    ///
    /// Runs on every statement, so the cheap checks are inlined into the caller.
    #[inline]
    pub(crate) fn wrap_decorated_loop_body(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.enabled {
            return;
        }
        Self::scope_decorated_loop_header(statement, ctx);
        if !matches!(statement, Statement::BlockStatement(_))
            && matches!(
                ctx.parent(),
                Ancestor::ForStatementBody(_)
                    | Ancestor::ForInStatementBody(_)
                    | Ancestor::ForOfStatementBody(_)
                    | Ancestor::WhileStatementBody(_)
                    | Ancestor::DoWhileStatementBody(_)
            )
        {
            self.wrap_loop_body_if_decorated(statement, ctx);
        }
    }

    #[inline(never)]
    fn wrap_loop_body_if_decorated(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        struct DecoratedClassFinder<'s, 'a> {
            found: bool,
            loop_classes: &'s mut FxHashMap<Address, bool>,
            state: &'s TransformState<'a>,
        }
        impl<'a> Visit<'a> for DecoratedClassFinder<'_, 'a> {
            fn visit_statement(&mut self, statement: &Statement<'a>) {
                let loop_address = matches!(
                    statement,
                    Statement::ForStatement(_)
                        | Statement::ForInStatement(_)
                        | Statement::ForOfStatement(_)
                        | Statement::WhileStatement(_)
                        | Statement::DoWhileStatement(_)
                )
                .then(|| statement.address());
                if let Some(address) = loop_address
                    && let Some(&found) = self.loop_classes.get(&address)
                {
                    self.found |= found;
                    return;
                }

                let found_before = mem::take(&mut self.found);
                oxc_ast_visit::walk::walk_statement(self, statement);
                if let Some(address) = loop_address {
                    // Reuse both positive and negative results for nested unbraced loops.
                    self.loop_classes.insert(address, self.found);
                }
                self.found |= found_before;
            }

            fn visit_class(&mut self, class: &Class<'a>) {
                self.found |=
                    self.state.class_lowering_before_entry(class).handles_standard_decorators();
                if !self.found {
                    oxc_ast_visit::walk::walk_class(self, class);
                }
            }
        }
        let found = {
            let mut finder = DecoratedClassFinder {
                found: false,
                loop_classes: &mut self.loop_classes,
                state: &ctx.state,
            };
            finder.visit_statement(statement);
            finder.found
        };
        if found {
            let scope_id = ctx.insert_scope_below_statement(statement, ScopeFlags::empty());
            let body = statement.take_in(ctx);
            *statement = Statement::new_block_statement_with_scope_id(SPAN, [body], scope_id, ctx);
        }
    }

    /// Move repeated header evaluations into a fresh block without introducing a function.
    /// This keeps await/yield and lexical this/arguments/super in their original context.
    fn scope_decorated_loop_header(statement: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let needs_first = match Self::loop_statement(statement) {
            Statement::WhileStatement(loop_stmt) => {
                if !Self::header_has_decorated_class(&loop_stmt.test, ctx) {
                    return;
                }
                false
            }
            Statement::ForStatement(loop_stmt) => {
                let move_update = loop_stmt
                    .update
                    .as_ref()
                    .is_some_and(|update| Self::header_has_decorated_class(update, ctx));
                if !move_update
                    && !loop_stmt
                        .test
                        .as_ref()
                        .is_some_and(|test| Self::header_has_decorated_class(test, ctx))
                {
                    return;
                }
                move_update
            }
            Statement::DoWhileStatement(loop_stmt) => {
                if !Self::header_has_decorated_class(&loop_stmt.test, ctx) {
                    return;
                }
                true
            }
            _ => return,
        };
        // Wrap the labels along with the loop so labelled continue still targets the loop.
        let first = needs_first.then(|| {
            let scope_id = ctx.insert_scope_below_statement(statement, ScopeFlags::empty());
            let binding = ctx.generate_uid("first", scope_id, SymbolFlags::BlockScopedVariable);
            (binding, scope_id)
        });
        match Self::loop_statement(statement) {
            Statement::WhileStatement(loop_stmt) => {
                let test = mem::replace(
                    &mut loop_stmt.test,
                    Expression::new_boolean_literal(SPAN, true, ctx),
                );
                let body = loop_stmt.body.take_in(ctx);
                loop_stmt.body = Self::guarded_loop_body(test, body, ctx.current_scope_id(), ctx);
            }
            Statement::ForStatement(loop_stmt) => {
                let mut test = loop_stmt
                    .test
                    .take()
                    .unwrap_or_else(|| Expression::new_boolean_literal(SPAN, true, ctx));
                if let Some((first, _)) = &first {
                    let update = loop_stmt.update.take().unwrap();
                    let next = Self::skip_first_evaluation(first, update, ctx);
                    test = Expression::new_sequence_expression(SPAN, [next, test], ctx);
                }
                let body = loop_stmt.body.take_in(ctx);
                loop_stmt.body = Self::guarded_loop_body(test, body, loop_stmt.scope_id(), ctx);
            }
            Statement::DoWhileStatement(loop_stmt) => {
                let (first, scope_id) = first.as_ref().unwrap();
                let test = mem::replace(
                    &mut loop_stmt.test,
                    Expression::new_boolean_literal(SPAN, true, ctx),
                );
                let test = Self::skip_first_evaluation(first, test, ctx);
                let body = loop_stmt.body.take_in(ctx);
                loop_stmt.body = Self::guarded_loop_body(test, body, *scope_id, ctx);
            }
            _ => unreachable!(),
        }
        if let Some((first, scope_id)) = first {
            let declaration =
                Self::create_let(&first, Expression::new_boolean_literal(SPAN, true, ctx), ctx);
            let loop_stmt = statement.take_in(ctx);
            *statement = Statement::new_block_statement_with_scope_id(
                SPAN,
                [declaration, loop_stmt],
                scope_id,
                ctx,
            );
        }
    }

    fn loop_statement<'s>(mut statement: &'s mut Statement<'a>) -> &'s mut Statement<'a> {
        while let Statement::LabeledStatement(label) = statement {
            statement = &mut label.body;
        }
        statement
    }

    fn header_has_decorated_class(expression: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        struct Finder<'s, 'a> {
            found: bool,
            state: &'s TransformState<'a>,
        }
        impl<'a> Visit<'a> for Finder<'_, 'a> {
            fn visit_class(&mut self, class: &Class<'a>) {
                self.found |= self.state.class_lowering_before_entry(class).has_decorators();
                if !self.found {
                    oxc_ast_visit::walk::walk_class(self, class);
                }
            }
            fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
            fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
        }
        let mut finder = Finder { found: false, state: &ctx.state };
        finder.visit_expression(expression);
        finder.found
    }

    /// Retain the original body as a nested statement so its lexicals cannot capture the header.
    fn guarded_loop_body(
        test: Expression<'a>,
        body: Statement<'a>,
        parent_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let guard = Statement::new_if_statement(
            SPAN,
            test,
            body,
            Some(Statement::new_break_statement(SPAN, None, ctx)),
            ctx,
        );
        let scope_id = ctx.insert_scope_below_statement_from_scope_id(
            &guard,
            parent_scope_id,
            ScopeFlags::empty(),
        );
        Statement::new_block_statement_with_scope_id(SPAN, [guard], scope_id, ctx)
    }

    /// Skip the first update (or the first do-while test), then evaluate it on every continue.
    fn skip_first_evaluation(
        first: &BoundIdentifier<'a>,
        next: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let clear =
            create_assignment(first, Expression::new_boolean_literal(SPAN, false, ctx), SPAN, ctx);
        let initial = Expression::new_sequence_expression(
            SPAN,
            [clear, Expression::new_boolean_literal(SPAN, true, ctx)],
            ctx,
        );
        Expression::new_conditional_expression(
            SPAN,
            first.create_read_expression(ctx),
            initial,
            next,
            ctx,
        )
    }

    /// Decorator context names use the property key after a single ToPropertyKey conversion.
    ///
    /// Entry phase: convert keys of decorated elements, and computed keys of fields, with
    /// `ToPropertyKey` before the class body is lowered.
    pub(crate) fn coerce_decorated_keys(body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        for element in &mut body.body {
            let (key, computed, decorated) = match element {
                // Lowered fields are defined later, from the constructor or after the class.
                // Convert their keys once, in element order, as class definition evaluation does.
                ClassElement::PropertyDefinition(prop) => {
                    let prop = &mut **prop;
                    let decorated = !prop.decorators.is_empty();
                    if !decorated
                        && prop.computed
                        && let Some(key) = prop.key.as_expression_mut()
                        && !matches!(
                            key,
                            Expression::StringLiteral(_)
                                | Expression::NumericLiteral(_)
                                | Expression::BigIntLiteral(_)
                        )
                    {
                        let value = key.take_in(ctx);
                        *key = helper_call_expr(
                            Helper::ToPropertyKey,
                            ArenaVec::from_value_in(Argument::from(value), ctx),
                            ctx,
                        );
                        continue;
                    }
                    (&mut prop.key, &mut prop.computed, decorated)
                }
                ClassElement::MethodDefinition(method) => {
                    let method = &mut **method;
                    (&mut method.key, &mut method.computed, !method.decorators.is_empty())
                }
                ClassElement::AccessorProperty(prop) => {
                    let prop = &mut **prop;
                    (&mut prop.key, &mut prop.computed, !prop.decorators.is_empty())
                }
                _ => continue,
            };
            if !decorated {
                continue;
            }
            if let Some(expression) = key.as_expression_mut() {
                if matches!(expression, Expression::StringLiteral(_)) {
                    continue;
                }
                let value = expression.take_in(ctx);
                *expression = helper_call_expr(
                    Helper::ToPropertyKey,
                    ArenaVec::from_value_in(Argument::from(value), ctx),
                    ctx,
                );
                *computed = true;
            }
        }
    }

    /// Entry phase: assign initializer slot indexes to decorated fields and accessors.
    ///
    /// Order matches the order `__decorateElement` is called in:
    /// static accessors, instance accessors, static fields, instance fields.
    pub(crate) fn prepare_class_decorators(
        body: &ClassBody<'a>,
        mut tables: ClassDecoratorTables<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ClassDecoratorState<'a> {
        let mut groups: [Vec<Address>; 4] = Default::default();
        let mut instance_method_extras = false;
        let mut static_method_extras = false;
        for element in &body.body {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    let address = prop.address();
                    if tables.accessor_storage.contains(&address) {
                        groups[DecoratorElementKind::Accessor.bucket(prop.r#static) as usize]
                            .push(address);
                    } else if !prop.decorators.is_empty() && !prop.declare {
                        groups[DecoratorElementKind::Field.bucket(prop.r#static) as usize]
                            .push(address);
                    }
                }
                ClassElement::MethodDefinition(method)
                    if !method.decorators.is_empty()
                        && !method.kind.is_constructor()
                        && !tables.accessor_getters.contains(&method.address()) =>
                {
                    if method.r#static {
                        static_method_extras = true;
                    } else {
                        instance_method_extras = true;
                    }
                }
                _ => {}
            }
        }

        // A computed name can collide with any other public method on the same object.
        // Capture all public methods on that object, including later undecorated overrides.
        let mut names = [FxHashMap::default(), FxHashMap::default()];
        let mut counts = [0, 0];
        let mut unknown = [false, false];
        let mut collisions = [false, false];
        let mut decorated = [false, false];
        let mut accessor_setter = false;
        for element in &body.body {
            let ClassElement::MethodDefinition(method) = element else { continue };
            if accessor_setter {
                accessor_setter = false;
                continue;
            }
            accessor_setter = tables.accessor_getters.contains(&method.address());
            if method.kind.is_constructor() || method.key.is_private_identifier() {
                continue;
            }
            let side = usize::from(method.r#static);
            counts[side] += 1;
            decorated[side] |= !method.decorators.is_empty();
            let kind = tables.method_decorator_kind(method);
            let parts = match kind {
                DecoratorElementKind::Method => 1,
                DecoratorElementKind::Getter => 2,
                DecoratorElementKind::Setter => 4,
                DecoratorElementKind::Accessor => 6,
                DecoratorElementKind::Field => unreachable!(),
            };
            if let Some(name) = method.key.static_name() {
                let previous = names[side].entry(name).or_insert(0);
                // A single getter/setter pair can use the native combined descriptor.
                collisions[side] |=
                    *previous != 0 && (parts == 1 || *previous == 1 || parts & *previous != 0);
                *previous |= parts;
            } else {
                unknown[side] = true;
            }
        }
        let capture = [0, 1].map(|side| {
            decorated[side] && (collisions[side] || (unknown[side] && counts[side] > 1))
        });
        let mut accessor_key = None;
        for element in &body.body {
            let ClassElement::MethodDefinition(method) = element else { continue };
            if method.kind.is_constructor()
                || method.key.is_private_identifier()
                || !capture[usize::from(method.r#static)]
            {
                continue;
            }
            let accessor_setter = accessor_key.is_some();
            let binding = accessor_key.take().unwrap_or_else(|| {
                let binding = ctx.generate_uid(
                    "key",
                    ctx.current_block_scope_id(),
                    SymbolFlags::BlockScopedVariable,
                );
                ctx.state.var_declarations.insert_let(&binding, None, &ctx.ast);
                binding
            });
            if tables.accessor_getters.contains(&method.address()) {
                accessor_key = Some(binding.clone());
            }
            tables
                .public_keys
                .insert(method.address(), PublicMethodKey { binding, accessor_setter });
        }

        let mut index = 0;
        for group in groups {
            for address in group {
                tables.fields.insert(address, index);
                index += 1;
            }
        }

        let init = ctx.generate_uid(
            "init",
            ctx.current_block_scope_id(),
            SymbolFlags::BlockScopedVariable,
        );
        ctx.state.var_declarations.insert_let(&init, None, &ctx.ast);
        ClassDecoratorState { init, instance_method_extras, static_method_extras, tables }
    }

    /// Exit phase: evaluate class decorators into `_Class_decorators = [...]` before class,
    /// and capture super class so its metadata can be inherited.
    pub(crate) fn take_class_decorators(
        &mut self,
        class: &mut Class<'a>,
        insert_before: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !class.decorators.is_empty() {
            let name = class.id.as_ref().map_or("Class", |id| id.name.as_str());
            let binding = ctx.generate_uid_in_current_hoist_scope(&format!("{name}_decorators"));
            ctx.state.var_declarations.insert_var(&binding, &ctx.ast);
            let array = Self::decorator_array(&mut class.decorators, ctx);
            insert_before.push(create_assignment(&binding, array, SPAN, ctx));
            self.exit.class_decorators = Some(binding);
        }
        self.exit.class_prefix_len = insert_before.len();

        if let Some(heritage) = &mut class.heritage {
            let binding = ctx.generate_uid_in_current_hoist_scope("superclass");
            ctx.state.var_declarations.insert_var(&binding, &ctx.ast);
            let expr = heritage.expression.take_in(ctx);
            heritage.expression = create_assignment(&binding, expr, SPAN, ctx);
            self.exit.super_class = Some(binding);
        }
    }

    /// Exit phase: element decorators and computed keys are evaluated after `extends` clause.
    /// If class has `extends`, move them into it: `extends (_superclass = Base, _x_dec = [dec], _superclass)`.
    /// This also evaluates them in the class scope, as the spec requires.
    pub(crate) fn fold_element_decorators_into_heritage(
        &mut self,
        class: &mut Class<'a>,
        insert_before: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let start = mem::take(&mut self.exit.class_prefix_len);
        let (Some(heritage), Some(binding)) = (&mut class.heritage, &self.exit.super_class) else {
            return;
        };
        if insert_before.len() <= start {
            return;
        }
        let mut exprs = ArenaVec::with_capacity_in(insert_before.len() - start + 2, ctx);
        exprs.push(heritage.expression.take_in(ctx));
        exprs.extend(insert_before.drain(start..));
        exprs.push(binding.create_read_expression(ctx));
        heritage.expression = Expression::new_sequence_expression(SPAN, exprs, ctx);
    }

    /// Exit phase: evaluate element decorators into `_x_dec = [...]` before class.
    pub(crate) fn take_element_decorators(
        decorators: &mut ArenaVec<'a, Decorator<'a>>,
        key: &PropertyKey<'a>,
        insert_before: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let hint = match key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
            _ => "member",
        };
        let binding = ctx.generate_uid_in_current_hoist_scope(&format!("{hint}_dec"));
        ctx.state.var_declarations.insert_var(&binding, &ctx.ast);
        let array = Self::decorator_array(decorators, ctx);
        insert_before.push(create_assignment(&binding, array, SPAN, ctx));
        binding
    }

    /// Decorators as `[receiver, function]` pairs, which the runtime expects.
    fn decorator_array(
        decorators: &mut ArenaVec<'a, Decorator<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut elements = ArenaVec::with_capacity_in(decorators.len(), ctx);
        for decorator in decorators.drain(..) {
            let receiver = Expression::new_void_0(SPAN, ctx);
            let pair = Expression::new_array_expression(
                SPAN,
                [
                    ArrayExpressionElement::from(receiver),
                    ArrayExpressionElement::from(decorator.expression),
                ],
                ctx,
            );
            elements.push(ArrayExpressionElement::from(pair));
        }
        Expression::new_array_expression(SPAN, elements, ctx)
    }

    /// Give a public method a temporary symbol key so later same-name methods cannot overwrite
    /// its descriptor before decoration. Keep it on the class to preserve `super`'s home object.
    pub(crate) fn capture_public_method(
        &mut self,
        method: &mut MethodDefinition<'a>,
        kind: DecoratorElementKind,
        key: Expression<'a>,
        binding: &BoundIdentifier<'a>,
        insert_before: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> usize {
        let symbol = helper_call_expr(Helper::DecoratorKey, ArenaVec::new_in(ctx), ctx);
        insert_before.push(create_assignment(binding, symbol, SPAN, ctx));
        if let PropertyKey::Identifier(key) = &method.key {
            ctx.delete_reference_for_identifier(key);
        }
        method.key = PropertyKey::from(binding.create_read_expression(ctx));
        method.computed = true;
        let flags = kind as u32 | if method.r#static { super::FLAG_STATIC } else { 0 };
        let record = Expression::new_array_expression(
            SPAN,
            [
                ArrayExpressionElement::new_numeric_literal(
                    SPAN,
                    f64::from(flags),
                    None,
                    NumberBase::Decimal,
                    ctx,
                ),
                ArrayExpressionElement::from(key),
                ArrayExpressionElement::from(binding.create_read_expression(ctx)),
            ],
            ctx,
        );
        let index = self.exit.public_descriptors.len();
        self.exit.public_descriptors.push(record);
        index
    }

    /// Memoize a static decorated field's computed key so it's evaluated once.
    pub(crate) fn memoize_decorated_key(
        key: &mut PropertyKey<'a>,
        insert_before: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(expr) = key.as_expression_mut()
            && key_needs_temp_var(expr, ctx)
        {
            let (assignment, temp) = create_computed_key_temp_var(expr.take_in(ctx), ctx);
            insert_before.push(assignment);
            *key = PropertyKey::from(temp);
        }
    }

    /// Expression for element name passed to `__decorateElement`.
    pub(crate) fn decorator_key_expr(
        key: &PropertyKey<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match key {
            PropertyKey::PrivateIdentifier(ident) => {
                let name = Str::from_strs_array_in(["#", &ident.name], ctx);
                Expression::new_string_literal(SPAN, name, None, ctx)
            }
            PropertyKey::StaticIdentifier(ident) => {
                Expression::new_string_literal(SPAN, ident.name, None, ctx)
            }
            PropertyKey::Identifier(ident) => {
                let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id();
                ctx.create_ident_expr(SPAN, ident.name, symbol_id, ReferenceFlags::Read)
            }
            PropertyKey::AssignmentExpression(assign) => {
                let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left else {
                    unreachable!("memoized computed key is always assignment to temp var");
                };
                let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id();
                ctx.create_ident_expr(SPAN, ident.name, symbol_id, ReferenceFlags::Read)
            }
            PropertyKey::StringLiteral(lit) => {
                Expression::StringLiteral(lit.clone_in(ctx.ast.allocator()))
            }
            PropertyKey::NumericLiteral(lit) => {
                Expression::NumericLiteral(lit.clone_in(ctx.ast.allocator()))
            }
            key => key.to_expression().clone_in(ctx.ast.allocator()),
        }
    }
}

impl<'a> ClassDecoratorTables<'a> {
    pub(crate) fn take_public_key(&mut self, method: Address) -> Option<PublicMethodKey<'a>> {
        self.public_keys.remove(&method)
    }

    /// Capture a decorated instance field's key before it is moved into the constructor.
    pub(crate) fn record_instance_field_for_decorators(
        &mut self,
        prop: &ArenaBox<'a, PropertyDefinition<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !prop.decorators.is_empty()
            && let Some(key) = prop.key.as_expression()
            && !key_needs_temp_var(key, ctx)
        {
            let key = StandardDecoratorState::decorator_key_expr(&prop.key, ctx);
            self.instance_keys.insert(prop.address(), key);
        }
    }

    /// Initializer slot index of a decorated field or accessor storage, if it has one.
    pub(crate) fn decorated_field_index(
        &self,
        prop: &ArenaBox<'a, PropertyDefinition<'a>>,
    ) -> Option<u32> {
        self.fields.get(&prop.address()).copied()
    }

    /// Key of a decorated instance field, if it was captured before the field was moved.
    pub(crate) fn take_instance_key(&mut self, property: Address) -> Option<Expression<'a>> {
        self.instance_keys.remove(&property)
    }

    pub(crate) fn method_decorator_kind(
        &self,
        method: &ArenaBox<'a, MethodDefinition<'a>>,
    ) -> DecoratorElementKind {
        if self.accessor_getters.contains(&method.address()) {
            return DecoratorElementKind::Accessor;
        }
        match method.kind {
            MethodDefinitionKind::Get => DecoratorElementKind::Getter,
            MethodDefinitionKind::Set => DecoratorElementKind::Setter,
            MethodDefinitionKind::Method | MethodDefinitionKind::Constructor => {
                DecoratorElementKind::Method
            }
        }
    }
}
