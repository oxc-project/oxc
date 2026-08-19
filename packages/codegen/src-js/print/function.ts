// Functions.

import { typeAssertIs } from "../asserts.ts";
import { printBindingPattern } from "./binding_pattern.ts";
import {
  CAT_CLOSE_BRACKET,
  CAT_IDENT,
  CAT_OTHER,
  CAT_START_OF_STMT,
  debugAssertLastFresh,
  markWithMapNoName,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapEnd,
  writeWithMapNoLast,
} from "./write.ts";
import { printDecorators } from "./class.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { printIndent } from "./indent.ts";
import { printDirectivesAndStatements } from "./statement.ts";
import { printTypeAnnotation, printTypeParameters } from "./typescript.ts";

import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print a function declaration or expression, from `async` through to the closing brace of its body.
 *
 * A function expression is parenthesized where the statement or an `export default` starts with it,
 * since it would otherwise be read as a declaration.
 */
export function printFunction(node: ESTree.Function, state: State): void {
  let wrap = false;
  if (node.type === "FunctionExpression") {
    debugAssertLastFresh(state);
    // `CAT_START_OF_STMT` or `CAT_START_OF_DEFAULT_EXPORT`, which are adjacent - see `write.ts`
    wrap = (state.last | 1) === CAT_START_OF_STMT;
  }

  if (wrap) write(state, "(", CAT_OTHER);

  printSpaceBeforeIdentifier(state);

  // The node's mapping goes on whichever of these is written first
  const declare = TS && node.declare;
  if (declare) {
    writeWithMap(state, "declare ", CAT_OTHER, node);
    write(state, node.async ? "async function" : "function", CAT_IDENT);
  } else {
    writeWithMap(state, node.async ? "async function" : "function", CAT_IDENT, node);
  }

  if (node.generator) write(state, "* ", CAT_OTHER);

  if (node.id != null) {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, node.id.name, CAT_IDENT, node.id);
  }

  if (TS) printTypeParameters(node.typeParameters, state);

  printParenParams(node.params, state);

  if (TS && node.returnType != null) printTypeAnnotation(node.returnType, state);

  if (node.body != null) {
    write(state, " ", CAT_OTHER);
    printFunctionBody(node.body, state);
  } else {
    write(state, ";", CAT_OTHER);
  }

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Print a parameter list in parentheses.
 */
export function printParenParams(params: ESTree.ParamPattern[], state: State): void {
  // `(params)`, as a single write when there are none
  if (params.length === 0) {
    write(state, "()", CAT_CLOSE_BRACKET);
    return;
  }

  write(state, "(", CAT_OTHER);
  printParams(params, state);
  write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Print the parameters themselves, without the parentheses around them.
 *
 * TypeScript parameter properties carry the modifiers which make them a property as well as a
 * parameter, and decorators can appear on either kind.
 */
function printParams(params: ESTree.ParamPattern[], state: State): void {
  const { length } = params;
  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);

    const param = params[i];

    // Oxc stores TypeScript's special `this` parameter separately from formal parameters
    // and prints it without a source mapping.
    if (TS && param.type === "Identifier" && param.name === "this") {
      write(state, "this", CAT_IDENT);
      if (param.typeAnnotation != null) printTypeAnnotation(param.typeAnnotation, state);
      continue;
    }

    const { decorators } = param;
    if (decorators != null && decorators.length > 0) {
      printDecorators(decorators, state);
    } else {
      markWithMapNoName(state, param);
    }

    if (TS && param.type === "TSParameterProperty") {
      if (param.accessibility != null) {
        printSpaceBeforeIdentifier(state);
        writeNoLast(state, param.accessibility);
        write(state, " ", CAT_OTHER);
      }

      if (param.override) {
        printSpaceBeforeIdentifier(state);
        write(state, "override ", CAT_OTHER);
      }
      if (param.readonly) {
        printSpaceBeforeIdentifier(state);
        write(state, "readonly ", CAT_OTHER);
      }

      printBindingPattern(param.parameter, state);
    } else {
      // The `TS &&` above defeats narrowing; `TSParameterProperty` was excluded by the check
      typeAssertIs<Exclude<typeof param, ESTree.TSParameterProperty>>(param);
      printBindingPattern(param, state);
    }
  }
}

/**
 * Print a function body in braces, empty ones tight.
 *
 * A function body has a directive prologue, so it goes through the same printer as a program does,
 * rather than through the block statement printer.
 */
export function printFunctionBody(body: ESTree.FunctionBody, state: State): void {
  // `body` is a BlockStatement holding directives + statements.
  const statements = body.body;
  if (statements.length === 0) {
    writeWithMapNoLast(state, "{", body);
    writeWithMapEnd(state, "}", CAT_OTHER, body);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, body);
  state.indentLevel++;
  printDirectivesAndStatements(statements, state);
  state.indentLevel--;
  printIndent(state);

  writeWithMapEnd(state, "}", CAT_OTHER, body);
}

/**
 * Print an arrow function's parameter list along with its `=>`.
 *
 * Oxc keeps the parentheses even around a lone parameter, so there is no single-parameter form.
 */
export function printParenParamsArrow(params: ESTree.ParamPattern[], state: State): void {
  // `(params) => `, as a single write when there are none
  if (params.length === 0) {
    write(state, "() => ", CAT_OTHER);
    return;
  }

  write(state, "(", CAT_OTHER);
  printParams(params, state);
  write(state, ") => ", CAT_OTHER);
}
