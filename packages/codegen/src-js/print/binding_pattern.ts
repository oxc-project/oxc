// Binding patterns.

import { typeAssertIs } from "../asserts.ts";
import { CAT_CLOSE_BRACKET, CAT_OTHER, CAT_QUESTION } from "./categories.ts";
import {
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapNamed,
  writeWithMapNamedPrivate,
} from "./write.ts";
import { printExpression } from "./expression.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { printLiteral } from "./literal.ts";
import { CTX_NONE } from "./operators.ts";
import { PREC_COMMA } from "./precedence.ts";
import { printString } from "./string.ts";
import { printTypeAnnotation } from "./typescript.ts";
import { throwUnknownNode } from "./types.ts";

import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * A binding pattern, or one of the nodes which can stand in for one.
 *
 * `RestElement` appears in object and array patterns.
 */
type BindingPatternNode =
  | ESTree.BindingPattern
  | ESTree.BindingRestElement
  | ESTree.FormalParameterRest;

/**
 * Print anything which can be bound to - a name, a destructuring pattern, or one carrying a default.
 *
 * @param node - Binding pattern
 * @param state - Printer state
 */
export function printBindingPattern(node: BindingPatternNode, state: State): void {
  switch (node.type) {
    case "Identifier":
      printSpaceBeforeIdentifier(state);
      writeWithMapNamed(state, node.name, node.start, node.end, node);
      if (TS && node.optional) write(state, "?", CAT_QUESTION);
      if (TS && node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);
      break;
    case "ObjectPattern":
      printObjectBindingPattern(node, state);
      if (TS && node.optional) write(state, "?", CAT_QUESTION);
      if (TS && node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);
      break;
    case "ArrayPattern":
      printArrayBindingPattern(node, state);
      if (TS && node.optional) write(state, "?", CAT_QUESTION);
      if (TS && node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);
      break;
    case "AssignmentPattern":
      printBindingPattern(node.left, state);
      write(state, " = ", CAT_OTHER);
      printExpression(node.right, state, PREC_COMMA, CTX_NONE);
      break;
    case "RestElement":
      writeWithMap(state, "...", CAT_OTHER, node.start, node.end, node);
      printBindingPattern(node.argument, state);
      if (TS && node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);
      break;
    default:
      throwUnknownNode(node, "binding pattern");
  }
}

/**
 * Print an object destructuring pattern, as in `const { a, b } = obj`.
 *
 * @param node - Object pattern
 * @param state - Printer state
 */
function printObjectBindingPattern(node: ESTree.ObjectPattern, state: State): void {
  const { properties } = node;
  const { length } = properties;

  if (length === 0) {
    writeWithMap(state, "{}", CAT_OTHER, node.start, node.end, node);
    return;
  }

  writeWithMap(state, "{ ", CAT_OTHER, node.start, node.end, node);

  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);
    const property = properties[i];
    if (property.type === "RestElement") {
      writeWithMap(state, "...", CAT_OTHER, property.start, property.end, property);
      printBindingPattern(property.argument, state);
    } else {
      printBindingProperty(property, state);
    }
  }

  write(state, " }", CAT_OTHER);
}

/**
 * Print one property of an object pattern, in shorthand where the key and the binding agree.
 *
 * @param node - Binding property
 * @param state - Printer state
 */
function printBindingProperty(node: ESTree.BindingProperty, state: State): void {
  // Shorthand is re-derived from names (matching Oxc), not from the flag.
  const { key, value } = node;

  let shorthand = false;
  if (!node.computed && key.type === "Identifier") {
    if (value.type === "Identifier" && key.name === value.name) {
      shorthand = true;
    } else if (
      value.type === "AssignmentPattern"
      && value.left.type === "Identifier"
      && key.name === value.left.name
    ) {
      shorthand = true;
    }
  }

  if (!shorthand) {
    if (node.computed) {
      write(state, "[", CAT_OTHER);
      typeAssertIs<ESTree.Expression>(key);
      printExpression(key, state, PREC_COMMA, CTX_NONE);
      write(state, "]", CAT_CLOSE_BRACKET);
    } else {
      printPropertyKey(key, state);
    }
    write(state, ": ", CAT_OTHER);
  }

  printBindingPattern(value, state);
}

/**
 * Print a property key which is not computed - a name, a private name, a string or a number.
 *
 * The caller decides between this and printing the key as an expression in brackets,
 * because whether a key must be computed depends on what it is, not on how the source wrote it.
 *
 * @param key - Property key
 * @param state - Printer state
 */
export function printPropertyKey(key: ESTree.PropertyKey, state: State): void {
  switch (key.type) {
    case "Identifier":
      printSpaceBeforeIdentifier(state);
      writeWithMapNamed(state, key.name, key.start, key.end, key);
      break;
    case "PrivateIdentifier":
      writeWithMapNamedPrivate(state, key.name, key.start, key.end, key);
      break;
    case "Literal":
      if (typeof key.value === "string") {
        printString(state, key.value, key.start, key.end, key);
      } else {
        printLiteral(key, state, PREC_COMMA, CTX_NONE);
      }
      break;
    default:
      printExpression(key, state, PREC_COMMA, CTX_NONE);
  }
}

/**
 * Print an array destructuring pattern, holes and rest element included.
 *
 * @param node - Array pattern
 * @param state - Printer state
 */
function printArrayBindingPattern(node: ESTree.ArrayPattern, state: State): void {
  writeWithMap(state, "[", CAT_OTHER, node.start, node.end, node);

  const { elements } = node,
    { length } = elements;

  if (length !== 0) {
    const lastElement = elements[length - 1];
    if (lastElement !== null && lastElement.type === "RestElement") {
      // Print elements before the `RestElement`
      const restIndex = length - 1;
      for (let i = 0; i < restIndex; i++) {
        const element = elements[i];
        if (element !== null) printBindingPattern(element, state);
        write(state, ", ", CAT_OTHER);
      }

      // Print `RestElement`
      writeWithMap(state, "...", CAT_OTHER, lastElement.start, lastElement.end, lastElement);
      printBindingPattern(lastElement.argument, state);
      if (TS) {
        const { typeAnnotation } = lastElement;
        if (typeAnnotation != null) printTypeAnnotation(typeAnnotation, state);
      }
    } else {
      // No `RestElement` - print all elements
      const firstElement = elements[0];
      if (firstElement !== null) printBindingPattern(firstElement, state);

      for (let i = 1; i < length; i++) {
        write(state, ", ", CAT_OTHER);

        const element = elements[i];
        if (element !== null) printBindingPattern(element, state);
      }

      // A trailing hole needs a comma after it
      if (lastElement === null) writeNoLast(state, ",");
    }
  }

  write(state, "]", CAT_CLOSE_BRACKET);
}
