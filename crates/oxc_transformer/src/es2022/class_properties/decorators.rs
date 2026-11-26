//! Integration of standard decorators with class bindings and private-element lowering.

use oxc_allocator::{ArenaBox, GetAddress, TakeIn, UnstableAddress};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Ident;
use oxc_syntax::scope::ScopeId;
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::class_lowering::ClassLowering,
    context::TraverseCtx,
    decorator::standard::{
        AccessorKeys, ClassDecoratorBindings, ClassDecoratorState, ClassDecoratorTables,
        DecoratedElement, DecoratorElementKind, PrivateDecoratorAccess, StandardDecoratorState,
    },
    utils::{
        ast_builder::create_to_property_key, decorated_class::class_body_has_static_auto_accessor,
    },
};

use super::{
    ClassBindings, ClassIdentity, ClassProperties, IdentIndexMap, PrivateProp,
    class_details::ClassTemps, utils::exprs_into_stmts,
};

/// Decorator setup produced before the class is pushed onto the class stack.
pub(super) struct DecoratorBodyEntry<'a> {
    pub state: Option<Box<ClassDecoratorState<'a>>>,
    pub static_accessor_binding: Option<BoundIdentifier<'a>>,
    /// Where the class declares its temporaries.
    pub temps: ClassTemps,
}

impl<'a> DecoratorBodyEntry<'a> {
    pub fn is_decorated(&self) -> bool {
        self.state.is_some()
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
        let static_brand =
            private_props.values().any(|prop| prop.is_static && prop.is_method()).then(|| {
                let brand = self.temps.create_binding("staticBrand", ctx);
                self.temps.declare(&brand, ctx);
                brand
            });
        bindings.identity = ClassIdentity::Decorated { static_brand };
    }
}

impl<'a> ClassProperties<'a> {
    /// Capture the names of anonymous classes in field and accessor initializers which standard
    /// decorator lowering names, before their keys are rewritten.
    pub(super) fn capture_initializer_names(
        &mut self,
        body: &mut ClassBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.decorator_state.enabled() {
            return;
        }
        for element in &mut body.body {
            let (key, value) = match element {
                ClassElement::PropertyDefinition(prop) => {
                    let prop = &mut **prop;
                    (&mut prop.key, &prop.value)
                }
                ClassElement::AccessorProperty(prop) => {
                    let prop = &mut **prop;
                    (&mut prop.key, &prop.value)
                }
                _ => continue,
            };
            if let Some(value) = value {
                self.decorator_state.capture_class_initializer_name(key, value, ctx);
            }
        }
    }

    /// Capture names before rewriting keys, then prepare this class's decorator state.
    pub(super) fn on_body_entry(
        &mut self,
        body: &mut ClassBody<'a>,
        class_name_binding: Option<&BoundIdentifier<'a>>,
        class_scope_id: ScopeId,
        lowering: ClassLowering,
        ctx: &mut TraverseCtx<'a>,
    ) -> DecoratorBodyEntry<'a> {
        let standard = self.decorator_state.enabled();
        let is_decorated = standard && lowering.has_decorators();
        let has_auto_accessor = standard && lowering.has_auto_accessor();
        let temps = ClassTemps::new(standard && lowering.handles_standard_decorators(), ctx);
        self.capture_initializer_names(body, ctx);
        debug_assert!(lowering.lowers_fields());
        let static_accessor_binding = (has_auto_accessor
            && class_body_has_static_auto_accessor(body))
        .then(|| temps.create_class_binding(class_name_binding, ctx));
        if is_decorated {
            StandardDecoratorState::report_decorated_private_method_writes(body, ctx);
            StandardDecoratorState::coerce_decorated_keys(body, ctx);
        }
        let mut tables = ClassDecoratorTables::default();
        if temps.is_per_evaluation() {
            self.decorator_state.lower_auto_accessors(
                body,
                class_scope_id,
                static_accessor_binding.as_ref(),
                AccessorKeys::Lowered,
                is_decorated.then_some(&mut tables),
                ctx,
            );
        }
        let state = if is_decorated {
            Some(Box::new(StandardDecoratorState::prepare_class_decorators(body, tables, ctx)))
        } else {
            None
        };

        DecoratorBodyEntry { state, static_accessor_binding, temps }
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

    /// Entry phase: wrap a decorated field's initializer and move it into the constructor.
    pub(super) fn on_instance_field_entry(
        &mut self,
        prop: &mut ArenaBox<'a, PropertyDefinition<'a>>,
        instance_inits: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let extra_initializers =
            self.classes_stack.last_mut().decorators.as_mut().and_then(|state| {
                state.tables.record_instance_field_for_decorators(prop, ctx);
                let index = state.tables.decorated_field_index(prop)?;
                Some(state.wrap_decorated_field(&mut prop.value, index, ctx))
            });
        self.convert_instance_property(prop, instance_inits, ctx);
        instance_inits.extend(extra_initializers);
    }

    /// Exit phase: collect the field's decorators and computed key.
    pub(super) fn on_instance_field_exit(
        &mut self,
        prop: &mut ArenaBox<'a, PropertyDefinition<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let address = prop.address();
        let prop = &mut **prop;
        if !prop.decorators.is_empty()
            && let Some(state) = self.classes_stack.last_mut().decorators.as_mut()
        {
            let decorators = StandardDecoratorState::take_element_decorators(
                &mut prop.decorators,
                &prop.key,
                &mut self.insert_before,
                ctx,
            );
            let key = state
                .tables
                .take_instance_key(address)
                .unwrap_or_else(|| StandardDecoratorState::decorator_key_expr(&prop.key, ctx));
            self.push_decorate_element(
                DecoratedElement {
                    kind: DecoratorElementKind::Field,
                    is_static: false,
                    key,
                    decorators: Some(&decorators),
                    public_descriptor: None,
                    span: prop.span,
                },
                prop.key.private_name(),
                ctx,
            );
        }
        if prop.key.is_computed() {
            self.extract_instance_prop_computed_key(prop, ctx);
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
        let span = prop.span;
        let decorators = if self.current_class().decorators.is_some() && !prop.decorators.is_empty()
        {
            let decorators = StandardDecoratorState::take_element_decorators(
                &mut prop.decorators,
                &prop.key,
                &mut self.insert_before,
                ctx,
            );
            StandardDecoratorState::memoize_decorated_key(
                &mut prop.key,
                &mut self.insert_before,
                ctx,
            );
            let key = StandardDecoratorState::decorator_key_expr(&prop.key, ctx);
            Some((decorators, key))
        } else {
            None
        };
        // `index` is only `Some` for a decorated class.
        let extra_initializers = index
            .zip(self.current_class().decorators.as_deref())
            .map(|(index, state)| state.wrap_decorated_field(&mut prop.value, index, ctx));
        self.convert_static_property(prop, ctx);
        if let Some(mut extra) = extra_initializers {
            self.transform_static_initializer(&mut extra, ctx);
            self.insert_expr_after_class(extra, ctx);
        }
        if let Some((decorators, key)) = decorators {
            self.push_decorate_element(
                DecoratedElement {
                    kind: DecoratorElementKind::Field,
                    is_static: true,
                    key,
                    decorators: Some(&decorators),
                    public_descriptor: None,
                    span,
                },
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
        let capture = self
            .current_class_mut()
            .decorators
            .as_mut()
            .and_then(|state| state.tables.take_public_key(method.address()));
        if let Some(capture) = &capture {
            if capture.accessor_setter {
                StandardDecoratorState::replace_method_key(method, &capture.binding, ctx);
                return None;
            }
            // The temporary symbol no longer performs the original key's coercion.
            if decorators.is_none()
                && method.key.is_computed()
                && let Some(key) = method.key.as_expression_mut()
            {
                *key = create_to_property_key(key.take_in(ctx), ctx);
            }
        }
        self.substitute_temp_var_for_method_computed_key(method, ctx);
        if decorators.is_some() || capture.is_some() {
            // Either is only `Some` for a decorated class.
            let Some(state) = self.current_class().decorators.as_deref() else { unreachable!() };
            let kind = state.tables.method_decorator_kind(method);
            let key = StandardDecoratorState::decorator_key_expr(&method.key, ctx);
            let private_name = method.key.private_name();
            let descriptor = capture.map(|capture| {
                let captured_key = StandardDecoratorState::decorator_key_expr(&method.key, ctx);
                self.decorator_state.capture_public_method(
                    method,
                    kind,
                    captured_key,
                    &capture.binding,
                    &mut self.insert_before,
                    ctx,
                )
            });
            self.push_decorate_element(
                DecoratedElement {
                    kind,
                    is_static: method.r#static,
                    key,
                    decorators: decorators.as_ref(),
                    public_descriptor: descriptor,
                    span: method.span,
                },
                private_name,
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
        let mut inferred_name =
            if !details.is_declaration && class.id.is_none() && details.temps.is_per_evaluation() {
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
                let stmts = exprs_into_stmts(exprs, &ctx.ast);
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

    pub(super) fn push_decorate_element(
        &mut self,
        element: DecoratedElement<'_, 'a>,
        private_name: Option<Ident<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let (kind, is_static) = (element.kind, element.is_static);
        let details = self.classes_stack.last();
        let private = private_name.map(|name| {
            let prop = details.private_props.lowered().unwrap().get(&name).unwrap();
            let brand = if kind == DecoratorElementKind::Field {
                prop.binding.clone()
            } else if is_static {
                details.bindings.identity.static_brand().unwrap().clone()
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
        let state =
            details.decorators.as_deref().expect("only decorated classes decorate elements");
        self.decorator_state.push_decorate_element(element, private.as_ref(), state, target, ctx);
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
                static_brand: bindings.identity.static_brand(),
            },
            ctx,
        )
    }
}
