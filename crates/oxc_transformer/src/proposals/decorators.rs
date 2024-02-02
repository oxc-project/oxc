use std::{collections::HashMap, rc::Rc};

use bitflags::bitflags;
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

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct DecoratorFlags: u8 {
        const Field = 0;
        const Accessor = 1;
        const Method = 2;
        const Getter = 3;
        const Setter = 4;
        const Static = 8;
        const DecoratorsHaveThis = 16;
    }
}

impl DecoratorFlags {
    pub fn get_flag_by_kind(kind: MethodDefinitionKind) -> Self {
        match kind {
            MethodDefinitionKind::Method => Self::Method,
            MethodDefinitionKind::Get => Self::Getter,
            MethodDefinitionKind::Set => Self::Setter,
            MethodDefinitionKind::Constructor => unreachable!(),
        }
    }
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

    pub fn get_call_with_this(&self, name: Atom) -> Statement<'a> {
        self.ast.expression_statement(
            SPAN,
            self.ast.call_expression(
                SPAN,
                self.ast.identifier_reference_expression(IdentifierReference::new(SPAN, name)),
                self.ast.new_vec_single(Argument::Expression(self.ast.this_expression(SPAN))),
                false,
                None,
            ),
        )
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

        let mut init_static_name = None;

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
            // https://github.com/babel/babel/blob/eccbd203383487f6957dcf086aa83d773691560b/packages/babel-helpers/src/helpers/applyDecs2305.ts#L7-L45
            let get_decorator_info = |key: &PropertyKey<'a>,
                                      flag: u8,
                                      decorator: &Decorator<'a>,
                                      ast: &AstBuilder<'a>| {
                let name = key.name();
                // [dec, flag, name, defaultValue | (o) => o.#a, (o, v) => o.#a = v]
                let mut decorator_elements = ast.new_vec_with_capacity(2);
                decorator_elements
                    .push(ArrayExpressionElement::Expression(ast.copy(&decorator.expression)));
                decorator_elements.push(ArrayExpressionElement::Expression(
                    ast.literal_number_expression(NumberLiteral::new(
                        SPAN,
                        0f64,
                        ast.new_str(flag.to_string().as_str()),
                        oxc_syntax::NumberBase::Decimal,
                    )),
                ));
                if let Some(name) = name {
                    decorator_elements.push(ArrayExpressionElement::Expression(
                        ast.literal_string_expression(StringLiteral::new(SPAN, name.clone())),
                    ));

                    if key.is_private_identifier() {
                        // (o) => o.#a
                        let mut items = ast.new_vec_single(ast.formal_parameter(
                            SPAN,
                            ast.binding_pattern(
                                ast.binding_pattern_identifier(BindingIdentifier::new(
                                    SPAN,
                                    "o".into(),
                                )),
                                None,
                                false,
                            ),
                            None,
                            false,
                            ast.new_vec(),
                        ));
                        let private_field = ast.private_field(
                            SPAN,
                            ast.identifier_reference_expression(IdentifierReference::new(
                                SPAN,
                                "o".into(),
                            )),
                            PrivateIdentifier::new(SPAN, name),
                            false,
                        );
                        let params = ast.formal_parameters(
                            SPAN,
                            FormalParameterKind::ArrowFormalParameters,
                            ast.copy(&items),
                            None,
                        );
                        decorator_elements.push(ArrayExpressionElement::Expression(
                            ast.arrow_expression(
                                SPAN,
                                true,
                                false,
                                params,
                                ast.function_body(
                                    SPAN,
                                    ast.new_vec(),
                                    ast.new_vec_single(ast.expression_statement(
                                        SPAN,
                                        ast.member_expression(ast.copy(&private_field)),
                                    )),
                                ),
                                None,
                                None,
                            ),
                        ));

                        {
                            // (o, v) => o.#a = v
                            items.push(ast.formal_parameter(
                                SPAN,
                                ast.binding_pattern(
                                    ast.binding_pattern_identifier(BindingIdentifier::new(
                                        SPAN,
                                        "v".into(),
                                    )),
                                    None,
                                    false,
                                ),
                                None,
                                false,
                                ast.new_vec(),
                            ));

                            let params = ast.formal_parameters(
                                SPAN,
                                FormalParameterKind::ArrowFormalParameters,
                                items,
                                None,
                            );

                            decorator_elements.push(ArrayExpressionElement::Expression(
                                ast.arrow_expression(
                                    SPAN,
                                    true,
                                    false,
                                    params,
                                    ast.function_body(
                                        SPAN,
                                        ast.new_vec(),
                                        ast.new_vec_single(ast.expression_statement(
                                            SPAN,
                                            ast.assignment_expression(
                                                SPAN,
                                                AssignmentOperator::Assign,
                                                ast.simple_assignment_target_member_expression(
                                                    private_field,
                                                ),
                                                ast.identifier_reference_expression(
                                                    IdentifierReference::new(SPAN, "v".into()),
                                                ),
                                            ),
                                        )),
                                    ),
                                    None,
                                    None,
                                ),
                            ));
                        }
                    }
                }
                ast.array_expression(SPAN, decorator_elements, None)
            };

            let mut is_proto = false;
            let mut is_static = false;

            class.body.body.iter_mut().for_each(|element| {
                if !element.has_decorator() {
                    return;
                }
                match element {
                    ClassElement::MethodDefinition(def) => {
                        if def.r#static {
                            is_static = def.r#static;
                        } else {
                            is_proto = true;
                        }
                        let mut flag = DecoratorFlags::get_flag_by_kind(def.kind).bits();
                        if def.r#static {
                            flag += DecoratorFlags::Static.bits();
                        }

                        def.decorators.iter().for_each(|decorator| {
                            member_decorators_vec.push(ArrayExpressionElement::Expression(
                                get_decorator_info(&def.key, flag, decorator, &self.ast),
                            ));
                        });
                        def.decorators.clear();
                    }
                    ClassElement::PropertyDefinition(def) => {
                        let flag = if def.r#static {
                            DecoratorFlags::Static
                        } else {
                            DecoratorFlags::Field
                        };

                        let init_name = self.get_unique_name(&if def.computed {
                            "init_computedKey".into()
                        } else {
                            format!("init_{}", def.key.name().unwrap()).into()
                        });

                        e_elements
                            .push(self.get_assignment_target_maybe_default(init_name.clone()));

                        def.decorators.iter().for_each(|decorator| {
                            member_decorators_vec.push(ArrayExpressionElement::Expression(
                                get_decorator_info(&def.key, flag.bits(), decorator, &self.ast),
                            ));
                        });
                        def.decorators.clear();

                        let mut arguments = self
                            .ast
                            .new_vec_single(Argument::Expression(self.ast.this_expression(SPAN)));

                        if let Some(value) = &mut def.value {
                            arguments.push(Argument::Expression(self.ast.move_expression(value)));
                        }

                        def.value = Some(self.ast.call_expression(
                            SPAN,
                            self.ast.identifier_reference_expression(IdentifierReference::new(
                                SPAN,
                                init_name.clone(),
                            )),
                            arguments,
                            false,
                            None,
                        ));

                        declarations.push(self.get_variable_declarator(init_name));
                    }
                    _ => {}
                }
            });

            if is_proto {
                // The class has method decorator and is not static
                let name = self.get_unique_name(&"initProto".into());
                e_elements.push(self.get_assignment_target_maybe_default(name.clone()));
                declarations.push(self.get_variable_declarator(name.clone()));

                // constructor() { _initProto(this) }
                if let Some(constructor_element) = class.body.body.iter_mut().find(
                    |element| matches!(element, ClassElement::MethodDefinition(def) if def.kind.is_constructor()),
                ) {
                    if let ClassElement::MethodDefinition(def) = constructor_element {
                        if let Some(body) = &mut def.value.body {
                            body.statements.insert(0, self.get_call_with_this(name));
                        }
                    } else {
                        unreachable!();
                    };
                } else {
                    // if the class has no constructor, insert a empty constructor and call initProto
                    class.body.body.insert(
                        0,
                        self.ast.class_constructor(
                            SPAN,
                            self.ast.function(
                                FunctionType::FunctionExpression,
                                SPAN,
                                None,
                                false,
                                false,
                                None,
                                self.ast.formal_parameters(
                                    SPAN,
                                    FormalParameterKind::FormalParameter,
                                    self.ast.new_vec(),
                                    None,
                                ),
                                Some(self.ast.function_body(
                                    SPAN,
                                    self.ast.new_vec(),
                                    self.ast.new_vec_single(self.get_call_with_this(name)),
                                )),
                                None,
                                None,
                                Modifiers::empty(),
                            ),
                        ),
                    );
                }
            }

            if is_static {
                let name = self.get_unique_name(&"initStatic".into());
                e_elements.push(self.get_assignment_target_maybe_default(name.clone()));
                declarations.push(self.get_variable_declarator(name.clone()));
                init_static_name = Some(name);
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
            // applyDecs2305(
            //     targetClass: any,
            //     memberDecs: DecoratorInfo[],
            //     classDecs: Function[],
            //     classDecsHaveThis: number,
            //     instanceBrand: Function,
            //     parentClass: any,
            //   ) {}
            // call babelHelpers.applyDecs2305
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

            if let Some(init_static_name) = init_static_name {
                // call initStatic
                let callee = self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    init_static_name,
                ));
                let arguments =
                    self.ast.new_vec_single(Argument::Expression(self.ast.this_expression(SPAN)));
                statements.push(self.ast.expression_statement(
                    SPAN,
                    self.ast.call_expression(SPAN, callee, arguments, false, None),
                ));
            }

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
