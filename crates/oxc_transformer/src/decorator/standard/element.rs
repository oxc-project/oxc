//! Element and class decorator collection.

use std::mem;

use rustc_hash::FxHashMap;

use oxc_allocator::{Address, ArenaBox, ArenaVec, CloneIn, GetAddress, GetAllocator, TakeIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::{reference::ReferenceFlags, scope::ScopeFlags, symbol::SymbolFlags};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse};

use crate::{
    common::{
        computed_key::{create_computed_key_temp_var, key_needs_temp_var},
        duplicate::duplicate_expression,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    es2020::OptionalChaining,
    utils::{ast_builder::create_assignment, decorated_class::class_lowered_for_decorators},
};

use super::{ClassDecoratorState, DecoratorElementKind, StandardDecoratorState};

impl<'a> StandardDecoratorState<'a> {
    pub(crate) fn take_instance_key(&mut self, property: Address) -> Option<Expression<'a>> {
        self.instance_keys.remove(&property)
    }

    /// Give an unbraced loop body a scope for per-evaluation decorator state.
    pub(crate) fn wrap_decorated_loop_body(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        struct DecoratedClassFinder<'s> {
            found: bool,
            loop_classes: &'s mut FxHashMap<Address, bool>,
        }
        impl<'a> Visit<'a> for DecoratedClassFinder<'_> {
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
                self.found |= class_lowered_for_decorators(class);
                if !self.found {
                    oxc_ast_visit::walk::walk_class(self, class);
                }
            }
        }
        use oxc_ast_visit::Visit;

        if !self.enabled
            || matches!(statement, Statement::BlockStatement(_))
            || !matches!(
                ctx.parent(),
                Ancestor::ForStatementBody(_)
                    | Ancestor::ForInStatementBody(_)
                    | Ancestor::ForOfStatementBody(_)
                    | Ancestor::WhileStatementBody(_)
                    | Ancestor::DoWhileStatementBody(_)
            )
        {
            return;
        }

        let mut finder =
            DecoratedClassFinder { found: false, loop_classes: &mut self.loop_classes };
        finder.visit_statement(statement);
        if finder.found {
            let scope_id = ctx.insert_scope_below_statement(statement, ScopeFlags::empty());
            let body = statement.take_in(ctx);
            *statement = Statement::new_block_statement_with_scope_id(SPAN, [body], scope_id, ctx);
        }
    }

    /// Preserve the receiver before private fields and `super` accesses are lowered.
    /// Each decorator is stored as `[receiver, function]` in evaluation order.
    pub(crate) fn capture_decorator_receiver(
        &self,
        decorator: &mut Decorator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.enabled {
            return;
        }
        let expression = decorator.expression.get_inner_expression_mut();
        if matches!(expression, Expression::ChainExpression(_)) {
            // Capture only after the optional checks have succeeded. Pulling a member's
            // object out of a chain would turn `null?.child.dec` into an early throw.
            OptionalChaining::new().enter_expression(expression, ctx);
            Self::capture_optional_decorator_receiver(expression, ctx);
        } else {
            Self::capture_decorator_receiver_pair(expression, ctx);
        }
    }

    fn capture_optional_decorator_receiver(
        expression: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match expression {
            Expression::ConditionalExpression(conditional) => {
                Self::capture_optional_decorator_receiver(&mut conditional.consequent, ctx);
                Self::capture_optional_decorator_receiver(&mut conditional.alternate, ctx);
            }
            Expression::SequenceExpression(sequence) => {
                Self::capture_optional_decorator_receiver(
                    sequence.expressions.last_mut().unwrap(),
                    ctx,
                );
            }
            expression => Self::capture_decorator_receiver_pair(expression, ctx),
        }
    }

    fn capture_decorator_receiver_pair(expression: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let receiver = if let Some(member) = expression.as_member_expression_mut() {
            let object = member.object_mut();
            if matches!(object.get_inner_expression(), Expression::Super(_)) {
                Expression::new_this_expression(SPAN, ctx)
            } else {
                let (receiver, reference) = duplicate_expression(object.take_in(ctx), true, ctx);
                *object = reference;
                receiver
            }
        } else {
            Expression::new_void_0(SPAN, ctx)
        };
        let function = expression.take_in(ctx);
        *expression = Expression::new_array_expression(
            SPAN,
            [ArrayExpressionElement::from(receiver), ArrayExpressionElement::from(function)],
            ctx,
        );
    }

    /// Decorator context names use the property key after a single ToPropertyKey conversion.
    pub(crate) fn coerce_decorated_keys(body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        for element in &mut body.body {
            let (key, computed, decorated) = match element {
                ClassElement::PropertyDefinition(prop) => {
                    let prop = &mut **prop;
                    (&mut prop.key, &mut prop.computed, !prop.decorators.is_empty())
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
        &mut self,
        body: &ClassBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ClassDecoratorState<'a> {
        let mut groups: [Vec<Address>; 4] = Default::default();
        let mut instance_method_extras = false;
        let mut static_method_extras = false;
        for element in &body.body {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    let address = prop.address();
                    if self.accessor_storage.contains(&address) {
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
                        && !self.accessor_getters.contains(&method.address()) =>
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

        let mut index = 0;
        for group in groups {
            for address in group {
                self.fields.insert(address, index);
                index += 1;
            }
        }

        let init = ctx.generate_uid(
            "init",
            ctx.current_block_scope_id(),
            SymbolFlags::BlockScopedVariable,
        );
        ctx.state.var_declarations.insert_let(&init, None, &ctx.ast);
        ClassDecoratorState { init, instance_method_extras, static_method_extras }
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
            let key = Self::decorator_key_expr(&prop.key, ctx);
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
            let elements = ArenaVec::from_iter_in(
                class.decorators.drain(..).map(|d| ArrayExpressionElement::from(d.expression)),
                ctx,
            );
            let array = Expression::new_array_expression(SPAN, elements, ctx);
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
        let elements = ArenaVec::from_iter_in(
            decorators.drain(..).map(|d| ArrayExpressionElement::from(d.expression)),
            ctx,
        );
        let array = Expression::new_array_expression(SPAN, elements, ctx);
        insert_before.push(create_assignment(&binding, array, SPAN, ctx));
        binding
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
