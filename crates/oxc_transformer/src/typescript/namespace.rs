use rustc_hash::FxHashSet;

use super::TypeScript;

use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, syntax_directed_operations::BoundNames};
use oxc_span::{Atom, SPAN};
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};

// TODO:
// 1. register scope for the newly created function: <https://github.com/babel/babel/blob/08b0472069cd207f043dd40a4d157addfdd36011/packages/babel-plugin-transform-typescript/src/namespace.ts#L38>
impl<'a> TypeScript<'a> {
    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    pub(super) fn transform_statements_for_namespace(&self, stmts: &mut Vec<'a, Statement<'a>>) {
        let mut names: FxHashSet<Atom<'a>> = FxHashSet::default();
        let mut new_stmts = vec![];

        for (index, stmt) in stmts.iter().enumerate() {
            match stmt {
                Statement::Declaration(Declaration::TSModuleDeclaration(decl)) => {
                    if !decl.modifiers.is_contains_declare() {
                        let name = decl.id.name().clone();
                        if let Some(transformed_stmt) =
                            self.handle_nested(self.ctx.ast.copy(decl), None)
                        {
                            if names.insert(name.clone()) {
                                new_stmts.push((
                                    index,
                                    Statement::Declaration(self.create_variable_declaration(&name)),
                                ));
                            }
                            new_stmts.push((index, transformed_stmt));
                        }
                    }
                }

                Statement::ModuleDeclaration(ref module_decl) => {
                    if let ModuleDeclaration::ExportNamedDeclaration(export_decl) = &**module_decl {
                        if let Some(Declaration::TSModuleDeclaration(decl)) =
                            &export_decl.declaration
                        {
                            if !decl.modifiers.is_contains_declare() {
                                let name = decl.id.name().clone();
                                if let Some(transformed_stmt) =
                                    self.handle_nested(self.ctx.ast.copy(decl), None)
                                {
                                    if names.insert(name.clone()) {
                                        new_stmts.push((
                                            index,
                                            self.ctx.ast.module_declaration(
                                                ModuleDeclaration::ExportNamedDeclaration(
                                                    self.ctx.ast.export_named_declaration(
                                                        SPAN,
                                                        Some(
                                                            self.create_variable_declaration(&name),
                                                        ),
                                                        self.ctx.ast.new_vec(),
                                                        None,
                                                        ImportOrExportKind::Value,
                                                        None,
                                                    ),
                                                ),
                                            ),
                                        ));
                                    }
                                    new_stmts.push((index, transformed_stmt));
                                    continue;
                                }
                            }
                        }
                    }
                    module_decl.bound_names(&mut |id| {
                        names.insert(id.name.clone());
                    });
                }
                // Collect bindings from class, function, variable and enum declarations
                Statement::Declaration(Declaration::FunctionDeclaration(ref decl)) => {
                    if let Some(ident) = &decl.id {
                        names.insert(ident.name.clone());
                    }
                }
                Statement::Declaration(Declaration::ClassDeclaration(ref decl)) => {
                    if let Some(ident) = &decl.id {
                        names.insert(ident.name.clone());
                    }
                }
                Statement::Declaration(Declaration::TSEnumDeclaration(ref decl)) => {
                    names.insert(decl.id.name.clone());
                }
                Statement::Declaration(Declaration::VariableDeclaration(ref decl)) => {
                    decl.bound_names(&mut |id| {
                        names.insert(id.name.clone());
                    });
                }
                _ => {}
            }
        }

        let mut deleted_indexes = FxHashSet::default();
        new_stmts.into_iter().rev().for_each(|(index, stmt)| {
            // remove original statement
            if deleted_indexes.insert(index) {
                stmts.remove(index);
            }
            stmts.insert(index, stmt);
        });
    }

    fn handle_nested(
        &self,
        decl: TSModuleDeclaration<'a>,
        parent_export: Option<Expression<'a>>,
    ) -> Option<Statement<'a>> {
        let mut names: FxHashSet<Atom<'a>> = FxHashSet::default();
        let real_name = decl.id.name();

        let name = self.ctx.ast.new_atom(&format!("_{}", real_name.clone())); // path.scope.generateUid(realName.name);

        let namespace_top_level = if let Some(body) = decl.body {
            match body {
                TSModuleDeclarationBody::TSModuleBlock(mut block) => {
                    self.ctx.ast.move_statement_vec(&mut block.body)
                }
                // We handle `namespace X.Y {}` as if it was
                //   namespace X {
                //     export namespace Y {}
                //   }
                TSModuleDeclarationBody::TSModuleDeclaration(declaration) => {
                    self.ctx.ast.new_vec_single(self.ctx.ast.module_declaration(
                        ModuleDeclaration::ExportNamedDeclaration(
                            self.ctx.ast.export_named_declaration(
                                SPAN,
                                Some(Declaration::TSModuleDeclaration(
                                    self.ctx.ast.copy(&declaration),
                                )),
                                self.ctx.ast.new_vec(),
                                None,
                                ImportOrExportKind::Value,
                                None,
                            ),
                        ),
                    ))
                }
            }
        } else {
            self.ctx.ast.new_vec()
        };

        let mut is_empty = true;

        let mut new_stmts = self.ctx.ast.new_vec();

        for stmt in namespace_top_level {
            match stmt {
                Statement::Declaration(Declaration::TSModuleDeclaration(decl)) => {
                    let module_name = decl.id.name().clone();
                    if let Some(transformed) = self.handle_nested(decl.unbox(), None) {
                        is_empty = false;
                        if names.insert(module_name.clone()) {
                            new_stmts.push(Statement::Declaration(
                                self.create_variable_declaration(&module_name),
                            ));
                        }
                        new_stmts.push(transformed);
                    }
                }
                Statement::Declaration(Declaration::ClassDeclaration(decl)) => {
                    is_empty = false;
                    decl.bound_names(&mut |id| {
                        names.insert(id.name.clone());
                    });
                    new_stmts.push(Statement::Declaration(Declaration::ClassDeclaration(decl)));
                }
                Statement::Declaration(decl) if decl.is_typescript_syntax() => {
                    is_empty = true;
                }
                Statement::ModuleDeclaration(decl) => {
                    if let ModuleDeclaration::ExportNamedDeclaration(export_decl) = decl.unbox() {
                        let export_decl = export_decl.unbox();
                        if let Some(decl) = export_decl.declaration {
                            if decl.modifiers().is_some_and(Modifiers::is_contains_declare) {
                                continue;
                            }
                            match decl {
                                Declaration::TSEnumDeclaration(enum_decl) => {
                                    is_empty = false;
                                    self.add_declaration(
                                        Some(enum_decl.id.name.clone()),
                                        Declaration::TSEnumDeclaration(enum_decl),
                                        &name,
                                        &mut names,
                                        &mut new_stmts,
                                    );
                                }
                                Declaration::FunctionDeclaration(func_decl) => {
                                    is_empty = false;
                                    self.add_declaration(
                                        func_decl.id.as_ref().map(|ident| ident.name.clone()),
                                        Declaration::FunctionDeclaration(func_decl),
                                        &name,
                                        &mut names,
                                        &mut new_stmts,
                                    );
                                }
                                Declaration::ClassDeclaration(class_decl) => {
                                    is_empty = false;
                                    self.add_declaration(
                                        class_decl.id.as_ref().map(|ident| ident.name.clone()),
                                        Declaration::ClassDeclaration(class_decl),
                                        &name,
                                        &mut names,
                                        &mut new_stmts,
                                    );
                                }
                                Declaration::VariableDeclaration(var_decl) => {
                                    is_empty = false;
                                    let stmt = self.handle_variable_declaration(var_decl, &name);
                                    new_stmts.push(stmt);
                                }
                                Declaration::TSModuleDeclaration(module_decl) => {
                                    let module_name = module_decl.id.name().clone();
                                    if let Some(transformed) = self.handle_nested(
                                        module_decl.unbox(),
                                        Some(self.ctx.ast.identifier_reference_expression(
                                            IdentifierReference::new(SPAN, name.clone()),
                                        )),
                                    ) {
                                        is_empty = false;
                                        if names.insert(module_name.clone()) {
                                            new_stmts.push(Statement::Declaration(
                                                self.create_variable_declaration(&module_name),
                                            ));
                                        }
                                        new_stmts.push(transformed);
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            new_stmts.push(Statement::ModuleDeclaration(self.ctx.ast.alloc(
                                ModuleDeclaration::ExportNamedDeclaration(
                                    self.ctx.ast.alloc(export_decl),
                                ),
                            )));
                        }
                    }
                }
                stmt => {
                    is_empty = false;
                    new_stmts.push(stmt);
                }
            }
        }

        if is_empty {
            return None;
        }

        let name = decl.id.name();
        let namespace = self.transform_namespace(name, real_name, new_stmts, parent_export);
        Some(namespace)
    }

    // fn transform_statement_for_namespace(
    // &self,
    // state: &mut State<'a>,
    // new_stmts: &mut Vec<'a, Statement<'a>>,
    // stmt: &mut Statement<'a>,
    // ) -> bool {
    // let mut is_export = false;
    // let ts_module_decl = match stmt {
    // Statement::Declaration(Declaration::TSModuleDeclaration(ts_module_decl)) => {
    // ts_module_decl
    // }
    // Statement::ModuleDeclaration(decl) => match &mut **decl {
    // ModuleDeclaration::ExportNamedDeclaration(decl) => {
    // if let Some(Declaration::TSModuleDeclaration(ts_module_decl)) =
    // decl.declaration.as_mut()
    // {
    // is_export = true;
    // ts_module_decl
    // } else {
    // return false;
    // }
    // }
    // _ => return false,
    // },
    // _ => return false,
    // };

    // if ts_module_decl.modifiers.is_contains_declare() {
    // return false;
    // }

    // let name = ts_module_decl.id.name().clone();

    // if state.names.insert(name.clone()) {
    // let stmt = self.create_variable_declaration_statement(&name, is_export);
    // new_stmts.push(stmt);
    // }

    // let arg_name = decl.id.name();
    // let namespace = self.transform_namespace(arg_name, real_name, ts_module_decl);
    // new_stmts.push(namespace);
    // true
    // }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                         ^^^^^^^
    fn create_variable_declaration(&self, name: &Atom<'a>) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Let;
        let declarator = {
            let ident = BindingIdentifier::new(SPAN, name.clone());
            let pattern_kind = self.ctx.ast.binding_pattern_identifier(ident);
            let binding = self.ctx.ast.binding_pattern(pattern_kind, None, false);
            let decl = self.ctx.ast.variable_declarator(SPAN, kind, binding, None, false);
            self.ctx.ast.new_vec_single(decl)
        };
        Declaration::VariableDeclaration(self.ctx.ast.variable_declaration(
            SPAN,
            kind,
            declarator,
            Modifiers::empty(),
        ))
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    fn transform_namespace(
        &self,
        arg_name: &Atom<'a>,
        real_name: &Atom<'a>,
        stmts: Vec<'a, Statement<'a>>,
        parent_export: Option<Expression<'a>>,
        // state: &mut State<'a>,
        // block: &mut Box<'a, TSModuleDeclaration<'a>>,
    ) -> Statement<'a> {
        // let body_statements = match &mut block.body {
        // Some(TSModuleDeclarationBody::TSModuleDeclaration(decl)) => {
        // let transformed_module_block = self.transform_namespace(state, decl);
        // self.ctx.ast.new_vec_single(transformed_module_block)
        // }
        // Some(TSModuleDeclarationBody::TSModuleBlock(ts_module_block)) => {
        // self.ctx.ast.move_statement_vec(&mut ts_module_block.body)
        // }
        // None => self.ctx.ast.new_vec(),
        // };

        // let name = block.id.name();

        // `(function (_N) { var x; })(N || (N = {}))`;
        //  ^^^^^^^^^^^^^^^^^^^^^^^^^^
        let callee = {
            let body = self.ctx.ast.function_body(SPAN, self.ctx.ast.new_vec(), stmts);
            // let arg_name = self.get_namespace_arg_name(state, name);
            let params = {
                let ident = self.ctx.ast.binding_pattern_identifier(BindingIdentifier::new(
                    SPAN,
                    self.ctx.ast.new_atom(&format!("_{arg_name}")),
                ));
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

        // (function (_N) { var M; (function (_M) { var x; })(M || (M = _N.M || (_N.M = {})));})(N || (N = {}));
        //                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^
        //                                                   Nested namespace arguments         Normal namespace arguments
        let arguments = {
            // M
            let logical_left = {
                let ident = IdentifierReference::new(SPAN, real_name.clone());
                self.ctx.ast.identifier_reference_expression(ident)
            };

            // (_N.M = {})
            let mut logical_right = {
                let assign_left = if let Some(parent_export) = self.ctx.ast.copy(&parent_export) {
                    self.ctx.ast.simple_assignment_target_member_expression(
                        self.ctx.ast.static_member(
                            SPAN,
                            parent_export,
                            self.ctx.ast.identifier_name(SPAN, real_name),
                            false,
                        ),
                    )
                } else {
                    self.ctx.ast.simple_assignment_target_identifier(IdentifierReference::new(
                        SPAN,
                        real_name.clone(),
                    ))
                };

                let assign_right =
                    self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
                let op = AssignmentOperator::Assign;
                let assign_expr =
                    self.ctx.ast.assignment_expression(SPAN, op, assign_left, assign_right);
                self.ctx.ast.parenthesized_expression(SPAN, assign_expr)
            };

            // (M = _N.M || (_N.M = {}))
            if let Some(parent_export) = parent_export {
                let assign_left = self.ctx.ast.simple_assignment_target_identifier(
                    IdentifierReference::new(SPAN, real_name.clone()),
                );

                let assign_right = self.ctx.ast.logical_expression(
                    SPAN,
                    self.ctx.ast.static_member_expression(
                        SPAN,
                        parent_export,
                        self.ctx.ast.identifier_name(SPAN, real_name),
                        false,
                    ),
                    LogicalOperator::Or,
                    logical_right,
                );

                let op = AssignmentOperator::Assign;
                logical_right = self.ctx.ast.parenthesized_expression(
                    SPAN,
                    self.ctx.ast.assignment_expression(SPAN, op, assign_left, assign_right),
                );
            }

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

    fn add_declaration(
        &self,
        id: Option<Atom<'a>>,
        decl: Declaration<'a>,
        name: &Atom<'a>,
        names: &mut FxHashSet<Atom<'a>>,
        new_stmts: &mut Vec<'a, Statement<'a>>,
    ) {
        if let Some(item_name) = id {
            names.insert(item_name.clone());
            new_stmts.push(Statement::Declaration(decl));
            let assignment_statement = self.create_assignment_statement(name, &item_name);
            new_stmts.push(assignment_statement);
        }
    }

    fn create_assignment_statement(&self, name: &Atom<'a>, item_name: &Atom<'a>) -> Statement<'a> {
        let ident = self.ctx.ast.identifier_reference(SPAN, name.as_str());
        let object = self.ctx.ast.identifier_reference_expression(ident);
        let property = IdentifierName::new(SPAN, item_name.clone());
        let left = self.ctx.ast.static_member(SPAN, object, property, false);
        let left = SimpleAssignmentTarget::MemberAssignmentTarget(self.ctx.ast.alloc(left));
        let left = AssignmentTarget::SimpleAssignmentTarget(left);
        let ident = self.ctx.ast.identifier_reference(SPAN, item_name.as_str());
        let right = self.ctx.ast.identifier_reference_expression(ident);
        let op = AssignmentOperator::Assign;
        let assign_expr = self.ctx.ast.assignment_expression(SPAN, op, left, right);
        self.ctx.ast.expression_statement(SPAN, assign_expr)
    }

    /// Convert `export const foo = 1` to `Namespace.foo = 1`;
    fn handle_variable_declaration(
        &self,
        mut var_decl: Box<'a, VariableDeclaration<'a>>,
        name: &Atom<'a>,
    ) -> Statement<'a> {
        var_decl.declarations.iter_mut().for_each(|declarator| {
            let Some(property_name) = declarator.id.get_identifier() else {
                return;
            };
            if let Some(init) = &declarator.init {
                declarator.init = Some(self.ctx.ast.assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    self.ctx.ast.simple_assignment_target_member_expression(
                        self.ctx.ast.static_member(
                            SPAN,
                            self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                                SPAN,
                                name.clone(),
                            )),
                            IdentifierName::new(SPAN, property_name.clone()),
                            false,
                        ),
                    ),
                    self.ctx.ast.copy(init),
                ));
            }
        });
        Statement::Declaration(Declaration::VariableDeclaration(var_decl))
    }
}
