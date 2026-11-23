// Looking through the parentheses an AST may carry, to the expression they wrap.

import { debugAssert } from "../asserts.ts";

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
 *
 * `nodeType` must be the `type` of `node`.
 *
 * @param node - Computed member object
 * @param nodeType - `node.type`
 */
export function isLetIdentifier(
  node: ESTree.Expression,
  nodeType: ESTree.Expression["type"],
): boolean {
  debugAssert(nodeType === node.type, "`nodeType` must equal `node.type`");

  for (;;) {
    switch (nodeType) {
      case "Identifier":
        debugAssert(node.type === nodeType);
        return node.name === "let";
      /* IF TS */
      case "TSAsExpression":
      case "TSInstantiationExpression":
      case "TSNonNullExpression":
      case "TSSatisfiesExpression":
      case "TSTypeAssertion":
      /* END_IF */
      case "ParenthesizedExpression":
        debugAssert(node.type === nodeType);
        node = node.expression;
        nodeType = node.type;
        break;
      default:
        return false;
    }
  }
}
