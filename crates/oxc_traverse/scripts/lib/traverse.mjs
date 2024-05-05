import {camelToSnake, toTypeName} from './utils.mjs';

export default function generateTraverseTraitCode(types) {
    let traverseMethods = '';
    for (const type of Object.values(types)) {
        const snakeName = camelToSnake(type.name),
            typeName = toTypeName(type);
        traverseMethods += `
            #[inline]
            fn enter_${snakeName}(&mut self, node: &mut ${typeName}, ctx: &TraverseCtx<'a>) {}
            #[inline]
            fn exit_${snakeName}(&mut self, node: &mut ${typeName}, ctx: &TraverseCtx<'a>) {}
        `;
    }

    return `
        use oxc_allocator::Vec;
        #[allow(clippy::wildcard_imports)]
        use oxc_ast::ast::*;

        use crate::TraverseCtx;

        #[allow(unused_variables)]
        pub trait Traverse<'a> {
            ${traverseMethods}

            #[inline]
            fn enter_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &TraverseCtx<'a>) {}
            #[inline]
            fn exit_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &TraverseCtx<'a>) {}
        }
    `;
}
