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
                type_annotations = self.ast.copy(&property.type_annotation);
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
                            value = Some(self.ast.copy(expr));
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
            self.ast.copy(&property.key),
            value,
            property.computed,
            property.r#static,
            property.declare,
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
            self.ast.copy(&function.id),
            false,
            false,
            false,
            self.ast.copy(&function.type_parameters),
            self.ast.copy(&function.this_param),
            params,
            Option::<FunctionBody>::None,
            return_type,
        );

        self.ast.class_element_method_definition(
            definition.r#type,
            definition.span,
            self.ast.vec(),
            self.ast.copy(&definition.key),
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
        key: PropertyKey<'a>,
        r#static: bool,
        r#override: bool,
        accessibility: Option<TSAccessibility>,
    ) -> ClassElement<'a> {
        self.ast.class_element_property_definition(
            r#type,
            SPAN,
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
            Option::<TSTypeAnnotation>::None,
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
                    self.ast.copy(&method.key),
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
                    Option::<BindingRestElement>::None,
                );
                self.transform_class_method_definition(method, params, None)
            }
            MethodDefinitionKind::Set => {
                let params = self.create_formal_parameters(
                    self.ast.binding_pattern_kind_binding_identifier(SPAN, "value"),
                    None,
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
                let type_annotation = if param.accessibility.is_some_and(|a| a.is_private()) {
                    None
                } else {
                    // transformed params will definitely have type annotation
                    self.ast.copy(&params.items[index].pattern.type_annotation)
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

    /// Infer get accessor return type from set accessor
    /// Infer set accessor parameter type from get accessor return type
    fn collect_inferred_accessor_types(
        &self,
        decl: &Class<'a>,
    ) -> FxHashMap<Atom, Box<'a, TSTypeAnnotation<'a>>> {
        let mut inferred_accessor_types: FxHashMap<Atom<'a>, Box<'a, TSTypeAnnotation<'a>>> =
            FxHashMap::default();

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
                let name = self.ast.atom(&name);
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
                                            .map(|x| self.ast.alloc_ts_type_annotation(SPAN, x))
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

        inferred_accessor_types
    }

    pub fn transform_class(
        &self,
        decl: &Class<'a>,
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

        let mut has_private_key = false;
        let mut elements = self.ast.vec();
        let mut is_function_overloads = false;
        for element in &decl.body.body {
            match element {
                ClassElement::StaticBlock(_) => {}
                ClassElement::MethodDefinition(ref method) => {
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
                    if method.accessibility.is_some_and(|a| a.is_private()) {
                        elements.push(self.transform_private_modifier_method(method));
                        continue;
                    }

                    let inferred_accessor_types = self.collect_inferred_accessor_types(decl);
                    let function = &method.value;
                    let params = if method.kind.is_set() {
                        method.key.static_name().map_or_else(
                            || self.transform_formal_parameters(&function.params),
                            |n| {
                                self.transform_set_accessor_params(
                                    &function.params,
                                    inferred_accessor_types
                                        .get(&self.ast.atom(&n))
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
                                    .get(&self.ast.atom(&name))
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
                    let new_element = self.ast.class_element_accessor_property(
                        property.r#type,
                        property.span,
                        self.ast.vec(),
                        self.ast.copy(&property.key),
                        None,
                        property.computed,
                        property.r#static,
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
            let ident = self.ast.property_key_private_identifier(SPAN, "private");
            let r#type = PropertyDefinitionType::PropertyDefinition;
            let decorators = self.ast.vec();
            let element = self.ast.class_element_property_definition(
                r#type,
                SPAN,
                decorators,
                ident,
                None,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                Option::<TSTypeAnnotation>::None,
                None,
            );

            elements.insert(0, element);
        }

        let body = self.ast.class_body(decl.body.span, elements);

        Some(self.ast.alloc_class(
            decl.r#type,
            decl.span,
            self.ast.vec(),
            self.ast.copy(&decl.id),
            self.ast.copy(&decl.super_class),
            body,
            self.ast.copy(&decl.type_parameters),
            self.ast.copy(&decl.super_type_parameters),
            self.ast.copy(&decl.implements),
            decl.r#abstract,
            declare.unwrap_or_else(|| self.is_declare()),
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
                || self.ast.binding_pattern_kind_binding_identifier(SPAN, "value"),
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
            self.ast.formal_parameter(SPAN, self.ast.vec(), pattern, None, false, false);
        let items = self.ast.vec1(parameter);
        self.ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::Signature,
            items,
            Option::<BindingRestElement>::None,
        )
    }
}
