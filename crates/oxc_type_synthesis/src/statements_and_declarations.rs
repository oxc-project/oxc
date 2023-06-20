use std::{borrow::Cow, collections::HashMap};

use ezno_checker::{
    self, context::VariableId, structures::variables::VariableMutability, CheckingData,
    Environment, FSResolver, RegisterOnExisting, TypeId,
};
use oxc_ast::{
    self,
    ast::{self, Statement},
};
use oxc_span::GetSpan;

use super::{oxc_span_to_source_map_span, types::synthesize_type_annotation};
use crate::{
    expressions::{self, synthesize_expression},
    functions::OxcFunction,
};

/// See `checking.md`s Hoisting section in docs for details
pub(crate) fn hoist_statements<T: FSResolver>(
    statements: &[ast::Statement],
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) {
    // TODO temp?
    let mut idx_to_types = HashMap::new();

    // First stage
    for (idx, statement) in statements.iter().enumerate() {
        if let Statement::Declaration(declaration) = statement {
            match declaration {
                ast::Declaration::VariableDeclaration(_)
                | ast::Declaration::FunctionDeclaration(_) => {}
                ast::Declaration::ClassDeclaration(_) => {}
                ast::Declaration::TSTypeAliasDeclaration(alias) => {
                    if alias.type_parameters.is_some() {
                        todo!()
                    }

                    // TODO eager and so won't work with hoisting
                    let to = synthesize_type_annotation(
                        &alias.type_annotation,
                        environment,
                        checking_data,
                    );

                    environment.new_alias(&alias.id.name.as_str(), to, &mut checking_data.types);
                }
                ast::Declaration::TSInterfaceDeclaration(interface) => {
                    let ty = environment.new_interface(
                        &interface.id.name.as_str(),
                        oxc_span_to_source_map_span(interface.span),
                        &mut checking_data.types,
                    );
                    idx_to_types.insert(idx, ty);
                }
                ast::Declaration::TSEnumDeclaration(_) => {}
                ast::Declaration::TSModuleDeclaration(_) => {}
                ast::Declaration::TSImportEqualsDeclaration(_) => {}
            }
        }
    }

    // Second stage
    for (idx, statement) in statements.iter().enumerate() {
        match statement {
            Statement::ModuleDeclaration(_) => todo!(),
            Statement::Declaration(declaration) => match declaration {
                ast::Declaration::VariableDeclaration(declaration) => {
                    let is_declare = declaration.modifiers.contains(ast::ModifierKind::Declare);
                    let is_const = matches!(declaration.kind, ast::VariableDeclarationKind::Const);

                    for declaration in declaration.declarations.iter() {
                        let ty = declaration.id.type_annotation.as_ref().map(|ta| {
                            synthesize_type_annotation(
                                &ta.type_annotation,
                                environment,
                                checking_data,
                            )
                        });

                        // TODO save ty
                        let behavior = if is_declare {
                            ezno_checker::context::VariableRegisterBehavior::Declare {
                                base: ty.unwrap(),
                            }
                        } else {
                            ezno_checker::context::VariableRegisterBehavior::Register {
                                mutability: if is_const {
                                    VariableMutability::Constant
                                } else {
                                    VariableMutability::Mutable { reassignment_constraint: ty }
                                },
                            }
                        };
                        register_variable(
                            &declaration.id.kind,
                            &declaration.span,
                            environment,
                            checking_data,
                            behavior,
                        );
                    }
                }
                ast::Declaration::FunctionDeclaration(func) => {
                    // TODO unsynthesized function? ...
                    let behavior = ezno_checker::context::VariableRegisterBehavior::Register {
                        // TODO
                        mutability:
                            ezno_checker::structures::variables::VariableMutability::Constant,
                    };
                    environment
                        .register_variable(
                            func.id.as_ref().unwrap().name.as_str(),
                            VariableId(oxc_span_to_source_map_span(func.span)),
                            behavior,
                            &mut checking_data.types,
                        )
                        .unwrap();
                }
                ast::Declaration::ClassDeclaration(_) => {}
                ast::Declaration::TSTypeAliasDeclaration(_) => {}
                ast::Declaration::TSInterfaceDeclaration(interface) => {
                    let ty = idx_to_types.remove(&idx).unwrap();
                    crate::interfaces::synthesize_interface(
                        interface,
                        ty,
                        environment,
                        checking_data,
                    )
                }
                ast::Declaration::TSEnumDeclaration(_) => {}
                ast::Declaration::TSModuleDeclaration(_) => {}
                ast::Declaration::TSImportEqualsDeclaration(_) => {}
            },
            _ => {}
        }
    }

    // Third stage
    for statement in statements {
        if let Statement::Declaration(declaration) = statement {
            match declaration {
                ast::Declaration::FunctionDeclaration(func) => environment.new_function(
                    checking_data,
                    &OxcFunction(&func, None),
                    RegisterOnExisting(func.id.as_ref().unwrap().name.as_str().to_owned()),
                ),
                _ => {}
            }
        }
    }
}

/// TODO different modes for parameters
///
/// Returns the type for reasons
pub(crate) fn register_variable<T: FSResolver>(
    pattern: &ast::BindingPatternKind,
    span: &oxc_span::Span,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
    behaviour: ezno_checker::context::VariableRegisterBehavior,
) -> TypeId {
    match &pattern {
        ast::BindingPatternKind::BindingIdentifier(ident) => environment
            .register_variable(
                ident.name.as_str(),
                VariableId(oxc_span_to_source_map_span(span.clone())),
                behaviour,
                &mut checking_data.types,
            )
            .unwrap(),
        ast::BindingPatternKind::ObjectPattern(item) => {
            checking_data.raise_unimplemented_error(
                "yield expression",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::BindingPatternKind::ArrayPattern(item) => {
            checking_data.raise_unimplemented_error(
                "yield expression",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
        ast::BindingPatternKind::AssignmentPattern(item) => {
            checking_data.raise_unimplemented_error(
                "yield expression",
                oxc_span_to_source_map_span(item.span),
            );
            TypeId::ERROR_TYPE
        }
    }
}

pub(crate) fn synthesize_statement<T: FSResolver>(
    statement: &ast::Statement,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) {
    match statement {
        ast::Statement::BlockStatement(block) => {
            synthesize_statements(&block.body, environment, checking_data);
        }
        ast::Statement::BreakStatement(item) => {
            checking_data.raise_unimplemented_error("break statement", oxc_span_to_source_map_span(item.span))
        }
        ast::Statement::ContinueStatement(item) => {
            checking_data.raise_unimplemented_error("continue statement", oxc_span_to_source_map_span(item.span))
        }
        ast::Statement::EmptyStatement(_) => {}
        ast::Statement::DebuggerStatement(_) => {}
        ast::Statement::DoWhileStatement(item) => {
            checking_data.raise_unimplemented_error("do while statement", oxc_span_to_source_map_span(item.span))
        }
        ast::Statement::ExpressionStatement(expr) => {
            expressions::synthesize_expression(&expr.expression, environment, checking_data);
        }
        ast::Statement::ForInStatement(_)
        | ast::Statement::ForOfStatement(_)
        | ast::Statement::ForStatement(_) => {
            checking_data.raise_unimplemented_error(
                "for statements",
                oxc_span_to_source_map_span(statement.span()),
            );
        }
        ast::Statement::IfStatement(if_stmt) => {
            synthesize_if_statement(if_stmt, environment, checking_data)
        }
        ast::Statement::LabeledStatement(item) => {
            checking_data.raise_unimplemented_error("labeled statement", oxc_span_to_source_map_span(item.span))
        }
        ast::Statement::ReturnStatement(ret_stmt) => {
            if let Some(ref value) = ret_stmt.argument {
                let returned =
                    expressions::synthesize_expression(value, environment, checking_data);
                environment.return_value(returned)
            } else {
                environment.return_value(TypeId::UNDEFINED_TYPE)
            }
        }
        ast::Statement::SwitchStatement(_) => {
            checking_data.raise_unimplemented_error(
                "switch statement",
                oxc_span_to_source_map_span(statement.span()),
            );
        }
        ast::Statement::ThrowStatement(throw_stmt) => {
            let thrown = expressions::synthesize_expression(
                &throw_stmt.argument,
                environment,
                checking_data,
            );
            environment.throw_value(thrown)
        }
        ast::Statement::TryStatement(stmt) => {
            synthesize_try_statement(stmt, environment, checking_data)
        }
        ast::Statement::WhileStatement(_) => {
            checking_data.raise_unimplemented_error(
                "while statement",
                oxc_span_to_source_map_span(statement.span()),
            );
        }
        ast::Statement::WithStatement(item) => {
            checking_data.raise_unimplemented_error("with statement", oxc_span_to_source_map_span(item.span))
        }
        ast::Statement::ModuleDeclaration(item) => {
            checking_data.raise_unimplemented_error("module declaration", oxc_span_to_source_map_span(item.span()))
        }
        ast::Statement::Declaration(declaration) => {
            if !matches!(declaration, ast::Declaration::FunctionDeclaration(..)) {
                synthesize_declaration(declaration, environment, checking_data)
            }
        }
    }
}

// TODO full type narrowing behavior
fn synthesize_if_statement<T: FSResolver>(
    if_stmt: &ast::IfStatement,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) {
    let condition = synthesize_expression(&if_stmt.test, environment, checking_data);

    if let ezno_checker::TruthyFalsy::Decidable(value) =
        environment.is_type_truthy_falsy(condition, &checking_data.types)
    {
        checking_data
            .raise_decidable_result_error(oxc_span_to_source_map_span(if_stmt.span), value);

        if value {
            synthesize_statement(&if_stmt.consequent, environment, checking_data);
            return;
        }
    } else {
        synthesize_statement(&if_stmt.consequent, environment, checking_data);
    }

    if let Some(ref alternative) = if_stmt.alternate {
        synthesize_statement(alternative, environment, checking_data)
    }
}

pub(crate) fn synthesize_statements<T: ezno_checker::FSResolver>(
    statements: &[oxc_ast::ast::Statement],
    environment: &mut Environment,
    checking_data: &mut ezno_checker::CheckingData<T>,
) {
    // TODO union this into one function
    hoist_statements(statements, environment, checking_data);

    for statement in statements.iter() {
        synthesize_statement(statement, environment, checking_data);
    }
}

// TODO some of this logic should be moved to the checker crate
fn synthesize_try_statement<T: FSResolver>(
    stmt: &ast::TryStatement,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) {
    let throw_type: TypeId =
        environment.new_try_context(checking_data, |environment, checking_data| {
            synthesize_statements(&stmt.block.body, environment, checking_data);
        });

    if let Some(ref handler) = stmt.handler {
        // TODO catch when never
        environment.new_lexical_environment_fold_into_parent(
            ezno_checker::Scope::Block {},
            checking_data,
            |environment, checking_data| {
                if let Some(ref clause) = handler.param {
                    // TODO clause.type_annotation
                    register_variable(
                        &clause.kind,
                        // TODO clause has no span
                        &handler.span,
                        environment,
                        checking_data,
                        ezno_checker::context::VariableRegisterBehavior::CatchVariable {
                            ty: throw_type,
                        },
                    );
                }
                synthesize_statements(&handler.body.body, environment, checking_data);
            },
        );
    }
    // TODO finally
}

pub(crate) fn synthesize_declaration<T: FSResolver>(
    declaration: &ast::Declaration,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) {
    match declaration {
        ast::Declaration::VariableDeclaration(variable_declaration) => {
            if variable_declaration.modifiers.contains(ast::ModifierKind::Declare) {
                return;
            }

            for declaration in variable_declaration.declarations.iter() {
                // TODO get from existing!!!!
                let var_ty_and_pos = declaration.id.type_annotation.as_ref().map(|ta| {
                    (
                        synthesize_type_annotation(&ta.type_annotation, environment, checking_data),
                        ta.span,
                    )
                });

                // TODO temp
                let value = declaration.init.as_ref().unwrap();
                let value_ty =
                    expressions::synthesize_expression(value, environment, checking_data);

                if let Some((var_ty, ta_pos)) = var_ty_and_pos {
                    // TODO temp
                    let ta_span = oxc_span_to_source_map_span(ta_pos);
                    let value_span = oxc_span_to_source_map_span(oxc_span::GetSpan::span(value));

                    ezno_checker::check_variable_initialization(
                        (var_ty, Cow::Owned(ta_span)),
                        (value_ty, Cow::Owned(value_span)),
                        environment,
                        checking_data,
                    );
                }

                let id = VariableId(oxc_span_to_source_map_span(declaration.span));
                environment.register_initial_variable_declaration_value(id, value_ty)
            }
        }
        ast::Declaration::FunctionDeclaration(_) => unreachable!("should be hoisted..."),
        ast::Declaration::ClassDeclaration(item) => {
            checking_data.raise_unimplemented_error("class declaration", oxc_span_to_source_map_span(item.span))
        }
        ast::Declaration::TSTypeAliasDeclaration(_) => {}
        ast::Declaration::TSInterfaceDeclaration(_) => {}
        ast::Declaration::TSEnumDeclaration(item) => {
            checking_data.raise_unimplemented_error("enum declaration", oxc_span_to_source_map_span(item.span))
        }
        ast::Declaration::TSModuleDeclaration(item) => {
            checking_data.raise_unimplemented_error("module declaration", oxc_span_to_source_map_span(item.span))
        }
        ast::Declaration::TSImportEqualsDeclaration(item) => {
            checking_data.raise_unimplemented_error("import equals declaration", oxc_span_to_source_map_span(item.span))
        }
    }
}
