// Binding patterns.

import { typeAssertIs } from "../asserts.ts";
import {
  CAT_CLOSE_BRACKET,
  CAT_IDENT,
  CAT_OTHER,
  CAT_QUESTION,
  write,
  writeWithMap,
  writeWithMapNoLast,
} from "./write.ts";
import { printExpression } from "./expression.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { printLiteral } from "./literal.ts";
import { CTX_NONE } from "./operators.ts";
import { PREC_COMMA } from "./precedence.ts";
import { printString } from "./string.ts";
import { printTypeAnnotation } from "./typescript.ts";

import type { State } from "../state.ts";
import type { LiteralExtras, UnknownNode } from "./types.ts";
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
 */
export function printBindingPattern(node: BindingPatternNode | UnknownNode, state: State): void {
  switch (node.type) {
    case "Identifier":
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, node.name, CAT_IDENT, node);
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
      writeWithMap(state, "...", CAT_OTHER, node);
      printBindingPattern(node.argument, state);
      if (TS && node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);
      break;
    default:
      throw new Error(`Unknown binding pattern type: ${node.type}`);
  }
}

/**
 * Print an object destructuring pattern, as in `const { a, b } = obj`.
 */
function printObjectBindingPattern(node: ESTree.ObjectPattern, state: State): void {
  const { properties } = node;
  const { length } = properties;

  if (length === 0) {
    writeWithMap(state, "{}", CAT_OTHER, node);
    return;
  }

  writeWithMap(state, "{ ", CAT_OTHER, node);

  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);
    const property = properties[i];
    if (property.type === "RestElement") {
      writeWithMap(state, "...", CAT_OTHER, property);
      printBindingPattern(property.argument, state);
    } else {
      printBindingProperty(property, state);
    }
  }

  write(state, " }", CAT_OTHER);
}

/**
 * Print one property of an object pattern, in shorthand where the key and the binding agree.
 */
function printBindingProperty(node: ESTree.BindingProperty, state: State): void {
  // Shorthand is re-derived from names (matching Oxc), not from the flag.
  const { key, value } = node;

  let shorthand = false;
  if (!node.computed && key.type === "Identifier") {
    if (value.type === "Identifier" && key.name === value.name) {
      shorthand = true;
    } else if (
      value.type === "AssignmentPattern" &&
      value.left.type === "Identifier" &&
      key.name === value.left.name
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
 */
export function printPropertyKey(key: ESTree.PropertyKey, state: State): void {
  switch (key.type) {
    case "Identifier":
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, key.name, CAT_IDENT, key);
      break;
    case "PrivateIdentifier":
      writeWithMapNoLast(state, "#", key);
      write(state, key.name, CAT_IDENT);
      break;
    case "Literal":
      typeAssertIs<LiteralExtras>(key);
      if (typeof key.value === "string") {
        printString(state, key.value, key);
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
 */
function printArrayBindingPattern(node: ESTree.ArrayPattern, state: State): void {
  // Oxc stores the rest element separately from the other elements.
  // A trailing comma is printed before it, and a space always precedes it.
  const { elements } = node;
  let { length } = elements;

  let rest: ESTree.BindingRestElement | ESTree.AssignmentTargetRest | null = null;
  if (length > 0) {
    const lastElement = elements[length - 1];
    if (lastElement != null && lastElement.type === "RestElement") {
      rest = lastElement;
      length--;
    }
  }

  writeWithMap(state, "[", CAT_OTHER, node);

  for (let i = 0; i < length; i++) {
    if (i !== 0) write(state, ", ", CAT_OTHER);

    const element = elements[i];
    if (element != null) printBindingPattern(element, state);

    if (i === length - 1 && (element == null || rest !== null)) write(state, ",", CAT_OTHER);
  }

  if (rest !== null) {
    write(state, " ", CAT_OTHER);
    printBindingPattern(rest, state);
  }

  write(state, "]", CAT_CLOSE_BRACKET);
}
