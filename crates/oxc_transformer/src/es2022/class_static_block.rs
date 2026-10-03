//! ES2022: Class Static Block
//!
//! Fold static blocks into following static field initializers. Defer trailing blocks through
//! a closure initialized inside the class and called after the class has been initialized.
//! This avoids adding private fields after a block has made the class non-extensible.
//!
//! > This plugin is included in `preset-env`, in ES2022
//!
//! ## Example
//!
//! Input:
//! ```js
//! class C {
//!   static {
//!     foo();
//!   }
//!   static {
//!     foo();
//!     bar();
//!   }
//! }
//! ```
//!
//! Output:
//! ```js
//! var _staticBlock;
//! let C = (class C {
//!   static #_ = _staticBlock = () => (foo(), (() => {
//!     foo();
//!     bar();
//!   })(), this);
//! }, _staticBlock());
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-class-static-block](https://babel.dev/docs/babel-plugin-transform-class-static-block).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-class-static-block>
//! * Class static initialization blocks TC39 proposal: <https://github.com/tc39/proposal-class-static-block>

use std::collections::hash_map::Entry;

use itoa::Buffer as ItoaBuffer;

use oxc_allocator::{Address, ArenaBox, ArenaVec, GetAddress, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::{
    operator::{BinaryOperator, LogicalOperator},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    state::TransformState,
    utils::ast_builder::{
        create_assignment, wrap_expression_in_arrow_function_iife,
        wrap_statements_in_arrow_function_iife,
    },
};

pub struct ClassStaticBlock<'a> {
    deferred: Vec<Option<BoundIdentifier<'a>>>,
    class_expression_deferred: Vec<Option<BoundIdentifier<'a>>>,
    class_names: FxHashMap<Address, InferredName<'a>>,
    scoped_classes: FxHashSet<Address>,
}

/// A property key that has already been evaluated and converted to a string or symbol.
enum InferredName<'a> {
    Static(Str<'a>),
    Computed(BoundIdentifier<'a>),
    Captured(BoundIdentifier<'a>, Str<'a>),
}

impl<'a> InferredName<'a> {
    fn expression(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match self {
            Self::Static(name) => Expression::new_string_literal(SPAN, *name, None, ctx),
            Self::Computed(binding) => binding.create_read_expression(ctx),
            Self::Captured(class, key) => Expression::new_private_field_expression(
                SPAN,
                class.create_read_expression(ctx),
                PrivateIdentifier::new(SPAN, *key, ctx),
                false,
                ctx,
            ),
        }
    }
}

impl ClassStaticBlock<'_> {
    pub fn new() -> Self {
        Self {
            deferred: Vec::new(),
            class_expression_deferred: Vec::new(),
            class_names: FxHashMap::default(),
            scoped_classes: FxHashSet::default(),
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ClassStaticBlock<'a> {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::ClassDeclaration(class) if Self::needs_declaration_initializer(class) => {
                let declaration = Self::class_declaration_initializer(class.take_in(ctx), ctx);
                *stmt = Statement::VariableDeclaration(declaration);
            }
            Statement::ExportDeclaration(export) => {
                if let Declaration::ClassDeclaration(class) = &mut export.declaration
                    && Self::needs_declaration_initializer(class)
                {
                    let declaration = Self::class_declaration_initializer(class.take_in(ctx), ctx);
                    export.declaration = Declaration::VariableDeclaration(declaration);
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                let ExportDefaultDeclarationKind::ClassDeclaration(class) = &mut export.declaration
                else {
                    return;
                };
                if !Self::needs_declaration_initializer(class) {
                    return;
                }
                let mut class = class.take_in(ctx);
                if let Some(id) = &class.id {
                    let binding = BoundIdentifier::from_binding_ident(id);
                    let declaration = Self::class_declaration_initializer(class, ctx);
                    let specifier = ExportSpecifier::new(
                        SPAN,
                        ModuleExportName::IdentifierReference(binding.create_read_reference(ctx)),
                        ModuleExportName::new_identifier_name(SPAN, "default", ctx),
                        ImportOrExportKind::Value,
                        ctx,
                    );
                    let export = Statement::new_export_named_declaration(
                        SPAN,
                        [specifier],
                        ImportOrExportKind::Value,
                        ctx,
                    );
                    ctx.state.statement_injector.insert_after(&declaration.address(), export);
                    *stmt = Statement::VariableDeclaration(declaration);
                } else {
                    class.r#type = ClassType::ClassExpression;
                    export.declaration =
                        ExportDefaultDeclarationKind::ClassExpression(ctx.alloc(class));
                }
            }
            _ => {}
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            Expression::ObjectExpression(object) => {
                for prop in &mut object.properties {
                    let ObjectPropertyKind::ObjectProperty(prop) = prop else { continue };
                    let prop = &mut **prop;
                    // A prototype setter does not perform NamedEvaluation.
                    if !prop.computed && prop.key.is_specific_static_name("__proto__") {
                        continue;
                    }
                    if let Some(class) = Self::anonymous_deferred_class(&prop.value) {
                        let address = class.body.address();
                        let name = Self::property_name(&mut prop.key, &mut prop.computed, ctx);
                        self.class_names.insert(address, name);
                    }
                }
            }
            Expression::ClassExpression(class)
                if class.id.is_none()
                    && (Self::has_trailing_blocks(class) || Self::needs_name_scope(class)) =>
            {
                let already_named = self.class_names.contains_key(&class.body.address());
                let name = ctx
                    .ancestors()
                    .find_map(|ancestor| match ancestor {
                        Ancestor::ParenthesizedExpressionExpression(_) => None,
                        Ancestor::VariableDeclaratorInit(decl) => {
                            Some(decl.id().get_identifier_name().map(Str::from))
                        }
                        Ancestor::AssignmentPatternRight(pattern) => {
                            Some(pattern.left().get_identifier_name().map(Str::from))
                        }
                        Ancestor::FormalParameterInitializer(param) => {
                            Some(param.pattern().get_identifier_name().map(Str::from))
                        }
                        Ancestor::AssignmentExpressionRight(assign)
                            if assign.operator().is_assign() || assign.operator().is_logical() =>
                        {
                            Some(Self::assignment_name(assign.left()))
                        }
                        Ancestor::AssignmentTargetWithDefaultInit(target) => {
                            Some(Self::assignment_name(target.binding()))
                        }
                        Ancestor::AssignmentTargetPropertyIdentifierInit(target) => {
                            Some(Some(Str::from(target.binding().name)))
                        }
                        Ancestor::ExportDefaultDeclarationDeclaration(_) => {
                            Some(Some(Str::from("default")))
                        }
                        _ => Some(None),
                    })
                    .flatten();
                if !already_named && let Some(name) = name {
                    self.class_names.insert(class.body.address(), InferredName::Static(name));
                }
            }
            _ => {}
        }
        if let Expression::ClassExpression(class) = expr
            && Self::needs_name_scope(class)
        {
            if ContainsSuspension::check(class) {
                // Await/yield must stay in the surrounding function. Store names used by
                // instance initializers in the class itself instead of wrapping evaluation.
                self.capture_suspended_names(class, ctx);
            } else if self.scoped_classes.insert(class.body.address()) {
                // An expression can run repeatedly without re-entering its enclosing block,
                // e.g. in a loop test. Capture names in a fresh class-evaluation scope.
                *expr = wrap_expression_in_arrow_function_iife(expr.take_in(ctx), ctx);
            }
        }
        // Parameter initializers have no statement list in which to declare per-call temps.
        if matches!(ctx.parent(), Ancestor::FormalParameterInitializer(_))
            && ContainsStaticBlocks::check(expr)
        {
            *expr = wrap_expression_in_arrow_function_iife(expr.take_in(ctx), ctx);
        }
    }

    fn enter_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let wrap = !prop.r#static && prop.value.as_ref().is_some_and(ContainsStaticBlocks::check);
        let class = prop.value.as_ref().and_then(|value| {
            if wrap {
                match value.without_parentheses() {
                    Expression::ClassExpression(class) if class.id.is_none() => Some(&**class),
                    _ => None,
                }
            } else {
                Self::anonymous_deferred_class(value)
            }
        });
        if let Some(class) = class {
            let address = class.body.address();
            // A folded initializer has already supplied the original name.
            if let Entry::Vacant(entry) = self.class_names.entry(address) {
                let name = Self::property_name(&mut prop.key, &mut prop.computed, ctx);
                entry.insert(name);
            }
        }
        // Instance initializers run on every construction, not when the enclosing class is
        // evaluated. Give their temporaries a fresh scope while retaining lexical this/super.
        if wrap {
            prop.value =
                Some(wrap_expression_in_arrow_function_iife(prop.value.take().unwrap(), ctx));
        }
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        // Loop through class body elements and:
        // 1. Find if there are any `StaticBlock`s.
        // 2. Collate list of private keys matching `#_` or `#_[1-9]...`.
        //
        // Don't collate private keys list conditionally only if a static block is found, as usually
        // there will be no matching private keys, so those checks are cheap and will not allocate.
        let mut has_static_block = false;
        let mut keys = Keys::default();
        for element in &body.body {
            let key = match element {
                ClassElement::StaticBlock(_) => {
                    has_static_block = true;
                    continue;
                }
                ClassElement::MethodDefinition(def) => &def.key,
                ClassElement::PropertyDefinition(def) => &def.key,
                ClassElement::AccessorProperty(def) => &def.key,
                ClassElement::TSIndexSignature(_) => continue,
            };

            if let PropertyKey::PrivateIdentifier(id) = key {
                keys.reserve(id.name.as_str());
            }
        }

        let has_static_property = body.body.iter().any(
            |element| matches!(element, ClassElement::PropertyDefinition(prop) if prop.r#static),
        );

        if has_static_property {
            let mut pending = Vec::new();
            let mut last_static_property = None;

            for (index, element) in body.body.iter_mut().enumerate() {
                match element {
                    ClassElement::StaticBlock(block) => {
                        pending.push(Self::convert_block_to_expression(block, ctx));
                    }
                    ClassElement::PropertyDefinition(prop) if prop.r#static => {
                        last_static_property = Some(index);
                        if !pending.is_empty() {
                            self.prepend_to_initializer(
                                prop,
                                std::mem::take(&mut pending).into_iter(),
                                ctx,
                            );
                        }
                    }
                    _ => {}
                }
            }

            let deferred = if pending.is_empty() {
                None
            } else {
                let index = last_static_property.expect("static property must exist");
                let expressions = std::mem::take(&mut pending);
                let is_class_expression = matches!(
                    ctx.parent(),
                    Ancestor::ClassBody(class) if *class.r#type() == ClassType::ClassExpression
                );
                let (binding, assignment) =
                    Self::create_deferred_initializer(expressions, is_class_expression, ctx);
                let ClassElement::PropertyDefinition(prop) = &mut body.body[index] else {
                    unreachable!();
                };
                self.append_deferred_initializer(prop, assignment, ctx);
                Some(binding)
            };

            body.body.retain(|element| !matches!(element, ClassElement::StaticBlock(_)));
            self.deferred.push(deferred);
            return;
        }

        if !has_static_block {
            self.deferred.push(None);
            return;
        }

        let expressions = body
            .body
            .iter_mut()
            .filter_map(|element| match element {
                ClassElement::StaticBlock(block) => {
                    Some(Self::convert_block_to_expression(block, ctx))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let is_class_expression = matches!(
            ctx.parent(),
            Ancestor::ClassBody(class) if *class.r#type() == ClassType::ClassExpression
        );
        let (binding, assignment) =
            Self::create_deferred_initializer(expressions, is_class_expression, ctx);
        let key = keys.get_unique(ctx);
        let key = PropertyKey::new_private_identifier(SPAN, key, ctx);
        let property = ClassElement::new_property_definition(
            SPAN,
            PropertyDefinitionType::PropertyDefinition,
            [],
            key,
            None,
            Some(assignment),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            None,
            ctx,
        );
        body.body.retain(|element| !matches!(element, ClassElement::StaticBlock(_)));
        body.body.push(property);
        self.deferred.push(Some(binding));
    }

    fn exit_class(&mut self, class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        let deferred = self.deferred.pop().expect("class body must be entered first");
        if class.r#type == ClassType::ClassExpression {
            self.class_expression_deferred.push(deferred);
        } else {
            debug_assert!(
                deferred.is_none(),
                "declarations with trailing blocks become initializers"
            );
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ClassExpression(class) = expr else { return };
        self.scoped_classes.remove(&class.body.address());
        let name = self.class_names.remove(&class.body.address());
        let deferred = self.class_expression_deferred.pop().flatten();
        if let Some(name) = name {
            // Let the engine perform NamedEvaluation before static initialization. Setting
            // `.name` afterwards is too late, and would overwrite a user-defined static name.
            // A computed key also handles symbols and the special `__proto__` spelling.
            let key = PropertyKey::from(name.expression(ctx));
            let computed =
                !matches!(&name, InferredName::Static(name) if name.as_str() != "__proto__");
            let property = ObjectPropertyKind::new_object_property(
                SPAN,
                PropertyKind::Init,
                key,
                expr.take_in(ctx),
                false,
                false,
                computed,
                ctx,
            );
            let object = Expression::new_object_expression(SPAN, [property], ctx);
            *expr = Expression::new_computed_member_expression(
                SPAN,
                object,
                name.expression(ctx),
                false,
                ctx,
            );
        }
        if let Some(deferred) = deferred {
            let class = expr.take_in(ctx);
            let call = Self::create_deferred_call(&deferred, ctx);
            *expr = Expression::new_sequence_expression(SPAN, [class, call], ctx);
        }
    }
}

impl<'a> ClassStaticBlock<'a> {
    /// Keep the outer declaration in its TDZ until all static initialization finishes.
    /// The class expression's separate inner binding remains available inside the class.
    fn class_declaration_initializer(
        mut class: Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaBox<'a, VariableDeclaration<'a>> {
        let id = class.id.take().expect("named declaration");
        let outer_symbol = id.symbol_id();
        let inner =
            ctx.generate_spanned_binding(id.name, id.span, class.scope_id(), SymbolFlags::Class);
        ClassBindingRebinder { outer_symbol, inner_symbol: inner.symbol_id, ctx }
            .visit_class(&class);
        *ctx.scoping_mut().symbol_flags_mut(outer_symbol) = SymbolFlags::BlockScopedVariable;
        class.id = Some(inner.create_spanned_binding_identifier(id.span, ctx));
        class.r#type = ClassType::ClassExpression;
        let span = class.span;
        let declaration = VariableDeclarator::new(
            span,
            BindingPattern::BindingIdentifier(ctx.alloc(id)),
            None,
            Some(Expression::ClassExpression(ctx.alloc(class))),
            false,
            ctx,
        );
        VariableDeclaration::boxed(span, VariableDeclarationKind::Let, [declaration], false, ctx)
    }

    fn needs_declaration_initializer(class: &Class<'_>) -> bool {
        Self::has_trailing_blocks(class)
            || (Self::needs_name_scope(class) && ContainsSuspension::check(class))
    }

    /// A suspending class cannot be moved into an arrow. Its computed keys are evaluated
    /// in one generator/async invocation, but instance initializers can outlive subsequent
    /// loop iterations. Capture those keys before any user static initialization runs.
    fn capture_suspended_names(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let indices = class
            .body
            .body
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                let ClassElement::PropertyDefinition(prop) = element else { return None };
                if prop.r#static || !prop.computed {
                    return None;
                }
                let Expression::ClassExpression(value) = prop.value.as_ref()?.without_parentheses()
                else {
                    return None;
                };
                (value.id.is_none() && ContainsStaticBlocks::check(prop.value.as_ref().unwrap()))
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        if indices.is_empty() {
            return;
        }

        let anonymous = class.id.is_none();
        let binding = if let Some(id) = &class.id {
            BoundIdentifier::from_binding_ident(id)
        } else {
            let binding = ctx.generate_uid("Class", class.scope_id(), SymbolFlags::Class);
            class.id = Some(binding.create_binding_identifier(ctx));
            binding
        };
        let mut keys = Keys::default();
        for element in &class.body.body {
            if let Some(PropertyKey::PrivateIdentifier(key)) = element.property_key() {
                keys.reserve(key.name.as_str());
            }
        }

        let mut captures = ArenaVec::new_in(ctx);
        for index in indices {
            let ClassElement::PropertyDefinition(prop) = &mut class.body.body[index] else {
                unreachable!();
            };
            let prop = &mut **prop;
            let name = Self::property_name(&mut prop.key, &mut prop.computed, ctx);
            let Expression::ClassExpression(value) =
                prop.value.as_ref().unwrap().without_parentheses()
            else {
                unreachable!();
            };
            let key = keys.get_unique(ctx);
            self.class_names
                .insert(value.body.address(), InferredName::Captured(binding.clone(), key));
            captures.push(ClassElement::new_property_definition(
                SPAN,
                PropertyDefinitionType::PropertyDefinition,
                [],
                PropertyKey::new_private_identifier(SPAN, key, ctx),
                None,
                Some(name.expression(ctx)),
                false,
                true,
                false,
                false,
                false,
                false,
                false,
                None,
                ctx,
            ));
        }

        if anonymous {
            let name = self
                .class_names
                .remove(&class.body.address())
                .unwrap_or(InferredName::Static(Str::from("")));
            // An explicit inner binding must not change the original inferred name. Static
            // methods named `name` suppress NamedEvaluation, including computed methods.
            let mut condition = None;
            let mut has_name_method = false;
            for element in &mut class.body.body {
                let ClassElement::MethodDefinition(method) = element else { continue };
                let method = &mut **method;
                if !method.r#static || matches!(method.key, PropertyKey::PrivateIdentifier(_)) {
                    continue;
                }
                if method.key.is_specific_static_name("name") {
                    has_name_method = true;
                    break;
                }
                if method.computed {
                    let key = Self::property_name(&mut method.key, &mut method.computed, ctx);
                    let check = Expression::new_binary_expression(
                        SPAN,
                        key.expression(ctx),
                        BinaryOperator::StrictInequality,
                        Expression::new_string_literal(SPAN, "name", None, ctx),
                        ctx,
                    );
                    condition = Some(if let Some(previous) = condition {
                        Expression::new_logical_expression(
                            SPAN,
                            previous,
                            LogicalOperator::And,
                            check,
                            ctx,
                        )
                    } else {
                        check
                    });
                }
            }
            if !has_name_method {
                let rename = helper_call_expr(
                    Helper::SetFunctionName,
                    ArenaVec::from_array_in(
                        [
                            Argument::from(Expression::new_this_expression(SPAN, ctx)),
                            Argument::from(name.expression(ctx)),
                        ],
                        ctx,
                    ),
                    ctx,
                );
                let rename = if let Some(condition) = condition {
                    Expression::new_logical_expression(
                        SPAN,
                        condition,
                        LogicalOperator::And,
                        rename,
                        ctx,
                    )
                } else {
                    rename
                };
                let ClassElement::PropertyDefinition(first) = &mut captures[0] else {
                    unreachable!()
                };
                first.value = Some(Expression::new_sequence_expression(
                    SPAN,
                    [rename, first.value.take().unwrap()],
                    ctx,
                ));
            }
        }
        captures.append(&mut class.body.body);
        class.body.body = captures;
    }

    fn assignment_name(target: &AssignmentTarget<'a>) -> Option<Str<'a>> {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => Some(ident.name.into()),
            _ => None,
        }
    }

    fn has_trailing_blocks(class: &Class<'_>) -> bool {
        for element in class.body.body.iter().rev() {
            match element {
                ClassElement::StaticBlock(_) => return true,
                ClassElement::PropertyDefinition(prop) if prop.r#static => return false,
                _ => {}
            }
        }
        false
    }

    fn anonymous_deferred_class<'e>(expr: &'e Expression<'a>) -> Option<&'e Class<'a>> {
        match expr.without_parentheses() {
            Expression::ClassExpression(class)
                if class.id.is_none()
                    && (Self::has_trailing_blocks(class) || Self::needs_name_scope(class)) =>
            {
                Some(class)
            }
            _ => None,
        }
    }

    fn needs_name_scope(class: &Class<'_>) -> bool {
        let has_blocks =
            class.body.body.iter().any(|element| matches!(element, ClassElement::StaticBlock(_)));
        class.body.body.iter().any(|element| {
            let ClassElement::PropertyDefinition(prop) = element else { return false };
            if matches!(
                prop.key,
                PropertyKey::StaticIdentifier(_)
                    | PropertyKey::PrivateIdentifier(_)
                    | PropertyKey::StringLiteral(_)
            ) {
                return false;
            }
            prop.value.as_ref().is_some_and(|value| {
                value.is_anonymous_function_definition()
                    && ((prop.r#static && has_blocks) || ContainsStaticBlocks::check(value))
            })
        })
    }

    fn property_name(
        key: &mut PropertyKey<'a>,
        computed: &mut bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> InferredName<'a> {
        let name = match key {
            PropertyKey::PrivateIdentifier(ident) => {
                Some(Str::from_strs_array_in(["#", ident.name.as_str()], ctx))
            }
            PropertyKey::StaticIdentifier(ident) => Some(Str::from(ident.name)),
            PropertyKey::StringLiteral(lit) => Some(lit.value),
            _ => None,
        };
        if let Some(name) = name {
            return InferredName::Static(name);
        }
        // Capture even an identifier: later keys/initializers can mutate its value. Coercion
        // belongs here too, since ToPropertyKey can have side effects or produce a symbol.
        *computed = true;
        let value = key.take_in(ctx).into_expression();
        let value = helper_call_expr(
            Helper::ToPropertyKey,
            ArenaVec::from_value_in(Argument::from(value), ctx),
            ctx,
        );
        let binding =
            ctx.generate_uid("key", ctx.current_block_scope_id(), SymbolFlags::BlockScopedVariable);
        ctx.state.var_declarations.insert_let(&binding, None, &ctx.ast);
        *key = PropertyKey::from(create_assignment(&binding, value, SPAN, ctx));
        InferredName::Computed(binding)
    }

    fn prepend_to_initializer(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        expressions: impl Iterator<Item = Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut expressions = expressions.collect::<Vec<_>>();
        expressions.push(self.take_initializer(prop, ctx));
        prop.value = Some(Expression::new_sequence_expression(
            SPAN,
            ArenaVec::from_iter_in(expressions, ctx),
            ctx,
        ));
    }

    fn take_initializer(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Some(mut value) = prop.value.take() {
            if value.is_anonymous_function_definition() {
                let name = Self::property_name(&mut prop.key, &mut prop.computed, ctx);
                if let Expression::ClassExpression(class) = value.without_parentheses() {
                    self.class_names.insert(class.body.address(), name);
                } else {
                    let name = name.expression(ctx);
                    value = helper_call_expr(
                        Helper::SetFunctionName,
                        ArenaVec::from_array_in([Argument::from(value), Argument::from(name)], ctx),
                        ctx,
                    );
                }
            }
            value
        } else {
            Expression::new_void_0(SPAN, ctx)
        }
    }

    /// Publish the deferred closure only after the last initializer has returned. A recursive
    /// evaluation of this class may otherwise overwrite the closure before we call it.
    fn append_deferred_initializer(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        assignment: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let value = self.take_initializer(prop, ctx);
        let scope_id = ctx.insert_scope_below_expression(
            &assignment,
            ScopeFlags::Function | ScopeFlags::Arrow | ScopeFlags::StrictMode,
        );
        let binding = ctx.generate_uid("value", scope_id, SymbolFlags::FunctionScopedVariable);
        let param = FormalParameter::new(
            SPAN,
            [],
            binding.create_binding_pattern(ctx),
            None,
            None,
            false,
            None,
            false,
            false,
            ctx,
        );
        let params = FormalParameters::boxed(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            [param],
            None,
            ctx,
        );
        let body = Expression::new_sequence_expression(
            SPAN,
            [assignment, binding.create_read_expression(ctx)],
            ctx,
        );
        let arrow = Expression::new_arrow_function_expression_with_scope_id_and_pure_and_pife(
            SPAN,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::from(body),
            scope_id,
            false,
            false,
            ctx,
        );
        prop.value = Some(Expression::new_call_expression(
            SPAN,
            arrow,
            None,
            [Argument::from(value)],
            false,
            ctx,
        ));
    }

    fn create_deferred_initializer(
        mut expressions: Vec<Expression<'a>>,
        returns_class: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> (BoundIdentifier<'a>, Expression<'a>) {
        let binding = ctx.generate_uid_in_current_hoist_scope("staticBlock");
        ctx.state.var_declarations.insert_var(&binding, &ctx.ast);
        if returns_class {
            expressions.push(Expression::new_this_expression(SPAN, ctx));
        }
        let body = if expressions.len() == 1 {
            expressions.pop().unwrap()
        } else {
            Expression::new_sequence_expression(SPAN, ArenaVec::from_iter_in(expressions, ctx), ctx)
        };
        let scope_id = ctx.insert_scope_below_expression(
            &body,
            ScopeFlags::Function | ScopeFlags::Arrow | ScopeFlags::StrictMode,
        );
        let params = FormalParameters::boxed(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            [],
            None,
            ctx,
        );
        let arrow = Expression::new_arrow_function_expression_with_scope_id_and_pure_and_pife(
            SPAN,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::from(body),
            scope_id,
            false,
            false,
            ctx,
        );
        (binding.clone(), create_assignment(&binding, arrow, SPAN, ctx))
    }

    fn create_deferred_call(
        binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        Expression::new_call_expression(
            SPAN,
            binding.create_read_expression(ctx),
            None,
            [],
            false,
            ctx,
        )
    }

    /// Convert static block to expression which will be value of private field.
    /// `static { foo }` -> `foo`
    /// `static { foo; bar; }` -> `(() => { foo; bar; })()`
    fn convert_block_to_expression(
        block: &mut StaticBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let scope_id = block.scope_id();

        // If block contains only a single `ExpressionStatement`, no need to wrap in an IIFE.
        // `static { foo }` -> `foo`
        // TODO(improve-on-babel): If block has no statements, could remove it entirely.
        let stmts = &mut block.body;
        if stmts.len() == 1
            && let Statement::ExpressionStatement(stmt) = stmts.first_mut().unwrap()
        {
            return Self::convert_block_with_single_expression_to_expression(
                &mut stmt.expression,
                scope_id,
                ctx,
            );
        }

        // Convert block to arrow function IIFE.
        // `static { foo; bar; }` -> `(() => { foo; bar; })()`

        // Re-use the static block's scope for the arrow function.
        // Always strict mode since we're in a class.
        *ctx.scoping_mut().scope_flags_mut(scope_id) =
            ScopeFlags::Function | ScopeFlags::Arrow | ScopeFlags::StrictMode;
        wrap_statements_in_arrow_function_iife(stmts.take_in(ctx), scope_id, block.span, ctx)
    }

    /// Convert static block to expression which will be value of private field,
    /// where the static block contains only a single expression.
    /// `static { foo }` -> `foo`
    fn convert_block_with_single_expression_to_expression(
        expr: &mut Expression<'a>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let expr = expr.take_in(ctx);

        // Remove the scope for the static block from the scope chain
        ctx.remove_scope_for_expression(scope_id, &expr);

        expr
    }
}

struct ClassBindingRebinder<'a, 'ctx> {
    outer_symbol: SymbolId,
    inner_symbol: SymbolId,
    ctx: &'ctx mut TraverseCtx<'a>,
}

#[derive(Default)]
struct ContainsStaticBlocks(bool);

impl ContainsStaticBlocks {
    fn check(expr: &Expression<'_>) -> bool {
        let mut visitor = Self::default();
        visitor.visit_expression(expr);
        visitor.0
    }
}

impl<'a> Visit<'a> for ContainsStaticBlocks {
    fn visit_static_block(&mut self, _block: &StaticBlock<'a>) {
        self.0 = true;
    }
    fn visit_function(&mut self, _function: &Function<'a>, _flags: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
}

#[derive(Default)]
struct ContainsSuspension(bool);

impl ContainsSuspension {
    fn check(class: &Class<'_>) -> bool {
        let mut visitor = Self::default();
        visitor.visit_class(class);
        visitor.0
    }
}

impl<'a> Visit<'a> for ContainsSuspension {
    fn visit_await_expression(&mut self, _expr: &AwaitExpression<'a>) {
        self.0 = true;
    }
    fn visit_yield_expression(&mut self, _expr: &YieldExpression<'a>) {
        self.0 = true;
    }
    fn visit_function(&mut self, _function: &Function<'a>, _flags: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
}

impl<'a> Visit<'a> for ClassBindingRebinder<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let reference_id = ident.reference_id();
        let scoping = self.ctx.scoping_mut();
        if scoping.get_reference(reference_id).symbol_id() == Some(self.outer_symbol) {
            scoping.get_reference_mut(reference_id).set_symbol_id(self.inner_symbol);
            scoping.delete_resolved_reference(self.outer_symbol, reference_id);
            scoping.add_resolved_reference(self.inner_symbol, reference_id);
        }
    }
}

/// Store of private identifier keys matching `#_` or `#_[1-9]...`.
///
/// Most commonly there will be no existing keys matching this pattern
/// (why would you prefix a private key with `_`?).
/// It's also uncommon to have more than 1 static block in a class.
///
/// Therefore common case is only 1 static block, which will use key `#_`.
/// So store whether `#_` is in set as a separate `bool`, to make a fast path this common case,
/// which does not involve any allocations (`numbered` will remain empty).
///
/// Use a `Vec` rather than a `HashMap`, because number of matching private keys is usually small,
/// and `Vec` is lower overhead in that case.
#[derive(Default)]
struct Keys<'a> {
    /// `true` if keys includes `#_`.
    underscore: bool,
    /// Keys matching `#_[1-9]...`. Stored without the `_` prefix.
    numbered: Vec<&'a str>,
}

impl<'a> Keys<'a> {
    /// Add a key to set.
    ///
    /// Key will only be added to set if it's `_`, or starts with `_[1-9]`.
    fn reserve(&mut self, key: &'a str) {
        let mut bytes = key.as_bytes().iter().copied();
        if bytes.next() != Some(b'_') {
            return;
        }

        match bytes.next() {
            None => {
                self.underscore = true;
            }
            Some(b'1'..=b'9') => {
                self.numbered.push(&key[1..]);
            }
            _ => {}
        }
    }

    /// Get a key which is not in the set.
    ///
    /// Returned key will be either `_`, or `_<integer>` starting with `_2`.
    #[inline]
    fn get_unique(&mut self, ctx: &TraverseCtx<'a>) -> Str<'a> {
        #[expect(clippy::if_not_else)]
        if !self.underscore {
            self.underscore = true;
            Str::from("_")
        } else {
            self.get_unique_slow(ctx)
        }
    }

    // `#[cold]` and `#[inline(never)]` as it should be very rare to need a key other than `#_`.
    #[cold]
    #[inline(never)]
    fn get_unique_slow(&mut self, ctx: &TraverseCtx<'a>) -> Str<'a> {
        // Source text is limited to `MAX_LEN` (defined in `oxc_parser`), which is less than `u32::MAX`.
        // So a class cannot have anywhere near `u32::MAX` private keys.
        // So `u32` is sufficient here, and `i` cannot overflow.
        let mut i = 2u32;
        let mut buffer = ItoaBuffer::new();
        let mut num_str;
        loop {
            num_str = buffer.format(i);
            if !self.numbered.contains(&num_str) {
                break;
            }
            i += 1;
        }

        let key = Str::from_strs_array_in(["_", num_str], ctx);
        self.numbered.push(&key.as_str()[1..]);

        key
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_semantic::Scoping;
    use oxc_traverse::ReusableTraverseCtx;

    use crate::state::TransformState;

    use super::Keys;

    macro_rules! setup {
        ($ctx:ident) => {
            let allocator = Allocator::default();
            let scoping = Scoping::default();
            let state = TransformState::default();
            let ctx = ReusableTraverseCtx::new(state, scoping, &allocator);
            // SAFETY: Macro user only gets a `&mut TransCtx`, which cannot be abused
            let mut ctx = unsafe { ctx.unwrap() };
            let $ctx = &mut ctx;
        };
    }

    #[test]
    fn keys_no_reserved() {
        setup!(ctx);

        let mut keys = Keys::default();

        assert_eq!(keys.get_unique(ctx), "_");
        assert_eq!(keys.get_unique(ctx), "_2");
        assert_eq!(keys.get_unique(ctx), "_3");
        assert_eq!(keys.get_unique(ctx), "_4");
        assert_eq!(keys.get_unique(ctx), "_5");
        assert_eq!(keys.get_unique(ctx), "_6");
        assert_eq!(keys.get_unique(ctx), "_7");
        assert_eq!(keys.get_unique(ctx), "_8");
        assert_eq!(keys.get_unique(ctx), "_9");
        assert_eq!(keys.get_unique(ctx), "_10");
        assert_eq!(keys.get_unique(ctx), "_11");
        assert_eq!(keys.get_unique(ctx), "_12");
    }

    #[test]
    fn keys_no_relevant_reserved() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("a");
        keys.reserve("foo");
        keys.reserve("__");
        keys.reserve("_0");
        keys.reserve("_1");
        keys.reserve("_a");
        keys.reserve("_foo");
        keys.reserve("_2foo");

        assert_eq!(keys.get_unique(ctx), "_");
        assert_eq!(keys.get_unique(ctx), "_2");
        assert_eq!(keys.get_unique(ctx), "_3");
    }

    #[test]
    fn keys_reserved_underscore() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_");

        assert_eq!(keys.get_unique(ctx), "_2");
        assert_eq!(keys.get_unique(ctx), "_3");
        assert_eq!(keys.get_unique(ctx), "_4");
    }

    #[test]
    fn keys_reserved_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_2");
        keys.reserve("_4");
        keys.reserve("_11");

        assert_eq!(keys.get_unique(ctx), "_");
        assert_eq!(keys.get_unique(ctx), "_3");
        assert_eq!(keys.get_unique(ctx), "_5");
        assert_eq!(keys.get_unique(ctx), "_6");
        assert_eq!(keys.get_unique(ctx), "_7");
        assert_eq!(keys.get_unique(ctx), "_8");
        assert_eq!(keys.get_unique(ctx), "_9");
        assert_eq!(keys.get_unique(ctx), "_10");
        assert_eq!(keys.get_unique(ctx), "_12");
    }

    #[test]
    fn keys_reserved_later_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_5");
        keys.reserve("_4");
        keys.reserve("_12");
        keys.reserve("_13");

        assert_eq!(keys.get_unique(ctx), "_");
        assert_eq!(keys.get_unique(ctx), "_2");
        assert_eq!(keys.get_unique(ctx), "_3");
        assert_eq!(keys.get_unique(ctx), "_6");
        assert_eq!(keys.get_unique(ctx), "_7");
        assert_eq!(keys.get_unique(ctx), "_8");
        assert_eq!(keys.get_unique(ctx), "_9");
        assert_eq!(keys.get_unique(ctx), "_10");
        assert_eq!(keys.get_unique(ctx), "_11");
        assert_eq!(keys.get_unique(ctx), "_14");
    }

    #[test]
    fn keys_reserved_underscore_and_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_2");
        keys.reserve("_4");
        keys.reserve("_");

        assert_eq!(keys.get_unique(ctx), "_3");
        assert_eq!(keys.get_unique(ctx), "_5");
        assert_eq!(keys.get_unique(ctx), "_6");
    }

    #[test]
    fn keys_reserved_underscore_and_later_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_5");
        keys.reserve("_4");
        keys.reserve("_");

        assert_eq!(keys.get_unique(ctx), "_2");
        assert_eq!(keys.get_unique(ctx), "_3");
        assert_eq!(keys.get_unique(ctx), "_6");
    }
}
