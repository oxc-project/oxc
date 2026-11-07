use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use oxc_ast::ast::{
    BinaryExpression, BindingPattern, Expression, IdentifierReference, Program, TSEnumDeclaration,
    UnaryExpression, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_ecmascript::{ToInt32, ToUint32};
use oxc_span::Span;
use oxc_str::{CompactStr, Ident, JSStr};
use oxc_syntax::{
    constant_value::ConstantValue,
    number::ToJsString,
    operator::{BinaryOperator, UnaryOperator},
    scope::ScopeId,
    symbol::SymbolId,
};

use crate::scoping::Scoping;

/// Context shared across recursive evaluation calls.
struct EnumEvalCtx<'a, 's> {
    scope_id: ScopeId,
    enum_symbol_id: Option<SymbolId>,
    scoping: &'s Scoping,
    constants: &'s mut EnumEvaluator<'a>,
    string_limit: usize,
}

pub fn evaluate_declarations<'a>(
    declarations: &[&TSEnumDeclaration<'a>],
    scoping: &mut Scoping,
    program: &'a Program<'a>,
) {
    let mut evaluator = EnumEvaluator {
        program,
        const_values: FxHashMap::default(),
        string_budget: program.source_text.len(),
        const_initializers: None,
    };
    for declaration in declarations {
        evaluate_enum_members(declaration, scoping, &mut evaluator);
    }
}

enum CachedConst {
    Evaluating,
    Unknown,
    String,
    Value(ConstantValue),
}

struct EnumEvaluator<'a> {
    program: &'a Program<'a>,
    const_initializers: Option<Vec<ConstInitializer<'a>>>,
    const_values: FxHashMap<SymbolId, CachedConst>,
    string_budget: usize,
}

struct ConstInitializer<'a> {
    symbol_id: SymbolId,
    expression: &'a Expression<'a>,
}

enum ConstEvalTask<'a> {
    Enter(SymbolId, Span),
    Exit(SymbolId, &'a Expression<'a>),
}

impl<'a> EnumEvaluator<'a> {
    fn evaluate_const(
        &mut self,
        symbol_id: SymbolId,
        reference_span: Span,
        scoping: &Scoping,
        string_limit: usize,
    ) -> Option<ConstantValue> {
        if !scoping.symbol_flags(symbol_id).is_const_variable()
            || scoping.symbol_span(symbol_id).start >= reference_span.start
        {
            return None;
        }
        if !self.const_values.contains_key(&symbol_id) {
            let mut tasks: SmallVec<[ConstEvalTask<'a>; 8]> = SmallVec::new();
            tasks.push(ConstEvalTask::Enter(symbol_id, reference_span));
            while let Some(task) = tasks.pop() {
                match task {
                    ConstEvalTask::Enter(symbol_id, reference_span) => {
                        if self.const_values.contains_key(&symbol_id)
                            || scoping.symbol_span(symbol_id).start >= reference_span.start
                        {
                            continue;
                        }
                        let Some(initializer) = self.initializer(symbol_id, scoping) else {
                            self.const_values.insert(symbol_id, CachedConst::Unknown);
                            continue;
                        };
                        self.const_values.insert(symbol_id, CachedConst::Evaluating);
                        tasks.push(ConstEvalTask::Exit(symbol_id, initializer));
                        ConstDependencies { scoping, tasks: &mut tasks }
                            .visit_expression(initializer);
                    }
                    ConstEvalTask::Exit(symbol_id, initializer) => {
                        let mut ctx = EnumEvalCtx {
                            scope_id: scoping.symbol_scope_id(symbol_id),
                            enum_symbol_id: None,
                            scoping,
                            // A short chain of string aliases can otherwise expand exponentially.
                            string_limit: self.string_budget,
                            constants: self,
                        };
                        let value = evaluate_expression(initializer, &mut ctx);
                        let is_string = matches!(value, Some(ConstantValue::String(_)))
                            || (value.is_none() && is_string_expression(initializer, &mut ctx));
                        let value = value.filter(|value| !matches!(value, ConstantValue::String(s) if s.len() > self.string_budget));
                        if let Some(ConstantValue::String(value)) = &value {
                            self.string_budget -= value.len();
                        }
                        let cached = match value {
                            Some(value) => CachedConst::Value(value),
                            None if is_string => CachedConst::String,
                            None => CachedConst::Unknown,
                        };
                        self.const_values.insert(symbol_id, cached);
                    }
                }
            }
        }
        match self.const_values.get(&symbol_id) {
            Some(CachedConst::Value(value)) if !matches!(value, ConstantValue::String(s) if s.len() > string_limit) => {
                Some(value.clone())
            }
            _ => None,
        }
    }

    fn initializer(
        &mut self,
        symbol_id: SymbolId,
        scoping: &Scoping,
    ) -> Option<&'a Expression<'a>> {
        let initializers = self.const_initializers.get_or_insert_with(|| {
            let count = scoping
                .symbol_ids()
                .filter(|&id| scoping.symbol_flags(id).is_const_variable())
                .count();
            let mut collector =
                ConstInitializers { scoping, initializers: Vec::with_capacity(count) };
            collector.visit_program(self.program);
            collector.initializers.sort_unstable_by_key(|init| init.symbol_id);
            collector.initializers
        });
        let index = initializers.binary_search_by_key(&symbol_id, |init| init.symbol_id).ok()?;
        Some(initializers[index].expression)
    }
}

struct ConstInitializers<'a, 's> {
    scoping: &'s Scoping,
    initializers: Vec<ConstInitializer<'a>>,
}

impl<'a> Visit<'a> for ConstInitializers<'a, '_> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(binding) = &decl.id
            && self.scoping.symbol_flags(binding.symbol_id()).is_const_variable()
            && let Some(init) = &decl.init
        {
            self.initializers.push(ConstInitializer {
                symbol_id: binding.symbol_id(),
                expression: self.alloc(init.get_inner_expression()),
            });
        }
        walk::walk_variable_declarator(self, decl);
    }
}

struct ConstDependencies<'a, 's> {
    scoping: &'s Scoping,
    tasks: &'s mut SmallVec<[ConstEvalTask<'a>; 8]>,
}

impl<'a> Visit<'a> for ConstDependencies<'a, '_> {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        match expr.without_parentheses() {
            Expression::Identifier(ident) => {
                if let Some(symbol_id) = ident
                    .reference_id
                    .get()
                    .and_then(|id| self.scoping.get_reference(id).symbol_id())
                    && self.scoping.symbol_flags(symbol_id).is_const_variable()
                {
                    self.tasks.push(ConstEvalTask::Enter(symbol_id, ident.span));
                }
            }
            Expression::BinaryExpression(expr) => {
                self.visit_expression(&expr.left);
                self.visit_expression(&expr.right);
            }
            Expression::UnaryExpression(expr) => self.visit_expression(&expr.argument),
            Expression::TemplateLiteral(expr) => {
                for expression in &expr.expressions {
                    self.visit_expression(expression);
                }
            }
            _ => {}
        }
    }
}

enum SiblingLookup {
    Missing,
    Unknown,
    Value(ConstantValue),
}

/// Evaluate all enum member values in a `TSEnumDeclaration` and store them in `Scoping`.
///
/// Runs during semantic analysis so the transformer can look up pre-computed values
/// to emit constant literals and decide whether const enum declarations can be removed.
///
/// ```ts
/// enum Color { Red, Green, Blue }
/// // Red → 0, Green → 1, Blue → 2 (auto-increment)
///
/// enum Flags { A = 1 << 0, B = 1 << 1, C = A | B }
/// // A → 1, B → 2, C → 3 (constant-folded expressions)
///
/// enum Mixed { X = "hello", Y }
/// // X → "hello", Y → None (can't auto-increment after string)
/// ```
fn evaluate_enum_members<'a>(
    decl: &TSEnumDeclaration<'a>,
    scoping: &mut Scoping,
    constants: &mut EnumEvaluator<'a>,
) {
    let Some(scope_id) = decl.body.scope_id.get() else { return };

    let enum_symbol_id = decl.id.symbol_id.get();
    if let Some(id) = enum_symbol_id {
        scoping.add_enum_body_scope(id, scope_id);
        if decl.r#const {
            scoping.add_const_enum(id);
        }
    }

    // Sentinel: the first member with no initializer evaluates to `-1.0 + 1.0 = 0.0`.
    // Once a member fails to resolve, `prev_value` becomes `None` and stays there, so
    // subsequent auto-increment members correctly propagate "unknown" instead of restarting at 0.
    let mut prev_value: Option<ConstantValue> = Some(ConstantValue::Number(-1.0));

    for member in &decl.body.members {
        let value = if let Some(init) = &member.initializer {
            let mut ctx = EnumEvalCtx {
                scope_id,
                enum_symbol_id,
                scoping,
                constants,
                string_limit: usize::MAX,
            };
            evaluate_expression(init, &mut ctx)
        } else {
            match &prev_value {
                Some(ConstantValue::Number(n)) => Some(ConstantValue::Number(n + 1.0)),
                None | Some(ConstantValue::String(_)) => None,
            }
        };

        let member_name = member.id.static_name();
        let member_symbol =
            member_name.as_str().and_then(|name| scoping.get_binding(scope_id, name.into()));
        let is_string = matches!(value, Some(ConstantValue::String(_)))
            || (value.is_none()
                && member.initializer.as_ref().is_some_and(|init| {
                    is_string_expression(
                        init,
                        &mut EnumEvalCtx {
                            scope_id,
                            enum_symbol_id,
                            scoping,
                            constants,
                            string_limit: usize::MAX,
                        },
                    )
                }));
        if let Some(ref value) = value
            && let Some(symbol_id) = member_symbol
        {
            scoping.set_enum_member_value(symbol_id, value.clone());
        } else if is_string {
            // Preserve the kind even when the constant or member name cannot convert to UTF-8.
            scoping.add_string_enum_member(scope_id, member_name);
        }

        prev_value = value;
    }
}

/// Determine string kind once, alongside constant evaluation, before transformation rewrites references.
fn is_string_expression(expr: &Expression<'_>, ctx: &mut EnumEvalCtx<'_, '_>) -> bool {
    match expr.without_parentheses() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::BinaryExpression(expr) if expr.operator == BinaryOperator::Addition => {
            is_string_expression(&expr.left, ctx) || is_string_expression(&expr.right, ctx)
        }
        Expression::Identifier(ident) => {
            if let Some(scopes) =
                ctx.enum_symbol_id.and_then(|id| ctx.scoping.get_enum_body_scopes(id))
            {
                for &scope_id in scopes {
                    if scope_id != ctx.scope_id
                        && ctx.scoping.get_binding(scope_id, ident.name).is_some()
                    {
                        return ctx.scoping.is_string_enum_member(scope_id, ident.name.into());
                    }
                }
            }
            resolve_identifier_symbol(ident, ctx).is_some_and(|symbol_id| {
                ctx.constants.evaluate_const(symbol_id, ident.span, ctx.scoping, ctx.string_limit);
                matches!(
                    ctx.constants.const_values.get(&symbol_id),
                    Some(CachedConst::String | CachedConst::Value(ConstantValue::String(_)))
                ) || ctx.scoping.is_string_enum_member(
                    ctx.scoping.symbol_scope_id(symbol_id),
                    ident.name.into(),
                )
            })
        }
        Expression::StaticMemberExpression(member) => {
            is_string_member(&member.object, member.property.name.into(), ctx)
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::StringLiteral(name) = &member.expression else { return false };
            is_string_member(&member.object, name.value, ctx)
        }
        _ => false,
    }
}

fn is_string_member(object: &Expression<'_>, name: JSStr<'_>, ctx: &EnumEvalCtx<'_, '_>) -> bool {
    let Expression::Identifier(ident) = object else { return false };
    resolve_identifier_symbol(ident, ctx)
        .and_then(|symbol_id| ctx.scoping.get_enum_body_scopes(symbol_id))
        .is_some_and(|scopes| {
            scopes.iter().any(|&scope| ctx.scoping.is_string_enum_member(scope, name))
        })
}

fn string_constant(value: &str, ctx: &EnumEvalCtx<'_, '_>) -> Option<ConstantValue> {
    (value.len() <= ctx.string_limit).then(|| ConstantValue::String(CompactStr::from(value)))
}

fn append_string(value: &mut String, suffix: &str, ctx: &EnumEvalCtx<'_, '_>) -> Option<()> {
    if value.len().checked_add(suffix.len())? > ctx.string_limit {
        return None;
    }
    value.push_str(suffix);
    Some(())
}

fn evaluate_expression(
    expr: &Expression<'_>,
    ctx: &mut EnumEvalCtx<'_, '_>,
) -> Option<ConstantValue> {
    let expr = expr.without_parentheses();
    match expr {
        Expression::Identifier(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => evaluate_ref(expr, ctx),
        Expression::BinaryExpression(expr) => eval_binary_expression(expr, ctx),
        Expression::UnaryExpression(expr) => eval_unary_expression(expr, ctx),
        Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
        Expression::StringLiteral(lit) => string_constant(lit.value.as_str()?, ctx),
        Expression::TemplateLiteral(lit) => {
            if let Some(quasi) = lit.single_quasi() {
                string_constant(quasi.as_str()?, ctx)
            } else {
                let mut value = String::new();
                for (i, quasi) in lit.quasis.iter().enumerate() {
                    let cooked_or_raw = match quasi.value.cooked {
                        Some(cooked) => cooked.as_str()?,
                        None => quasi.value.raw.as_str(),
                    };
                    append_string(&mut value, cooked_or_raw, ctx)?;
                    if i < lit.expressions.len() {
                        match evaluate_expression(&lit.expressions[i], ctx)? {
                            ConstantValue::String(s) => append_string(&mut value, &s, ctx)?,
                            ConstantValue::Number(n) => {
                                append_string(&mut value, &n.to_js_string(), ctx)?;
                            }
                        }
                    }
                }
                Some(ConstantValue::String(CompactStr::from(value.as_str())))
            }
        }
        _ => None,
    }
}

/// Resolve an identifier or member expression to a constant value.
///
/// Uses a three-level lookup strategy for bare identifiers:
/// 1. Sibling enum fallback — `enum A { X = 1 } enum A { Y = X }` (merged declarations)
/// 2. Lexical binding — an enum member or a const variable
/// 3. Unbound globals — `Infinity` and `NaN`
///
/// Also handles cross-enum member access:
/// ```ts
/// enum A { X = 1 }
/// enum B { Y = A.X + 1 }   // StaticMemberExpression
/// enum C { Z = A["X"] + 1 } // ComputedMemberExpression
/// ```
fn evaluate_ref(expr: &Expression<'_>, ctx: &mut EnumEvalCtx<'_, '_>) -> Option<ConstantValue> {
    match expr {
        Expression::Identifier(ident) => {
            match find_in_sibling_enum_scopes(
                ident.name.as_str(),
                ctx.scope_id,
                ctx.enum_symbol_id,
                ctx.scoping,
            ) {
                SiblingLookup::Value(value) => return Some(value),
                SiblingLookup::Unknown => return None,
                SiblingLookup::Missing => {}
            }

            if let Some(symbol_id) = resolve_identifier_symbol(ident, ctx) {
                if let Some(value) = ctx.scoping.get_enum_member_value(symbol_id) {
                    return (!matches!(value, ConstantValue::String(s) if s.len() > ctx.string_limit)).then(|| value.clone());
                }
                return ctx.constants.evaluate_const(
                    symbol_id,
                    ident.span,
                    ctx.scoping,
                    ctx.string_limit,
                );
            }

            match ident.name.as_str() {
                "Infinity" => return Some(ConstantValue::Number(f64::INFINITY)),
                "NaN" => return Some(ConstantValue::Number(f64::NAN)),
                _ => {}
            }

            None
        }
        Expression::StaticMemberExpression(member_expr) => {
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, ctx)?;
            find_in_enum_body_scopes(
                member_expr.property.name,
                obj_symbol_id,
                ctx.scoping,
                ctx.string_limit,
            )
        }
        Expression::ComputedMemberExpression(member_expr) => {
            let Expression::Identifier(obj_ident) = &member_expr.object else { return None };
            let Expression::StringLiteral(prop_lit) = &member_expr.expression else {
                return None;
            };
            let obj_symbol_id = resolve_identifier_symbol(obj_ident, ctx)?;
            find_in_enum_body_scopes(
                Ident::from(prop_lit.value.as_str()?),
                obj_symbol_id,
                ctx.scoping,
                ctx.string_limit,
            )
        }
        _ => None,
    }
}

/// Resolve an identifier to its symbol, trying the resolved reference first,
/// then falling back to scope binding lookup.
fn resolve_identifier_symbol(
    ident: &IdentifierReference<'_>,
    ctx: &EnumEvalCtx<'_, '_>,
) -> Option<SymbolId> {
    if let Some(ref_id) = ident.reference_id.get()
        && let Some(symbol_id) = ctx.scoping.get_reference(ref_id).symbol_id()
    {
        return Some(symbol_id);
    }
    ctx.scoping.find_binding(ctx.scope_id, ident.name.as_str().into())
}

fn eval_binary_expression(
    expr: &BinaryExpression<'_>,
    ctx: &mut EnumEvalCtx<'_, '_>,
) -> Option<ConstantValue> {
    let left = evaluate_expression(&expr.left, ctx)?;
    let right = evaluate_expression(&expr.right, ctx)?;

    if matches!(expr.operator, BinaryOperator::Addition)
        && (matches!(left, ConstantValue::String(_)) || matches!(right, ConstantValue::String(_)))
    {
        let mut result = match &left {
            ConstantValue::String(s) => s.to_string(),
            ConstantValue::Number(v) => v.to_js_string(),
        };
        match &right {
            ConstantValue::String(s) => append_string(&mut result, s, ctx)?,
            ConstantValue::Number(v) => append_string(&mut result, &v.to_js_string(), ctx)?,
        }
        return Some(ConstantValue::String(CompactStr::from(result.as_str())));
    }

    let left = match left {
        ConstantValue::Number(v) => v,
        ConstantValue::String(_) => return None,
    };
    let right = match right {
        ConstantValue::Number(v) => v,
        ConstantValue::String(_) => return None,
    };

    match expr.operator {
        BinaryOperator::ShiftRight => Some(ConstantValue::Number(f64::from(
            left.to_int_32().wrapping_shr(right.to_uint_32()),
        ))),
        BinaryOperator::ShiftRightZeroFill => Some(ConstantValue::Number(f64::from(
            left.to_uint_32().wrapping_shr(right.to_uint_32()),
        ))),
        BinaryOperator::ShiftLeft => Some(ConstantValue::Number(f64::from(
            left.to_int_32().wrapping_shl(right.to_uint_32()),
        ))),
        BinaryOperator::BitwiseXOR => {
            Some(ConstantValue::Number(f64::from(left.to_int_32() ^ right.to_int_32())))
        }
        BinaryOperator::BitwiseOR => {
            Some(ConstantValue::Number(f64::from(left.to_int_32() | right.to_int_32())))
        }
        BinaryOperator::BitwiseAnd => {
            Some(ConstantValue::Number(f64::from(left.to_int_32() & right.to_int_32())))
        }
        BinaryOperator::Multiplication => Some(ConstantValue::Number(left * right)),
        BinaryOperator::Division => Some(ConstantValue::Number(left / right)),
        BinaryOperator::Addition => Some(ConstantValue::Number(left + right)),
        BinaryOperator::Subtraction => Some(ConstantValue::Number(left - right)),
        BinaryOperator::Remainder => Some(ConstantValue::Number(left % right)),
        BinaryOperator::Exponential => Some(ConstantValue::Number(left.powf(right))),
        _ => None,
    }
}

fn eval_unary_expression(
    expr: &UnaryExpression<'_>,
    ctx: &mut EnumEvalCtx<'_, '_>,
) -> Option<ConstantValue> {
    let value = evaluate_expression(&expr.argument, ctx)?;

    // TypeScript leaves unary operations on strings unevaluated (computed members),
    // so we decline here too — folding them would need ToNumber coercion, and a
    // folded value that does not match the runtime result is worse than none at
    // all (`-"3"` must produce -3, `~"4"` -5).
    let value = match value {
        ConstantValue::Number(v) => v,
        ConstantValue::String(_) => return None,
    };

    match expr.operator {
        UnaryOperator::UnaryPlus => Some(ConstantValue::Number(value)),
        UnaryOperator::UnaryNegation => Some(ConstantValue::Number(-value)),
        UnaryOperator::BitwiseNot => Some(ConstantValue::Number(f64::from(!value.to_int_32()))),
        _ => None,
    }
}

/// Search all body scopes of an enum for a member by name.
///
/// Handles merged enums where a single enum symbol has multiple body scopes:
/// ```ts
/// enum A { X = 1 }
/// enum A { Y = 2 }
/// // The symbol `A` has two body scopes — this searches both.
/// ```
fn find_in_enum_body_scopes(
    member_name: Ident<'_>,
    enum_symbol_id: SymbolId,
    scoping: &Scoping,
    string_limit: usize,
) -> Option<ConstantValue> {
    let body_scopes = scoping.get_enum_body_scopes(enum_symbol_id)?;
    for &body_scope in body_scopes {
        if let Some(member_symbol_id) = scoping.get_binding(body_scope, member_name)
            && let Some(value) = scoping.get_enum_member_value(member_symbol_id)
            && !matches!(value, ConstantValue::String(s) if s.len() > string_limit)
        {
            return Some(value.clone());
        }
    }
    None
}

/// For merged enum declarations, find a bare identifier in a *sibling* body scope.
///
/// This handles the case where a later declaration references a member from an
/// earlier declaration without qualification:
/// ```ts
/// enum Foo { A = 1 }
/// enum Foo { B = A + 1 }  // `A` is in the first Foo's scope, not the current one
/// ```
///
/// Returns [`SiblingLookup::Unknown`] when the member exists but could not be evaluated, so
/// callers do not fall back to a global with the same name.
fn find_in_sibling_enum_scopes(
    name: &str,
    current_scope_id: ScopeId,
    enum_symbol_id: Option<SymbolId>,
    scoping: &Scoping,
) -> SiblingLookup {
    let Some(enum_symbol_id) = enum_symbol_id else { return SiblingLookup::Missing };
    let Some(body_scopes) = scoping.get_enum_body_scopes(enum_symbol_id) else {
        return SiblingLookup::Missing;
    };

    for &body_scope in body_scopes {
        if body_scope != current_scope_id
            && let Some(member_sym) = scoping.get_binding(body_scope, name.into())
        {
            return scoping
                .get_enum_member_value(member_sym)
                .cloned()
                .map_or(SiblingLookup::Unknown, SiblingLookup::Value);
        }
    }
    SiblingLookup::Missing
}
