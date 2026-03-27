use std::{collections::HashMap, sync::Arc};

use oxc_ast::ast::{BindingPattern, Declaration, Function, Program, Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Semantic;
use oxc_span::{CompactStr, GetSpan};
use oxc_syntax::symbol::SymbolId;
use oxc_types::{IntrinsicType, ObjectFlags, TypeArena, TypeData, TypeFlags, TypeId, UnionType};
use smallvec::SmallVec;

/// TypeScript type checker.
///
/// The checker runs after semantic analysis and resolves types for all
/// expressions and declarations, emitting diagnostics for type errors.
///
/// Types are stored in a `TypeArena` and referenced by `TypeId`.
/// Well-known types (primitives) are pre-allocated during construction.
pub struct Checker<'a> {
    /// The semantic analysis result for the program being checked.
    pub(crate) semantic: Semantic<'a>,

    /// Arena storing all types created during checking.
    pub(crate) type_arena: TypeArena,

    /// Diagnostics collected during type checking.
    pub(crate) diagnostics: Vec<OxcDiagnostic>,

    /// Recursion depth counter to prevent stack overflow.
    pub(crate) recursion_depth: u32,

    /// Cache for deduplicating union types. Key is sorted constituent TypeIds.
    union_types: HashMap<Arc<SmallVec<[TypeId; 4]>>, TypeId>,

    /// Cache of resolved symbol types. Each symbol's type is computed at most
    /// once and stored here. Mirrors tsgo's valueSymbolLinks.resolvedType.
    pub(crate) symbol_type_cache: HashMap<SymbolId, TypeId>,

    /// Stack of symbols currently being resolved, for cycle detection.
    /// If we encounter a symbol already on this stack, we have a circular
    /// reference and return `any_type` to break the cycle.
    /// Mirrors tsgo's pushTypeResolution/popTypeResolution.
    pub(crate) resolving_symbols: Vec<SymbolId>,

    /// Cache of declared types for type-namespace symbols (type aliases,
    /// interfaces, classes, enums). Mirrors tsgo's getDeclaredTypeOfSymbol
    /// caching.
    pub(crate) declared_type_cache: HashMap<SymbolId, TypeId>,

    /// Global types from lib.d.ts (Array, Promise, etc.).
    pub(crate) global_types: HashMap<CompactStr, TypeId>,

    /// Stack of return types for enclosing functions.
    /// `Some(type_id)` = function has a declared return type annotation.
    /// `None` = function has no return type annotation (returns are unchecked).
    /// Empty = we're at the top level (return statements are invalid but the
    /// parser handles that).
    pub(crate) return_type_stack: Vec<Option<TypeId>>,

    // Well-known types, pre-allocated during construction.
    pub any_type: TypeId,
    pub unknown_type: TypeId,
    pub string_type: TypeId,
    pub number_type: TypeId,
    pub bigint_type: TypeId,
    pub boolean_type: TypeId,
    pub es_symbol_type: TypeId,
    pub void_type: TypeId,
    pub undefined_type: TypeId,
    pub null_type: TypeId,
    pub never_type: TypeId,
    /// The `object` non-primitive type (not `Object` interface).
    pub non_primitive_type: TypeId,
    /// The `true` literal type.
    pub true_type: TypeId,
    /// The `false` literal type.
    pub false_type: TypeId,
}

impl<'a> Checker<'a> {
    /// Create a new type checker from a completed semantic analysis.
    pub fn new(semantic: Semantic<'a>) -> Self {
        let mut type_arena = TypeArena::with_capacity(64);
        let global_types =
            crate::global_types::GlobalTypes::from_lib(&mut type_arena);
        Self::new_inner(semantic, type_arena, global_types.types)
    }

    fn new_inner(
        semantic: Semantic<'a>,
        mut type_arena: TypeArena,
        global_types: HashMap<CompactStr, TypeId>,
    ) -> Self {

        let any_type = new_intrinsic(&mut type_arena, TypeFlags::Any, "any");
        let unknown_type = new_intrinsic(&mut type_arena, TypeFlags::Unknown, "unknown");
        let string_type = new_intrinsic(&mut type_arena, TypeFlags::String, "string");
        let number_type = new_intrinsic(&mut type_arena, TypeFlags::Number, "number");
        let bigint_type = new_intrinsic(&mut type_arena, TypeFlags::BigInt, "bigint");
        let boolean_type = new_intrinsic(&mut type_arena, TypeFlags::Boolean, "boolean");
        let es_symbol_type = new_intrinsic(&mut type_arena, TypeFlags::ESSymbol, "symbol");
        let void_type = new_intrinsic(&mut type_arena, TypeFlags::Void, "void");
        let undefined_type = new_intrinsic(&mut type_arena, TypeFlags::Undefined, "undefined");
        let null_type = new_intrinsic(&mut type_arena, TypeFlags::Null, "null");
        let never_type = new_intrinsic(&mut type_arena, TypeFlags::Never, "never");
        let non_primitive_type =
            new_intrinsic(&mut type_arena, TypeFlags::NonPrimitive, "object");

        let true_type = type_arena.new_type(
            TypeFlags::BooleanLiteral,
            ObjectFlags::None,
            TypeData::Literal(oxc_types::LiteralType::Boolean(true)),
            None,
        );
        let false_type = type_arena.new_type(
            TypeFlags::BooleanLiteral,
            ObjectFlags::None,
            TypeData::Literal(oxc_types::LiteralType::Boolean(false)),
            None,
        );

        Self {
            semantic,
            type_arena,
            diagnostics: Vec::new(),
            recursion_depth: 0,
            union_types: HashMap::new(),
            symbol_type_cache: HashMap::new(),
            resolving_symbols: Vec::new(),
            declared_type_cache: HashMap::new(),
            global_types,
            return_type_stack: Vec::new(),
            any_type,
            unknown_type,
            string_type,
            number_type,
            bigint_type,
            boolean_type,
            es_symbol_type,
            void_type,
            undefined_type,
            null_type,
            never_type,
            non_primitive_type,
            true_type,
            false_type,
        }
    }

    /// Run the type checker on a program and return collected diagnostics.
    pub fn check_program(mut self, program: &Program<'a>) -> Vec<OxcDiagnostic> {
        self.check_source_elements(&program.body);
        std::mem::take(&mut self.diagnostics)
    }

    /// Check a single AST node, dispatching by kind.
    /// Equivalent to tsgo's `checkSourceElementWorker`.
    fn check_source_element(&mut self, stmt: &Statement<'a>) {
        match stmt {
            // Declarations
            Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    self.check_variable_declarator(declarator);
                }
            }
            Statement::FunctionDeclaration(func) => {
                self.check_function_body(func);
            }

            // Compound statements — recurse into children
            Statement::BlockStatement(block) => {
                self.check_source_elements(&block.body);
            }
            Statement::IfStatement(if_stmt) => {
                self.check_source_element(&if_stmt.consequent);
                if let Some(alt) = &if_stmt.alternate {
                    self.check_source_element(alt);
                }
            }
            Statement::ForStatement(for_stmt) => {
                self.check_source_element(&for_stmt.body);
            }
            Statement::ForInStatement(for_in) => {
                self.check_source_element(&for_in.body);
            }
            Statement::ForOfStatement(for_of) => {
                self.check_source_element(&for_of.body);
            }
            Statement::WhileStatement(while_stmt) => {
                self.check_source_element(&while_stmt.body);
            }
            Statement::DoWhileStatement(do_while) => {
                self.check_source_element(&do_while.body);
            }
            Statement::SwitchStatement(switch_stmt) => {
                for case in &switch_stmt.cases {
                    self.check_source_elements(&case.consequent);
                }
            }
            Statement::TryStatement(try_stmt) => {
                self.check_source_elements(&try_stmt.block.body);
                if let Some(handler) = &try_stmt.handler {
                    self.check_source_elements(&handler.body.body);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    self.check_source_elements(&finalizer.body);
                }
            }
            Statement::LabeledStatement(labeled) => {
                self.check_source_element(&labeled.body);
            }
            Statement::WithStatement(with_stmt) => {
                self.check_source_element(&with_stmt.body);
            }

            // Export declarations — recurse into inner declaration
            Statement::ExportNamedDeclaration(export) => {
                if let Some(decl) = &export.declaration {
                    self.check_exported_declaration(decl);
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                use oxc_ast::ast::ExportDefaultDeclarationKind;
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func) =
                    &export.declaration
                {
                    self.check_function_body(func);
                }
            }

            Statement::ExpressionStatement(expr_stmt) => {
                self.check_expression_statement(expr_stmt);
            }

            Statement::ReturnStatement(ret) => {
                self.check_return_statement(ret);
            }

            // Leaf statements / not yet implemented — no-op
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::EmptyStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::ClassDeclaration(_)
            | Statement::TSTypeAliasDeclaration(_)
            | Statement::TSInterfaceDeclaration(_)
            | Statement::TSEnumDeclaration(_)
            | Statement::TSModuleDeclaration(_)
            | Statement::TSGlobalDeclaration(_)
            | Statement::TSImportEqualsDeclaration(_)
            | Statement::ImportDeclaration(_)
            | Statement::ExportAllDeclaration(_)
            | Statement::TSExportAssignment(_)
            | Statement::TSNamespaceExportDeclaration(_) => {}
        }
    }

    /// Check a list of statements. Equivalent to tsgo's `checkSourceElements`.
    fn check_source_elements(&mut self, stmts: &[Statement<'a>]) {
        for stmt in stmts {
            self.check_source_element(stmt);
        }
    }

    /// Handle a declaration inside an export statement.
    fn check_exported_declaration(&mut self, decl: &Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    self.check_variable_declarator(declarator);
                }
            }
            Declaration::FunctionDeclaration(func) => {
                self.check_function_body(func);
            }
            Declaration::ClassDeclaration(_)
            | Declaration::TSTypeAliasDeclaration(_)
            | Declaration::TSInterfaceDeclaration(_)
            | Declaration::TSEnumDeclaration(_)
            | Declaration::TSModuleDeclaration(_)
            | Declaration::TSGlobalDeclaration(_)
            | Declaration::TSImportEqualsDeclaration(_) => {}
        }
    }

    fn check_variable_declarator(
        &mut self,
        decl: &oxc_ast::ast::VariableDeclarator<'a>,
    ) {
        // Only check when there's both a type annotation and initializer
        let Some(annotation) = &decl.type_annotation else {
            return;
        };
        let Some(init) = &decl.init else {
            return;
        };

        let declared_type = self.get_type_from_type_node(&annotation.type_annotation);
        let init_type = self.get_type_of_expression(init);

        if !self.is_type_assignable_to(init_type, declared_type) {
            let source_str = self.type_to_string(init_type);
            let target_str = self.type_to_string(declared_type);

            // Get the span of the binding identifier for the error label
            let label_span = if let BindingPattern::BindingIdentifier(id) = &decl.id {
                id.span
            } else {
                decl.id.span()
            };

            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Type '{source_str}' is not assignable to type '{target_str}'."
                ))
                .with_error_code("ts", "2322")
                .with_label(label_span),
            );
        }
    }

    /// Check a function's body with the return type context pushed.
    fn check_function_body(&mut self, func: &Function<'a>) {
        let return_type = func
            .return_type
            .as_ref()
            .map(|rt| self.get_type_from_type_node(&rt.type_annotation));
        self.return_type_stack.push(return_type);
        if let Some(body) = &func.body {
            self.check_source_elements(&body.statements);
        }
        self.return_type_stack.pop();
    }

    /// Check a return statement against the enclosing function's return type.
    fn check_return_statement(&mut self, ret: &oxc_ast::ast::ReturnStatement<'a>) {
        let Some(Some(expected_return_type)) = self.return_type_stack.last().copied() else {
            // No enclosing function or no return type annotation — skip
            return;
        };

        let actual_type = if let Some(arg) = &ret.argument {
            self.get_type_of_expression(arg)
        } else {
            self.undefined_type
        };

        if !self.is_type_assignable_to(actual_type, expected_return_type) {
            let source_str = self.type_to_string(actual_type);
            let target_str = self.type_to_string(expected_return_type);
            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Type '{source_str}' is not assignable to type '{target_str}'."
                ))
                .with_error_code("ts", "2322")
                .with_label(ret.span),
            );
        }
    }

    /// Look up a global type by name (e.g., "Array", "Promise").
    /// Returns `any_type` if the name is not found.
    pub fn get_global_type(&self, name: &str) -> TypeId {
        self.global_types
            .get(name)
            .copied()
            .unwrap_or(self.any_type)
    }

    /// Get the type arena (for testing/inspection).
    pub fn type_arena(&self) -> &TypeArena {
        &self.type_arena
    }

    /// Get the semantic analysis result.
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
    }

    /// Get or create a deduplicated union type from a list of constituent type IDs.
    ///
    /// Handles normalization: filters `never`, deduplicates, sorts.
    /// Returns `never` for empty, unwraps single-element unions.
    pub fn get_or_create_union_type(&mut self, mut types: Vec<TypeId>) -> TypeId {
        // Filter out `never` types
        types.retain(|&t| !self.type_arena.get_flags(t).intersects(TypeFlags::Never));

        if types.is_empty() {
            return self.never_type;
        }
        if types.len() == 1 {
            return types[0];
        }

        types.sort();
        types.dedup();

        if types.len() == 1 {
            return types[0];
        }

        let key: Arc<SmallVec<[TypeId; 4]>> = SmallVec::from_vec(types).into();

        let type_id = self.union_types.entry(key.clone()).or_insert_with_key(|key| {
            self.type_arena.new_type(
                TypeFlags::Union,
                ObjectFlags::None,
                TypeData::Union(UnionType { types: key.clone() }),
                None
            )
        });

        *type_id
    }

    /// Create an intersection type from a list of constituent type IDs.
    ///
    /// Handles normalization: deduplicates, sorts.
    /// Returns `unknown` for empty, unwraps single-element intersections.
    pub fn get_or_create_intersection_type(&mut self, mut types: Vec<TypeId>) -> TypeId {
        types.sort();
        types.dedup();

        if types.is_empty() {
            return self.unknown_type;
        }
        if types.len() == 1 {
            return types[0];
        }

        self.type_arena.new_type(
            TypeFlags::Intersection,
            ObjectFlags::None,
            TypeData::Intersection(oxc_types::IntersectionType {
                types: SmallVec::from_vec(types),
            }),
            None,
        )
    }
}

fn new_intrinsic(arena: &mut TypeArena, flags: TypeFlags, name: &'static str) -> TypeId {
    arena.new_type(
        flags,
        ObjectFlags::None,
        TypeData::Intrinsic(IntrinsicType { intrinsic_name: name }),
        None,
    )
}
