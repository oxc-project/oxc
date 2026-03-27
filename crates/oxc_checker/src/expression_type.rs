use oxc_ast::ast::{BinaryExpression, Expression, UnaryExpression};
use oxc_span::CompactStr;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use oxc_syntax::symbol::SymbolId;
use oxc_types::{LiteralType, ObjectFlags, TypeData, TypeFlags, TypeId};

use crate::Checker;

impl Checker<'_> {
    /// Get the type of an expression.
    ///
    /// For literals, returns the corresponding literal type.
    /// For identifiers, resolves the symbol and returns its declared type.
    /// Unimplemented expressions fall back to `any`.
    pub fn get_type_of_expression(&mut self, expr: &Expression<'_>) -> TypeId {
        // Guard against infinite recursion (e.g., `const x = x`)
        if self.recursion_depth > 100 {
            return self.any_type;
        }
        self.recursion_depth += 1;
        let result = self.get_type_of_expression_inner(expr);
        self.recursion_depth -= 1;
        result
    }

    fn get_type_of_expression_inner(&mut self, expr: &Expression<'_>) -> TypeId {
        match expr {
            Expression::StringLiteral(lit) => self.type_arena.new_type(
                TypeFlags::StringLiteral,
                ObjectFlags::None,
                TypeData::Literal(LiteralType::String(CompactStr::new(&lit.value))),
                None,
            ),
            Expression::NumericLiteral(lit) => self.type_arena.new_type(
                TypeFlags::NumberLiteral,
                ObjectFlags::None,
                TypeData::Literal(LiteralType::Number(lit.value)),
                None,
            ),
            Expression::BigIntLiteral(lit) => self.type_arena.new_type(
                TypeFlags::BigIntLiteral,
                ObjectFlags::None,
                TypeData::Literal(LiteralType::BigInt(CompactStr::new(
                    lit.value.as_str(),
                ))),
                None,
            ),
            Expression::BooleanLiteral(lit) => {
                if lit.value {
                    self.true_type
                } else {
                    self.false_type
                }
            }
            Expression::NullLiteral(_) => self.null_type,
            Expression::Identifier(ident) => self.get_type_of_identifier(ident),
            Expression::ParenthesizedExpression(paren) => {
                self.get_type_of_expression(&paren.expression)
            }
            // Type assertions — return the asserted type
            Expression::TSAsExpression(expr) => {
                self.get_type_from_type_node(&expr.type_annotation)
            }
            Expression::TSTypeAssertion(expr) => {
                self.get_type_from_type_node(&expr.type_annotation)
            }
            // `satisfies` checks but returns the expression's type, not the annotation
            Expression::TSSatisfiesExpression(expr) => {
                self.get_type_of_expression(&expr.expression)
            }
            // Non-null assertion — return the expression type (TODO: remove null/undefined)
            Expression::TSNonNullExpression(expr) => {
                self.get_type_of_expression(&expr.expression)
            }

            // Unary expressions
            Expression::UnaryExpression(expr) => self.get_type_of_unary_expression(expr),

            // Binary expressions
            Expression::BinaryExpression(expr) => self.get_type_of_binary_expression(expr),

            // Conditional (ternary) — union of both branches
            Expression::ConditionalExpression(expr) => {
                let true_type = self.get_type_of_expression(&expr.consequent);
                let false_type = self.get_type_of_expression(&expr.alternate);
                self.get_or_create_union_type(vec![true_type, false_type])
            }

            // Template literals — always string (simplified; tsc can produce literal types)
            Expression::TemplateLiteral(_) => self.string_type,

            // Sequence expression — type of the last element
            Expression::SequenceExpression(expr) => {
                if let Some(last) = expr.expressions.last() {
                    self.get_type_of_expression(last)
                } else {
                    self.undefined_type
                }
            }

            // void x — always undefined
            // (handled in unary, but keeping note)

            // Logical expressions — simplified to union of both sides
            Expression::LogicalExpression(expr) => {
                let left_type = self.get_type_of_expression(&expr.left);
                let right_type = self.get_type_of_expression(&expr.right);
                self.get_or_create_union_type(vec![left_type, right_type])
            }

            // ++x, x++ etc — returns number (simplified; bigint not handled)
            Expression::UpdateExpression(_) => self.number_type,

            // Not yet implemented — return `any`
            Expression::RegExpLiteral(_)
            | Expression::MetaProperty(_)
            | Expression::Super(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::CallExpression(_)
            | Expression::ChainExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ImportExpression(_)
            | Expression::NewExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::ThisExpression(_)
            | Expression::YieldExpression(_)
            | Expression::PrivateInExpression(_)
            | Expression::JSXElement(_)
            | Expression::JSXFragment(_)
            | Expression::TSInstantiationExpression(_)
            | Expression::V8IntrinsicExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_) => self.any_type,
        }
    }

    /// Resolve an identifier reference to its type.
    ///
    /// Looks up the reference -> symbol -> declaration -> type annotation.
    pub(crate) fn get_type_of_identifier(
        &mut self,
        ident: &oxc_ast::ast::IdentifierReference<'_>,
    ) -> TypeId {
        let Some(reference_id) = ident.reference_id.get() else {
            return self.any_type;
        };

        let reference = self.semantic().scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else {
            // Unresolved reference (global variable)
            return self.any_type;
        };

        self.get_type_of_symbol(symbol_id)
    }

    /// Get the type of a symbol, with caching and cycle detection.
    ///
    /// On first call, resolves the symbol's type from its declaration and caches
    /// the result. Subsequent calls return the cached type. If the symbol is
    /// already being resolved (circular reference), returns `any_type`.
    ///
    /// Mirrors tsgo's `getTypeOfSymbol` with `valueSymbolLinks.resolvedType`
    /// caching and `pushTypeResolution`/`popTypeResolution` cycle detection.
    pub fn get_type_of_symbol(&mut self, symbol_id: SymbolId) -> TypeId {
        // Check cache
        if let Some(&cached) = self.symbol_type_cache.get(&symbol_id) {
            return cached;
        }

        // Cycle detection: if this symbol is already being resolved, break cycle
        if self.resolving_symbols.contains(&symbol_id) {
            return self.any_type;
        }

        // Push onto resolution stack, resolve, pop, cache
        self.resolving_symbols.push(symbol_id);
        let result = self.resolve_symbol_type(symbol_id);
        self.resolving_symbols.pop();
        self.symbol_type_cache.insert(symbol_id, result);
        result
    }

    /// Resolve the type of a symbol from its declaration.
    ///
    /// This is the uncached inner logic — callers should use `get_type_of_symbol`.
    fn resolve_symbol_type(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        match node.kind() {
            AstKind::VariableDeclarator(decl) => {
                if let Some(annotation) = &decl.type_annotation {
                    self.get_type_from_type_node(&annotation.type_annotation)
                } else if let Some(init) = &decl.init {
                    // Infer type from initializer if no annotation
                    self.get_type_of_expression(init)
                } else {
                    self.any_type
                }
            }
            AstKind::FormalParameter(param) => {
                if let Some(annotation) = &param.type_annotation {
                    self.get_type_from_type_node(&annotation.type_annotation)
                } else {
                    self.any_type
                }
            }
            // TODO: functions, classes, enums, type aliases, interfaces, etc.
            _ => self.any_type,
        }
    }

    /// Get the result type of a unary expression.
    fn get_type_of_unary_expression(&mut self, expr: &UnaryExpression<'_>) -> TypeId {
        match expr.operator {
            // typeof always returns a string
            UnaryOperator::Typeof => self.string_type,
            // void always returns undefined
            UnaryOperator::Void => self.undefined_type,
            // ! returns boolean
            UnaryOperator::LogicalNot => self.boolean_type,
            // delete returns boolean
            UnaryOperator::Delete => self.boolean_type,
            // +x always returns number
            UnaryOperator::UnaryPlus => self.number_type,
            // -x and ~x return number or bigint depending on operand
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                let operand_type = self.get_type_of_expression(&expr.argument);
                let operand_flags = self.type_arena.get_flags(operand_type);
                if operand_flags.intersects(TypeFlags::BigIntLike) {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }
        }
    }

    /// Get the result type of a binary expression.
    fn get_type_of_binary_expression(&mut self, expr: &BinaryExpression<'_>) -> TypeId {
        match expr.operator {
            // Comparison and equality operators always return boolean
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::In
            | BinaryOperator::Instanceof => self.boolean_type,

            // Arithmetic operators (not +) return number or bigint
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd => {
                let left_type = self.get_type_of_expression(&expr.left);
                let right_type = self.get_type_of_expression(&expr.right);
                let left_flags = self.type_arena.get_flags(left_type);
                let right_flags = self.type_arena.get_flags(right_type);
                if left_flags.intersects(TypeFlags::BigIntLike)
                    && right_flags.intersects(TypeFlags::BigIntLike)
                {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }

            // + is special: string concat if either side is string-like, otherwise number
            BinaryOperator::Addition => {
                let left_type = self.get_type_of_expression(&expr.left);
                let right_type = self.get_type_of_expression(&expr.right);
                let left_flags = self.type_arena.get_flags(left_type);
                let right_flags = self.type_arena.get_flags(right_type);
                if left_flags.intersects(TypeFlags::StringLike)
                    || right_flags.intersects(TypeFlags::StringLike)
                {
                    self.string_type
                } else if left_flags.intersects(TypeFlags::BigIntLike)
                    && right_flags.intersects(TypeFlags::BigIntLike)
                {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }
        }
    }
}
