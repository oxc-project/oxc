//! Random arena-backed Oxc AST generation.
//!
//! This crate deliberately only constructs ASTs. It does not print, parse, or
//! semantically validate the generated trees.
//! JavaScript and TypeScript are supported; JSX and TSX are not yet supported.

use std::cell::Cell;

use rand::{Rng, RngExt as _};

use oxc_allocator::{Allocator, Box as ArenaBox, Vec as ArenaVec};
use oxc_ast::{
    ast::{CommentNewlines, RegExpFlags, SourceType, Span},
    builder::AstBuilder,
};
use oxc_str::{Ident, Str};

mod custom;
mod generated;

pub use generated::Generate;

/// Limits for random AST generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(clippy::struct_field_names)]
pub struct AstGeneratorOptions {
    /// Maximum number of statements in one generated root.
    pub max_stmts: usize,
    /// Maximum number of statements in each braced block.
    pub max_stmts_per_block: usize,
    /// Maximum nesting depth of expressions.
    pub max_expr_depth: usize,
}

impl Default for AstGeneratorOptions {
    fn default() -> Self {
        Self { max_stmts: 100, max_stmts_per_block: 10, max_expr_depth: 10 }
    }
}

/// Generator for random arena-backed Oxc AST values.
pub struct AstGenerator<'a, 'r, R: Rng + ?Sized> {
    ast: AstBuilder<'a>,
    rng: &'r mut R,
    source_type: SourceType,
    options: AstGeneratorOptions,
    is_generating: bool,
    stmts: usize,
    expr_depth: usize,
    context: Context,
}

impl<'a, 'r, R: Rng + ?Sized> AstGenerator<'a, 'r, R> {
    /// Create a generator using caller-provided randomness.
    ///
    /// # Panics
    ///
    /// Panics for parser-input-only, declaration-file, JSX, or TSX source types, which are not
    /// supported yet.
    pub fn new(allocator: &'a Allocator, rng: &'r mut R, source_type: SourceType) -> Self {
        Self::new_with_options(allocator, rng, source_type, AstGeneratorOptions::default())
    }

    /// Create a generator using caller-provided randomness and generation limits.
    ///
    /// # Panics
    ///
    /// Panics if `max_stmts` or `max_expr_depth` is zero, or for parser-input-only,
    /// declaration-file, JSX, or TSX source types.
    pub fn new_with_options(
        allocator: &'a Allocator,
        rng: &'r mut R,
        source_type: SourceType,
        options: AstGeneratorOptions,
    ) -> Self {
        assert!(options.max_stmts > 0, "max_stmts must be greater than zero");
        assert!(options.max_expr_depth > 0, "max_expr_depth must be greater than zero");
        assert!(!source_type.is_jsx(), "JSX and TSX AST generation is not implemented");
        assert!(
            !source_type.is_unambiguous(),
            "unambiguous source types are parser inputs and cannot be stored on generated ASTs"
        );
        assert!(
            !source_type.is_typescript_definition(),
            "TypeScript declaration-file AST generation is not implemented"
        );
        Self {
            ast: AstBuilder::new(allocator),
            rng,
            source_type,
            options,
            is_generating: false,
            stmts: 0,
            expr_depth: 0,
            context: Context {
                in_function: source_type.is_commonjs(),
                allow_new_target: source_type.is_commonjs(),
                ..Context::default()
            },
        }
    }

    /// Generate a random AST value.
    pub fn generate<T: Generate<'a>>(&mut self) -> T {
        let is_root = !self.is_generating;
        if is_root {
            self.is_generating = true;
            self.stmts = 0;
            self.expr_depth = 0;
        }
        let value = T::generate(self);
        if is_root {
            self.is_generating = false;
        }
        value
    }

    #[inline]
    pub(crate) fn ast(&self) -> &AstBuilder<'a> {
        &self.ast
    }

    #[inline]
    pub(crate) fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[inline]
    pub(crate) fn is_typescript(&self) -> bool {
        self.source_type.is_typescript()
    }

    #[inline]
    pub(crate) fn at_limit(&self) -> bool {
        self.at_stmt_limit() || self.at_expr_limit()
    }

    #[inline]
    pub(crate) fn at_stmt_limit(&self) -> bool {
        self.stmts >= self.options.max_stmts
    }

    #[inline]
    pub(crate) fn at_expr_limit(&self) -> bool {
        self.expr_depth >= self.options.max_expr_depth
    }

    #[inline]
    pub(crate) fn can_nest_exprs(&self, additional_depth: usize) -> bool {
        self.expr_depth.saturating_add(additional_depth) <= self.options.max_expr_depth
    }

    pub(crate) fn with_expr_depth<T>(&mut self, generate: impl FnOnce(&mut Self) -> T) -> T {
        self.expr_depth += 1;
        let value = generate(self);
        self.expr_depth -= 1;
        value
    }

    #[inline]
    pub(crate) fn remaining_stmts(&self) -> usize {
        self.options.max_stmts.saturating_sub(self.stmts)
    }

    #[inline]
    pub(crate) fn reserve_stmt(&mut self) -> bool {
        if self.at_stmt_limit() {
            false
        } else {
            self.stmts += 1;
            true
        }
    }

    #[inline]
    pub(crate) fn max_stmts_per_block(&self) -> usize {
        self.options.max_stmts_per_block
    }

    #[inline]
    pub(crate) fn random_bool(&mut self) -> bool {
        self.rng.random()
    }

    #[inline]
    pub(crate) fn random_index(&mut self, len: usize) -> usize {
        self.rng.random_range(0..len)
    }

    pub(crate) fn random_weighted(&mut self, weights: &[u32]) -> u32 {
        let total = weights.iter().copied().sum::<u32>();
        let mut ticket = self.rng.random_range(0..total);
        for (index, &weight) in weights.iter().enumerate() {
            if ticket < weight {
                return u32::try_from(index).unwrap();
            }
            ticket -= weight;
        }
        unreachable!()
    }

    pub(crate) fn random_ident(&mut self) -> Ident<'a> {
        const NAMES: &[&str] = &["a", "b", "c", "x", "y", "value", "item", "result"];
        let name = NAMES[self.random_index(NAMES.len())];
        Ident::from_str_in(name, self.ast())
    }

    pub(crate) fn random_str(&mut self) -> Str<'a> {
        const VALUES: &[&str] = &["", "a", "value", "hello", "0", "key"];
        let value = VALUES[self.random_index(VALUES.len())];
        Str::from_str_in(value, self.ast())
    }

    pub(crate) fn random_vec<T: Generate<'a>>(&mut self) -> ArenaVec<'a, T> {
        let mut values = ArenaVec::new_in(self.ast());
        while !self.at_limit() && self.random_bool() {
            values.push(self.generate());
        }
        values
    }

    pub(crate) fn random_option<T: Generate<'a>>(&mut self) -> Option<T> {
        (!self.at_limit() && self.random_bool()).then(|| self.generate())
    }

    pub(crate) fn random_box<T: Generate<'a>>(&mut self) -> ArenaBox<'a, T> {
        ArenaBox::new_in(self.generate(), self.ast())
    }

    pub(crate) fn context(&self) -> Context {
        self.context
    }

    pub(crate) fn with_context<T>(
        &mut self,
        update: impl FnOnce(&mut Context),
        generate: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.context;
        update(&mut self.context);
        let value = generate(self);
        self.context = previous;
        value
    }
}

/// Context used by handwritten generators to avoid contextually illegal nodes.
#[derive(Clone, Copy, Default)]
pub(crate) struct Context {
    pub in_function: bool,
    pub allow_new_target: bool,
    pub in_async: bool,
    pub in_generator: bool,
    pub in_loop: bool,
    pub in_switch: bool,
    #[expect(dead_code)]
    pub in_class: bool,
    #[expect(dead_code)]
    pub in_derived_class: bool,
}

impl Context {
    fn enter_function(&mut self, is_async: bool, is_generator: bool) {
        self.in_function = true;
        self.allow_new_target = true;
        self.in_async = is_async;
        self.in_generator = is_generator;
        self.in_loop = false;
        self.in_switch = false;
    }

    fn enter_arrow_function(&mut self, is_async: bool) {
        self.in_function = true;
        self.in_async = is_async;
        self.in_generator = false;
        self.in_loop = false;
        self.in_switch = false;
    }
}

macro_rules! generate_random {
    ($($ty:ty),* $(,)?) => {$ (
        impl Generate<'_> for $ty {
            fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'_, '_, R>) -> Self {
                generator.rng.random()
            }
        }
    )* };
}

generate_random!(bool, u8, u16, u32, u64, i8, i16, i32, i64);

impl Generate<'_> for f64 {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'_, '_, R>) -> Self {
        // Keep values finite so they always have a straightforward source representation.
        f64::from(generator.rng.random_range(-1_000_i32..=1_000))
    }
}

impl<'a> Generate<'a> for &'a str {
    fn generate<R: Rng + ?Sized>(_generator: &mut AstGenerator<'a, '_, R>) -> Self {
        ""
    }
}

impl<'a> Generate<'a> for Str<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        generator.random_str()
    }
}

impl<'a> Generate<'a> for Ident<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        generator.random_ident()
    }
}

impl<'a, T: Generate<'a>> Generate<'a> for Option<T> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        generator.random_option()
    }
}

impl<'a, T: Generate<'a>> Generate<'a> for ArenaBox<'a, T> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        generator.random_box()
    }
}

impl<'a, T: Generate<'a>> Generate<'a> for ArenaVec<'a, T> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        generator.random_vec()
    }
}

impl<T: Default> Generate<'_> for Cell<T> {
    fn generate<R: Rng + ?Sized>(_generator: &mut AstGenerator<'_, '_, R>) -> Self {
        Self::default()
    }
}

impl Generate<'_> for Span {
    fn generate<R: Rng + ?Sized>(_generator: &mut AstGenerator<'_, '_, R>) -> Self {
        oxc_span::SPAN
    }
}

impl Generate<'_> for RegExpFlags {
    fn generate<R: Rng + ?Sized>(_generator: &mut AstGenerator<'_, '_, R>) -> Self {
        Self::empty()
    }
}

impl Generate<'_> for CommentNewlines {
    fn generate<R: Rng + ?Sized>(_generator: &mut AstGenerator<'_, '_, R>) -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use oxc_ast::ast::{
        AccessorPropertyType, ArrayExpressionElement, Expression, FormalParameterKind,
        FunctionType, ImportOrExportKind, MethodDefinitionType, ObjectProperty, Program,
        PropertyDefinitionType, PropertyKey, PropertyKind, SourceType, Statement, TSAsExpression,
        TSType, TSTypeAliasDeclaration,
    };
    use rand::{SeedableRng, rngs::StdRng};

    use super::{AstGenerator, AstGeneratorOptions, Context};

    #[test]
    fn generates_javascript_roots() {
        for seed in 0..8 {
            let allocator = oxc_allocator::Allocator::default();
            let mut rng = StdRng::seed_from_u64(seed);
            let mut generator = AstGenerator::new(&allocator, &mut rng, SourceType::mjs());

            let program = generator.generate::<Program<'_>>();
            assert_eq!(program.span, oxc_span::SPAN);
            assert_eq!(program.source_text, "");
            assert!(program.comments.is_empty());
            assert!(program.body.len() <= AstGeneratorOptions::default().max_stmts);

            let _ = generator.generate::<Statement<'_>>();
            let _ = generator.generate::<Expression<'_>>();
        }

        assert_eq!(
            AstGeneratorOptions::default(),
            AstGeneratorOptions { max_stmts: 100, max_stmts_per_block: 10, max_expr_depth: 10 }
        );
    }

    #[test]
    fn generates_typescript_roots() {
        for seed in 0..8 {
            let allocator = oxc_allocator::Allocator::default();
            let mut rng = StdRng::seed_from_u64(seed);
            let mut generator =
                AstGenerator::new(&allocator, &mut rng, SourceType::ts().with_module(true));

            let program = generator.generate::<Program<'_>>();
            assert!(program.source_type.is_typescript());
            let _ = generator.generate::<TSType<'_>>();
            let _ = generator.generate::<TSTypeAliasDeclaration<'_>>();
            let _ = generator.generate::<TSAsExpression<'_>>();
        }
    }

    #[test]
    fn typescript_enum_generators_include_typescript_variants() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let mut generator =
            AstGenerator::new(&allocator, &mut rng, SourceType::ts().with_module(true));
        let mut saw_ts_function = false;
        let mut saw_type_import = false;

        for _ in 0..64 {
            saw_ts_function |= matches!(
                generator.generate::<FunctionType>(),
                FunctionType::TSDeclareFunction | FunctionType::TSEmptyBodyFunctionExpression
            );
            saw_type_import |=
                generator.generate::<ImportOrExportKind>() == ImportOrExportKind::Type;
        }

        assert!(saw_ts_function);
        assert!(saw_type_import);
    }

    #[test]
    #[should_panic(expected = "JSX and TSX AST generation is not implemented")]
    fn rejects_jsx_source_type() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let _ = AstGenerator::new(&allocator, &mut rng, SourceType::jsx());
    }

    #[test]
    #[should_panic(expected = "JSX and TSX AST generation is not implemented")]
    fn rejects_tsx_source_type() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let _ = AstGenerator::new(&allocator, &mut rng, SourceType::tsx());
    }

    #[test]
    #[should_panic(
        expected = "unambiguous source types are parser inputs and cannot be stored on generated ASTs"
    )]
    fn rejects_unambiguous_source_type() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let _ = AstGenerator::new(&allocator, &mut rng, SourceType::ts());
    }

    #[test]
    #[should_panic(expected = "TypeScript declaration-file AST generation is not implemented")]
    fn rejects_typescript_definition_source_type() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let _ = AstGenerator::new(&allocator, &mut rng, SourceType::d_ts());
    }

    #[test]
    #[should_panic(expected = "TSType generation requires a TypeScript source type")]
    fn rejects_typescript_node_for_javascript_source_type() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let mut generator = AstGenerator::new(&allocator, &mut rng, SourceType::mjs());
        let _ = generator.generate::<TSType<'_>>();
    }

    #[test]
    fn resets_statement_budget_for_each_root() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let options = AstGeneratorOptions { max_stmts: 1, ..AstGeneratorOptions::default() };
        let mut generator =
            AstGenerator::new_with_options(&allocator, &mut rng, SourceType::mjs(), options);
        generator.stmts = generator.options.max_stmts;

        let _ = generator.generate::<Statement<'_>>();

        assert_eq!(generator.stmts, 1);
    }

    #[test]
    #[should_panic(expected = "max_stmts must be greater than zero")]
    fn rejects_zero_max_statements() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let options = AstGeneratorOptions { max_stmts: 0, ..AstGeneratorOptions::default() };
        let _ = AstGenerator::new_with_options(&allocator, &mut rng, SourceType::mjs(), options);
    }

    #[test]
    #[should_panic(expected = "max_expr_depth must be greater than zero")]
    fn rejects_zero_max_expression_depth() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let options = AstGeneratorOptions { max_expr_depth: 0, ..AstGeneratorOptions::default() };
        let _ = AstGenerator::new_with_options(&allocator, &mut rng, SourceType::mjs(), options);
    }

    #[test]
    fn function_context_cannot_jump_to_outer_control_flow() {
        let mut context = Context { in_loop: true, in_switch: true, ..Context::default() };

        context.enter_function(false, false);

        assert!(!context.in_loop);
        assert!(!context.in_switch);
    }

    #[test]
    fn inherited_expression_generation_keeps_context_guards() {
        for seed in 0..64 {
            let allocator = oxc_allocator::Allocator::default();
            let mut rng = StdRng::seed_from_u64(seed);
            let mut generator = AstGenerator::new(&allocator, &mut rng, SourceType::script());

            let element = generator.generate::<ArrayExpressionElement<'_>>();

            assert!(!matches!(
                element,
                ArrayExpressionElement::AwaitExpression(_)
                    | ArrayExpressionElement::YieldExpression(_)
                    | ArrayExpressionElement::Super(_)
                    | ArrayExpressionElement::ImportMeta(_)
                    | ArrayExpressionElement::NewTarget(_)
            ));
        }
    }

    #[test]
    fn javascript_enum_generators_exclude_typescript_variants() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let mut generator = AstGenerator::new(&allocator, &mut rng, SourceType::mjs());

        for _ in 0..32 {
            assert!(matches!(
                generator.generate::<FunctionType>(),
                FunctionType::FunctionDeclaration | FunctionType::FunctionExpression
            ));
            assert!(!matches!(
                generator.generate::<FormalParameterKind>(),
                FormalParameterKind::Signature
            ));
            assert_eq!(
                generator.generate::<MethodDefinitionType>(),
                MethodDefinitionType::MethodDefinition
            );
            assert_eq!(
                generator.generate::<PropertyDefinitionType>(),
                PropertyDefinitionType::PropertyDefinition
            );
            assert_eq!(
                generator.generate::<AccessorPropertyType>(),
                AccessorPropertyType::AccessorProperty
            );
        }
    }

    #[test]
    fn object_properties_have_coherent_fields() {
        let allocator = oxc_allocator::Allocator::default();
        let mut rng = StdRng::seed_from_u64(0);
        let mut generator = AstGenerator::new(&allocator, &mut rng, SourceType::mjs());

        for _ in 0..32 {
            let property = generator.generate::<ObjectProperty<'_>>();
            assert_eq!(property.kind, PropertyKind::Init);
            assert!(matches!(property.key, PropertyKey::StaticIdentifier(_)));
            assert!(!property.method);
            assert!(!property.shorthand);
            assert!(!property.computed);
        }
    }
}
