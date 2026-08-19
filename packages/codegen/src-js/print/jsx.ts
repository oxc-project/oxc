// JSX.
// Port of `gen.rs` JSX section.

import {
  CAT_OTHER,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapEnd,
  writeWithMapNoLast,
} from "./write.ts";
import { printExpression } from "./expression.ts";
import { CTX_NONE } from "./operators.ts";
import { PREC_COMMA, PREC_LOWEST } from "./precedence.ts";
import { printTypeArguments } from "./typescript.ts";

import type { State } from "../state.ts";
import type { UnknownNode } from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print a JSX element, its attributes and its children.
 *
 * An element with no closing element is self-closing and gets ` />`, which is also the only place
 * a space is required before the slash.
 */
export function printJSXElement(node: ESTree.JSXElement, state: State): void {
  const { openingElement } = node;

  writeWithMapNoLast(state, "<", openingElement);
  printJSXElementName(openingElement.name, state);

  if (TS) printTypeArguments(openingElement.typeArguments, state);

  const { attributes } = openingElement;
  const { length } = attributes;
  for (let i = 0; i < length; i++) {
    writeNoLast(state, " ");
    const attribute = attributes[i];
    if (attribute.type === "JSXSpreadAttribute") {
      writeWithMap(state, "{...", CAT_OTHER, attribute);
      printExpression(attribute.argument, state, PREC_COMMA, CTX_NONE);
      writeWithMapEnd(state, "}", CAT_OTHER, attribute);
    } else {
      printJSXAttribute(attribute, state);
    }
  }

  const { closingElement } = node;
  if (closingElement == null) {
    write(state, " />", CAT_OTHER);
    return;
  }

  writeNoLast(state, ">");

  const { children } = node;
  const childCount = children.length;
  for (let i = 0; i < childCount; i++) {
    printJSXChild(children[i], state);
  }

  writeWithMapNoLast(state, "</", closingElement);
  printJSXElementName(closingElement.name, state);
  write(state, ">", CAT_OTHER);
}

/**
 * Print the name in an opening or closing tag - a name, a namespaced name, a member chain, or `this`.
 */
function printJSXElementName(
  node: ESTree.JSXElementName | ESTree.ThisExpression | UnknownNode,
  state: State,
): void {
  switch (node.type) {
    case "JSXIdentifier":
      writeWithMapNoLast(state, node.name, node);
      break;
    case "JSXMemberExpression":
      printJSXElementName(node.object, state);
      writeNoLast(state, ".");
      printJSXElementName(node.property, state);
      break;
    case "JSXNamespacedName":
      writeWithMapNoLast(state, node.namespace.name, node.namespace);
      writeNoLast(state, ":");
      writeWithMapNoLast(state, node.name.name, node.name);
      break;
    case "ThisExpression":
      writeWithMapNoLast(state, "this", node);
      break;
    default:
      throw new Error(`Unknown JSX name type: ${node.type}`);
  }
}

/**
 * Print one attribute, with its value where it has one - a bare attribute is `true`.
 */
function printJSXAttribute(node: ESTree.JSXAttribute, state: State): void {
  // Attribute names never update `last`.
  // Everything which can follow one - `=`, the next attribute's leading space, `>`, ` />` -
  // is punctuation needing no separation from what precedes it, and nothing between here
  // and the next real write reads `last`.
  const { name } = node;
  if (name.type === "JSXNamespacedName") {
    writeWithMapNoLast(state, name.namespace.name, name.namespace);
    writeNoLast(state, ":");
    writeWithMapNoLast(state, name.name.name, name.name);
  } else {
    writeWithMapNoLast(state, name.name, name);
  }

  const { value } = node;
  if (value != null) {
    writeNoLast(state, "=");
    printJSXAttributeValue(value, state);
  }
}

/**
 * Print an attribute's value.
 *
 * A string value is printed from its raw text, not through the string escaper, because JSX attribute strings
 * have no escape sequences - the quote is picked to suit the contents instead.
 */
function printJSXAttributeValue(node: ESTree.JSXAttributeValue | UnknownNode, state: State): void {
  switch (node.type) {
    case "Literal": {
      // JSX strings have no escape sequences and HTML entities are not decoded by parser at present.
      // Print the raw source text (matching Oxc), choosing only the quote.
      //
      // Once HTML entities are decoded (which they should be), this check stops being enough -
      // `<Foo bar="'&quot;" />` decodes to a value holding both quote characters, which no choice
      // of quote mark can contain, so the text will have to be escaped.
      // The cases in `test/print.test.ts` pin the current output, so they will fail the moment
      // the parser starts decoding, rather than the change slipping through here unnoticed.
      const { raw } = node;
      const text = raw != null ? raw.slice(1, -1) : String(node.value);

      const quote = text.includes('"') ? "'" : '"';
      writeNoLast(state, quote);
      writeNoLast(state, text);
      writeNoLast(state, quote);
      break;
    }
    case "JSXExpressionContainer":
      printJSXExpressionContainer(node, state);
      break;
    case "JSXElement":
      printJSXElement(node, state);
      break;
    case "JSXFragment":
      printJSXFragment(node, state);
      break;
    default:
      throw new Error(`Unknown JSX attribute value type: ${node.type}`);
  }
}

/**
 * Print a `{ … }` container, which may hold nothing but a comment.
 */
function printJSXExpressionContainer(node: ESTree.JSXExpressionContainer, state: State): void {
  write(state, "{", CAT_OTHER);

  if (node.expression.type !== "JSXEmptyExpression") {
    printExpression(node.expression, state, PREC_LOWEST, CTX_NONE);
  }

  // `}` needs no category. What follows it in either caller context (attribute value, child)
  // is punctuation or JSX text, none of which needs separating from it, and nothing reads
  // `last` before the next real write.
  writeNoLast(state, "}");
}

/**
 * Print a `<>...</>` fragment.
 */
export function printJSXFragment(node: ESTree.JSXFragment, state: State): void {
  writeWithMapNoLast(state, "<>", node.openingFragment);

  const { children } = node;
  const { length } = children;
  for (let i = 0; i < length; i++) {
    printJSXChild(children[i], state);
  }

  writeWithMap(state, "</>", CAT_OTHER, node.closingFragment);
}

/**
 * Print one child of an element or fragment.
 *
 * Text is written from its raw source, since whitespace between elements is significant
 * and any entities in it must survive untouched.
 */
function printJSXChild(node: ESTree.JSXChild | UnknownNode, state: State): void {
  switch (node.type) {
    case "JSXText":
      writeWithMapNoLast(state, node.raw != null ? node.raw : node.value, node);
      break;
    case "JSXExpressionContainer":
      printJSXExpressionContainer(node, state);
      break;
    case "JSXElement":
      printJSXElement(node, state);
      break;
    case "JSXFragment":
      printJSXFragment(node, state);
      break;
    case "JSXSpreadChild":
      write(state, "{...", CAT_OTHER);
      printExpression(node.expression, state, PREC_LOWEST, CTX_NONE);
      writeNoLast(state, "}");
      break;
    default:
      throw new Error(`Unknown JSX child type: ${node.type}`);
  }
}
