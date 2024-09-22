use oxc_allocator::{Box, CloneIn};
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, NONE};
use oxc_span::{GetSpan, SPAN};
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
            | PropertyKey::BigIntLiteral(_) => true,
            PropertyKey::TemplateLiteral(l) => l.expressions.is_empty(),
            PropertyKey::UnaryExpression(expr) => {
                expr.operator.is_arithmetic()
                    && matches!(
                        expr.argument,
                        Expression::NumericLiteral(_) | Expression::BigIntLiteral(_)
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
        let mut type_annotations = None;
        let mut value = None;

        if property.accessibility.map_or(true, |a| !a.is_private()) {
            if property.type_annotation.is_some() {
                // SAFETY: `ast.copy` is unsound! We need to fix.
                type_annotations = unsafe { self.ast.copy(&property.type_annotation) };
            } else if let Some(expr) = property.value.as_ref() {
                let ts_type = if property.readonly {
                    // `field = 'string'` remain `field = 'string'` instead of `field: 'string'`
                    if Self::is_need_to_infer_type_from_expression(expr) {
                        self.transform_expression_to_ts_type(expr)
                    } else {
                        if let Expression::TemplateLiteral(lit) = expr {
                            value = self
                                .transform_template_to_string(lit)
                                .map(Expression::StringLiteral);
                        } else {
                            // SAFETY: `ast.copy` is unsound! We need to fix.
                            value = Some(unsafe { self.ast.copy(expr) });
                        }
                        None
                    }
                } else {
                    self.infer_type_from_expression(expr)
                };

                type_annotations = ts_type.map(|t| self.ast.alloc_ts_type_annotation(SPAN, t));
            }

            if type_annotations.is_none() && value.is_none() {
                self.error(property_must_have_explicit_type(property.key.span()));
            }
        }

        self.ast.class_element_property_definition(
            property.r#type,
            property.span,
            self.ast.vec(),
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&property.key) },
            value,
            property.computed,
            property.r#static,
            false,
            property.r#override,
            property.optional,
            property.definite,
            property.readonly,
            type_annotations,
            self.transform_accessibility(property.accessibility),
        )
    }

    fn transform_class_method_definition(
        &self,
        definition: &MethodDefinition<'a>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    ) -> ClassElement<'a> {
        let function = &definition.value;

        let value = self.ast.alloc_function(
            FunctionType::TSEmptyBodyFunctionExpression,
            function.span,
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&function.id) },
            false,
            false,
            false,
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&function.type_parameters) },
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&function.this_param) },
            params,
            return_type,
            NONE,
        );

        self.ast.class_element_method_definition(
            definition.r#type,
            definition.span,
            self.ast.vec(),
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&definition.key) },
            value,
            definition.kind,
            definition.computed,
            definition.r#static,
            definition.r#override,
            definition.optional,
            self.transform_accessibility(definition.accessibility),
        )
    }

    fn create_class_property(
        &self,
        r#type: PropertyDefinitionType,
        span: Span,
        key: PropertyKey<'a>,
        r#static: bool,
        r#override: bool,
        accessibility: Option<TSAccessibility>,
    ) -> ClassElement<'a> {
        self.ast.class_element_property_definition(
            r#type,
            span,
            self.ast.vec(),
            key,
            None,
            false,
            r#static,
            false,
            r#override,
            false,
            false,
            false,
            NONE,
            accessibility,
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
        let key = self.ast.property_key_identifier_name(SPAN, ident_name);
        Some(self.ast.class_element_property_definition(
            PropertyDefinitionType::PropertyDefinition,
            param.span,
            self.ast.vec(),
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
                    method.span,
                    // SAFETY: `ast.copy` is unsound! We need to fix.
                    unsafe { self.ast.copy(&method.key) },
                    method.r#static,
                    method.r#override,
                    self.transform_accessibility(method.accessibility),
                )
            }
            MethodDefinitionKind::Get | MethodDefinitionKind::Constructor => {
                let params = self.ast.alloc_formal_parameters(
                    SPAN,
                    FormalParameterKind::Signature,
                    self.ast.vec(),
                    NONE,
                );
                self.transform_class_method_definition(method, params, None)
            }
            MethodDefinitionKind::Set => {
                let params = self.create_formal_parameters(
                    self.ast.binding_pattern_kind_binding_identifier(SPAN, "value"),
                );
                self.transform_class_method_definition(method, params, None)
            }
        }
    }

    fn transform_constructor_params_to_class_properties(
        &self,
        function: &Function<'a>,
        params: &FormalParameters<'a>,
    ) -> oxc_allocator::Vec<'a, ClassElement<'a>> {
        let mut elements = self.ast.vec();
        for (index, param) in function.params.items.iter().enumerate() {
            if param.accessibility.is_some() || param.readonly {
                let type_annotation =
                    if param.accessibility.is_some_and(TSAccessibility::is_private) {
                        None
                    } else {
                        // transformed params will definitely have type annotation
                        // SAFETY: `ast.copy` is unsound! We need to fix.
                        unsafe { self.ast.copy(&params.items[index].pattern.type_annotation) }
                    };
                if let Some(new_element) =
                    self.transform_formal_parameter_to_class_property(param, type_annotation)
                {
                    elements.push(new_element);
                }
            }
        }
        elements
    }

    /// Transform getter and setter methods
    ///
    /// ### Getter
    ///
    /// 1. If it has no return type, infer it from the function body
    /// 2. If it cannot be inferred from the function body, try to infer it from setter method's first parameter
    ///
    /// ### Setter
    ///
    /// 1. If it has no parameter, create a parameter with the name `value`
    /// 2. If it has no parameter type, infer it from the getter method's return type
    fn transform_getter_or_setter_methods(&self, decl: &mut Class<'a>) {
        let mut method_annotations: FxHashMap<_, (bool, _, _)> = FxHashMap::default();
        for element in decl.body.body.iter_mut() {
            if let ClassElement::MethodDefinition(method) = element {
                if method.key.is_private_identifier()
                    && (method.computed && !self.is_literal_key(&method.key))
                {
                    continue;
                }

                let Some(name) = method.key.static_name() else {
                    continue;
                };

                match method.kind {
                    MethodDefinitionKind::Set => {
                        let params = &mut method.value.params;
                        if params.items.is_empty() {
                            *params = self.create_formal_parameters(
                                self.ast.binding_pattern_kind_binding_identifier(SPAN, "value"),
                            );
                        }
                        let Some(first_param) = method.value.params.items.first_mut() else {
                            continue;
                        };
                        let entry = method_annotations.entry(name).or_default();
                        entry.0 |= first_param.pattern.type_annotation.is_none();
                        entry.1 = Some(&mut first_param.pattern.type_annotation);
                    }
                    MethodDefinitionKind::Get => {
                        let function = &mut method.value;
                        if function.return_type.is_none() {
                            function.return_type = self.infer_function_return_type(function);
                        };
                        let return_type = &mut function.return_type;
                        let entry = method_annotations.entry(name).or_default();
                        entry.0 |= return_type.is_none();
                        entry.2 = Some(&mut function.return_type);
                    }
                    _ => continue,
                };
            }
        }

        for (requires_inference, param, return_type) in method_annotations.into_values() {
            if requires_inference {
                if let (Some(Some(annotation)), Some(option))
                | (Some(option), Some(Some(annotation))) = (param, return_type)
                {
                    option.replace(annotation.clone_in(self.ast.allocator));
                }
            }
        }
    }

    pub fn transform_class(
        &self,
        decl: &mut Class<'a>,
        declare: Option<bool>,
    ) -> Option<Box<'a, Class<'a>>> {
        if decl.declare {
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

        self.transform_getter_or_setter_methods(decl);
        let mut has_private_key = false;
        let mut elements = self.ast.vec();
        let mut is_function_overloads = false;
        for element in &decl.body.body {
            match element {
                ClassElement::StaticBlock(_) => {}
                ClassElement::MethodDefinition(ref method) => {
                    if self.has_internal_annotation(method.span) {
                        continue;
                    }
                    if !(method.r#type.is_abstract() || method.optional)
                        && method.value.body.is_none()
                    {
                        is_function_overloads = true;
                    } else if is_function_overloads {
                        // Skip implementation of function overloads
                        is_function_overloads = false;
                        continue;
                    }
                    if method.key.is_private_identifier() {
                        has_private_key = true;
                        continue;
                    }
                    if self.report_property_key(&method.key, method.computed) {
                        continue;
                    }

                    let function = &method.value;
                    let params = match method.kind {
                        MethodDefinitionKind::Set => function.params.clone_in(self.ast.allocator),
                        MethodDefinitionKind::Constructor => {
                            let params = self.transform_formal_parameters(&function.params);
                            elements.splice(
                                0..0,
                                self.transform_constructor_params_to_class_properties(
                                    function, &params,
                                ),
                            );

                            if method.accessibility.is_some_and(TSAccessibility::is_private) {
                                elements.push(self.transform_private_modifier_method(method));
                                continue;
                            }

                            params
                        }
                        _ => {
                            if method.accessibility.is_some_and(TSAccessibility::is_private) {
                                elements.push(self.transform_private_modifier_method(method));
                                continue;
                            }
                            self.transform_formal_parameters(&function.params)
                        }
                    };

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
                            let rt = method.value.return_type.clone_in(self.ast.allocator);
                            if method.value.return_type.is_none() {
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
                    if self.has_internal_annotation(property.span) {
                        continue;
                    }

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
                    if self.has_internal_annotation(property.span) {
                        continue;
                    }

                    if self.report_property_key(&property.key, property.computed) {
                        return None;
                    }

                    if property.key.is_private_identifier() {
                        has_private_key = true;
                        continue;
                    }

                    // FIXME: missing many fields
                    let new_element = self.ast.class_element_accessor_property(
                        property.r#type,
                        property.span,
                        self.ast.vec(),
                        // SAFETY: `ast.copy` is unsound! We need to fix.
                        unsafe { self.ast.copy(&property.key) },
                        None,
                        property.computed,
                        property.r#static,
                        property.definite,
                        // SAFETY: `ast.copy` is unsound! We need to fix.
                        unsafe { self.ast.copy(&property.type_annotation) },
                        property.accessibility,
                    );
                    elements.push(new_element);
                }
                ClassElement::TSIndexSignature(signature) => elements.push({
                    if self.has_internal_annotation(signature.span) {
                        continue;
                    }
                    // SAFETY: `ast.copy` is unsound! We need to fix.
                    unsafe { self.ast.copy(element) }
                }),
            }
        }

        if has_private_key {
            // <https://github.com/microsoft/TypeScript/blob/64d2eeea7b9c7f1a79edf42cb99f302535136a2e/src/compiler/transformers/declarations.ts#L1699-L1709>
            // When the class has at least one private identifier, create a unique constant identifier to retain the nominal typing behavior
            // Prevents other classes with the same public members from being used in place of the current class
            let ident = self.ast.property_key_private_identifier(SPAN, "private");
            let r#type = PropertyDefinitionType::PropertyDefinition;
            let decorators = self.ast.vec();
            let element = self.ast.class_element_property_definition(
                r#type, SPAN, decorators, ident, None, false, false, false, false, false, false,
                false, NONE, None,
            );

            elements.insert(0, element);
        }

        let body = self.ast.class_body(decl.body.span, elements);

        Some(self.ast.alloc_class(
            decl.r#type,
            decl.span,
            self.ast.vec(),
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&decl.id) },
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&decl.type_parameters) },
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&decl.super_class) },
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&decl.super_type_parameters) },
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&decl.implements) },
            body,
            decl.r#abstract,
            declare.unwrap_or_else(|| self.is_declare()),
        ))
    }

    pub fn create_formal_parameters(
        &self,
        kind: BindingPatternKind<'a>,
    ) -> Box<'a, FormalParameters<'a>> {
        let pattern = self.ast.binding_pattern(kind, None::<TSTypeAnnotation<'a>>, false);
        let parameter =
            self.ast.formal_parameter(SPAN, self.ast.vec(), pattern, None, false, false);
        let items = self.ast.vec1(parameter);
        self.ast.alloc_formal_parameters(SPAN, FormalParameterKind::Signature, items, NONE)
    }
}
