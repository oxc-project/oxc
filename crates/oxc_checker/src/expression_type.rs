use oxc_ast::ast::Expression;
use oxc_span::CompactStr;
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
            // Literal expressions
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

            // Identifier — resolve through the symbol table
            Expression::Identifier(ident) => self.get_type_of_identifier(ident),

            // Not yet implemented
            _ => self.any_type,
        }
    }

    /// Resolve an identifier reference to its type.
    ///
    /// Looks up the reference -> symbol -> declaration -> type annotation.
    fn get_type_of_identifier(
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

    /// Get the type of a symbol by looking at its declaration's type annotation.
    pub fn get_type_of_symbol(&mut self, symbol_id: oxc_syntax::symbol::SymbolId) -> TypeId {
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
            // TODO: functions, parameters, classes, etc.
            _ => self.any_type,
        }
    }
}
