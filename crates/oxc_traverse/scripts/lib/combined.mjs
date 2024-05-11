import {camelToSnake} from './utils.mjs';

export default function generateCombinedTraverseImplCode(types) {
    const typesArr = Object.values(types);
    typesArr.push({name: 'Statements', rawName: "Vec<'a, Statement<'a>>"});

    let methods = '';
    for (const type of typesArr) {
        const snakeName = camelToSnake(type.name);
        methods += `
            #[inline]
            fn enter_${snakeName}(&mut self, node: &mut ${type.rawName}, ctx: &TraverseCtx<'a>) {
                self.traverse1.enter_${snakeName}(node, ctx);
                self.traverse2.enter_${snakeName}(node, ctx);
            }
            #[inline]
            fn exit_${snakeName}(&mut self, node: &mut ${type.rawName}, ctx: &TraverseCtx<'a>) {
                self.traverse2.exit_${snakeName}(node, ctx);
                self.traverse1.exit_${snakeName}(node, ctx);
            }
        `;
    }

    return `
        use oxc_allocator::Vec;
        #[allow(clippy::wildcard_imports)]
        use oxc_ast::ast::*;

        use crate::{CombinedTraverse, Traverse, TraverseCtx};

        #[allow(unused_variables)]
        impl<'a, Tr1, Tr2> Traverse<'a> for CombinedTraverse<'a, Tr1, Tr2>
        where
            Tr1: Traverse<'a>,
            Tr2: Traverse<'a>,
        {
            ${methods}
        }
    `;
}
