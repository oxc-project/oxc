//! ES2022: Class Properties
//! Lowering of standard (TC39 2023-11) decorators.
//!
//! Based on esbuild's `lowerClass` (`internal/js_parser/js_parser_lower_class.go`).
//!
//! Decorated classes are lowered entirely: all fields, private names and static blocks, the same
//! as esbuild's `computeClassLoweringInfo` does when any element is decorated. Decorator calls are
//! emitted after the class using esbuild's runtime helpers:
//!
//! ```js
//! _init = __decoratorStart(base);
//! __decorateElement(_init, flags, key, _x_dec, Class);   // per element, spec order
//! Class = __decorateElement(_init, 0, "Class", _Class_decorators, Class);
//! __runInitializers(_init, 3, Class);                     // static method extra initializers
//! /* static fields and blocks */
//! __runInitializers(_init, 1, Class);                     // class extra initializers
//! ```

use std::mem;

use oxc_allocator::{Address, ArenaVec, GetAddress, TakeIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::{
    number::NumberBase,
    reference::ReferenceFlags,
    scope::{ScopeFlags, ScopeId},
};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse};

use crate::{
    common::{
        computed_key::{create_computed_key_temp_var, key_needs_temp_var},
        duplicate::duplicate_expression,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    es2020::OptionalChaining,
    utils::ast_builder::{
        create_accessor_method, create_assignment, create_export_default_class_reference,
        create_export_named_class_reference,
    },
};

use super::ClassProperties;

const KIND_METHOD: u32 = 1;
const KIND_GETTER: u32 = 2;
const KIND_SETTER: u32 = 3;
const KIND_ACCESSOR: u32 = 4;
const KIND_FIELD: u32 = 5;
const FLAG_STATIC: u32 = 8;
const FLAG_PRIVATE: u32 = 16;

/// Decorator state for a decorated class. Stored in `ClassDetails`.
pub(super) struct ClassDecoratorState<'a> {
    /// `_init` var holding the decorator context array.
    pub init: BoundIdentifier<'a>,
    /// Class contains a decorated instance method / getter / setter.
    pub instance_method_extras: bool,
    /// Class contains a decorated static method / getter / setter.
    pub static_method_extras: bool,
}

/// Bucket indexes for `__decorateElement` calls, in the order they are evaluated.
#[derive(Clone, Copy)]
#[expect(clippy::enum_variant_names)]
pub(super) enum DecoratorBucket {
    StaticNonField = 0,
    InstanceNonField = 1,
    StaticField = 2,
    InstanceField = 3,
}

/// Returns `true` if class properties transform lowers this class because of standard decorators:
/// class is decorated, or contains `accessor` properties.
///
/// Other transforms (TypeScript) use this to know class fields of this class are handled by
/// class properties transform, even when the target supports class fields.
pub fn class_lowered_for_decorators(class: &Class<'_>) -> bool {
    !class.declare && class_body_lowered_for_decorators(!class.decorators.is_empty(), &class.body)
}

/// Same as [`class_lowered_for_decorators`], for use when only the class body is accessible.
pub fn class_body_lowered_for_decorators(has_class_decorators: bool, body: &ClassBody<'_>) -> bool {
    has_class_decorators
        || ClassProperties::class_body_has_decorators(body)
        || body
            .body
            .iter()
            .any(|e| matches!(e, ClassElement::AccessorProperty(p) if !p.r#type.is_abstract()))
}

fn number<'a>(value: u32, ctx: &TraverseCtx<'a>) -> Argument<'a> {
    Argument::new_numeric_literal(SPAN, f64::from(value), None, NumberBase::Decimal, ctx)
}

impl<'a> ClassProperties<'a> {
    /// Give an unbraced loop body a scope for per-evaluation decorator state.
    pub(in crate::es2022) fn wrap_decorated_loop_body(
        &self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        #[derive(Default)]
        struct DecoratedClassFinder(bool);
        impl<'a> oxc_ast_visit::Visit<'a> for DecoratedClassFinder {
            fn visit_class(&mut self, class: &Class<'a>) {
                self.0 |= class_lowered_for_decorators(class);
                if !self.0 {
                    oxc_ast_visit::walk::walk_class(self, class);
                }
            }
        }
        use oxc_ast_visit::Visit;

        if !self.standard_decorators
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

        let mut finder = DecoratedClassFinder::default();
        finder.visit_statement(statement);
        if finder.0 {
            let scope_id = ctx.insert_scope_below_statement(statement, ScopeFlags::empty());
            let body = statement.take_in(ctx);
            *statement = Statement::new_block_statement_with_scope_id(SPAN, [body], scope_id, ctx);
        }
    }

    /// Preserve the receiver before private fields and `super` accesses are lowered.
    /// Each decorator is stored as `[receiver, function]` in evaluation order.
    pub(in crate::es2022) fn capture_decorator_receiver(
        &self,
        decorator: &mut Decorator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.standard_decorators {
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

    /// Returns `true` if class or any of its elements are decorated.
    pub(super) fn class_body_has_decorators(body: &ClassBody<'a>) -> bool {
        body.body.iter().any(|element| match element {
            ClassElement::MethodDefinition(method) => !method.decorators.is_empty(),
            ClassElement::PropertyDefinition(prop) => !prop.decorators.is_empty(),
            ClassElement::AccessorProperty(prop) => !prop.decorators.is_empty(),
            ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => false,
        })
    }

    /// Decorator context names use the property key after a single ToPropertyKey conversion.
    pub(super) fn coerce_decorated_keys(body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
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

    /// Entry phase: lower `accessor` properties to private storage + getter/setter pair.
    ///
    /// `@dec accessor x = 1` -> `#_x_accessor_storage = 1; @dec get x() { return this.#_x_accessor_storage; } set x(value) { ... }`
    ///
    /// Decorators move to the getter, which is recorded so it's decorated with `accessor` kind.
    pub(super) fn lower_auto_accessors(
        &mut self,
        body: &mut ClassBody<'a>,
        class_scope_id: ScopeId,
        static_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !body
            .body
            .iter()
            .any(|e| matches!(e, ClassElement::AccessorProperty(p) if !p.r#type.is_abstract()))
        {
            return;
        }

        // Private names are matched by name, so generated storage names must not clash
        // with existing private names, or with each other (`accessor x` + `static accessor x`).
        let mut used_names: FxHashSet<&str> = body
            .body
            .iter()
            .filter_map(|element| match element {
                ClassElement::PropertyDefinition(p) => p.key.private_name(),
                ClassElement::MethodDefinition(m) => m.key.private_name(),
                ClassElement::AccessorProperty(p) => p.key.private_name(),
                _ => None,
            })
            .map(|name| name.as_str())
            .collect();
        let mut unique_name = |base: Str<'a>, ctx: &TraverseCtx<'a>| -> Str<'a> {
            let mut name = base;
            let mut n = 2u32;
            while used_names.contains(name.as_str()) {
                name = Str::from_strs_array_in([&base, &n.to_string()], ctx);
                n += 1;
            }
            used_names.insert(name.as_str());
            name
        };

        let mut new_body = ArenaVec::with_capacity_in(body.body.len() + 4, ctx);
        let mut counter = 0u32;
        for element in body.body.drain(..) {
            let ClassElement::AccessorProperty(accessor) = element else {
                new_body.push(element);
                continue;
            };
            if accessor.r#type.is_abstract() {
                new_body.push(ClassElement::AccessorProperty(accessor));
                continue;
            }

            let mut accessor = accessor.unbox();
            let is_static = accessor.r#static;
            let computed = accessor.computed;
            let decorators = mem::replace(&mut accessor.decorators, ArenaVec::new_in(ctx));
            let is_decorated = !decorators.is_empty();

            let original_name = Self::static_key_name(&accessor.key, ctx);
            let (storage_name, getter_key, setter_key) = match accessor.key {
                PropertyKey::StaticIdentifier(ident) => {
                    let name =
                        Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                    let key = PropertyKey::StaticIdentifier(ident);
                    let key2 = PropertyKey::new_static_identifier(
                        SPAN,
                        match &key {
                            PropertyKey::StaticIdentifier(ident) => ident.name,
                            _ => unreachable!(),
                        },
                        ctx,
                    );
                    (name, key, key2)
                }
                PropertyKey::PrivateIdentifier(ident) => {
                    let name =
                        Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                    let key2 = PropertyKey::new_private_identifier(SPAN, ident.name, ctx);
                    (name, PropertyKey::PrivateIdentifier(ident), key2)
                }
                key => {
                    let counter_str = counter.to_string();
                    counter += 1;
                    let name =
                        Str::from_strs_array_in(["_", &counter_str, "_accessor_storage"], ctx);
                    let key_expr = key.into_expression();
                    let (assignment, reference) = duplicate_expression(key_expr, true, ctx);
                    (name, PropertyKey::from(assignment), PropertyKey::from(reference))
                }
            };

            let storage_name = unique_name(storage_name, ctx);

            // `#_x_accessor_storage = value`
            let storage = ClassElement::new_property_definition(
                SPAN,
                PropertyDefinitionType::PropertyDefinition,
                [],
                PropertyKey::new_private_identifier(SPAN, storage_name, ctx),
                None,
                accessor.value.take(),
                false,
                is_static,
                false,
                false,
                false,
                false,
                false,
                None,
                ctx,
            );
            if let ClassElement::PropertyDefinition(prop) = &storage {
                if is_decorated {
                    self.decorator_accessor_storage.insert(prop.address());
                }
                if let Some(name) = &original_name {
                    self.decorator_accessor_names.insert(prop.address(), *name);
                }
            }
            new_body.push(storage);

            let getter = create_accessor_method(
                decorators,
                getter_key,
                MethodDefinitionKind::Get,
                computed,
                is_static,
                storage_name,
                if is_static { static_binding } else { None },
                class_scope_id,
                None,
                ctx,
            );
            if is_decorated && let ClassElement::MethodDefinition(method) = &getter {
                self.decorator_accessor_getters.insert(method.address());
            }
            new_body.push(getter);

            new_body.push(create_accessor_method(
                ArenaVec::new_in(ctx),
                setter_key,
                MethodDefinitionKind::Set,
                computed,
                is_static,
                storage_name,
                if is_static { static_binding } else { None },
                class_scope_id,
                None,
                ctx,
            ));
        }
        body.body = new_body;
    }

    /// Entry phase: assign initializer slot indexes to decorated fields and accessors.
    ///
    /// Order matches the order `__decorateElement` is called in:
    /// static accessors, instance accessors, static fields, instance fields.
    pub(super) fn prepare_class_decorators(
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
                    if self.decorator_accessor_storage.contains(&address) {
                        groups[usize::from(!prop.r#static)].push(address);
                    } else if !prop.decorators.is_empty() && !prop.declare {
                        groups[2 + usize::from(!prop.r#static)].push(address);
                    }
                }
                ClassElement::MethodDefinition(method)
                    if !method.decorators.is_empty()
                        && !method.kind.is_constructor()
                        && !self.decorator_accessor_getters.contains(&method.address()) =>
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
                self.decorated_fields.insert(address, index);
                index += 1;
            }
        }

        let init = ctx.generate_uid(
            "init",
            ctx.current_block_scope_id(),
            oxc_syntax::symbol::SymbolFlags::BlockScopedVariable,
        );
        ctx.state.var_declarations.insert_let(&init, None, &ctx.ast);
        ClassDecoratorState { init, instance_method_extras, static_method_extras }
    }

    /// Capture a decorated instance field's key before it is moved into the constructor.
    pub(super) fn record_instance_field_for_decorators(
        &mut self,
        prop: &ArenaBox<'a, PropertyDefinition<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !prop.decorators.is_empty()
            && let Some(key) = prop.key.as_expression()
            && !key_needs_temp_var(key, ctx)
        {
            let key = Self::decorator_key_expr(&prop.key, ctx);
            self.decorator_instance_keys.insert(prop.address(), key);
        }
    }

    /// Initializer slot index of a decorated field or accessor storage, if it has one.
    pub(super) fn decorated_field_index(
        &self,
        prop: &ArenaBox<'a, PropertyDefinition<'a>>,
    ) -> Option<u32> {
        self.decorated_fields.get(&prop.address()).copied()
    }

    fn decorator_init(&self) -> BoundIdentifier<'a> {
        self.current_class().decorators.as_ref().unwrap().init.clone()
    }

    /// `__runInitializers(_init, (4 + 2 * index) << 1, this, value)`
    pub(super) fn wrap_decorated_field_value(
        &self,
        value: Option<Expression<'a>>,
        index: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let init = self.decorator_init();
        let mut arguments = ArenaVec::with_capacity_in(4, ctx);
        arguments.push(Argument::from(init.create_read_expression(ctx)));
        arguments.push(number((4 + 2 * index) << 1, ctx));
        arguments.push(Argument::new_this_expression(SPAN, ctx));
        if let Some(value) = value {
            arguments.push(Argument::from(value));
        }
        helper_call_expr(Helper::RunInitializers, arguments, ctx)
    }

    /// `__runInitializers(_init, ((5 + 2 * index) << 1) | 1, this)`
    pub(super) fn decorated_field_extra_initializers(
        &self,
        index: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.run_initializers_call(
            ((5 + 2 * index) << 1) | 1,
            Expression::new_this_expression(SPAN, ctx),
            ctx,
        )
    }

    /// `__runInitializers(_init, flags, target)`
    pub(super) fn run_initializers_call(
        &self,
        flags: u32,
        target: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let init = self.decorator_init();
        let arguments = ArenaVec::from_array_in(
            [
                Argument::from(init.create_read_expression(ctx)),
                number(flags, ctx),
                Argument::from(target),
            ],
            ctx,
        );
        helper_call_expr(Helper::RunInitializers, arguments, ctx)
    }

    /// Exit phase: evaluate class decorators into `_Class_decorators = [...]` before class,
    /// and capture super class so its metadata can be inherited.
    pub(super) fn take_class_decorators(
        &mut self,
        class: &mut Class<'a>,
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
            self.insert_before.push(create_assignment(&binding, array, SPAN, ctx));
            self.decorator_class_decorators = Some(binding);
        }
        self.decorator_class_prefix_len = self.insert_before.len();

        if let Some(heritage) = &mut class.heritage {
            let binding = ctx.generate_uid_in_current_hoist_scope("superclass");
            ctx.state.var_declarations.insert_var(&binding, &ctx.ast);
            let expr = heritage.expression.take_in(ctx);
            heritage.expression = create_assignment(&binding, expr, SPAN, ctx);
            self.decorator_super_class = Some(binding);
        }
    }

    /// Exit phase: element decorators and computed keys are evaluated after `extends` clause.
    /// If class has `extends`, move them into it: `extends (_superclass = Base, _x_dec = [dec], _superclass)`.
    /// This also evaluates them in the class scope, as the spec requires.
    pub(super) fn fold_element_decorators_into_heritage(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let start = mem::take(&mut self.decorator_class_prefix_len);
        let (Some(heritage), Some(binding)) = (&mut class.heritage, &self.decorator_super_class)
        else {
            return;
        };
        if self.insert_before.len() <= start {
            return;
        }
        let mut exprs = ArenaVec::with_capacity_in(self.insert_before.len() - start + 2, ctx);
        exprs.push(heritage.expression.take_in(ctx));
        exprs.extend(self.insert_before.drain(start..));
        exprs.push(binding.create_read_expression(ctx));
        heritage.expression = Expression::new_sequence_expression(SPAN, exprs, ctx);
    }

    /// Exit phase: evaluate element decorators into `_x_dec = [...]` before class.
    pub(super) fn take_element_decorators(
        &mut self,
        decorators: &mut ArenaVec<'a, Decorator<'a>>,
        key: &PropertyKey<'a>,
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
        self.insert_before.push(create_assignment(&binding, array, SPAN, ctx));
        binding
    }

    /// Memoize a static decorated field's computed key so it's evaluated once.
    pub(super) fn memoize_decorated_key(
        &mut self,
        key: &mut PropertyKey<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(expr) = key.as_expression_mut()
            && key_needs_temp_var(expr, ctx)
        {
            let (assignment, temp) = create_computed_key_temp_var(expr.take_in(ctx), ctx);
            self.insert_before.push(assignment);
            *key = PropertyKey::from(temp);
        }
    }

    /// Expression for element name passed to `__decorateElement`.
    pub(super) fn decorator_key_expr(
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

    /// Name of a non-computed property key, for named evaluation of anonymous classes.
    fn static_key_name(key: &PropertyKey<'a>, ctx: &TraverseCtx<'a>) -> Option<oxc_str::JSStr<'a>> {
        match key {
            PropertyKey::StaticIdentifier(ident) => Some(ident.name.into()),
            PropertyKey::StringLiteral(lit) => Some(lit.value),
            PropertyKey::PrivateIdentifier(ident) => {
                Some(Str::from_strs_array_in(["#", &ident.name], ctx).into())
            }
            _ => None,
        }
    }

    /// Preserve named evaluation before a property initializer is moved or its key is rewritten.
    pub(super) fn capture_class_initializer_name(
        &mut self,
        key: &mut PropertyKey<'a>,
        computed: bool,
        value: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = value.without_parentheses() else { return };
        if !self.standard_decorators || class.id.is_some() || !class_lowered_for_decorators(class) {
            return;
        }
        let name = if computed {
            let key_expr = key.as_expression_mut().unwrap();
            let value = helper_call_expr(
                Helper::ToPropertyKey,
                ArenaVec::from_value_in(Argument::from(key_expr.take_in(ctx)), ctx),
                ctx,
            );
            let (assignment, reference) = create_computed_key_temp_var(value, ctx);
            *key_expr = assignment;
            reference
        } else {
            Self::decorator_key_expr(key, ctx)
        };
        self.decorator_named_classes.insert(class.address(), name);
    }

    pub(in crate::es2022) fn capture_object_property_class_name(
        &mut self,
        prop: &mut ObjectProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // A prototype setter does not perform named evaluation.
        if !prop.computed
            && Self::static_key_name(&prop.key, ctx).is_some_and(|name| name == "__proto__")
        {
            return;
        }
        self.capture_class_initializer_name(&mut prop.key, prop.computed, &prop.value, ctx);
    }

    /// Infer name of an anonymous class expression (NamedEvaluation), from its parent node.
    pub(super) fn infer_class_expression_name(
        &mut self,
        class_address: Address,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if let Some(name) = self.decorator_named_classes.remove(&class_address) {
            return Some(name);
        }
        let parent = ctx
            .ancestors()
            .find(|ancestor| !matches!(ancestor, Ancestor::ParenthesizedExpressionExpression(_)))?;
        let name = match parent {
            Ancestor::PropertyDefinitionValue(prop) => {
                self.decorator_accessor_names.get(&parent.address()).copied().or_else(|| {
                    if *prop.computed() { None } else { Self::static_key_name(prop.key(), ctx) }
                })
            }
            _ => super::super::class_name::contextual_class_name(&parent).map(Into::into),
        };
        name.map(|name| Expression::new_string_literal(SPAN, name, None, ctx))
    }

    pub(super) fn set_inferred_class_name(
        class: Expression<'a>,
        name: Option<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Some(name) = name else { return class };
        helper_call_expr(
            Helper::SetClassName,
            ArenaVec::from_array_in([Argument::from(class), Argument::from(name)], ctx),
            ctx,
        )
    }

    /// Expression referring to the class from outside the class body.
    pub(super) fn decorated_class_ref(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let class_details = self.current_class();
        let bindings = &class_details.bindings;
        let binding = if class_details.is_declaration {
            bindings.name.as_ref().or(bindings.temp.as_ref())
        } else {
            bindings.temp.as_ref()
        };
        binding.expect("decorated class always has a binding").create_read_expression(ctx)
    }

    /// Exit phase: `__decorateElement(_init, flags, key, _x_dec, Class)`
    pub(super) fn push_decorate_element(
        &mut self,
        bucket: DecoratorBucket,
        kind: u32,
        is_static: bool,
        key: Expression<'a>,
        decorators: &BoundIdentifier<'a>,
        private_name: Option<oxc_str::Ident<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let flags = kind
            | if is_static { FLAG_STATIC } else { 0 }
            | if private_name.is_some() { FLAG_PRIVATE } else { 0 };
        let init = self.decorator_init();
        let target = self.decorated_class_ref(ctx);
        let mut arguments = ArenaVec::from_array_in(
            [
                Argument::from(init.create_read_expression(ctx)),
                number(flags, ctx),
                Argument::from(key),
                Argument::from(decorators.create_read_expression(ctx)),
                Argument::from(target),
            ],
            ctx,
        );
        let mut writebacks = Vec::new();
        if let Some(name) = private_name {
            let details = self.current_class();
            let prop = details.private_props.as_ref().unwrap().get(&name).unwrap();
            let brand = if kind == KIND_FIELD {
                prop.binding.clone()
            } else if is_static {
                details.bindings.decorator_static_brand.as_ref().unwrap().clone()
            } else {
                details.bindings.brand().clone()
            };
            let (getter, setter) = if matches!(prop.method_kind, Some(MethodDefinitionKind::Set)) {
                (prop.binding2.clone(), Some(prop.binding.clone()))
            } else {
                (Some(prop.binding.clone()), prop.binding2.clone())
            };
            if kind == KIND_METHOD {
                writebacks.push(("value", prop.binding.clone()));
            }
            if (kind == KIND_GETTER || kind == KIND_ACCESSOR)
                && let Some(getter) = getter
            {
                writebacks.push(("get", getter));
            }
            if (kind == KIND_SETTER || kind == KIND_ACCESSOR)
                && let Some(setter) = setter
            {
                writebacks.push(("set", setter));
            }
            let mut properties = ArenaVec::with_capacity_in(writebacks.len(), ctx);
            for (key, binding) in &writebacks {
                let key = PropertyKey::new_static_identifier(SPAN, Str::from_str_in(key, ctx), ctx);
                let value = binding.create_read_expression(ctx);
                properties.push(ObjectPropertyKind::new_object_property(
                    SPAN,
                    PropertyKind::Init,
                    key,
                    value,
                    false,
                    false,
                    false,
                    ctx,
                ));
            }
            let descriptor = Expression::new_object_expression(SPAN, properties, ctx);
            let info = Expression::new_array_expression(
                SPAN,
                [
                    ArrayExpressionElement::from(brand.create_read_expression(ctx)),
                    ArrayExpressionElement::from(descriptor),
                ],
                ctx,
            );
            arguments.push(Argument::from(info));
        }
        let mut call = helper_call_expr(Helper::DecorateElement, arguments, ctx);
        if kind == KIND_ACCESSOR && private_name.is_some() {
            let descriptor = ctx.generate_uid_in_current_hoist_scope("descriptor");
            ctx.state.var_declarations.insert_var(&descriptor, &ctx.ast);
            let mut expressions = ArenaVec::new_in(ctx);
            expressions.push(create_assignment(&descriptor, call, SPAN, ctx));
            for (key, binding) in writebacks {
                let value = crate::utils::ast_builder::create_property_access(
                    SPAN,
                    descriptor.create_read_expression(ctx),
                    key,
                    ctx,
                );
                expressions.push(create_assignment(&binding, value, SPAN, ctx));
            }
            call = Expression::new_sequence_expression(SPAN, expressions, ctx);
        } else if let Some((_, binding)) = writebacks.first() {
            call = create_assignment(binding, call, SPAN, ctx);
        }
        self.decorator_elements[bucket as usize].push(call);
    }

    pub(super) fn method_decorator_kind(&self, method: &ArenaBox<'a, MethodDefinition<'a>>) -> u32 {
        if self.decorator_accessor_getters.contains(&method.address()) {
            return KIND_ACCESSOR;
        }
        match method.kind {
            MethodDefinitionKind::Get => KIND_GETTER,
            MethodDefinitionKind::Set => KIND_SETTER,
            MethodDefinitionKind::Method | MethodDefinitionKind::Constructor => KIND_METHOD,
        }
    }

    pub(super) const fn field_decorator_kind() -> u32 {
        KIND_FIELD
    }

    /// Exit phase: build the decorator application sequence which runs after the class definition,
    /// and the trailing class extra initializers.
    ///
    /// Returns `(before_static_members, after_static_members)`.
    pub(super) fn build_decorator_sequence(
        &mut self,
        class: &Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Vec<Expression<'a>>, Option<Expression<'a>>) {
        let state = self.current_class().decorators.as_ref().unwrap();
        let init = state.init.clone();
        let static_method_extras = state.static_method_extras;

        let mut exprs = Vec::new();

        // `_init = __decoratorStart(base)`
        let base = match &self.decorator_super_class.take() {
            Some(binding) => binding.create_read_expression(ctx),
            None => Expression::new_null_literal(SPAN, ctx),
        };
        let start = helper_call_expr(
            Helper::DecoratorStart,
            ArenaVec::from_value_in(Argument::from(base), ctx),
            ctx,
        );
        exprs.push(create_assignment(&init, start, SPAN, ctx));

        for bucket in &mut self.decorator_elements {
            exprs.append(bucket);
        }

        if let Some(brand) = self.current_class().bindings.decorator_static_brand.clone() {
            let weakset = super::class::create_new_weakset(ctx);
            exprs.insert(1, create_assignment(&brand, weakset, SPAN, ctx));
            let target = self.decorated_class_ref(ctx);
            let callee = crate::utils::ast_builder::create_property_access(
                SPAN,
                brand.create_read_expression(ctx),
                "add",
                ctx,
            );
            exprs.push(Expression::new_call_expression(
                SPAN,
                callee,
                None,
                [Argument::from(target)],
                false,
                ctx,
            ));
        }

        let class_decorators = self.decorator_class_decorators.take();
        if let Some(decorators) = &class_decorators {
            // `Class = __decorateElement(_init, 0, "Class", _Class_decorators, Class)`
            let inferred_name = self.decorator_class_name.take();
            let name = match &class.id {
                Some(id) => Expression::new_string_literal(SPAN, id.name, None, ctx),
                None => inferred_name
                    .unwrap_or_else(|| Expression::new_string_literal(SPAN, "", None, ctx)),
            };
            let target = self.decorated_class_ref(ctx);
            let arguments = ArenaVec::from_array_in(
                [
                    Argument::from(init.create_read_expression(ctx)),
                    number(0, ctx),
                    Argument::from(name),
                    Argument::from(decorators.create_read_expression(ctx)),
                    Argument::from(target),
                ],
                ctx,
            );
            let mut call = helper_call_expr(Helper::DecorateElement, arguments, ctx);

            let class_details = self.current_class();
            let bindings = &class_details.bindings;
            let mut targets = Vec::with_capacity(2);
            if class_details.is_declaration
                && let Some(name) = &bindings.name
            {
                targets.push(name.clone());
            }
            if let Some(temp) = &bindings.temp
                && targets.iter().all(|t| t.symbol_id != temp.symbol_id)
            {
                targets.push(temp.clone());
            }
            for target in targets {
                call = create_assignment(&target, call, SPAN, ctx);
            }
            exprs.push(call);
        } else {
            // `__decoratorMetadata(_init, Class)`
            let target = self.decorated_class_ref(ctx);
            let arguments = ArenaVec::from_array_in(
                [Argument::from(init.create_read_expression(ctx)), Argument::from(target)],
                ctx,
            );
            exprs.push(helper_call_expr(Helper::DecoratorMetadata, arguments, ctx));
        }

        if static_method_extras {
            let target = self.decorated_class_ref(ctx);
            exprs.push(self.run_initializers_call(3, target, ctx));
        }

        let class_extras = class_decorators.map(|_| {
            let target = self.decorated_class_ref(ctx);
            self.run_initializers_call(1, target, ctx)
        });

        (exprs, class_extras)
    }
}

use oxc_allocator::{ArenaBox, CloneIn, GetAllocator};
use rustc_hash::FxHashSet;

/// Rewrite references to the class's own name inside class body to `replacement`.
///
/// Inside the class body, the class name refers to the inner (immutable) class binding,
/// which a class decorator may replace. Element decorators are moved outside the class body,
/// where the name would refer to a different binding.
///
/// `replacement` is created lazily on first reference, if `create_in_scope` is `Some`.
pub(super) struct ClassNameRewriter<'a, 'ctx> {
    symbol_id: oxc_syntax::symbol::SymbolId,
    name: oxc_str::Ident<'a>,
    replacement: Option<BoundIdentifier<'a>>,
    create_in_scope: Option<ScopeId>,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a, 'ctx> ClassNameRewriter<'a, 'ctx> {
    pub fn new(
        symbol_id: oxc_syntax::symbol::SymbolId,
        name: oxc_str::Ident<'a>,
        replacement: Option<BoundIdentifier<'a>>,
        create_in_scope: Option<ScopeId>,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) -> Self {
        Self { symbol_id, name, replacement, create_in_scope, ctx }
    }

    pub fn rewrite(mut self, body: &mut ClassBody<'a>) -> Option<BoundIdentifier<'a>> {
        use oxc_ast_visit::VisitMut;
        self.visit_class_body(body);
        self.replacement
    }
}

impl<'a> oxc_ast_visit::VisitMut<'a> for ClassNameRewriter<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        let symbol_id = self.ctx.scoping().get_reference(ident.reference_id()).symbol_id();
        if symbol_id != Some(self.symbol_id) {
            return;
        }
        let replacement = if let Some(replacement) = &self.replacement {
            replacement.clone()
        } else {
            let Some(scope_id) = self.create_in_scope else { return };
            let binding = self.ctx.generate_uid(
                &self.name,
                scope_id,
                oxc_syntax::symbol::SymbolFlags::BlockScopedVariable,
            );
            self.replacement = Some(binding.clone());
            binding
        };
        let reference_id = ident.reference_id();
        let flags = self.ctx.scoping().get_reference(reference_id).flags();
        self.ctx.scoping_mut().delete_resolved_reference(self.symbol_id, reference_id);
        *ident = replacement.create_reference(flags, self.ctx);
    }
}

impl<'a> ClassProperties<'a> {
    /// Exit phase: convert a decorated class declaration whose name is referenced inside the class body.
    ///
    /// `class Foo {}` -> `let _Foo = class Foo {}; /* decorators */ let Foo = _Foo;`
    pub(super) fn rewrite_decorated_class_statement(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        enum Export {
            None,
            Named,
            Default,
        }
        let (class, export) = match stmt {
            Statement::ClassDeclaration(class) => (class, Export::None),
            Statement::ExportDeclaration(export) => match &mut export.declaration {
                Declaration::ClassDeclaration(class) => (class, Export::Named),
                _ => return,
            },
            Statement::ExportDefaultDeclaration(export) => match &mut export.declaration {
                ExportDefaultDeclarationKind::ClassDeclaration(class) => (class, Export::Default),
                _ => return,
            },
            _ => return,
        };
        let Some((inner, outer)) = self.decorator_declaration_rewrites.remove(&class.address())
        else {
            return;
        };

        // Class name becomes a binding in class scope. Outer binding becomes `let Foo`.
        let class_scope_id = class.scope_id();
        let id = class.id.as_mut().unwrap();
        let class_name = ctx.generate_spanned_binding(
            id.name,
            id.span,
            class_scope_id,
            oxc_syntax::symbol::SymbolFlags::Class,
        );
        id.symbol_id.set(Some(class_name.symbol_id));
        *ctx.scoping_mut().symbol_flags_mut(outer.symbol_id) =
            oxc_syntax::symbol::SymbolFlags::BlockScopedVariable;

        class.r#type = ClassType::ClassExpression;
        let class_expr = Expression::ClassExpression(class.take_in_box(ctx));
        let new_stmt = Self::create_let(&inner, class_expr, ctx);
        let outer_stmt = Self::create_let(&outer, inner.create_read_expression(ctx), ctx);

        ctx.state.statement_injector.move_insertions(stmt, &new_stmt);
        ctx.state.statement_injector.insert_after(&new_stmt, outer_stmt);
        match export {
            Export::None => {}
            Export::Named => {
                let export = create_export_named_class_reference(&outer, ctx);
                ctx.state.statement_injector.insert_after(&new_stmt, export);
            }
            Export::Default => {
                let export = create_export_default_class_reference(&outer, ctx);
                ctx.state.statement_injector.insert_after(&new_stmt, export);
            }
        }
        *stmt = new_stmt;
    }

    pub(super) fn create_let(
        binding: &BoundIdentifier<'a>,
        init: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        let declarator = VariableDeclarator::new(
            SPAN,
            binding
                .create_spanned_binding_pattern(ctx.scoping().symbol_span(binding.symbol_id), ctx),
            None,
            Some(init),
            false,
            ctx,
        );
        Statement::from(Declaration::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Let,
            [declarator],
            false,
            ctx,
        ))
    }
}
