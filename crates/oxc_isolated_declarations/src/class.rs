use oxc_allocator::Box;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_span::{Atom, GetSpan, SPAN};
use rustc_hash::FxHashMap;

use crate::{
    diagnostics::{
        accessor_must_have_explicit_return_type, computed_property_name, extends_clause_expression,
        method_must_have_explicit_return_type, property_must_have_explicit_type,
    },
    IsolatedDeclarations,
};

impl<'a> IsolatedDeclarations<'a> {
    pub fn is_literal_key(&self, key: &PropertyKey<'a>) -> bool {
        match key {
            PropertyKey::StringLiteral(_)
            | PropertyKey::NumericLiteral(_)
            | PropertyKey::BigintLiteral(_) => true,
            PropertyKey::TemplateLiteral(l) => l.expressions.is_empty(),
            PropertyKey::UnaryExpression(expr) => {
                expr.operator.is_arithmetic()
                    && matches!(
                        expr.argument,
                        Expression::NumericLiteral(_) | Expression::BigintLiteral(_)
                    )
            }
            _ => false,
        }
    }

    pub fn report_property_key(&self, key: &PropertyKey<'a>, computed: bool) -> bool {
        if computed && !self.is_literal_key(key) {
            self.error(computed_property_name(key.span()));
            true
        } else {
            false
        }
    }

    pub fn transform_accessibility(
        &self,
        accessibility: Option<TSAccessibility>,
    ) -> Option<TSAccessibility> {
        if accessibility.is_none() || accessibility.is_some_and(|a| a == TSAccessibility::Public) {
            None
        } else {
            accessibility
        }
    }

    fn transform_class_property_definition(
        &self,
        property: &PropertyDefinition<'a>,
    ) -> ClassElement<'a> {
        let type_annotations = if property.accessibility.is_some_and(|a| a.is_private()) {
            None
        } else {
            property
                .type_annotation
                .as_ref()
                .map(|type_annotation| self.ast.copy(type_annotation))
                .or_else(|| {
                    property
                        .value
                        .as_ref()
                        .and_then(|expr| {
                            let ts_type = self.infer_type_from_expression(expr);
                            if ts_type.is_none() {
                                self.error(property_must_have_explicit_type(property.key.span()));
                            }
                            ts_type
                        })
                        .map(|ts_type| self.ast.ts_type_annotation(SPAN, ts_type))
                })
        };

        self.ast.class_property(
            property.r#type,
            property.span,
            self.ast.copy(&property.key),
            None,
            property.computed,
            property.r#static,
            property.declare,
            property.r#override,
            property.optional,
            property.definite,
            property.readonly,
            type_annotations,
            self.transform_accessibility(property.accessibility),
            self.ast.new_vec(),
        )
    }

    fn transform_class_method_definition(
        &self,
        definition: &MethodDefinition<'a>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> ClassElement<'a> {
        let function = &definition.value;

        let value = self.ast.function(
            FunctionType::TSEmptyBodyFunctionExpression,
            function.span,
            self.ast.copy(&function.id),
            function.generator,
            function.r#async,
            self.ast.copy(&function.this_param),
            params,
            None,
            self.ast.copy(&function.type_parameters),
            return_type,
            Modifiers::empty(),
        );

        self.ast.class_method(
            definition.r#type,
            definition.span,
            self.ast.copy(&definition.key),
            definition.kind,
            value,
            definition.computed,
            definition.r#static,
            definition.r#override,
            definition.optional,
            self.transform_accessibility(definition.accessibility),
            self.ast.new_vec(),
        )
    }

    fn create_class_property(
        &self,
        r#type: PropertyDefinitionType,
        key: PropertyKey<'a>,
        r#override: bool,
        accessibility: Option<TSAccessibility>,
    ) -> ClassElement<'a> {
        self.ast.class_property(
            r#type,
            SPAN,
            key,
            None,
            false,
            false,
            false,
            r#override,
            false,
            false,
            false,
            None,
            accessibility,
            self.ast.new_vec(),
        )
    }

    fn transform_formal_parameter_to_class_property(
        &self,
        param: &FormalParameter<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Option<ClassElement<'a>> {
        let Some(ident_name) = param.pattern.get_identifier() else {
            // A parameter property may not be declared using a binding pattern.(1187)
            return None;
        };
        let key = self.ast.property_key_identifier(IdentifierName::new(SPAN, ident_name.clone()));
        Some(self.ast.class_property(
            PropertyDefinitionType::PropertyDefinition,
            param.span,
            key,
            None,
            false,
            false,
            false,
            param.r#override,
            param.pattern.optional,
            false,
            param.readonly,
            type_annotation,
            self.transform_accessibility(param.accessibility),
            self.ast.new_vec(),
        ))
    }

    fn transform_private_modifier_method(&self, method: &MethodDefinition<'a>) -> ClassElement<'a> {
        match method.kind {
            MethodDefinitionKind::Method => {
                let r#type = match method.r#type {
                    MethodDefinitionType::MethodDefinition => {
                        PropertyDefinitionType::PropertyDefinition
                    }
                    MethodDefinitionType::TSAbstractMethodDefinition => {
                        PropertyDefinitionType::TSAbstractPropertyDefinition
                    }
                };
                self.create_class_property(
                    r#type,
                    self.ast.copy(&method.key),
                    method.r#override,
                    self.transform_accessibility(method.accessibility),
                )
            }
            MethodDefinitionKind::Get => {
                let params = self.ast.formal_parameters(
                    SPAN,
                    FormalParameterKind::Signature,
                    self.ast.new_vec(),
                    None,
                );
                self.transform_class_method_definition(method, params, None)
            }
            MethodDefinitionKind::Set => {
                let params = self.create_formal_parameters(
                    self.ast.binding_pattern_identifier(BindingIdentifier::new(
                        SPAN,
                        self.ast.new_atom("value"),
                    )),
                    None,
                );
                self.transform_class_method_definition(method, params, None)
            }
            MethodDefinitionKind::Constructor => {
                unreachable!()
            }
        }
    }

    fn transform_constructor_params_to_class_properties(
        &self,
        function: &Function<'a>,
        params: &FormalParameters<'a>,
    ) -> oxc_allocator::Vec<'a, ClassElement<'a>> {
        let mut elements = self.ast.new_vec();
        for (index, param) in function.params.items.iter().enumerate() {
            if param.accessibility.is_some() {
                // transformed params will definitely have type annotation
                let type_annotation = self.ast.copy(&params.items[index].pattern.type_annotation);
                if let Some(new_element) =
                    self.transform_formal_parameter_to_class_property(param, type_annotation)
                {
                    elements.push(new_element);
                }
            }
        }
        elements
    }

    pub fn transform_class(
        &self,
        decl: &Class<'a>,
        modifiers: Option<Modifiers<'a>>,
    ) -> Option<Box<'a, Class<'a>>> {
        if decl.is_declare() {
            return None;
        }

        if let Some(super_class) = &decl.super_class {
            let is_not_allowed = match super_class {
                Expression::Identifier(_) => false,
                Expression::StaticMemberExpression(expr) => {
                    !expr.get_first_object().is_identifier_reference()
                }
                _ => true,
            };
            if is_not_allowed {
                self.error(extends_clause_expression(super_class.span()));
            }
        }

        let mut inferred_accessor_types: FxHashMap<Atom<'a>, Box<'a, TSTypeAnnotation<'a>>> =
            FxHashMap::default();

        // Infer get accessor return type from set accessor
        // Infer set accessor parameter type from get accessor
        for element in &decl.body.body {
            if let ClassElement::MethodDefinition(method) = element {
                if method.key.is_private_identifier()
                    || method.accessibility.is_some_and(|a| a.is_private())
                    || (method.computed && !self.is_literal_key(&method.key))
                {
                    continue;
                }
                let Some(name) = method.key.static_name() else {
                    continue;
                };
                let name = self.ast.new_atom(&name);
                if inferred_accessor_types.contains_key(&name) {
                    // We've inferred that accessor type already
                    continue;
                }
                let function = &method.value;
                match method.kind {
                    MethodDefinitionKind::Get => {
                        let return_type = self.infer_function_return_type(function);
                        if let Some(return_type) = return_type {
                            inferred_accessor_types.insert(name, self.ast.copy(&return_type));
                        }
                    }
                    MethodDefinitionKind::Set => {
                        if let Some(param) = function.params.items.first() {
                            let type_annotation =
                                param.pattern.type_annotation.as_ref().map_or_else(
                                    || {
                                        self.infer_type_from_formal_parameter(param)
                                            .map(|x| self.ast.ts_type_annotation(SPAN, x))
                                    },
                                    |t| Some(self.ast.copy(t)),
                                );
                            if let Some(type_annotation) = type_annotation {
                                inferred_accessor_types.insert(name, type_annotation);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut has_private_key = false;
        let mut elements = self.ast.new_vec();
        for element in &decl.body.body {
            match element {
                ClassElement::StaticBlock(_) => {}
                ClassElement::MethodDefinition(ref method) => {
                    if method.key.is_private_identifier() {
                        has_private_key = true;
                        continue;
                    }
                    if self.report_property_key(&method.key, method.computed) {
                        continue;
                    }
                    if method.accessibility.is_some_and(|a| a.is_private()) {
                        elements.push(self.transform_private_modifier_method(method));
                        continue;
                    }
                    let function = &method.value;
                    let params = if method.kind.is_set() {
                        method.key.static_name().map_or_else(
                            || self.transform_formal_parameters(&function.params),
                            |n| {
                                self.transform_set_accessor_params(
                                    &function.params,
                                    inferred_accessor_types
                                        .get(&self.ast.new_atom(&n))
                                        .map(|t| self.ast.copy(t)),
                                )
                            },
                        )
                    } else {
                        self.transform_formal_parameters(&function.params)
                    };

                    if let MethodDefinitionKind::Constructor = method.kind {
                        elements.extend(
                            self.transform_constructor_params_to_class_properties(
                                function, &params,
                            ),
                        );
                    }

                    let return_type = match method.kind {
                        MethodDefinitionKind::Method => {
                            let rt = self.infer_function_return_type(function);
                            if rt.is_none() {
                                self.error(method_must_have_explicit_return_type(
                                    method.key.span(),
                                ));
                            }
                            rt
                        }
                        MethodDefinitionKind::Get => {
                            let rt = method.key.static_name().and_then(|name| {
                                inferred_accessor_types
                                    .get(&self.ast.new_atom(&name))
                                    .map(|t| self.ast.copy(t))
                            });
                            if rt.is_none() {
                                self.error(accessor_must_have_explicit_return_type(
                                    method.key.span(),
                                ));
                            }
                            rt
                        }
                        MethodDefinitionKind::Set | MethodDefinitionKind::Constructor => None,
                    };
                    let new_element =
                        self.transform_class_method_definition(method, params, return_type);
                    elements.push(new_element);
                }
                ClassElement::PropertyDefinition(property) => {
                    if self.report_property_key(&property.key, property.computed) {
                        continue;
                    }

                    if property.key.is_private_identifier() {
                        has_private_key = true;
                    } else {
                        elements.push(self.transform_class_property_definition(property));
                    }
                }
                ClassElement::AccessorProperty(property) => {
                    if self.report_property_key(&property.key, property.computed) {
                        return None;
                    }

                    if property.key.is_private_identifier() {
                        has_private_key = true;
                        continue;
                    }

                    // FIXME: missing many fields
                    let new_element = self.ast.accessor_property(
                        property.r#type,
                        property.span,
                        self.ast.copy(&property.key),
                        None,
                        property.computed,
                        property.r#static,
                        self.ast.new_vec(),
                    );
                    elements.push(new_element);
                }
                ClassElement::TSIndexSignature(_) => elements.push(self.ast.copy(element)),
            }
        }

        if has_private_key {
            // <https://github.com/microsoft/TypeScript/blob/64d2eeea7b9c7f1a79edf42cb99f302535136a2e/src/compiler/transformers/declarations.ts#L1699-L1709>
            // When the class has at least one private identifier, create a unique constant identifier to retain the nominal typing behavior
            // Prevents other classes with the same public members from being used in place of the current class
            let ident = self
                .ast
                .property_key_private_identifier(PrivateIdentifier::new(SPAN, "private".into()));
            let r#type = PropertyDefinitionType::PropertyDefinition;
            let decorators = self.ast.new_vec();
            let element = self.ast.class_property(
                r#type, SPAN, ident, None, false, false, false, false, false, false, false, None,
                None, decorators,
            );

            elements.insert(0, element);
        }

        let body = self.ast.class_body(decl.body.span, elements);

        let mut modifiers = modifiers.unwrap_or_else(|| self.modifiers_declare());
        if decl.modifiers.is_contains_abstract() {
            modifiers.add_modifier(Modifier { span: SPAN, kind: ModifierKind::Abstract });
        };

        Some(self.ast.class(
            decl.r#type,
            decl.span,
            self.ast.copy(&decl.id),
            self.ast.copy(&decl.super_class),
            body,
            self.ast.copy(&decl.type_parameters),
            self.ast.copy(&decl.super_type_parameters),
            self.ast.copy(&decl.implements),
            self.ast.new_vec(),
            modifiers,
        ))
    }

    pub fn transform_set_accessor_params(
        &self,
        params: &Box<'a, FormalParameters<'a>>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Box<'a, FormalParameters<'a>> {
        let items = &params.items;
        if items.first().map_or(true, |item| item.pattern.type_annotation.is_none()) {
            let kind = items.first().map_or_else(
                || {
                    self.ast.binding_pattern_identifier(BindingIdentifier::new(
                        SPAN,
                        self.ast.new_atom("value"),
                    ))
                },
                |item| self.ast.copy(&item.pattern.kind),
            );

            self.create_formal_parameters(kind, type_annotation)
        } else {
            self.transform_formal_parameters(params)
        }
    }

    pub fn create_formal_parameters(
        &self,
        kind: BindingPatternKind<'a>,
        type_annotation: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> Box<'a, FormalParameters<'a>> {
        let pattern = BindingPattern { kind, type_annotation, optional: false };
        let parameter =
            self.ast.formal_parameter(SPAN, pattern, None, false, false, self.ast.new_vec());
        let items = self.ast.new_vec_single(parameter);
        self.ast.formal_parameters(SPAN, FormalParameterKind::Signature, items, None)
    }
}
