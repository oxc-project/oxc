// Classes.

import { typeAssertIs } from "../asserts.ts";
import { printPropertyKey } from "./binding_pattern.ts";
import {
  CAT_CLOSE_BRACKET,
  CAT_IDENT,
  CAT_OP_UN_NOT,
  CAT_OTHER,
  CAT_QUESTION,
  CAT_START_OF_STMT,
  debugAssertLastFresh,
  markWithMap,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapEnd,
  writeWithMapNoLast,
} from "./write.ts";
import { printExpression } from "./expression.ts";
import { printFunctionBody, printParenParams } from "./function.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { printIndent } from "./indent.ts";
import { CTX_NONE } from "./operators.ts";
import { PREC_CALL, PREC_COMMA, PREC_LOWEST, PREC_POSTFIX } from "./precedence.ts";
import { printStatement } from "./statement.ts";
import {
  printTSIndexSignature,
  printTypeAnnotation,
  printTypeArguments,
  printTypeParameters,
} from "./typescript.ts";

import type { State } from "../state.ts";
import type {
  AccessorPropertyNode,
  ClassBodyNode,
  MethodDefinitionNode,
  PropertyDefinitionNode,
} from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print a class declaration or expression, decorators and heritage clauses included.
 *
 * A class expression is parenthesized where the statement or an `export default` starts with it,
 * as `printFunction` does for functions.
 */
export function printClass(node: ESTree.Class, state: State): void {
  let wrap = false;
  if (node.type === "ClassExpression") {
    debugAssertLastFresh(state);
    // `CAT_START_OF_STMT` or `CAT_START_OF_DEFAULT_EXPORT`, which are adjacent - see `write.ts`
    wrap = (state.last | 1) === CAT_START_OF_STMT;
  }
  if (wrap) write(state, "(", CAT_OTHER);

  const { decorators } = node;
  if (decorators != null && decorators.length > 0) printDecorators(decorators, state);

  printSpaceBeforeIdentifier(state);

  const declare = TS && node.declare;
  const abstract = TS && node.abstract;

  // The node's mapping goes on whichever of these is written first
  if (declare) writeWithMap(state, "declare ", CAT_OTHER, node);
  if (abstract) {
    if (declare) {
      write(state, "abstract ", CAT_OTHER);
    } else {
      writeWithMap(state, "abstract ", CAT_OTHER, node);
    }
  }
  if (declare || abstract) {
    write(state, "class", CAT_IDENT);
  } else {
    writeWithMap(state, "class", CAT_IDENT, node);
  }

  if (node.id != null) {
    write(state, " ", CAT_OTHER);
    writeWithMap(state, node.id.name, CAT_IDENT, node.id);
  }

  if (TS) printTypeParameters(node.typeParameters, state);

  if (node.superClass != null) {
    write(state, " extends ", CAT_OTHER);
    printExpression(node.superClass, state, PREC_POSTFIX, CTX_NONE);
    if (TS) printTypeArguments(node.superTypeArguments, state);
  }

  const { implements: implementsClauses } = node;
  if (TS && implementsClauses != null && implementsClauses.length > 0) {
    write(state, " implements ", CAT_OTHER);

    const { length } = implementsClauses;
    for (let i = 0; i < length; i++) {
      if (i > 0) write(state, ", ", CAT_OTHER);
      const clause = implementsClauses[i];
      printExpression(clause.expression, state, PREC_CALL, CTX_NONE);
      printTypeArguments(clause.typeArguments, state);
    }
  }

  write(state, " ", CAT_OTHER);

  printClassBody(node.body, state);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Print a run of decorators separated by spaces, parenthesizing any whose expression needs it.
 */
export function printDecorators(decorators: ESTree.Decorator[], state: State): void {
  const { length } = decorators;
  for (let i = 0; i < length; i++) {
    const decorator = decorators[i];

    writeWithMap(state, "@", CAT_OTHER, decorator);

    const { expression } = decorator;
    const wrap = decoratorNeedsWrap(expression);
    if (wrap) write(state, "(", CAT_OTHER);
    printExpression(expression, state, PREC_LOWEST, CTX_NONE);
    if (wrap) write(state, ")", CAT_CLOSE_BRACKET);

    write(state, " ", CAT_OTHER);
  }
}

/**
 * Whether a decorator's expression has to be parenthesized.
 *
 * The grammar allows a name, a chain of plain member accesses, and a call on either, and nothing
 * else - so a computed access needs parentheses even though a static one does not.
 */
function decoratorNeedsWrap(expr: ESTree.Expression): boolean {
  for (;;) {
    switch (expr.type) {
      case "Identifier":
        return false;
      case "MemberExpression":
        // `@a.b` / `@this.b` / `@x!.y` need no parens; computed access does
        return expr.computed;
      case "CallExpression":
        expr = expr.callee;
        break;
      default:
        return true;
    }
  }
}

/**
 * Print the members of a class in braces, empty ones tight.
 */
function printClassBody(node: ClassBodyNode, state: State): void {
  const { body } = node;
  const { length } = body;
  if (length === 0) {
    writeWithMapNoLast(state, "{", node);
    writeWithMapEnd(state, "}", CAT_OTHER, node);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, node);

  state.indentLevel++;

  for (let i = 0; i < length; i++) {
    printIndent(state);

    const element = body[i];
    switch (element.type) {
      case "MethodDefinition":
      /* IF TS */
      case "TSAbstractMethodDefinition":
        /* END_IF */
        printMethodDefinition(element, state);
        write(state, "\n", CAT_OTHER);
        break;
      case "PropertyDefinition":
      /* IF TS */
      case "TSAbstractPropertyDefinition":
        /* END_IF */
        printPropertyDefinition(element, state);
        write(state, ";\n", CAT_OTHER);
        break;
      case "StaticBlock":
        printStaticBlock(element, state);
        write(state, "\n", CAT_OTHER);
        break;
      case "AccessorProperty":
      /* IF TS */
      case "TSAbstractAccessorProperty":
        /* END_IF */
        printAccessorProperty(element, state);
        write(state, ";\n", CAT_OTHER);
        break;
      /* IF TS */
      case "TSIndexSignature":
        printTSIndexSignature(element, state);
        write(state, ";\n", CAT_OTHER);
        break;
      /* END_IF */
      default:
        throw new Error(`Unknown class element type: ${element.type}`);
    }
  }

  state.indentLevel--;

  printIndent(state);
  writeWithMapEnd(state, "}", CAT_OTHER, node);
}

/**
 * Print a method, including getters, setters, constructors and their modifiers.
 */
function printMethodDefinition(node: MethodDefinitionNode, state: State): void {
  markWithMap(state, node);

  const { decorators } = node;
  if (decorators != null && decorators.length > 0) printDecorators(decorators, state);

  if (TS && node.accessibility != null) {
    printSpaceBeforeIdentifier(state);
    writeNoLast(state, node.accessibility);
    write(state, " ", CAT_OTHER);
  }

  if (TS && (node.type === "TSAbstractMethodDefinition" || node.abstract)) {
    printSpaceBeforeIdentifier(state);
    write(state, "abstract ", CAT_OTHER);
  }

  if (node.static) {
    printSpaceBeforeIdentifier(state);
    write(state, "static ", CAT_OTHER);
  }
  if (TS && node.override) {
    printSpaceBeforeIdentifier(state);
    write(state, "override ", CAT_OTHER);
  }

  const { kind } = node;
  if (kind === "get") {
    printSpaceBeforeIdentifier(state);
    write(state, "get ", CAT_OTHER);
  } else if (kind === "set") {
    printSpaceBeforeIdentifier(state);
    write(state, "set ", CAT_OTHER);
  }

  const func = node.value;
  if (func.async) {
    printSpaceBeforeIdentifier(state);
    write(state, "async ", CAT_OTHER);
  }
  if (func.generator) write(state, "*", CAT_OTHER);

  if (node.computed) {
    write(state, "[", CAT_OTHER);
    typeAssertIs<ESTree.Expression>(node.key);
    printExpression(node.key, state, PREC_COMMA, CTX_NONE);
    write(state, "]", CAT_CLOSE_BRACKET);
  } else {
    printPropertyKey(node.key, state);
  }

  if (TS && node.optional) write(state, "?", CAT_QUESTION);

  if (TS) printTypeParameters(func.typeParameters, state);

  printParenParams(func.params, state);

  if (TS && func.returnType != null) {
    printTypeAnnotation(func.returnType, state);
  }

  if (func.body != null) {
    write(state, " ", CAT_OTHER);
    printFunctionBody(func.body, state);
  } else {
    write(state, ";", CAT_OTHER);
  }
}

/**
 * Print a class field, with its modifiers and initializer.
 */
function printPropertyDefinition(node: PropertyDefinitionNode, state: State): void {
  markWithMap(state, node);

  const { decorators } = node;
  if (decorators != null && decorators.length > 0) printDecorators(decorators, state);

  if (TS && node.declare) {
    printSpaceBeforeIdentifier(state);
    write(state, "declare ", CAT_OTHER);
  }

  if (TS && node.accessibility != null) {
    printSpaceBeforeIdentifier(state);
    writeNoLast(state, node.accessibility);
    write(state, " ", CAT_OTHER);
  }

  if (TS && (node.type === "TSAbstractPropertyDefinition" || node.abstract)) {
    printSpaceBeforeIdentifier(state);
    write(state, "abstract ", CAT_OTHER);
  }

  if (node.static) {
    printSpaceBeforeIdentifier(state);
    write(state, "static ", CAT_OTHER);
  }
  if (TS && node.override) {
    printSpaceBeforeIdentifier(state);
    write(state, "override ", CAT_OTHER);
  }
  if (TS && node.readonly) {
    printSpaceBeforeIdentifier(state);
    write(state, "readonly ", CAT_OTHER);
  }

  if (node.computed) {
    write(state, "[", CAT_OTHER);
    typeAssertIs<ESTree.Expression>(node.key);
    printExpression(node.key, state, PREC_COMMA, CTX_NONE);
    write(state, "]", CAT_CLOSE_BRACKET);
  } else {
    printPropertyKey(node.key, state);
  }

  if (TS && node.optional) write(state, "?", CAT_QUESTION);
  if (TS && node.definite) write(state, "!", CAT_OP_UN_NOT);

  if (TS && node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);

  if (node.value != null) {
    write(state, " = ", CAT_OTHER);
    printExpression(node.value, state, PREC_COMMA, CTX_NONE);
  }
}

/**
 * Print a `static { … }` block.
 *
 * Its body has no directive prologue, unlike a function body, so a string statement at the top of
 * one needs no protection from being read back as a directive.
 */
function printStaticBlock(node: ESTree.StaticBlock, state: State): void {
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "static ", CAT_OTHER, node);

  const { body } = node;
  const { length } = body;
  if (length === 0) {
    writeWithMapNoLast(state, "{", node);
    writeWithMapEnd(state, "}", CAT_OTHER, node);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, node);

  state.indentLevel++;

  for (let i = 0; i < length; i++) {
    printStatement(body[i], state);
  }

  state.indentLevel--;

  printIndent(state);
  writeWithMapEnd(state, "}", CAT_OTHER, node);
}

/**
 * Print an `accessor` field.
 */
function printAccessorProperty(node: AccessorPropertyNode, state: State): void {
  markWithMap(state, node);

  const { decorators } = node;
  if (decorators != null && decorators.length > 0) printDecorators(decorators, state);

  if (TS && (node.type === "TSAbstractAccessorProperty" || node.abstract)) {
    printSpaceBeforeIdentifier(state);
    write(state, "abstract ", CAT_OTHER);
  }

  if (TS && node.accessibility != null) {
    printSpaceBeforeIdentifier(state);
    writeNoLast(state, node.accessibility);
    write(state, " ", CAT_OTHER);
  }

  if (node.static) {
    printSpaceBeforeIdentifier(state);
    write(state, "static ", CAT_OTHER);
  }
  if (TS && node.override) {
    printSpaceBeforeIdentifier(state);
    write(state, "override ", CAT_OTHER);
  }

  printSpaceBeforeIdentifier(state);
  write(state, "accessor", CAT_IDENT);

  if (node.computed) {
    write(state, " [", CAT_OTHER);
    typeAssertIs<ESTree.Expression>(node.key);
    printExpression(node.key, state, PREC_COMMA, CTX_NONE);
    write(state, "]", CAT_CLOSE_BRACKET);
  } else {
    write(state, " ", CAT_OTHER);
    printPropertyKey(node.key, state);
  }

  if (TS && node.definite) write(state, "!", CAT_OP_UN_NOT);
  if (TS && node.typeAnnotation != null) printTypeAnnotation(node.typeAnnotation, state);

  if (node.value != null) {
    write(state, " = ", CAT_OTHER);
    printExpression(node.value, state, PREC_COMMA, CTX_NONE);
  }
}
