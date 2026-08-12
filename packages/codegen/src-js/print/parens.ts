// Looking through the parentheses an AST may carry, to the expression they wrap.

import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Look through any `ParenthesizedExpression` wrappers to the expression inside.
 *
 * Parsing with `preserveParens: false` produces none of these, but the printer accepts an AST
 * which has them, and decides parenthesisation from precedence rather than from what the source had.
 */
export function withoutParens(node: ESTree.Expression): ESTree.Expression {
  while (node.type === "ParenthesizedExpression") {
    node = node.expression;
  }
  return node;
}
