//! ES2022: Class Static Block
//!
//! TODO: Fill in docs

use itoa::Buffer as ItoaBuffer;

use oxc_allocator::String as AString;
use oxc_ast::{ast::*, NONE};
use oxc_span::SPAN;
use oxc_syntax::scope::ScopeFlags;
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
                // TODO: Handle `AccessorProperty`?
                _ => continue,
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
        let stmts = &mut block.body;
        if stmts.len() == 1 {
            if let Statement::ExpressionStatement(stmt) = stmts.first_mut().unwrap() {
                // Static block contains a single `ExpressionStatement`. No need to wrap in an IIFE.
                // `static { foo }` -> `foo`
                let expr = ctx.ast.move_expression(&mut stmt.expression);

                // TODO: Tidy up + add comments to the new scoping functions.
                // TODO: Add tests where nested scopes within static block (if none already).
                ctx.scoping.remove_scope_expression(scope_id, &expr);

                return expr;
            }
        }

        // Convert block to arrow function IIFE.
        // `static { foo; bar; }` -> `(() => { foo; bar; })()`

        // Always strict mode since we're in a class
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
        let arrow = ctx.ast.expression_arrow_function_with_scope_id(
            SPAN, false, false, NONE, params, NONE, body, scope_id,
        );
        ctx.ast.expression_call(SPAN, arrow, NONE, ctx.ast.vec(), false)
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

        let mut key = AString::with_capacity_in(num_str.len() + 1, ctx.ast.allocator);
        key.push('_');
        key.push_str(num_str);
        let key = Atom::from(key.into_bump_str());

        self.numbered.push(&key.as_str()[1..]);

        key
    }
}
