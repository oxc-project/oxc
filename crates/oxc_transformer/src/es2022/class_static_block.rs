//! ES2022: Class Static Block
//!
//! This plugin transforms class static blocks (`class C { static { foo } }`) to an equivalent
//! using private fields (`class C { static #_ = foo }`).
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
//! class C {
//!   static #_ = foo();
//!   static #_2 = (() => {
//!     foo();
//!     bar();
//!   })();
//! }
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-class-static-block](https://babel.dev/docs/babel-plugin-transform-class-static-block).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-class-static-block>
//! * Class static initialization blocks TC39 proposal: <https://github.com/tc39/proposal-class-static-block>

use itoa::Buffer as ItoaBuffer;

use oxc_allocator::String as ArenaString;
use oxc_ast::{ast::*, Visit, NONE};
use oxc_semantic::SymbolTable;
use oxc_span::SPAN;
use oxc_syntax::{
    reference::ReferenceFlags,
    scope::{ScopeFlags, ScopeId},
};
use oxc_traverse::{Traverse, TraverseCtx};

pub struct ClassStaticBlock;

impl ClassStaticBlock {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> Traverse<'a> for ClassStaticBlock {
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

        // Transform static blocks
        if !has_static_block {
            return;
        }

        for element in body.body.iter_mut() {
            if let ClassElement::StaticBlock(block) = element {
                *element = Self::convert_block_to_private_field(block, &mut keys, ctx);
            }
        }
    }
}

impl ClassStaticBlock {
    /// Convert static block to private field.
    /// `static { foo }` -> `static #_ = foo;`
    /// `static { foo; bar; }` -> `static #_ = (() => { foo; bar; })();`
    fn convert_block_to_private_field<'a>(
        block: &mut StaticBlock<'a>,
        keys: &mut Keys<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ClassElement<'a> {
        let expr = Self::convert_block_to_expression(block, ctx);

        let key = keys.get_unique(ctx);
        let key = ctx.ast.property_key_private_identifier(SPAN, key);

        ctx.ast.class_element_property_definition(
            PropertyDefinitionType::PropertyDefinition,
            block.span,
            ctx.ast.vec(),
            key,
            Some(expr),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            NONE,
            None,
        )
    }

    /// Convert static block to expression which will be value of private field.
    /// `static { foo }` -> `foo`
    /// `static { foo; bar; }` -> `(() => { foo; bar; })()`
    fn convert_block_to_expression<'a>(
        block: &mut StaticBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let scope_id = block.scope_id.get().unwrap();

        // If block contains only a single `ExpressionStatement`, no need to wrap in an IIFE.
        // `static { foo }` -> `foo`
        // TODO(improve-on-babel): If block has no statements, could remove it entirely.
        let stmts = &mut block.body;
        if stmts.len() == 1 {
            if let Statement::ExpressionStatement(stmt) = stmts.first_mut().unwrap() {
                return Self::convert_block_with_single_expression_to_expression(
                    &mut stmt.expression,
                    scope_id,
                    ctx,
                );
            }
        }

        // Convert block to arrow function IIFE.
        // `static { foo; bar; }` -> `(() => { foo; bar; })()`

        // Re-use the static block's scope for the arrow function.
        // Always strict mode since we're in a class.
        *ctx.scopes_mut().get_flags_mut(scope_id) =
            ScopeFlags::Function | ScopeFlags::Arrow | ScopeFlags::StrictMode;

        let stmts = ctx.ast.move_vec(stmts);
        let params = ctx.ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            ctx.ast.vec(),
            NONE,
        );
        let body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), stmts);
        let arrow = Expression::ArrowFunctionExpression(
            ctx.ast.alloc_arrow_function_expression_with_scope_id(
                SPAN, false, false, NONE, params, NONE, body, scope_id,
            ),
        );
        ctx.ast.expression_call(SPAN, arrow, NONE, ctx.ast.vec(), false)
    }

    /// Convert static block to expression which will be value of private field,
    /// where the static block contains only a single expression.
    /// `static { foo }` -> `foo`
    fn convert_block_with_single_expression_to_expression<'a>(
        expr: &mut Expression<'a>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let expr = ctx.ast.move_expression(expr);

        // Remove the scope for the static block from the scope chain
        ctx.remove_scope_for_expression(scope_id, &expr);

        // If expression is an assignment, left side has moved from a write-only position to a read + write one.
        // `static { x = 1; }` -> `static #_ = x = 1;`
        // So set `ReferenceFlags::Read` on the left side.
        if let Expression::AssignmentExpression(assign_expr) = &expr {
            if assign_expr.operator == AssignmentOperator::Assign {
                let mut setter = ReferenceFlagsSetter { symbols: ctx.symbols_mut() };
                setter.visit_assignment_target(&assign_expr.left);
            }
        }

        expr
    }
}

/// Visitor which sets `ReferenceFlags::Read` flag on all `IdentifierReference`s.
/// It skips `MemberExpression`s, because their flags are not affected by the change in position.
struct ReferenceFlagsSetter<'s> {
    symbols: &'s mut SymbolTable,
}

impl<'a, 's> Visit<'a> for ReferenceFlagsSetter<'s> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let reference_id = ident.reference_id().unwrap();
        let reference = self.symbols.get_reference_mut(reference_id);
        *reference.flags_mut() |= ReferenceFlags::Read;
    }

    fn visit_member_expression(&mut self, _member_expr: &MemberExpression<'a>) {
        // Don't traverse further
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
    fn get_unique(&mut self, ctx: &mut TraverseCtx<'a>) -> Atom<'a> {
        #[expect(clippy::if_not_else)]
        if !self.underscore {
            self.underscore = true;
            Atom::from("_")
        } else {
            self.get_unique_slow(ctx)
        }
    }

    // `#[cold]` and `#[inline(never)]` as it should be very rare to need a key other than `#_`.
    #[cold]
    #[inline(never)]
    fn get_unique_slow(&mut self, ctx: &mut TraverseCtx<'a>) -> Atom<'a> {
        // Source text length is limited to `u32::MAX` so impossible to have more than `u32::MAX`
        // private keys. So `u32` is sufficient here.
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

        let mut key = ArenaString::with_capacity_in(num_str.len() + 1, ctx.ast.allocator);
        key.push('_');
        key.push_str(num_str);
        let key = Atom::from(key.into_bump_str());

        self.numbered.push(&key.as_str()[1..]);

        key
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_semantic::{ScopeTree, SymbolTable};
    use oxc_traverse::TraverseCtx;

    use super::Keys;

    macro_rules! setup {
        ($ctx:ident) => {
            let allocator = Allocator::default();
            let scopes = ScopeTree::default();
            let symbols = SymbolTable::default();
            let mut $ctx = TraverseCtx::new(scopes, symbols, &allocator);
        };
    }

    #[test]
    fn keys_no_reserved() {
        setup!(ctx);

        let mut keys = Keys::default();

        assert_eq!(keys.get_unique(&mut ctx), "_");
        assert_eq!(keys.get_unique(&mut ctx), "_2");
        assert_eq!(keys.get_unique(&mut ctx), "_3");
        assert_eq!(keys.get_unique(&mut ctx), "_4");
        assert_eq!(keys.get_unique(&mut ctx), "_5");
        assert_eq!(keys.get_unique(&mut ctx), "_6");
        assert_eq!(keys.get_unique(&mut ctx), "_7");
        assert_eq!(keys.get_unique(&mut ctx), "_8");
        assert_eq!(keys.get_unique(&mut ctx), "_9");
        assert_eq!(keys.get_unique(&mut ctx), "_10");
        assert_eq!(keys.get_unique(&mut ctx), "_11");
        assert_eq!(keys.get_unique(&mut ctx), "_12");
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

        assert_eq!(keys.get_unique(&mut ctx), "_");
        assert_eq!(keys.get_unique(&mut ctx), "_2");
        assert_eq!(keys.get_unique(&mut ctx), "_3");
    }

    #[test]
    fn keys_reserved_underscore() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_");

        assert_eq!(keys.get_unique(&mut ctx), "_2");
        assert_eq!(keys.get_unique(&mut ctx), "_3");
        assert_eq!(keys.get_unique(&mut ctx), "_4");
    }

    #[test]
    fn keys_reserved_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_2");
        keys.reserve("_4");
        keys.reserve("_11");

        assert_eq!(keys.get_unique(&mut ctx), "_");
        assert_eq!(keys.get_unique(&mut ctx), "_3");
        assert_eq!(keys.get_unique(&mut ctx), "_5");
        assert_eq!(keys.get_unique(&mut ctx), "_6");
        assert_eq!(keys.get_unique(&mut ctx), "_7");
        assert_eq!(keys.get_unique(&mut ctx), "_8");
        assert_eq!(keys.get_unique(&mut ctx), "_9");
        assert_eq!(keys.get_unique(&mut ctx), "_10");
        assert_eq!(keys.get_unique(&mut ctx), "_12");
    }

    #[test]
    fn keys_reserved_later_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_5");
        keys.reserve("_4");
        keys.reserve("_12");
        keys.reserve("_13");

        assert_eq!(keys.get_unique(&mut ctx), "_");
        assert_eq!(keys.get_unique(&mut ctx), "_2");
        assert_eq!(keys.get_unique(&mut ctx), "_3");
        assert_eq!(keys.get_unique(&mut ctx), "_6");
        assert_eq!(keys.get_unique(&mut ctx), "_7");
        assert_eq!(keys.get_unique(&mut ctx), "_8");
        assert_eq!(keys.get_unique(&mut ctx), "_9");
        assert_eq!(keys.get_unique(&mut ctx), "_10");
        assert_eq!(keys.get_unique(&mut ctx), "_11");
        assert_eq!(keys.get_unique(&mut ctx), "_14");
    }

    #[test]
    fn keys_reserved_underscore_and_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_2");
        keys.reserve("_4");
        keys.reserve("_");

        assert_eq!(keys.get_unique(&mut ctx), "_3");
        assert_eq!(keys.get_unique(&mut ctx), "_5");
        assert_eq!(keys.get_unique(&mut ctx), "_6");
    }

    #[test]
    fn keys_reserved_underscore_and_later_numbers() {
        setup!(ctx);

        let mut keys = Keys::default();
        keys.reserve("_5");
        keys.reserve("_4");
        keys.reserve("_");

        assert_eq!(keys.get_unique(&mut ctx), "_2");
        assert_eq!(keys.get_unique(&mut ctx), "_3");
        assert_eq!(keys.get_unique(&mut ctx), "_6");
    }
}
