use rustc_hash::{FxHashMap, FxHashSet};

use super::TypeScript;

use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};

#[derive(Default)]
struct State<'a> {
    /// Deduplicate the `let` declarations` for namespace concatenation.
    /// `namespace foo {}; namespace {}` creates a single `let foo;`.
    names: FxHashSet<Atom<'a>>,

    /// Increment the argument name to avoid name clashes.
    arg_names: FxHashMap<Atom<'a>, usize>,
}

fn is_namespace(decl: &Declaration<'_>) -> bool {
    matches!(decl, Declaration::TSModuleDeclaration(decl) if !decl.modifiers.is_contains_declare())
}

// TODO:
// 1. register scope for the newly created function: <https://github.com/babel/babel/blob/08b0472069cd207f043dd40a4d157addfdd36011/packages/babel-plugin-transform-typescript/src/namespace.ts#L38>
impl<'a> TypeScript<'a> {
    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    pub(super) fn transform_statements_for_namespace(&self, stmts: &mut Vec<'a, Statement<'a>>) {
        // Only do the transform if a namespace declaration is found.
        if !stmts.iter().any(|stmt| match stmt {
            Statement::Declaration(decl) => is_namespace(decl),
            Statement::ModuleDeclaration(decl) => match &**decl {
                ModuleDeclaration::ExportNamedDeclaration(decl) => {
                    decl.declaration.as_ref().is_some_and(is_namespace)
                }
                _ => false,
            },
            _ => false,
        }) {
            return;
        }

        // Recreate the statements vec for memory efficiency.
        // Inserting the `let` declaration multiple times will reallocate the whole statements vec
        // every time a namespace declaration is encountered.
        let mut new_stmts = self.ctx.ast.new_vec();

        let mut state = State::default();

        for mut stmt in self.ctx.ast.move_statement_vec(stmts) {
            if !self.transform_statement_for_namespace(&mut state, &mut new_stmts, &mut stmt) {
                new_stmts.push(stmt);
            }
        }

        *stmts = new_stmts;
    }

    fn transform_statement_for_namespace(
        &self,
        state: &mut State<'a>,
        new_stmts: &mut Vec<'a, Statement<'a>>,
        stmt: &mut Statement<'a>,
    ) -> bool {
        let mut is_export = false;
        let ts_module_decl = match stmt {
            Statement::Declaration(Declaration::TSModuleDeclaration(ts_module_decl)) => {
                ts_module_decl
            }
            Statement::ModuleDeclaration(decl) => match &mut **decl {
                ModuleDeclaration::ExportNamedDeclaration(decl) => {
                    if let Some(Declaration::TSModuleDeclaration(ts_module_decl)) =
                        decl.declaration.as_mut()
                    {
                        is_export = true;
                        ts_module_decl
                    } else {
                        return false;
                    }
                }
                _ => return false,
            },
            _ => return false,
        };

        if ts_module_decl.modifiers.is_contains_declare() {
            return false;
        }

        let name = ts_module_decl.id.name().clone();

        if state.names.insert(name.clone()) {
            let stmt = self.create_variable_declaration_statement(&name, is_export);
            new_stmts.push(stmt);
        }

        let namespace = self.transform_namespace(state, ts_module_decl);
        new_stmts.push(namespace);
        true
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                         ^^^^^^^
    fn create_variable_declaration_statement(
        &self,
        name: &Atom<'a>,
        is_export: bool,
    ) -> Statement<'a> {
        let kind = VariableDeclarationKind::Let;
        let declarators = {
            let ident = BindingIdentifier::new(SPAN, name.clone());
            let pattern_kind = self.ctx.ast.binding_pattern_identifier(ident);
            let binding = self.ctx.ast.binding_pattern(pattern_kind, None, false);
            let decl = self.ctx.ast.variable_declarator(SPAN, kind, binding, None, false);
            self.ctx.ast.new_vec_single(decl)
        };
        let decl = Declaration::VariableDeclaration(self.ctx.ast.variable_declaration(
            SPAN,
            kind,
            declarators,
            Modifiers::empty(),
        ));
        if is_export {
            self.ctx.ast.module_declaration(ModuleDeclaration::ExportNamedDeclaration(
                self.ctx.ast.export_named_declaration(
                    SPAN,
                    Some(decl),
                    self.ctx.ast.new_vec(),
                    None,
                    ImportOrExportKind::Value,
                    None,
                ),
            ))
        } else {
            Statement::Declaration(decl)
        }
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    fn transform_namespace(
        &self,
        state: &mut State<'a>,
        block: &mut Box<'a, TSModuleDeclaration<'a>>,
    ) -> Statement<'a> {
        let body_statements = match &mut block.body {
            Some(TSModuleDeclarationBody::TSModuleDeclaration(decl)) => {
                let transformed_module_block = self.transform_namespace(state, decl);
                self.ctx.ast.new_vec_single(transformed_module_block)
            }
            Some(TSModuleDeclarationBody::TSModuleBlock(ts_module_block)) => {
                self.ctx.ast.move_statement_vec(&mut ts_module_block.body)
            }
            None => self.ctx.ast.new_vec(),
        };

        let name = block.id.name();

        // `(function (_N) { var x; })(N || (N = {}))`;
        //  ^^^^^^^^^^^^^^^^^^^^^^^^^^
        let callee = {
            let body = self.ctx.ast.function_body(SPAN, self.ctx.ast.new_vec(), body_statements);
            let arg_name = self.get_namespace_arg_name(state, name);
            let params = {
                let ident =
                    self.ctx.ast.binding_pattern_identifier(BindingIdentifier::new(SPAN, arg_name));
                let pattern = self.ctx.ast.binding_pattern(ident, None, false);
                let items =
                    self.ctx.ast.new_vec_single(self.ctx.ast.plain_formal_parameter(SPAN, pattern));
                self.ctx.ast.formal_parameters(
                    SPAN,
                    FormalParameterKind::FormalParameter,
                    items,
                    None,
                )
            };
            let function = self.ctx.ast.plain_function(
                FunctionType::FunctionExpression,
                SPAN,
                None,
                params,
                Some(body),
            );
            let function_expr = self.ctx.ast.function_expression(function);
            self.ctx.ast.parenthesized_expression(SPAN, function_expr)
        };

        // `(function (_N) { var x; })(N || (N = {}))`;
        //                             ^^^^^^^^^^^^^
        let arguments = {
            let logical_left = {
                let ident = IdentifierReference::new(SPAN, name.clone());
                self.ctx.ast.identifier_reference_expression(ident)
            };
            let logical_right = {
                let assign_left = self.ctx.ast.simple_assignment_target_identifier(
                    IdentifierReference::new(SPAN, name.clone()),
                );
                let assign_right =
                    self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
                let op = AssignmentOperator::Assign;
                let assign_expr =
                    self.ctx.ast.assignment_expression(SPAN, op, assign_left, assign_right);
                self.ctx.ast.parenthesized_expression(SPAN, assign_expr)
            };
            self.ctx.ast.new_vec_single(Argument::Expression(self.ctx.ast.logical_expression(
                SPAN,
                logical_left,
                LogicalOperator::Or,
                logical_right,
            )))
        };
        let expr = self.ctx.ast.call_expression(SPAN, callee, arguments, false, None);
        self.ctx.ast.expression_statement(SPAN, expr)
    }

    fn get_namespace_arg_name(&self, state: &mut State<'a>, name: &Atom<'a>) -> Atom<'a> {
        let count = state.arg_names.entry(name.clone()).or_insert(0);
        *count += 1;
        let name = if *count > 1 { format!("_{name}{count}") } else { format!("_{name}") };
        self.ctx.ast.new_atom(&name)
    }
}
