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

    pub fn get_variable_declarator(&self, name: Atom) -> VariableDeclarator<'a> {
        self.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            BindingPattern::new_with_kind(
                self.ast.binding_pattern_identifier(BindingIdentifier::new(SPAN, name)),
            ),
            None,
            false,
        )
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn get_assignment_target_maybe_default(
        &self,
        name: Atom,
    ) -> Option<AssignmentTargetMaybeDefault<'a>> {
        Some(AssignmentTargetMaybeDefault::AssignmentTarget(
            self.ast.simple_assignment_target_identifier(IdentifierReference::new(SPAN, name)),
        ))
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
                                if !Self::can_transform(class) {
                                    return None;
                                }
                                let has_decorator = !class.decorators.is_empty();
                                let class_name = if has_decorator {
                                    class
                                        .id
                                        .clone()
                                        .map(|id| self.get_unique_name(&id.name))
                                        .or_else(|| Some(self.get_unique_name(&"class".into())))
                                } else {
                                    None
                                };

                                if has_decorator {
                                    self.bottom_statements.push(self.ast.module_declaration(
                                        ModuleDeclaration::ExportNamedDeclaration(
                                            self.ast.export_named_declaration(
                                                SPAN,
                                                None,
                                                self.ast.new_vec_single(ExportSpecifier::new(
                                                    SPAN,
                                                    ModuleExportName::Identifier(
                                                        IdentifierName::new(
                                                            SPAN,
                                                            class_name.clone().unwrap(),
                                                        ),
                                                    ),
                                                    ModuleExportName::Identifier(
                                                        IdentifierName::new(
                                                            SPAN,
                                                            class.id.clone().unwrap().name,
                                                        ),
                                                    ),
                                                )),
                                                None,
                                                ImportOrExportKind::Value,
                                            ),
                                        ),
                                    ));
                                }

                                let new_declaration = self.transform_class(class, class_name);
                                if has_decorator {
                                    return Some(Statement::Declaration(new_declaration));
                                }
                                *declaration = new_declaration;
                                return None;
                            }
                            None
                        },
                    )
                }
                ModuleDeclaration::ExportDefaultDeclaration(export) => {
                    if let ExportDefaultDeclarationKind::ClassDeclaration(class) =
                        &mut export.declaration
                    {
                        if !Self::can_transform(class) {
                            return;
                        }
                        let class_has_decorator = !class.decorators.is_empty();
                        let class_name = if class_has_decorator {
                            class
                                .id
                                .clone()
                                .map(|id| self.get_unique_name(&id.name))
                                .or_else(|| Some(self.get_unique_name(&"class".into())))
                        } else {
                            None
                        };

                        if class_has_decorator {
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
                                                "default".into(),
                                            )),
                                        )),
                                        None,
                                        ImportOrExportKind::Value,
                                    ),
                                ),
                            ));
                        }

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
                if Self::can_transform(class) {
                    Some(self.transform_class(class, None))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(new_decl) = new_decl {
            *decl = new_decl;
        }
    }

    pub fn can_transform(class: &Class<'a>) -> bool {
        !class.decorators.is_empty() || class.body.body.iter().any(ClassElement::has_decorator)
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

        let has_decorator = !class.decorators.is_empty();
        let has_member_decorator = class.body.body.iter().any(ClassElement::has_decorator);

        let mut declarations = self.ast.new_vec();

        let mut c_elements = self.ast.new_vec();
        let mut e_elements = self.ast.new_vec();

        // insert member decorators
        let mut member_decorators_vec = self.ast.new_vec();
        let mut class_decorators_argument =
            Argument::Expression(self.ast.array_expression(SPAN, self.ast.new_vec(), None));

        if has_decorator {
            let class_name = class_name.unwrap_or_else(|| self.get_unique_name(&"class".into()));

            let class_decs_name = self.get_unique_name(&"classDecs".into());
            let init_class_name = self.get_unique_name(&"initClass".into());

            {
                // insert var _initClass, _classDecs;
                declarations.push(self.get_variable_declarator(init_class_name.clone()));
                declarations.push(self.get_variable_declarator(class_decs_name.clone()));
            }

            {
                // insert _classDecs = decorators;
                let left = self.ast.simple_assignment_target_identifier(IdentifierReference::new(
                    SPAN,
                    class_decs_name.clone(),
                ));

                let right = self.ast.array_expression(
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
                let declarations =
                    self.ast.new_vec_single(self.get_variable_declarator(class_name.clone()));
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

            c_elements.push(self.get_assignment_target_maybe_default(class_name));
            c_elements.push(self.get_assignment_target_maybe_default(init_class_name.clone()));

            class_decorators_argument =
                Argument::Expression(self.ast.identifier_reference_expression(
                    IdentifierReference::new(SPAN, class_decs_name),
                ));

            {
                // call _initClass
                let callee = self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    init_class_name,
                ));
                let call_expr =
                    self.ast.call_expression(SPAN, callee, self.ast.new_vec(), false, None);
                let statements =
                    self.ast.new_vec_single(self.ast.expression_statement(SPAN, call_expr));
                let static_block = self.ast.static_block(SPAN, statements);
                class.body.body.insert(0, static_block);
            }
        } else if has_member_decorator {
            let elements: std::vec::Vec<_> = {
                class
                    .body
                    .body
                    .iter_mut()
                    .enumerate()
                    .filter_map(|(index, element)| match element {
                        ClassElement::MethodDefinition(def) => {
                            if def.decorators.is_empty() {
                                None
                            } else {
                                Some((index, def.key.name(), def.computed, &def.decorators))
                            }
                        }
                        ClassElement::PropertyDefinition(def) => {
                            if def.decorators.is_empty() {
                                None
                            } else {
                                Some((index, def.key.name(), def.computed, &def.decorators))
                            }
                        }
                        _ => None,
                    })
                    .collect()
            };

            let mut replace_elements = HashMap::new();

            for (index, name, computed, decorators) in elements {
                let init_name = self.get_unique_name(&if computed {
                    "init_computedKey".into()
                } else {
                    format!("init_{}", name.clone().unwrap()).into()
                });
                decorators.iter().for_each(|decorator| {
                    let dec_name = self.get_unique_name(&"dec".into());
                    declarations.push(self.get_variable_declarator(dec_name.clone()));

                    let left = self.ast.simple_assignment_target_identifier(
                        IdentifierReference::new(SPAN, dec_name.clone()),
                    );
                    let right = self.ast.copy(&decorator.expression);
                    let dec_expr = self.ast.assignment_expression(
                        SPAN,
                        AssignmentOperator::Assign,
                        left,
                        right,
                    );
                    e_elements.push(self.get_assignment_target_maybe_default(init_name.clone()));
                    let mut decorator_elements = self.ast.new_vec_with_capacity(2);
                    decorator_elements.push(ArrayExpressionElement::Expression(
                        self.ast.identifier_reference_expression(IdentifierReference::new(
                            SPAN, dec_name,
                        )),
                    ));
                    decorator_elements.push(ArrayExpressionElement::Expression(
                        self.ast.literal_number_expression(NumberLiteral::new(
                            SPAN,
                            0f64,
                            "0",
                            oxc_syntax::NumberBase::Decimal,
                        )),
                    ));
                    if let Some(name) = name.clone() {
                        decorator_elements.push(ArrayExpressionElement::Expression(
                            self.ast.literal_string_expression(StringLiteral::new(SPAN, name)),
                        ));
                    }
                    member_decorators_vec.push(ArrayExpressionElement::Expression(
                        self.ast.array_expression(SPAN, decorator_elements, None),
                    ));

                    self.top_statements.push(self.ast.expression_statement(SPAN, dec_expr));
                });

                declarations.push(self.get_variable_declarator(init_name.clone()));

                if let Some(name) = name.clone() {
                    replace_elements.insert(
                        index,
                        self.ast.class_property(
                            SPAN,
                            self.ast
                                .property_key_identifier(IdentifierName::new(SPAN, name.clone())),
                            Some(self.ast.call_expression(
                                SPAN,
                                self.ast.identifier_reference_expression(IdentifierReference::new(
                                    SPAN, init_name,
                                )),
                                self.ast.new_vec_single(Argument::Expression(
                                    self.ast.this_expression(SPAN),
                                )),
                                false,
                                None,
                            )),
                            false,
                            false,
                            self.ast.new_vec(),
                        ),
                    );
                }
            }

            // replace the element with `name = init_name(this)`
            for (index, element) in class.body.body.iter_mut().enumerate() {
                if let Some(new_element) = replace_elements.remove(&index) {
                    *element = new_element;
                }
            }
        }

        {
            // insert all variable_declarator in same variable_declaration
            let variable_declaration = self.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                declarations,
                Modifiers::empty(),
            );

            self.top_statements.insert(
                0,
                Statement::Declaration(Declaration::VariableDeclaration(variable_declaration)),
            );
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

            let mut arguments =
                self.ast.new_vec_single(Argument::Expression(self.ast.this_expression(SPAN)));
            arguments.push(Argument::Expression(self.ast.array_expression(
                SPAN,
                member_decorators_vec,
                None,
            )));
            arguments.push(class_decorators_argument);

            let mut call_expr = self.ast.call_expression(SPAN, callee, arguments, false, None);

            if has_decorator && has_decorator == has_member_decorator {
                // TODO: support this case
            } else if has_decorator || has_member_decorator {
                call_expr = self.ast.static_member_expression(
                    SPAN,
                    call_expr,
                    IdentifierName::new(SPAN, if has_decorator { "c".into() } else { "e".into() }),
                    false,
                );
            }

            let left = self.ast.array_assignment_target(ArrayAssignmentTarget::new_with_elements(
                SPAN,
                if c_elements.is_empty() { e_elements } else { c_elements },
            ));
            let new_expr =
                self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, call_expr);

            let mut statements = self.ast.new_vec();
            statements.push(self.ast.expression_statement(SPAN, new_expr));
            let static_block = self.ast.static_block(SPAN, statements);
            class.body.body.insert(0, static_block);
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
                self.ast.new_vec_single(self.get_variable_declarator(class_identifier_name)),
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
