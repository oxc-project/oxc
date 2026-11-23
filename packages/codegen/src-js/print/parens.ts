// Looking through the parentheses an AST may carry, to the expression they wrap.

import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Look through any `ParenthesizedExpression` wrappers to the expression inside.
 *
 * Parsing with `preserveParens: false` produces none of these, but the printer accepts an AST
 * which has them, and decides parenthesisation from precedence rather than from what the source had.
 *
 * @param node - Expression
 */
export function withoutParens(node: ESTree.Expression): ESTree.Expression {
  while (node.type === "ParenthesizedExpression") {
    node = node.expression;
  }
  return node;
}

/**
 * Whether a computed member's object needs parentheses to avoid starting a `let` declaration.
 * @param node - Computed member object
 */
export function isLetIdentifier(node: ESTree.Expression): boolean {
  for (;;) {
    switch (node.type) {
      case "Identifier":
        return node.name === "let";
      /* IF TS */
      case "TSAsExpression":
      case "TSInstantiationExpression":
      case "TSNonNullExpression":
      case "TSSatisfiesExpression":
      case "TSTypeAssertion":
      /* END_IF */
      case "ParenthesizedExpression":
        node = node.expression;
        break;
      default:
        return false;
    }
  }
}
