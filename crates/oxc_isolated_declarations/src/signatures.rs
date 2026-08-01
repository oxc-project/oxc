use rustc_hash::FxHashMap;

use oxc_allocator::{ArenaVec, CloneIn, GetAllocator};
use oxc_ast::ast::{
    ClassElement, MethodDefinitionKind, TSAccessibility, TSMethodSignatureKind, TSSignature,
};
use oxc_span::GetSpan;

use crate::{
    IsolatedDeclarations,
    diagnostics::{accessor_must_have_explicit_return_type, method_must_have_explicit_return_type},
};

impl<'a> IsolatedDeclarations<'a> {
    /// Transform setter signature or getter return type to match the other
    ///
    /// Infer get accessor return type from set accessor's param type
    /// Infer set accessor parameter type from get accessor return type
    pub fn transform_ts_signatures(&self, signatures: &mut ArenaVec<'a, TSSignature<'a>>) {
        // <name, (requires_inference, first_param_annotation, return_type)>
        let mut method_annotations: FxHashMap<_, (bool, _, _)> = FxHashMap::default();

        // Strip internal signatures
        signatures.retain(|signature| !self.has_internal_annotation(signature.span()));

        // Static ETS interfaces may contain concrete-looking methods and fields.
        // Declaration output must retain their ETS modifiers and annotations while
        // removing executable bodies and non-declaration initializers.
        for signature in signatures.iter_mut() {
            match signature {
                TSSignature::MethodDefinition(method) => {
                    let function = &method.value;
                    let params = self.transform_formal_parameters(
                        &function.params,
                        method.accessibility.is_some_and(TSAccessibility::is_private),
                    );
                    let return_type = match method.kind {
                        MethodDefinitionKind::Method => {
                            let return_type = self.infer_function_return_type(function);
                            if return_type.is_none() {
                                self.error(method_must_have_explicit_return_type(
                                    method.key.span(),
                                ));
                            }
                            return_type
                        }
                        MethodDefinitionKind::Get => {
                            let return_type = self.infer_function_return_type(function);
                            if return_type.is_none() {
                                self.error(accessor_must_have_explicit_return_type(
                                    method.key.span(),
                                ));
                            }
                            return_type
                        }
                        MethodDefinitionKind::Set | MethodDefinitionKind::Constructor => None,
                    };
                    let transformed =
                        self.transform_class_method_definition(method, params, return_type);
                    let ClassElement::MethodDefinition(transformed) = transformed else {
                        unreachable!();
                    };
                    *method = transformed;
                }
                TSSignature::PropertyDefinition(property) => {
                    let transformed = self.transform_class_property_definition(property);
                    let ClassElement::PropertyDefinition(transformed) = transformed else {
                        unreachable!();
                    };
                    *property = transformed;
                }
                _ => {}
            }
        }

        signatures.iter_mut().for_each(|signature| {
            if let TSSignature::TSMethodSignature(method) = signature {
                let Some(name) = method.key.static_name() else {
                    return;
                };
                match method.kind {
                    TSMethodSignatureKind::Method => {}
                    TSMethodSignatureKind::Set => {
                        let Some(first_param) = method.params.items.first_mut() else {
                            return;
                        };

                        let entry = method_annotations.entry(name.clone()).or_default();
                        entry.0 |= first_param.type_annotation.is_none();
                        entry.1 = Some(&mut first_param.type_annotation);
                    }
                    TSMethodSignatureKind::Get => {
                        let entry = method_annotations.entry(name.clone()).or_default();
                        entry.0 |= method.return_type.is_none();
                        entry.2 = Some(&mut method.return_type);
                    }
                }
            }
        });

        for (requires_inference, param, return_type) in method_annotations.into_values() {
            if requires_inference
                && let (Some(Some(annotation)), Some(option))
                | (Some(option), Some(Some(annotation))) = (param, return_type)
            {
                option.replace(annotation.clone_in(self.allocator()));
            }
        }
    }
}
