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
//! class C {
//!   static #_ = _staticBlock = () => (foo(), (() => {
//!     foo();
//!     bar();
//!   })());
//! }
//! _staticBlock();
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-class-static-block](https://babel.dev/docs/babel-plugin-transform-class-static-block).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/v7.26.2/packages/babel-plugin-transform-class-static-block>
//! * Class static initialization blocks TC39 proposal: <https://github.com/tc39/proposal-class-static-block>

use std::collections::hash_map::Entry;

use itoa::Buffer as ItoaBuffer;

use oxc_allocator::{Address, ArenaVec, GetAddress, TakeIn, UnstableAddress};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse};
use rustc_hash::FxHashMap;

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    state::TransformState,
    utils::ast_builder::{create_assignment, wrap_statements_in_arrow_function_iife},
};

pub struct ClassStaticBlock<'a> {
    deferred: Vec<Option<BoundIdentifier<'a>>>,
    class_expression_deferred: Vec<Option<BoundIdentifier<'a>>>,
    class_names: FxHashMap<Address, InferredName<'a>>,
}

/// A property key that has already been evaluated and converted to a string or symbol.
enum InferredName<'a> {
    Static(Str<'a>),
    Computed(BoundIdentifier<'a>),
}

impl<'a> InferredName<'a> {
    fn expression(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match self {
            Self::Static(name) => Expression::new_string_literal(SPAN, *name, None, ctx),
            Self::Computed(binding) => binding.create_read_expression(ctx),
        }
    }
}

impl ClassStaticBlock<'_> {
    pub fn new() -> Self {
        Self {
            deferred: Vec::new(),
            class_expression_deferred: Vec::new(),
            class_names: FxHashMap::default(),
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ClassStaticBlock<'a> {
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
                if class.id.is_none() && Self::has_trailing_blocks(class) =>
            {
                if self.class_names.contains_key(&class.body.address()) {
                    return;
                }
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
                if let Some(name) = name {
                    self.class_names.insert(class.body.address(), InferredName::Static(name));
                }
            }
            _ => {}
        }
    }

    fn enter_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(class) = prop.value.as_ref().and_then(Self::anonymous_deferred_class) {
            let address = class.body.address();
            // A folded initializer has already supplied the original name.
            if let Entry::Vacant(entry) = self.class_names.entry(address) {
                let name = Self::property_name(&mut prop.key, &mut prop.computed, ctx);
                entry.insert(name);
            }
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
                self.prepend_to_initializer(prop, std::iter::once(assignment), ctx);
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

    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let deferred = self.deferred.pop().expect("class body must be entered first");
        if class.r#type == ClassType::ClassExpression {
            self.class_expression_deferred.push(deferred);
        } else if let Some(binding) = deferred {
            let stmt_address = match ctx.parent() {
                parent @ (Ancestor::ExportDefaultDeclarationDeclaration(_)
                | Ancestor::ExportDeclarationDeclaration(_)) => parent.address(),
                _ => class.unstable_address(),
            };
            let call = Self::create_deferred_call(&binding, ctx);
            let statement = Statement::new_expression_statement(SPAN, call, ctx);
            ctx.state.statement_injector.insert_after(&stmt_address, statement);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ClassExpression(class) = expr else { return };
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
                if class.id.is_none() && Self::has_trailing_blocks(class) =>
            {
                Some(class)
            }
            _ => None,
        }
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
        let binding = ctx.generate_uid_in_current_hoist_scope("key");
        ctx.state.var_declarations.insert_var(&binding, &ctx.ast);
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
            expressions.push(value);
        } else {
            expressions.push(Expression::new_void_0(SPAN, ctx));
        }
        prop.value = Some(Expression::new_sequence_expression(
            SPAN,
            ArenaVec::from_iter_in(expressions, ctx),
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
