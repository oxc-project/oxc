use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::FormalParameters;

/// Copy TypeScript metadata onto parameters rebuilt by codegen.
pub fn copy_param_ts_metadata<'a>(
    allocator: &'a Allocator,
    new_params: &mut FormalParameters<'a>,
    source_params: &FormalParameters<'a>,
) {
    for (param, source) in new_params.items.iter_mut().zip(source_params.items.iter()) {
        param.decorators = source.decorators.clone_in_with_semantic_ids(allocator);
        param.type_annotation = source.type_annotation.clone_in_with_semantic_ids(allocator);
        param.optional = source.optional;
        param.accessibility = source.accessibility;
        param.readonly = source.readonly;
        param.r#override = source.r#override;
    }
    if let (Some(rest), Some(source_rest)) = (&mut new_params.rest, &source_params.rest) {
        rest.decorators = source_rest.decorators.clone_in_with_semantic_ids(allocator);
        rest.type_annotation = source_rest.type_annotation.clone_in_with_semantic_ids(allocator);
    }
}
