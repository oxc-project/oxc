use std::{collections::HashMap, rc::Rc};

use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, SPAN};
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};
use serde::Deserialize;

use crate::{context::TransformerCtx, options::TransformOptions};

/// Proposal: Decorators
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-proposal-decorators>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-proposal-decorators>
/// * <https://github.com/tc39/proposal-decorators>
pub struct Decorators<'a> {
    ast: Rc<AstBuilder<'a>>,
    _ctx: TransformerCtx<'a>,
    options: DecoratorsOptions,
    // Insert to the top of the program
    top_statements: Vec<'a, Statement<'a>>,
    // Insert to the bottom of the program
    bottom_statements: Vec<'a, Statement<'a>>,
    uid_map: HashMap<Atom, u32>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoratorsOptions {
    version: Version,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Version {
    Legacy,
    #[serde(rename = "2023-05")]
    #[default]
    Year202305,
}
impl Version {
    fn is_legacy(self) -> bool {
        matches!(self, Self::Legacy)
    }
}

impl<'a> Decorators<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        let top_statements = ast.new_vec();
        let bottom_statements = ast.new_vec();
        options.decorators.map(|options| Self {
            ast,
            _ctx: ctx,
            options,
            top_statements,
            bottom_statements,
            uid_map: HashMap::new(),
        })
    }

    pub fn get_variable_declarator(&self, name: &str) -> VariableDeclarator<'a> {
        self.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            BindingPattern::new_with_kind(
                self.ast.binding_pattern_identifier(BindingIdentifier::new(SPAN, name.into())),
            ),
            None,
            false,
        )
    }

    // TODO: use generate_uid of scope to generate unique name
    pub fn get_unique_name(&mut self, name: &Atom) -> Atom {
        let uid = self.uid_map.entry(name.clone()).or_insert(0);
        *uid += 1;
        Atom::from(format!("_{name}{}", if *uid == 1 { String::new() } else { uid.to_string() }))
    }

    pub fn transform_program(&mut self, program: &mut Program<'a>) {
        program.body.splice(0..0, self.top_statements.drain(..));
        program.body.append(&mut self.bottom_statements);
    }

    pub fn transform_statement(&mut self, stmt: &mut Statement<'a>) {
        if let Statement::ModuleDeclaration(decl) = stmt {
            let new_stmt = match &mut **decl {
                ModuleDeclaration::ExportNamedDeclaration(export) => {
                    // remove export
                    export.declaration.as_mut().map_or_else(
                        || None,
                        |declaration| {
                            if let Declaration::ClassDeclaration(class) = declaration {
                                if class.decorators.is_empty() {
                                    return None;
                                }
                                let class_name = class
                                    .id
                                    .clone()
                                    .map(|id| self.get_unique_name(&id.name))
                                    .or_else(|| Some(self.get_unique_name(&"class".into())));

                                self.bottom_statements.push(self.ast.module_declaration(
                                    ModuleDeclaration::ExportNamedDeclaration(
                                        self.ast.export_named_declaration(
                                            SPAN,
                                            None,
                                            self.ast.new_vec_single(ExportSpecifier::new(
                                                SPAN,
                                                ModuleExportName::Identifier(IdentifierName::new(
                                                    SPAN,
                                                    class_name.clone().unwrap(),
                                                )),
                                                ModuleExportName::Identifier(IdentifierName::new(
                                                    SPAN,
                                                    class.id.clone().unwrap().name,
                                                )),
                                            )),
                                            None,
                                            ImportOrExportKind::Value,
                                        ),
                                    ),
                                ));

                                return Some(Statement::Declaration(
                                    self.transform_class(class, class_name),
                                ));
                            }
                            None
                        },
                    )
                }
                ModuleDeclaration::ExportDefaultDeclaration(export) => {
                    if let ExportDefaultDeclarationKind::ClassDeclaration(class) =
                        &mut export.declaration
                    {
                        if class.decorators.is_empty() {
                            return;
                        }
                        let class_name = class
                            .id
                            .clone()
                            .map(|id| self.get_unique_name(&id.name))
                            .or_else(|| Some(self.get_unique_name(&"class".into())));

                        self.bottom_statements.push(self.ast.module_declaration(
                            ModuleDeclaration::ExportNamedDeclaration(
                                self.ast.export_named_declaration(
                                    SPAN,
                                    None,
                                    self.ast.new_vec_single(ExportSpecifier::new(
                                        SPAN,
                                        ModuleExportName::Identifier(IdentifierName::new(
                                            SPAN,
                                            class_name.clone().unwrap(),
                                        )),
                                        ModuleExportName::Identifier(IdentifierName::new(
                                            SPAN,
                                            Atom::from("default"),
                                        )),
                                    )),
                                    None,
                                    ImportOrExportKind::Value,
                                ),
                            ),
                        ));

                        Some(Statement::Declaration(self.transform_class(class, class_name)))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(new_stmt) = new_stmt {
                *stmt = new_stmt;
            }
        }
    }
    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>) {
        let new_decl = match decl {
            Declaration::ClassDeclaration(class) => {
                if class.decorators.is_empty() {
                    None
                } else {
                    Some(self.transform_class(class, None))
                }
            }
            _ => None,
        };
        if let Some(new_decl) = new_decl {
            *decl = new_decl;
        }
    }

    /// transform version: 2023-05
    pub fn transform_class(
        &mut self,
        class: &mut Box<'a, Class<'a>>,
        class_name: Option<Atom>,
    ) -> Declaration<'a> {
        if self.options.version.is_legacy() {
            return self.transform_class_legacy(class, class_name);
        }
        let class_name = class_name.unwrap_or_else(|| self.get_unique_name(&"class".into()));

        let class_decs_name = self.get_unique_name(&"classDecs".into());
        let init_class_name = self.get_unique_name(&"initClass".into());

        {
            // insert var _initClass, _classDecs;
            let mut declarations = self.ast.new_vec_with_capacity(2);
            declarations.push(self.get_variable_declarator(&init_class_name));
            declarations.push(self.get_variable_declarator(&class_decs_name));
            let variable_declaration = self.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                declarations,
                Modifiers::empty(),
            );

            self.top_statements.push(Statement::Declaration(Declaration::VariableDeclaration(
                variable_declaration,
            )));
        }

        {
            // insert _classDecs = decorators;
            let left = self.ast.simple_assignment_target_identifier(IdentifierReference::new(
                SPAN,
                class_decs_name.clone(),
            ));

            let right =
                self.ast.array_expression(
                    SPAN,
                    {
                        let mut elements = self.ast.new_vec();
                        elements.extend(class.decorators.drain(..).map(|d| {
                            ArrayExpressionElement::Expression(self.ast.copy(&d.expression))
                        }));
                        elements
                    },
                    None,
                );
            let assign_class_decs = self.ast.expression_statement(
                SPAN,
                self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right),
            );
            self.top_statements.push(assign_class_decs);
        };

        {
            // insert let _className

            let declarations = self.ast.new_vec_single(self.get_variable_declarator(&class_name));
            let variable_declaration = self.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Let,
                declarations,
                Modifiers::empty(),
            );
            self.top_statements.push(Statement::Declaration(Declaration::VariableDeclaration(
                variable_declaration,
            )));
        }

        {
            // call  babelHelpers.applyDecs2305
            let callee = self.ast.static_member_expression(
                SPAN,
                self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    "babelHelpers".into(),
                )),
                IdentifierName::new(SPAN, "applyDecs2305".into()),
                false,
            );
            let mut arguments = self.ast.new_vec();
            arguments.push(Argument::Expression(self.ast.this_expression(SPAN)));
            let decs = self.ast.new_vec();
            arguments.push(Argument::Expression(self.ast.array_expression(SPAN, decs, None)));
            arguments.push(Argument::Expression(
                self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    class_decs_name,
                )),
            ));
            let object = self.ast.call_expression(SPAN, callee, arguments, false, None);
            let call_expr = self.ast.static_member_expression(
                SPAN,
                object,
                IdentifierName::new(SPAN, "c".into()),
                false,
            );

            let mut elements = self.ast.new_vec();
            elements.push(Some(AssignmentTargetMaybeDefault::AssignmentTarget(
                self.ast.simple_assignment_target_identifier(IdentifierReference::new(
                    SPAN, class_name,
                )),
            )));
            elements.push(Some(AssignmentTargetMaybeDefault::AssignmentTarget(
                self.ast.simple_assignment_target_identifier(IdentifierReference::new(
                    SPAN,
                    init_class_name.clone(),
                )),
            )));
            let left = self
                .ast
                .array_assignment_target(ArrayAssignmentTarget::new_with_elements(SPAN, elements));
            let new_expr =
                self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, call_expr);

            let mut statements = self.ast.new_vec();
            statements.push(self.ast.expression_statement(SPAN, new_expr));
            let static_block = self.ast.static_block(SPAN, statements);
            class.body.body.insert(0, static_block);
        }

        {
            // call _initClass
            let callee = self
                .ast
                .identifier_reference_expression(IdentifierReference::new(SPAN, init_class_name));
            let call_expr = self.ast.call_expression(SPAN, callee, self.ast.new_vec(), false, None);
            let statements =
                self.ast.new_vec_single(self.ast.expression_statement(SPAN, call_expr));
            let static_block = self.ast.static_block(SPAN, statements);
            class.body.body.insert(1, static_block);
        }

        Declaration::ClassDeclaration(self.ast.copy(class))
    }

    /// transform version: legacy
    pub fn transform_class_legacy(
        &mut self,
        class: &mut Box<'a, Class<'a>>,
        class_name: Option<Atom>,
    ) -> Declaration<'a> {
        let class_binding_identifier = &class.id.clone().unwrap_or_else(|| {
            BindingIdentifier::new(
                SPAN,
                class_name.unwrap_or_else(|| self.get_unique_name(&"class".into())),
            )
        });
        let class_name = BindingPattern::new_with_kind(
            self.ast.binding_pattern_identifier(self.ast.copy(class_binding_identifier)),
        );

        let init = {
            let class_identifier_name: Atom = self.get_unique_name(&"class".into());
            let class_identifier = IdentifierReference::new(SPAN, class_identifier_name.clone());

            let decl = self.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                self.ast.new_vec_single(self.get_variable_declarator(&class_identifier_name)),
                Modifiers::empty(),
            );
            self.top_statements
                .push(Statement::Declaration(Declaration::VariableDeclaration(decl)));

            let left = self.ast.simple_assignment_target_identifier(class_identifier.clone());
            let right = self.ast.class_expression(self.ast.copy(class));
            let new_expr =
                self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right);

            let new_expr = class.decorators.drain(..).fold(new_expr, |new_expr, decorator| {
                match &decorator.expression {
                    Expression::Identifier(identifier) => self.ast.call_expression(
                        SPAN,
                        self.ast.identifier_reference_expression(IdentifierReference::new(
                            SPAN,
                            identifier.name.clone(),
                        )),
                        self.ast.new_vec_single(Argument::Expression(self.ast.copy(&new_expr))),
                        false,
                        None,
                    ),
                    _ => new_expr,
                }
            });

            self.ast.logical_expression(
                SPAN,
                new_expr,
                LogicalOperator::Or,
                self.ast.identifier_reference_expression(class_identifier),
            )
        };

        let declarator = self.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Let,
            class_name,
            Some(init),
            false,
        );

        Declaration::VariableDeclaration(self.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Let,
            self.ast.new_vec_single(declarator),
            Modifiers::empty(),
        ))
    }
}
