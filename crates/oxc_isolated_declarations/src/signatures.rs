use rustc_hash::FxHashMap;

use oxc_allocator::{CloneIn, Vec};
use oxc_ast::ast::{TSMethodSignatureKind, TSSignature};
use oxc_span::GetSpan;

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    /// Transform setter signature or getter return type to match the other
    ///
    /// Infer get accessor return type from set accessor's param type
    /// Infer set accessor parameter type from get accessor return type
    pub fn transform_ts_signatures(&mut self, signatures: &mut Vec<'a, TSSignature<'a>>) {
        // <name, (requires_inference, first_param_annotation, return_type)>
        let mut method_annotations: FxHashMap<_, (bool, _, _)> = FxHashMap::default();

        // Strip internal signatures
        signatures.retain(|signature| !self.has_internal_annotation(signature.span()));

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
                        entry.0 |= first_param.pattern.type_annotation.is_none();
                        entry.1 = Some(&mut first_param.pattern.type_annotation);
                    }
                    TSMethodSignatureKind::Get => {
                        let entry = method_annotations.entry(name.clone()).or_default();
                        entry.0 |= method.return_type.is_none();
                        entry.2 = Some(&mut method.return_type);
                    }
                };
            }
        });

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
}
