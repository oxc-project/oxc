use std::{borrow::Cow, collections::HashMap};

use bitflags::bitflags;
use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::{Atom, CompactStr, SPAN};
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};
use serde::Deserialize;

use crate::context::TransformerCtx;

/// Proposal: Decorators
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-proposal-decorators>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-proposal-decorators>
/// * <https://github.com/tc39/proposal-decorators>
pub struct Decorators<'a> {
    ctx: TransformerCtx<'a>,
    options: DecoratorsOptions,
    // Insert to the top of the program
    top_statements: Vec<'a, Statement<'a>>,
    // Insert to the bottom of the program
    bottom_statements: Vec<'a, Statement<'a>>,
    uid_map: HashMap<CompactStr, u32>,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct DecoratorFlags: u8 {
        // flag is 0
        const Field = 1 << 0;
        // flag is 1
        const Accessor = 1 << 1;
        // flag is 2
        const Method = 1 << 2;
        // flag is 3
        const Getter = 1 << 3;
        // flag is 4
        const Setter = 1 << 4;
        // flag is 8
        const Static = 1 << 5;
        // flag is 16
        const DecoratorsHaveThis = 1 << 6;
    }
}

impl DecoratorFlags {
    pub fn is_static(self) -> bool {
        self.contains(Self::Static)
    }
    pub fn get_flag_by_kind(kind: MethodDefinitionKind) -> Self {
        match kind {
            MethodDefinitionKind::Method => Self::Method,
            MethodDefinitionKind::Get => Self::Getter,
            MethodDefinitionKind::Set => Self::Setter,
            MethodDefinitionKind::Constructor => unreachable!(),
        }
    }
    pub fn to_value(self) -> u8 {
        if self.contains(DecoratorFlags::DecoratorsHaveThis) {
            return 16;
        }
        let mut value: u8 = 0;
        if self.contains(DecoratorFlags::Accessor) {
            value += 1;
        }
        if self.contains(DecoratorFlags::Method) {
            value += 2;
        }
        if self.contains(DecoratorFlags::Getter) {
            value += 3;
        }
        if self.contains(DecoratorFlags::Setter) {
            value += 4;
        }
        if self.contains(DecoratorFlags::Static) {
            value += 8;
        }
        value
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
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        let top_statements = ctx.ast.new_vec();
        let bottom_statements = ctx.ast.new_vec();
        ctx.options.decorators.map(|options| Self {
            ctx,
            options,
            top_statements,
            bottom_statements,
            uid_map: HashMap::new(),
        })
    }

    pub fn get_variable_declarator(&self, name: &str) -> VariableDeclarator<'a> {
        let name = self.ctx.ast.new_atom(name);
        self.ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            BindingPattern::new_with_kind(
                self.ctx.ast.binding_pattern_identifier(BindingIdentifier::new(SPAN, name)),
            ),
            None,
            false,
        )
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn get_assignment_target_maybe_default(
        &self,
        name: &str,
    ) -> Option<AssignmentTargetMaybeDefault<'a>> {
        let name = self.ctx.ast.new_atom(name);
        Some(AssignmentTargetMaybeDefault::AssignmentTarget(
            self.ctx.ast.simple_assignment_target_identifier(IdentifierReference::new(SPAN, name)),
        ))
    }

    // TODO: use generate_uid of scope to generate unique name
    pub fn get_unique_name(&mut self, name: &str) -> CompactStr {
        let uid = self.uid_map.entry(CompactStr::new(name)).or_insert(0);
        *uid += 1;
        CompactStr::from(format!(
            "_{name}{}",
            if *uid == 1 { String::new() } else { uid.to_string() }
        ))
    }

    pub fn get_call_with_this(&self, name: &str) -> Expression<'a> {
        self.get_call_with_arguments(
            name,
            self.ctx.ast.new_vec_single(Argument::Expression(self.ctx.ast.this_expression(SPAN))),
        )
    }

    pub fn get_call_with_arguments(
        &self,
        name: &str,
        arguments: Vec<'a, Argument<'a>>,
    ) -> Expression<'a> {
        let name = self.ctx.ast.new_atom(name);
        self.ctx.ast.call_expression(
            SPAN,
            self.ctx.ast.identifier_reference_expression(IdentifierReference::new(SPAN, name)),
            arguments,
            false,
            None,
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
                                        .or_else(|| Some(self.get_unique_name("class")))
                                } else {
                                    None
                                };

                                if has_decorator {
                                    self.bottom_statements.push(self.ctx.ast.module_declaration(
                                        ModuleDeclaration::ExportNamedDeclaration(
                                            self.ctx.ast.export_named_declaration(
                                                SPAN,
                                                None,
                                                self.ctx.ast.new_vec_single(ExportSpecifier::new(
                                                    SPAN,
                                                    ModuleExportName::Identifier(
                                                        IdentifierName::new(
                                                            SPAN,
                                                            self.ctx.ast.new_atom(
                                                                class_name.as_ref().unwrap(),
                                                            ),
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
                                                None,
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
                                .or_else(|| Some(self.get_unique_name("class")))
                        } else {
                            None
                        };

                        if class_has_decorator {
                            self.bottom_statements.push(self.ctx.ast.module_declaration(
                                ModuleDeclaration::ExportNamedDeclaration(
                                    self.ctx.ast.export_named_declaration(
                                        SPAN,
                                        None,
                                        self.ctx.ast.new_vec_single(ExportSpecifier::new(
                                            SPAN,
                                            ModuleExportName::Identifier(IdentifierName::new(
                                                SPAN,
                                                self.ctx.ast.new_atom(class_name.as_ref().unwrap()),
                                            )),
                                            ModuleExportName::Identifier(IdentifierName::new(
                                                SPAN,
                                                self.ctx.ast.new_atom("default"),
                                            )),
                                        )),
                                        None,
                                        ImportOrExportKind::Value,
                                        None,
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
        class_name: Option<CompactStr>,
    ) -> Declaration<'a> {
        if self.options.version.is_legacy() {
            return self.transform_class_legacy(class, class_name);
        }

        let has_decorator = !class.decorators.is_empty();
        let has_member_decorator = class.body.body.iter().any(ClassElement::has_decorator);

        let mut declarations = self.ctx.ast.new_vec();

        let mut c_elements = self.ctx.ast.new_vec();
        let mut e_elements = self.ctx.ast.new_vec();

        let mut private_in_expression = None;

        let mut init_static_name = None;

        // insert member decorators
        let mut member_decorators_vec = self.ctx.ast.new_vec();
        let mut class_decorators_argument =
            Argument::Expression(self.ctx.ast.array_expression(SPAN, self.ctx.ast.new_vec(), None));

        if has_decorator {
            let class_name = class_name.unwrap_or_else(|| {
                self.get_unique_name(
                    class.id.as_ref().map_or_else(|| "class".into(), |id| id.name.clone()).as_ref(),
                )
            });

            let init_class_name = self.get_unique_name("initClass");

            {
                // insert var _initClass, _classDecs;
                declarations.push(self.get_variable_declarator(&init_class_name));
            }

            {
                let decorators_exists = class.decorators.iter().any(|decorator| {
                    decorator
                        .name()
                        .is_some_and(|name| // TODO: We should use current node's scope id
                    self.ctx.scopes().has_binding(self.ctx.scopes().root_scope_id(), name))
                });

                if decorators_exists {
                    let mut elements = self.ctx.ast.new_vec();

                    elements.extend(class.decorators.drain(..).map(|decorator| {
                        ArrayExpressionElement::Expression(self.ctx.ast.copy(&decorator.expression))
                    }));
                    class_decorators_argument =
                        Argument::Expression(self.ctx.ast.array_expression(SPAN, elements, None));
                } else {
                    let class_decs_name = self.get_unique_name("classDecs");

                    // insert var _classDecs;
                    declarations.push(self.get_variable_declarator(&class_decs_name));

                    // insert _classDecs = decorators;
                    let left = self.ctx.ast.simple_assignment_target_identifier(
                        IdentifierReference::new(SPAN, self.ctx.ast.new_atom(&class_decs_name)),
                    );

                    let right = self.ctx.ast.array_expression(
                        SPAN,
                        {
                            let mut elements = self.ctx.ast.new_vec();
                            elements.extend(class.decorators.drain(..).map(|d| {
                                ArrayExpressionElement::Expression(self.ctx.ast.copy(&d.expression))
                            }));
                            elements
                        },
                        None,
                    );
                    let assign_class_decs = self.ctx.ast.expression_statement(
                        SPAN,
                        self.ctx.ast.assignment_expression(
                            SPAN,
                            AssignmentOperator::Assign,
                            left,
                            right,
                        ),
                    );
                    self.top_statements.push(assign_class_decs);

                    class_decorators_argument =
                        Argument::Expression(self.ctx.ast.identifier_reference_expression(
                            IdentifierReference::new(SPAN, self.ctx.ast.new_atom(&class_decs_name)),
                        ));
                }
            };

            {
                // insert let _className
                let declarations =
                    self.ctx.ast.new_vec_single(self.get_variable_declarator(&class_name));
                let variable_declaration = self.ctx.ast.variable_declaration(
                    SPAN,
                    VariableDeclarationKind::Let,
                    declarations,
                    Modifiers::empty(),
                );
                self.top_statements.push(Statement::Declaration(Declaration::VariableDeclaration(
                    variable_declaration,
                )));
            }

            c_elements.push(self.get_assignment_target_maybe_default(&class_name));
            c_elements.push(self.get_assignment_target_maybe_default(&init_class_name));

            {
                // call _initClass
                let callee = self.ctx.ast.identifier_reference_expression(
                    IdentifierReference::new(SPAN, self.ctx.ast.new_atom(&init_class_name)),
                );
                let call_expr =
                    self.ctx.ast.call_expression(SPAN, callee, self.ctx.ast.new_vec(), false, None);
                let statements =
                    self.ctx.ast.new_vec_single(self.ctx.ast.expression_statement(SPAN, call_expr));
                let static_block = self.ctx.ast.static_block(SPAN, statements);
                class.body.body.insert(0, static_block);
            }
        }

        if has_member_decorator {
            let mut is_proto = false;
            let mut is_static = false;

            let mut name;
            for element in class.body.body.iter_mut() {
                if !element.has_decorator() {
                    continue;
                }
                match element {
                    ClassElement::MethodDefinition(def) => {
                        if def.r#static {
                            is_static = def.r#static;
                        } else {
                            is_proto = true;
                        }
                        let mut flag = DecoratorFlags::get_flag_by_kind(def.kind);
                        if def.r#static {
                            flag |= DecoratorFlags::Static;
                        }

                        for decorator in &def.decorators {
                            member_decorators_vec.push(ArrayExpressionElement::Expression(
                                self.get_decorator_info(
                                    &def.key,
                                    Some(self.ctx.ast.copy(&def.value)),
                                    flag,
                                    decorator,
                                ),
                            ));
                        }

                        def.decorators.clear();

                        if def.key.is_private_identifier() {
                            {
                                if !flag.is_static() {
                                    // _ => #a in _;
                                    private_in_expression =
                                        Some(self.get_is_private_function(
                                            def.key.private_name().unwrap(),
                                        ));
                                }
                            }

                            name = self.get_unique_name(&if def.computed {
                                Cow::Borrowed("init_computedKey")
                            } else {
                                Cow::Owned(format!("call_{}", def.key.name().unwrap()))
                            });

                            let mut arguments = self.ctx.ast.new_vec_with_capacity(2);
                            arguments
                                .push(Argument::Expression(self.ctx.ast.this_expression(SPAN)));
                            arguments.push(Argument::Expression(
                                self.ctx.ast.identifier_reference_expression(
                                    IdentifierReference::new(SPAN, self.ctx.ast.new_atom("v")),
                                ),
                            ));

                            let is_setter = def.kind == MethodDefinitionKind::Set;

                            def.value = self.ctx.ast.function(
                                def.value.r#type,
                                def.value.span,
                                self.ctx.ast.copy(&def.value.id),
                                def.value.generator,
                                def.value.r#async,
                                self.ctx.ast.copy(&def.value.this_param),
                                self.ctx.ast.formal_parameters(
                                    SPAN,
                                    FormalParameterKind::FormalParameter,
                                    if is_setter {
                                        self.ctx.ast.new_vec_single(self.ctx.ast.formal_parameter(
                                            SPAN,
                                            self.ctx.ast.binding_pattern(
                                                self.ctx.ast.binding_pattern_identifier(
                                                    BindingIdentifier::new(
                                                        SPAN,
                                                        self.ctx.ast.new_atom("v"),
                                                    ),
                                                ),
                                                None,
                                                false,
                                            ),
                                            None,
                                            false,
                                            false,
                                            self.ctx.ast.new_vec(),
                                        ))
                                    } else {
                                        self.ctx.ast.new_vec()
                                    },
                                    None,
                                ),
                                Some(self.ctx.ast.function_body(
                                    SPAN,
                                    self.ctx.ast.new_vec(),
                                    self.ctx.ast.new_vec_single(if is_setter {
                                        self.ctx.ast.expression_statement(
                                            SPAN,
                                            self.get_call_with_arguments(&name, arguments),
                                        )
                                    } else {
                                        self.ctx.ast.return_statement(
                                            SPAN,
                                            Some(self.get_call_with_this(&name)),
                                        )
                                    }),
                                )),
                                self.ctx.ast.copy(&def.value.type_parameters),
                                self.ctx.ast.copy(&def.value.return_type),
                                self.ctx.ast.copy(&def.value.modifiers),
                            );
                        } else {
                            continue;
                        }
                    }
                    ClassElement::PropertyDefinition(def) => {
                        let flag = if def.r#static {
                            DecoratorFlags::Static
                        } else {
                            DecoratorFlags::Field
                        };

                        if def.key.is_private_identifier() && !flag.is_static() {
                            private_in_expression =
                                Some(self.get_is_private_function(def.key.private_name().unwrap()));
                        }

                        name = self.get_unique_name(&if def.computed {
                            Cow::Borrowed("init_computedKey")
                        } else {
                            Cow::Owned(format!("init_{}", def.key.name().unwrap()))
                        });

                        for decorator in &def.decorators {
                            member_decorators_vec.push(ArrayExpressionElement::Expression(
                                self.get_decorator_info(&def.key, None, flag, decorator),
                            ));
                        }
                        def.decorators.clear();

                        let mut arguments = self.ctx.ast.new_vec_single(Argument::Expression(
                            self.ctx.ast.this_expression(SPAN),
                        ));

                        if let Some(value) = &mut def.value {
                            arguments
                                .push(Argument::Expression(self.ctx.ast.move_expression(value)));
                        }

                        def.value = Some(self.ctx.ast.call_expression(
                            SPAN,
                            self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                                SPAN,
                                self.ctx.ast.new_atom(&name),
                            )),
                            arguments,
                            false,
                            None,
                        ));
                    }
                    _ => continue,
                }

                e_elements.push(self.get_assignment_target_maybe_default(&name));
                declarations.push(self.get_variable_declarator(&name));
            }

            if is_proto {
                // The class has method decorator and is not static
                let name = self.get_unique_name("initProto");
                e_elements.push(self.get_assignment_target_maybe_default(&name));
                declarations.push(self.get_variable_declarator(&name));

                // constructor() { _initProto(this) }
                if let Some(constructor_element) = class.body.body.iter_mut().find(
                    |element| matches!(element, ClassElement::MethodDefinition(def) if def.kind.is_constructor()),
                ) {
                    if let ClassElement::MethodDefinition(def) = constructor_element {
                        if let Some(body) = &mut def.value.body {
                            body.statements.insert(0, self.ctx.ast.expression_statement(SPAN, self.get_call_with_this(&name)));
                        }
                    } else {
                        unreachable!();
                    };
                } else {
                    // if the class has no constructor, insert a empty constructor and call initProto
                    class.body.body.insert(
                        0,
                        self.ctx.ast.class_constructor(
                            SPAN,
                            self.ctx.ast.function(
                                FunctionType::FunctionExpression,
                                SPAN,
                                None,
                                false,
                                false,
                                None,
                                self.ctx.ast.formal_parameters(
                                    SPAN,
                                    FormalParameterKind::FormalParameter,
                                    self.ctx.ast.new_vec(),
                                    None,
                                ),
                                Some(self.ctx.ast.function_body(
                                    SPAN,
                                    self.ctx.ast.new_vec(),
                                    self.ctx.ast.new_vec_single(
                                        self.ctx.ast.expression_statement(SPAN, self.get_call_with_this(&name))
                                    ),
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
                let name = self.get_unique_name("initStatic");
                e_elements.push(self.get_assignment_target_maybe_default(&name));
                declarations.push(self.get_variable_declarator(&name));
                init_static_name = Some(name);
            }
        }

        {
            // insert all variable_declarator in same variable_declaration
            let variable_declaration = self.ctx.ast.variable_declaration(
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
            let callee = self.ctx.ast.static_member_expression(
                SPAN,
                self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    self.ctx.ast.new_atom("babelHelpers"),
                )),
                IdentifierName::new(SPAN, self.ctx.ast.new_atom("applyDecs2305")),
                false,
            );

            let mut arguments = self
                .ctx
                .ast
                .new_vec_single(Argument::Expression(self.ctx.ast.this_expression(SPAN)));
            arguments.push(Argument::Expression(self.ctx.ast.array_expression(
                SPAN,
                member_decorators_vec,
                None,
            )));
            arguments.push(class_decorators_argument);
            if let Some(private_in_expression) = private_in_expression {
                // classDecsHaveThis
                arguments.push(Argument::Expression(self.ctx.ast.literal_number_expression(
                    // TODO: use correct number instead of `0`
                    self.ctx.ast.number_literal(SPAN, 0f64, "0", oxc_syntax::NumberBase::Decimal),
                )));
                // instanceBrand
                arguments.push(Argument::Expression(private_in_expression));
            }

            let mut call_expr = self.ctx.ast.call_expression(SPAN, callee, arguments, false, None);

            if has_decorator && has_decorator == has_member_decorator {
                let mut properties = self.ctx.ast.new_vec_with_capacity(2);
                properties.push(self.ctx.ast.assignment_target_property_property(
                    SPAN,
                    self.ctx.ast.property_key_identifier(IdentifierName::new(SPAN, "e".into())),
                    self.ctx.ast.array_assignment_target_maybe_default(
                        ArrayAssignmentTarget::new_with_elements(SPAN, e_elements),
                    ),
                ));
                properties.push(self.ctx.ast.assignment_target_property_property(
                    SPAN,
                    self.ctx.ast.property_key_identifier(IdentifierName::new(SPAN, "c".into())),
                    self.ctx.ast.array_assignment_target_maybe_default(
                        ArrayAssignmentTarget::new_with_elements(SPAN, c_elements),
                    ),
                ));
                call_expr = self.ctx.ast.assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    self.ctx.ast.object_assignment_target(
                        ObjectAssignmentTarget::new_with_properties(SPAN, properties),
                    ),
                    call_expr,
                );
            } else if has_decorator || has_member_decorator {
                call_expr = self.ctx.ast.static_member_expression(
                    SPAN,
                    call_expr,
                    IdentifierName::new(
                        SPAN,
                        self.ctx.ast.new_atom(if has_decorator { "c" } else { "e" }),
                    ),
                    false,
                );

                let left =
                    self.ctx.ast.array_assignment_target(ArrayAssignmentTarget::new_with_elements(
                        SPAN,
                        if has_decorator { c_elements } else { e_elements },
                    ));

                call_expr = self.ctx.ast.assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    left,
                    call_expr,
                );
            }

            let mut statements = self.ctx.ast.new_vec();
            statements.push(self.ctx.ast.expression_statement(SPAN, call_expr));

            if let Some(init_static_name) = init_static_name {
                // call initStatic
                let callee = self.ctx.ast.identifier_reference_expression(
                    IdentifierReference::new(SPAN, self.ctx.ast.new_atom(&init_static_name)),
                );
                let arguments = self
                    .ctx
                    .ast
                    .new_vec_single(Argument::Expression(self.ctx.ast.this_expression(SPAN)));
                statements.push(self.ctx.ast.expression_statement(
                    SPAN,
                    self.ctx.ast.call_expression(SPAN, callee, arguments, false, None),
                ));
            }

            let static_block = self.ctx.ast.static_block(SPAN, statements);
            class.body.body.insert(0, static_block);
        }

        Declaration::ClassDeclaration(self.ctx.ast.copy(class))
    }

    /// transform version: legacy
    pub fn transform_class_legacy(
        &mut self,
        class: &mut Box<'a, Class<'a>>,
        class_name: Option<CompactStr>,
    ) -> Declaration<'a> {
        let class_binding_identifier = &class.id.clone().unwrap_or_else(|| {
            let class_name = class_name.unwrap_or_else(|| self.get_unique_name("class"));
            BindingIdentifier::new(SPAN, self.ctx.ast.new_atom(&class_name))
        });
        let class_name = BindingPattern::new_with_kind(
            self.ctx.ast.binding_pattern_identifier(self.ctx.ast.copy(class_binding_identifier)),
        );

        let init = {
            let class_identifier_name = self.get_unique_name("class");
            let class_identifier =
                IdentifierReference::new(SPAN, self.ctx.ast.new_atom(&class_identifier_name));

            let decl = self.ctx.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                self.ctx.ast.new_vec_single(self.get_variable_declarator(&class_identifier_name)),
                Modifiers::empty(),
            );
            self.top_statements
                .push(Statement::Declaration(Declaration::VariableDeclaration(decl)));

            let left = self.ctx.ast.simple_assignment_target_identifier(class_identifier.clone());
            let right = self.ctx.ast.class_expression(self.ctx.ast.copy(class));
            let new_expr =
                self.ctx.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right);

            let new_expr = class.decorators.drain(..).fold(new_expr, |new_expr, decorator| {
                match &decorator.expression {
                    Expression::Identifier(identifier) => self.ctx.ast.call_expression(
                        SPAN,
                        self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                            SPAN,
                            identifier.name.clone(),
                        )),
                        self.ctx
                            .ast
                            .new_vec_single(Argument::Expression(self.ctx.ast.copy(&new_expr))),
                        false,
                        None,
                    ),
                    _ => new_expr,
                }
            });

            self.ctx.ast.logical_expression(
                SPAN,
                new_expr,
                LogicalOperator::Or,
                self.ctx.ast.identifier_reference_expression(class_identifier),
            )
        };

        let declarator = self.ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Let,
            class_name,
            Some(init),
            false,
        );

        Declaration::VariableDeclaration(self.ctx.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Let,
            self.ctx.ast.new_vec_single(declarator),
            Modifiers::empty(),
        ))
    }

    /// https://github.com/babel/babel/blob/eccbd203383487f6957dcf086aa83d773691560b/packages/babel-helpers/src/helpers/applyDecs2305.ts#L7-L45
    fn get_decorator_info(
        &self,
        key: &PropertyKey<'a>,
        value: Option<Box<'a, Function<'a>>>,
        flag: DecoratorFlags,
        decorator: &Decorator<'a>,
    ) -> Expression<'a> {
        let name = key.name();
        // [dec, flag, name, defaultValue | (o) => o.#a, (o, v) => o.#a = v]
        let mut decorator_elements = self.ctx.ast.new_vec_with_capacity(2);
        decorator_elements
            .push(ArrayExpressionElement::Expression(self.ctx.ast.copy(&decorator.expression)));
        decorator_elements.push(ArrayExpressionElement::Expression(
            self.ctx.ast.literal_number_expression(NumericLiteral::new(
                SPAN,
                0f64,
                self.ctx.ast.new_str(flag.to_value().to_string().as_str()),
                oxc_syntax::NumberBase::Decimal,
            )),
        ));
        if let Some(name) = name {
            decorator_elements.push(ArrayExpressionElement::Expression(
                self.ctx.ast.literal_string_expression(StringLiteral::new(
                    SPAN,
                    self.ctx.ast.new_atom(&name),
                )),
            ));

            if key.is_private_identifier() {
                if let Some(value) = value {
                    decorator_elements.push(ArrayExpressionElement::Expression(
                        Expression::FunctionExpression(value),
                    ));
                } else {
                    // o => o.#a
                    let mut items = self.ctx.ast.new_vec_single(self.ctx.ast.formal_parameter(
                        SPAN,
                        self.ctx.ast.binding_pattern(
                            self.ctx.ast.binding_pattern_identifier(BindingIdentifier::new(
                                SPAN,
                                self.ctx.ast.new_atom("o"),
                            )),
                            None,
                            false,
                        ),
                        None,
                        false,
                        false,
                        self.ctx.ast.new_vec(),
                    ));
                    let private_field = self.ctx.ast.private_field(
                        SPAN,
                        self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                            SPAN,
                            self.ctx.ast.new_atom("o"),
                        )),
                        PrivateIdentifier::new(SPAN, self.ctx.ast.new_atom(&name)),
                        false,
                    );
                    let params = self.ctx.ast.formal_parameters(
                        SPAN,
                        FormalParameterKind::ArrowFormalParameters,
                        self.ctx.ast.copy(&items),
                        None,
                    );
                    decorator_elements.push(ArrayExpressionElement::Expression(
                        self.ctx.ast.arrow_function_expression(
                            SPAN,
                            true,
                            false,
                            params,
                            self.ctx.ast.function_body(
                                SPAN,
                                self.ctx.ast.new_vec(),
                                self.ctx.ast.new_vec_single(
                                    self.ctx.ast.expression_statement(
                                        SPAN,
                                        self.ctx
                                            .ast
                                            .member_expression(self.ctx.ast.copy(&private_field)),
                                    ),
                                ),
                            ),
                            None,
                            None,
                        ),
                    ));

                    {
                        // (o, v) => o.#a = v
                        items.push(self.ctx.ast.formal_parameter(
                            SPAN,
                            self.ctx.ast.binding_pattern(
                                self.ctx.ast.binding_pattern_identifier(BindingIdentifier::new(
                                    SPAN,
                                    self.ctx.ast.new_atom("v"),
                                )),
                                None,
                                false,
                            ),
                            None,
                            false,
                            false,
                            self.ctx.ast.new_vec(),
                        ));

                        let params = self.ctx.ast.formal_parameters(
                            SPAN,
                            FormalParameterKind::ArrowFormalParameters,
                            items,
                            None,
                        );

                        decorator_elements.push(ArrayExpressionElement::Expression(
                            self.ctx.ast.arrow_function_expression(
                                SPAN,
                                true,
                                false,
                                params,
                                self.ctx.ast.function_body(
                                    SPAN,
                                    self.ctx.ast.new_vec(),
                                    self.ctx.ast.new_vec_single(
                                        self.ctx.ast.expression_statement(
                                            SPAN,
                                            self.ctx.ast.assignment_expression(
                                                SPAN,
                                                AssignmentOperator::Assign,
                                                self.ctx
                                                    .ast
                                                    .simple_assignment_target_member_expression(
                                                        private_field,
                                                    ),
                                                self.ctx.ast.identifier_reference_expression(
                                                    IdentifierReference::new(
                                                        SPAN,
                                                        self.ctx.ast.new_atom("v"),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                                None,
                                None,
                            ),
                        ));
                    }
                }
            }
        }
        self.ctx.ast.array_expression(SPAN, decorator_elements, None)
    }

    // _ => #a in _;
    fn get_is_private_function(&self, name: &Atom<'a>) -> Expression<'a> {
        self.ctx.ast.arrow_function_expression(
            SPAN,
            true,
            false,
            self.ctx.ast.formal_parameters(
                SPAN,
                FormalParameterKind::ArrowFormalParameters,
                self.ctx.ast.new_vec_single(self.ctx.ast.formal_parameter(
                    SPAN,
                    self.ctx.ast.binding_pattern(
                        self.ctx.ast.binding_pattern_identifier(BindingIdentifier::new(
                            SPAN,
                            self.ctx.ast.new_atom("_"),
                        )),
                        None,
                        false,
                    ),
                    None,
                    false,
                    false,
                    self.ctx.ast.new_vec(),
                )),
                None,
            ),
            self.ctx.ast.function_body(
                SPAN,
                self.ctx.ast.new_vec(),
                self.ctx.ast.new_vec_single(self.ctx.ast.expression_statement(
                    SPAN,
                    self.ctx.ast.private_in_expression(
                        SPAN,
                        PrivateIdentifier::new(SPAN, name.clone()),
                        self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                            SPAN,
                            self.ctx.ast.new_atom("_"),
                        )),
                    ),
                )),
            ),
            None,
            None,
        )
    }
}
