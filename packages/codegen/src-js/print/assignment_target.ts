// Assignment targets.

import { typeAssertIs } from "../asserts.ts";
import { printPropertyKey } from "./binding_pattern.ts";
import { CAT_CLOSE_BRACKET, CAT_OTHER } from "./categories.ts";
import { write, writeNoLast, writeWithMap, writeWithMapNamed } from "./write.ts";
import { printExpression, printMemberExpression } from "./expression.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { CTX_NONE } from "./operators.ts";
import { PREC_COMMA, PREC_COMPARE, PREC_PREFIX } from "./precedence.ts";

import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print the left hand side of an assignment.
 *
 * Destructuring targets look like object and array literals but are a separate set of node types,
 * with their own rules - which is why they are printed here rather than by the expression printer.
 *
 * @param node - Assignment target
 * @param state - Printer state
 */
export function printAssignmentTarget(node: ESTree.AssignmentTarget, state: State): void {
  switch (node.type) {
    // A simple target is only ever a name, a member access, or (in TS) one of the assertion expressions below.
    // For speed, the two common ones are printed here instead of through the expression printer's own dispatch.
    case "Identifier":
      printSpaceBeforeIdentifier(state);
      writeWithMapNamed(state, node.name, node.start, node.end, node);
      break;
    case "MemberExpression":
      printMemberExpression(node, state, CTX_NONE);
      break;
    case "ObjectPattern":
      printObjectAssignmentTarget(node, state);
      break;
    case "ArrayPattern":
      printArrayAssignmentTarget(node, state);
      break;
    /* IF TS */
    case "TSAsExpression":
    case "TSSatisfiesExpression":
      // `(x as T) = y` - the target must be parenthesized
      printExpression(node, state, PREC_COMPARE, CTX_NONE);
      break;
    case "TSTypeAssertion":
      printExpression(node, state, PREC_PREFIX, CTX_NONE);
      break;
    /* END_IF */
    default:
      printExpression(node, state, PREC_COMMA, CTX_NONE);
  }
}

/**
 * Print an object destructuring target, as in `({ a, b: c } = obj)`.
 *
 * @param node - Object assignment target
 * @param state - Printer state
 */
function printObjectAssignmentTarget(node: ESTree.ObjectAssignmentTarget, state: State): void {
  writeWithMap(state, "{", CAT_OTHER, node.start, node.end, node);

  const { properties } = node;
  const { length } = properties;
  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);

    const property = properties[i];
    if (property.type === "RestElement") {
      writeWithMap(state, "...", CAT_OTHER, property.start, property.end, property);
      printAssignmentTarget(property.argument, state);
    } else {
      printAssignmentTargetProperty(property, state);
    }
  }

  write(state, "}", CAT_OTHER);
}

/**
 * Print one property of an object destructuring target, in shorthand where the AST says so.
 *
 * @param node - Assignment target property
 * @param state - Printer state
 */
function printAssignmentTargetProperty(node: ESTree.AssignmentTargetProperty, state: State): void {
  // Shorthand from the AST flag (assignment targets are not re-derived).
  if (node.shorthand) {
    // `{ a }` or `{ a = 1 }`
    const { value } = node;
    if (value.type === "AssignmentPattern") {
      typeAssertIs<ESTree.IdentifierReference>(value.left);
      writeWithMapNamed(state, value.left.name, value.left.start, value.left.end, value.left);
      write(state, " = ", CAT_OTHER);
      printExpression(value.right, state, PREC_COMMA, CTX_NONE);
    } else {
      writeWithMapNamed(state, value.name, value.start, value.end, value);
    }
  } else {
    const { key } = node;

    if (node.computed) {
      write(state, "[", CAT_OTHER);
      typeAssertIs<ESTree.Expression>(key);
      printExpression(key, state, PREC_COMMA, CTX_NONE);
      write(state, "]", CAT_CLOSE_BRACKET);
    } else {
      printPropertyKey(key, state);
    }

    write(state, ": ", CAT_OTHER);
    printAssignmentTargetMaybeDefault(node.value, state);
  }
}

/**
 * Print a destructuring target which may carry a default, as the `a = 1` in `({ a = 1 } = obj)`.
 *
 * @param node - Assignment target, possibly with a default
 * @param state - Printer state
 */
function printAssignmentTargetMaybeDefault(
  node: ESTree.AssignmentTargetMaybeDefault,
  state: State,
): void {
  if (node.type === "AssignmentPattern") {
    printAssignmentTarget(node.left, state);
    write(state, " = ", CAT_OTHER);
    printExpression(node.right, state, PREC_COMMA, CTX_NONE);
  } else {
    printAssignmentTarget(node, state);
  }
}

/**
 * Print an array destructuring target, holes and rest element included.
 *
 * @param node - Array assignment target
 * @param state - Printer state
 */
function printArrayAssignmentTarget(node: ESTree.ArrayAssignmentTarget, state: State): void {
  writeWithMap(state, "[", CAT_OTHER, node.start, node.end, node);

  const { elements } = node,
    { length } = elements;

  if (length !== 0) {
    const lastElement = elements[length - 1];
    if (lastElement !== null && lastElement.type === "RestElement") {
      // Print elements before the `RestElement`
      const restIndex = length - 1;
      for (let i = 0; i < restIndex; i++) {
        // `RestElement` can only be last, and this loop stops before it
        const element = elements[i] as ESTree.AssignmentTargetMaybeDefault | null;
        if (element !== null) printAssignmentTargetMaybeDefault(element, state);
        write(state, ", ", CAT_OTHER);
      }

      // Print `RestElement`
      writeWithMap(state, "...", CAT_OTHER, lastElement.start, lastElement.end, lastElement);
      printAssignmentTarget(lastElement.argument, state);
    } else {
      // `RestElement` can only be last, and the last element is not one
      typeAssertIs<(ESTree.AssignmentTargetMaybeDefault | null)[]>(elements);

      const firstElement = elements[0];
      if (firstElement !== null) printAssignmentTargetMaybeDefault(firstElement, state);

      for (let i = 1; i < length; i++) {
        write(state, ", ", CAT_OTHER);

        const element = elements[i];
        if (element !== null) printAssignmentTargetMaybeDefault(element, state);
      }

      // A trailing hole needs a comma after it
      if (lastElement === null) writeNoLast(state, ",");
    }
  }

  write(state, "]", CAT_CLOSE_BRACKET);
}
