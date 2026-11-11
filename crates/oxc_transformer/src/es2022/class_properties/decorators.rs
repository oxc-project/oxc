//! Integration of standard decorators with class bindings and private-element lowering.

use oxc_allocator::{ArenaBox, GetAddress, UnstableAddress};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Ident;
use oxc_syntax::{scope::ScopeId, symbol::SymbolFlags};
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::class_lowering::ClassLoweringMode,
    context::TraverseCtx,
    decorator::standard::{
        ClassDecoratorBindings, ClassDecoratorState, ClassDecoratorTables, ClassNameRewriter,
        DecoratorElementKind, PrivateDecoratorAccess, StandardDecoratorState,
    },
};

use super::{ClassBindings, ClassProperties, IdentIndexMap, PrivateProp};

/// Decorator setup produced before the class is pushed onto the class stack.
pub(super) struct DecoratorBodyEntry<'a> {
    pub state: Option<ClassDecoratorState<'a>>,
    pub static_accessor_binding: Option<BoundIdentifier<'a>>,
    pub has_auto_accessor: bool,
}

impl<'a> DecoratorBodyEntry<'a> {
    pub fn is_decorated(&self) -> bool {
        self.state.is_some()
    }

    /// Decorated classes and lowered auto-accessors also own their static blocks.
    pub fn lowers_static_blocks(&self) -> bool {
        self.is_decorated() || self.has_auto_accessor
    }

    pub fn create_binding(&self, name: &str, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        if self.is_decorated() {
            ctx.generate_uid(name, ctx.current_block_scope_id(), SymbolFlags::BlockScopedVariable)
        } else {
            ctx.generate_uid_in_current_hoist_scope(name)
        }
    }

    pub fn private_method_scope(
        &self,
        in_class_sequence: bool,
        ctx: &TraverseCtx<'a>,
    ) -> (ScopeId, SymbolFlags) {
        if !in_class_sequence {
            (ctx.current_block_scope_id(), SymbolFlags::Function)
        } else if self.is_decorated() {
            (ctx.current_block_scope_id(), SymbolFlags::BlockScopedVariable)
        } else {
            (ctx.current_hoist_scope_id(), SymbolFlags::FunctionScopedVariable)
        }
    }

    pub fn create_temp_binding(
        &self,
        name: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        if self.is_decorated() {
            self.create_binding(name.map_or("Class", |binding| binding.name.as_str()), ctx)
        } else {
            ClassBindings::create_temp_binding(name, ctx.current_hoist_scope_id(), ctx)
        }
    }

    pub fn declare_temp(&self, binding: &BoundIdentifier<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_decorated() {
            ctx.state.var_declarations.insert_let(binding, None, &ctx.ast);
        } else {
            ctx.state.var_declarations.insert_var(binding, &ctx.ast);
        }
    }

    pub fn configure_private_props(&self, private_props: &mut IdentIndexMap<'a, PrivateProp<'a>>) {
        if self.is_decorated() {
            for prop in private_props.values_mut() {
                prop.uses_weakmap = prop.is_static && !prop.is_method();
            }
        }
    }

    pub fn configure_bindings(
        &self,
        bindings: &mut ClassBindings<'a>,
        private_props: &IdentIndexMap<'a, PrivateProp<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_decorated() {
            return;
        }
        bindings.force_private_spec = true;
        if private_props.values().any(|prop| prop.is_static && prop.is_method()) {
            let brand = self.create_binding("staticBrand", ctx);
            ctx.state.var_declarations.insert_let(&brand, None, &ctx.ast);
            bindings.decorator_static_brand = Some(brand);
        }
    }

    pub fn rewrite_expression_name(
        &self,
        body: &mut ClassBody<'a>,
        bindings: &ClassBindings<'a>,
        is_declaration: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.is_decorated()
            && !is_declaration
            && let (Some(name), Some(temp)) = (&bindings.name, &bindings.temp)
        {
            ClassNameRewriter::new(name.symbol_id, name.name, Some(temp.clone()), None, ctx)
                .rewrite(body);
        }
    }
}

/// Instance field initializers move on entry; decorators and keys are collected on exit.
pub(super) enum InstanceFieldPhase<'b, 'a> {
    Entry(&'b mut Vec<Expression<'a>>),
    Exit,
}

impl<'a> ClassProperties<'a> {
    /// Capture names before rewriting keys, then prepare this class's decorator state.
    pub(super) fn on_body_entry(
        &mut self,
        body: &mut ClassBody<'a>,
        class_name_binding: &mut Option<BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> DecoratorBodyEntry<'a> {
        let Ancestor::ClassBody(class) = ctx.parent() else { unreachable!() };
        let class_scope_id = class.scope_id().get().unwrap();
        let class_address = class.address();
        let is_declaration = *class.r#type() == ClassType::ClassDeclaration;
        let lowering = ctx.state.class_lowering_of(class_scope_id);
        let standard = self.decorator_state.enabled();
        let is_decorated = standard && lowering.lowered_for_decorators();
        let has_auto_accessor = standard && lowering.has_auto_accessor();
        if standard {
            for element in &mut body.body {
                let (key, computed, value) = match element {
                    ClassElement::PropertyDefinition(prop) => {
                        let prop = &mut **prop;
                        (&mut prop.key, prop.computed, &prop.value)
                    }
                    ClassElement::AccessorProperty(prop) => {
                        let prop = &mut **prop;
                        (&mut prop.key, prop.computed, &prop.value)
                    }
                    _ => continue,
                };
                if let Some(value) = value {
                    self.decorator_state.capture_class_initializer_name(key, computed, value, ctx);
                }
            }
        }
        if matches!(lowering.mode(), ClassLoweringMode::Native | ClassLoweringMode::Preserve) {
            return DecoratorBodyEntry {
                state: None,
                static_accessor_binding: None,
                has_auto_accessor,
            };
        }
        let static_accessor_binding = (has_auto_accessor
            && body.body.iter().any(|element| {
                matches!(element, ClassElement::AccessorProperty(prop) if prop.r#static && !prop.r#type.is_abstract())
            }))
        .then(|| {
            if is_decorated {
                let name = class_name_binding.as_ref().map_or("Class", |binding| binding.name.as_str());
                ctx.generate_uid(name, ctx.current_block_scope_id(), SymbolFlags::BlockScopedVariable)
            } else {
                ClassBindings::create_temp_binding(
                    class_name_binding.as_ref(),
                    ctx.current_hoist_scope_id(),
                    ctx,
                )
            }
        });
        if is_decorated {
            StandardDecoratorState::coerce_decorated_keys(body, ctx);
        }
        let mut tables = ClassDecoratorTables::default();
        if is_decorated || has_auto_accessor {
            self.decorator_state.lower_auto_accessors(
                body,
                class_scope_id,
                static_accessor_binding.as_ref(),
                false,
                is_decorated.then_some(&mut tables),
                ctx,
            );
        }
        let state = if is_decorated {
            Some(StandardDecoratorState::prepare_class_decorators(body, tables, ctx))
        } else {
            None
        };

        // A class decorator can replace the outer binding while the class keeps its inner name.
        if is_decorated
            && is_declaration
            && let Some(name_binding) = class_name_binding
        {
            let outer_scope_id = ctx.scoping().scope_parent_id(class_scope_id).unwrap();
            let inner = ClassNameRewriter::new(
                name_binding.symbol_id,
                name_binding.name,
                None,
                Some(outer_scope_id),
                ctx,
            )
            .rewrite(body);
            if let Some(inner) = inner {
                self.decorator_state.record_declaration_rewrite(
                    class_address,
                    (inner.clone(), name_binding.clone()),
                );
                *name_binding = inner;
            }
        }
        DecoratorBodyEntry { state, static_accessor_binding, has_auto_accessor }
    }

    pub(super) fn push_instance_method_initializers(
        &self,
        instance_inits: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(state) = &self.current_class().decorators
            && state.instance_method_extras
        {
            let this = Expression::new_this_expression(SPAN, ctx);
            instance_inits.push(state.run_initializers_call(5, this, ctx));
        }
    }

    /// Wrap the initializer on entry, and collect its decorator and computed key on exit.
    pub(super) fn on_instance_field(
        &mut self,
        prop: &mut ArenaBox<'a, PropertyDefinition<'a>>,
        phase: InstanceFieldPhase<'_, 'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match phase {
            InstanceFieldPhase::Entry(instance_inits) => {
                if let Some(state) = self.current_class_mut().decorators.as_mut() {
                    state.tables.record_instance_field_for_decorators(prop, ctx);
                }
                let state = self.current_class().decorators.as_ref();
                let index = state.and_then(|state| state.tables.decorated_field_index(prop));
                if let Some(index) = index {
                    let value = prop.value.take();
                    prop.value = Some(state.unwrap().wrap_decorated_field_value(value, index, ctx));
                }
                self.convert_instance_property(prop, instance_inits, ctx);
                if let Some(index) = index {
                    let state = self.current_class().decorators.as_ref().unwrap();
                    instance_inits.push(state.decorated_field_extra_initializers(index, ctx));
                }
            }
            InstanceFieldPhase::Exit => {
                let address = prop.address();
                let prop = &mut **prop;
                if self.current_class().decorators.is_some() && !prop.decorators.is_empty() {
                    let decorators = StandardDecoratorState::take_element_decorators(
                        &mut prop.decorators,
                        &prop.key,
                        &mut self.insert_before,
                        ctx,
                    );
                    let key = self
                        .current_class_mut()
                        .decorators
                        .as_mut()
                        .unwrap()
                        .tables
                        .take_instance_key(address)
                        .unwrap_or_else(|| {
                            StandardDecoratorState::decorator_key_expr(&prop.key, ctx)
                        });
                    self.push_decorate_element(
                        DecoratorElementKind::Field,
                        false,
                        key,
                        &decorators,
                        prop.key.private_name(),
                        ctx,
                    );
                }
                if prop.computed {
                    self.extract_instance_prop_computed_key(prop, ctx);
                }
            }
        }
    }

    /// Evaluate decorators before the key, lower the field, then append its extra initializers.
    pub(super) fn on_static_field(
        &mut self,
        prop: &mut ArenaBox<'a, PropertyDefinition<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let index = self
            .current_class()
            .decorators
            .as_ref()
            .and_then(|state| state.tables.decorated_field_index(prop));
        let prop = &mut **prop;
        let private_name = prop.key.private_name();
        let decorators = if self.current_class().decorators.is_some() && !prop.decorators.is_empty()
        {
            Some(StandardDecoratorState::take_element_decorators(
                &mut prop.decorators,
                &prop.key,
                &mut self.insert_before,
                ctx,
            ))
        } else {
            None
        };
        if decorators.is_some() {
            StandardDecoratorState::memoize_decorated_key(
                &mut prop.key,
                &mut self.insert_before,
                ctx,
            );
        }
        let key =
            decorators.as_ref().map(|_| StandardDecoratorState::decorator_key_expr(&prop.key, ctx));
        if let Some(index) = index {
            let value = prop.value.take();
            let state = self.current_class().decorators.as_ref().unwrap();
            prop.value = Some(state.wrap_decorated_field_value(value, index, ctx));
        }
        self.convert_static_property(prop, ctx);
        if let Some(index) = index {
            let mut extra = self
                .current_class()
                .decorators
                .as_ref()
                .unwrap()
                .decorated_field_extra_initializers(index, ctx);
            self.transform_static_initializer(&mut extra, ctx);
            self.insert_expr_after_class(extra, ctx);
        }
        if let (Some(decorators), Some(key)) = (&decorators, key) {
            self.push_decorate_element(
                DecoratorElementKind::Field,
                true,
                key,
                decorators,
                private_name,
                ctx,
            );
        }
    }

    /// Collect decorators before moving a method's key or lowering its private implementation.
    pub(super) fn on_method(
        &mut self,
        method: &mut ArenaBox<'a, MethodDefinition<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        let decorators =
            if self.current_class().decorators.is_some() && !method.decorators.is_empty() {
                let method = &mut **method;
                Some(StandardDecoratorState::take_element_decorators(
                    &mut method.decorators,
                    &method.key,
                    &mut self.insert_before,
                    ctx,
                ))
            } else {
                None
            };
        self.substitute_temp_var_for_method_computed_key(method, ctx);
        if let Some(decorators) = &decorators {
            let kind = self
                .current_class()
                .decorators
                .as_ref()
                .unwrap()
                .tables
                .method_decorator_kind(method);
            let key = StandardDecoratorState::decorator_key_expr(&method.key, ctx);
            self.push_decorate_element(
                kind,
                method.r#static,
                key,
                decorators,
                method.key.private_name(),
                ctx,
            );
        }
        self.convert_private_method(method, ctx)
    }

    /// Surround element extraction with class decorator evaluation and application.
    pub(super) fn on_class_exit(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let details = self.current_class();
        let is_decorated = details.decorators.is_some();
        let mut inferred_name = if !details.is_declaration
            && class.id.is_none()
            && (is_decorated || details.has_auto_accessor)
        {
            self.decorator_state.infer_class_expression_name(class.unstable_address(), ctx)
        } else {
            None
        };
        if is_decorated {
            let name = if self.current_class().is_declaration {
                (class.id.is_none()
                    && matches!(ctx.parent(), Ancestor::ExportDefaultDeclarationDeclaration(_)))
                .then(|| Expression::new_string_literal(SPAN, "default", None, ctx))
            } else if class.decorators.is_empty() {
                None
            } else {
                inferred_name.take()
            };
            self.decorator_state.begin_class_exit(name);
            self.decorator_state.take_class_decorators(class, &mut self.insert_before, ctx);
        }

        let class_methods_count = self.transform_class_elements(class, ctx);

        if is_decorated {
            self.decorator_state.fold_element_decorators_into_heritage(
                class,
                &mut self.insert_before,
                ctx,
            );
            // Apply decorators before running the extracted static initializers.
            let (exprs, class_extras) = self.build_decorator_sequence(class, ctx);
            if self.current_class().is_declaration {
                let stmts = exprs
                    .into_iter()
                    .map(|expr| Statement::new_expression_statement(SPAN, expr, ctx));
                self.insert_after_stmts.splice(class_methods_count..class_methods_count, stmts);
                if let Some(expr) = class_extras {
                    self.insert_after_stmts
                        .push(Statement::new_expression_statement(SPAN, expr, ctx));
                }
            } else {
                self.insert_after_exprs.splice(0..0, exprs);
                if let Some(expr) = class_extras {
                    self.insert_after_exprs.push(expr);
                }
            }
        }
        inferred_name
    }

    pub(super) fn get_or_init_original_class_for_super(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<&BoundIdentifier<'a>> {
        if !self.decorator_state.has_class_decorators() {
            return None;
        }
        let state = self.current_class_mut().decorators.as_mut().unwrap();
        Some(state.get_or_init_original_class_for_super(ctx))
    }

    pub(super) fn push_decorate_element(
        &mut self,
        kind: DecoratorElementKind,
        is_static: bool,
        key: Expression<'a>,
        decorators: &BoundIdentifier<'a>,
        private_name: Option<Ident<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let details = self.classes_stack.last();
        let private = private_name.map(|name| {
            let prop = details.private_props.lowered().unwrap().get(&name).unwrap();
            let brand = if kind == DecoratorElementKind::Field {
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
            PrivateDecoratorAccess { brand, value: prop.binding.clone(), getter, setter }
        });
        let target = if details.is_declaration {
            details.bindings.name.as_ref().or(details.bindings.temp.as_ref())
        } else {
            details.bindings.temp.as_ref()
        }
        .expect("decorated class always has a binding");
        self.decorator_state.push_decorate_element(
            kind,
            is_static,
            key,
            decorators,
            private.as_ref(),
            details.decorators.as_ref().unwrap(),
            target,
            ctx,
        );
    }

    pub(super) fn build_decorator_sequence(
        &mut self,
        class: &Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Vec<Expression<'a>>, Option<Expression<'a>>) {
        let details = self.classes_stack.last();
        let bindings = &details.bindings;
        let target = if details.is_declaration {
            bindings.name.as_ref().or(bindings.temp.as_ref())
        } else {
            bindings.temp.as_ref()
        }
        .expect("decorated class always has a binding");
        self.decorator_state.build_decorator_sequence(
            class,
            details.decorators.as_ref().unwrap(),
            &ClassDecoratorBindings {
                target,
                declaration: if details.is_declaration { bindings.name.as_ref() } else { None },
                temporary: bindings.temp.as_ref(),
                static_brand: bindings.decorator_static_brand.as_ref(),
            },
            ctx,
        )
    }
}
