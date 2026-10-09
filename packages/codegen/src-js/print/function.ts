// Functions.

import {
  startNodeComments,
  finishNodeComments,
  hasInsideComments,
  printInsideComments,
  printContainerComments,
  printDeferredLeadingComments,
} from "./comments.ts";
import { typeAssertIs } from "../asserts.ts";
import { printBindingPattern } from "./binding_pattern.ts";
import { CAT_CLOSE_BRACKET, CAT_IDENT, CAT_OTHER, CAT_START_OF_STMT } from "./categories.ts";
import {
  debugAssertLastFresh,
  markMapStart,
  write,
  writeIdent,
  writeNoLast,
  writeWithMap,
  writeWithMapEnd,
  writeWithMapNamed,
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
 * The keywords a named function begins with, indexed as described in `printFunction`.
 *
 * Each ends with a space, which separates it from the name.
 * After `*` the space is not needed, but matches `oxc_codegen`'s style.
 *
 * Only TS builds include the `declare` forms.
 */
const NAMED_FUNCTION_PREFIXES = TS
  ? [
      "function ",
      "async function ",
      "function* ",
      "async function* ",
      "declare function ",
      "declare async function ",
      "declare function* ",
      "declare async function* ",
    ]
  : ["function ", "async function ", "function* ", "async function* "];

/**
 * The keywords an anonymous function begins with, indexed as described in `printFunction`.
 *
 * No trailing space is needed. The generator forms have one after `*` anyway, to match `oxc_codegen`'s style.
 *
 * Only TS builds include the `declare` forms.
 */
const ANONYMOUS_FUNCTION_PREFIXES = TS
  ? [
      "function",
      "async function",
      "function* ",
      "async function* ",
      "declare function",
      "declare async function",
      "declare function* ",
      "declare async function* ",
    ]
  : ["function", "async function", "function* ", "async function* "];

/**
 * Print a function declaration or expression, from `async` through to the closing brace of its body.
 *
 * A function expression is parenthesized where the statement or an `export default` starts with it,
 * since it would otherwise be read as a declaration.
 */
export function printFunction(node: ESTree.Function, state: State): void {
  const commentOwner = COMMENTS && startNodeComments(node, state);
  let wrap = false;
  if (node.type === "FunctionExpression") {
    debugAssertLastFresh(state);
    // `CAT_START_OF_STMT` or `CAT_START_OF_DEFAULT_EXPORT`, which are adjacent - see `categories.ts`
    wrap = (state.last | 1) === CAT_START_OF_STMT;
  }

  if (wrap) write(state, "(", CAT_OTHER);

  if (COMMENTS) printDeferredLeadingComments(node, state);

  printSpaceBeforeIdentifier(state);

  // The node's mapping goes on whatever keyword begins the function, which is written as a single string.
  //
  // The string comes from a table, rather than from testing each flag in turn.
  // Real code mixes `async` and plain functions freely, so a branch on `async` would often be mispredicted.
  // Bit 0 of the index is `async`, bit 1 is generator, and bit 2 is `declare` (TS builds only).
  //
  // V8 compiles each `flag === true` to a single compare against the `true` object, with no branch.
  // Any value other than `true` (e.g. `null` or `undefined`) counts as `false`.
  const generator = node.generator === true;
  const prefixIndex = TS
    ? ((node.async === true) as unknown as number)
      | (((generator === true) as unknown as number) << 1)
      | (((node.declare === true) as unknown as number) << 2)
    : ((node.async === true) as unknown as number)
      | (((generator === true) as unknown as number) << 1);

  const { id } = node;
  if (id != null) {
    writeWithMapNoLast(state, NAMED_FUNCTION_PREFIXES[prefixIndex], node.start, node.end, node);
    const nameCommentOwner = COMMENTS && startNodeComments(id, state);
    writeWithMapNamed(state, id.name, id.start, id.end, id);
    if (nameCommentOwner) finishNodeComments(id, state);
  } else {
    writeWithMap(
      state,
      ANONYMOUS_FUNCTION_PREFIXES[prefixIndex],
      // Only the generator prefixes end with a space
      generator === true ? CAT_OTHER : CAT_IDENT,
      node.start,
      node.end,
      node,
    );
  }

  if (TS) printTypeParameters(node.typeParameters, state);

  printParenParams(node.params, state, COMMENTS ? node : undefined);

  if (TS && node.returnType != null) printTypeAnnotation(node.returnType, state);

  if (node.body != null) {
    write(state, " ", CAT_OTHER);
    printFunctionBody(node.body, state);
  } else {
    write(state, ";", CAT_OTHER);
  }

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);

  if (commentOwner) finishNodeComments(node, state);
}

/**
 * Print a parameter list in parentheses.
 */
export function printParenParams(
  params: ESTree.ParamPattern[],
  state: State,
  owner?: ESTree.Node,
): void {
  // `(params)`, as a single write when there are none
  if (params.length === 0 && (!COMMENTS || !hasInsideComments(owner, state, "FormalParameters"))) {
    write(state, "()", CAT_CLOSE_BRACKET);
    return;
  }

  if (COMMENTS && owner != null) {
    printContainerComments(owner, state, "FormalParameters", "leading");
  }
  write(state, "(", CAT_OTHER);
  printParams(params, state);
  if (COMMENTS) printInsideComments(owner, state, "FormalParameters");
  write(state, ")", CAT_CLOSE_BRACKET);
  if (COMMENTS && owner != null) {
    printContainerComments(owner, state, "FormalParameters", "trailing");
  }
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
    const commentOwner = COMMENTS && startNodeComments(param, state);
    const container = param.type === "RestElement" ? "FormalParameterRest" : "FormalParameter";
    if (COMMENTS) printContainerComments(param, state, container, "leading");

    // Oxc stores TypeScript's special `this` parameter separately from formal parameters
    // and prints it without a source mapping.
    if (TS && param.type === "Identifier" && param.name === "this") {
      writeIdent(state, "this");
      if (param.typeAnnotation != null) printTypeAnnotation(param.typeAnnotation, state);
      continue;
    }

    const { decorators } = param;
    if (decorators != null && decorators.length > 0) {
      printDecorators(decorators, state);
    } else {
      markMapStart(state, param.start, param.end, param);
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
    if (COMMENTS) {
      printInsideComments(param, state, container);
      printContainerComments(param, state, container, "trailing");
    }
    if (commentOwner) finishNodeComments(param, state);
  }
}

/**
 * Print a function body in braces, empty ones tight.
 *
 * A function body has a directive prologue, so it goes through the same printer as a program does,
 * rather than through the block statement printer.
 */
export function printFunctionBody(body: ESTree.FunctionBody, state: State): void {
  const commentOwner = COMMENTS && startNodeComments(body, state);
  // `body` is a BlockStatement holding directives + statements.
  const statements = body.body;
  if (statements.length === 0 && (!COMMENTS || !hasInsideComments(body, state))) {
    writeWithMapNoLast(state, "{", body.start, body.end, body);
    writeWithMapEnd(state, "}", CAT_OTHER, body.start, body.end, body);
    if (commentOwner) finishNodeComments(body, state);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, body.start, body.end, body);
  state.indentLevel++;
  printDirectivesAndStatements(statements, state);
  if (COMMENTS) {
    printInsideComments(body, state);
    if (state.commentLastChar !== "\n") write(state, "\n", CAT_OTHER);
  }
  state.indentLevel--;
  printIndent(state);

  writeWithMapEnd(state, "}", CAT_OTHER, body.start, body.end, body);

  if (commentOwner) finishNodeComments(body, state);
}

/**
 * Print an arrow function's parameter list along with its `=>`.
 *
 * Oxc keeps the parentheses even around a lone parameter, so there is no single-parameter form.
 */
export function printParenParamsArrow(
  params: ESTree.ParamPattern[],
  state: State,
  owner?: ESTree.Node,
): void {
  // `(params) => `, as a single write when there are none
  if (params.length === 0 && (!COMMENTS || !hasInsideComments(owner, state, "FormalParameters"))) {
    write(state, "() => ", CAT_OTHER);
    return;
  }

  if (COMMENTS && owner != null) {
    printContainerComments(owner, state, "FormalParameters", "leading");
  }
  write(state, "(", CAT_OTHER);
  printParams(params, state);
  if (COMMENTS) printInsideComments(owner, state, "FormalParameters");
  write(state, ")", CAT_CLOSE_BRACKET);
  if (COMMENTS && owner != null) {
    printContainerComments(owner, state, "FormalParameters", "trailing");
  }
  write(state, " => ", CAT_OTHER);
}
