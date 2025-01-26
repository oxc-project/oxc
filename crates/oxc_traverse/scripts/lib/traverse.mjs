import { camelToSnake } from './utils.mjs';

/**
 * @param {import('./parse.mjs').Types} types
 */
export default function generateTraverseTraitCode(types) {
  const typesArr = Object.values(types);
  typesArr.push({ name: 'Statements', rawName: "Vec<'a, Statement<'a>>" });

  let traverseMethods = '';
  for (const type of typesArr) {
    const snakeName = camelToSnake(type.name);
    traverseMethods += `
      #[inline]
      fn enter_${snakeName}(&mut self, node: &mut ${type.rawName}, ctx: &mut TraverseCtx<'a>) {}
      #[inline]
      fn exit_${snakeName}(&mut self, node: &mut ${type.rawName}, ctx: &mut TraverseCtx<'a>) {}
    `;
  }

  return `
    use oxc_allocator::Vec;
    use oxc_ast::ast::*;

    use crate::TraverseCtx;

    #[expect(unused_variables)]
    pub trait Traverse<'a> {
      ${traverseMethods}
    }
  `;
}
