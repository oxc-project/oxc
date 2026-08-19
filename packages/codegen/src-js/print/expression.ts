// Expressions.

import { typeAssertIs } from "../asserts.ts";
import { printAssignmentTarget } from "./assignment_target.ts";
import { printBinaryish } from "./binary.ts";
import { printPropertyKey } from "./binding_pattern.ts";
import {
  CAT_CLOSE_BRACKET,
  CAT_IDENT,
  CAT_INT_DIGIT,
  CAT_LT,
  CAT_OP_UN_NOT,
  CAT_OP_UN_NOT_AFTER_LT,
  CAT_OTHER,
  CAT_QUESTION,
  CAT_START_OF_ARROW_EXPR,
  CAT_START_OF_STMT,
} from "./write.ts";
import { printClass } from "./class.ts";
import {
  printFunction,
  printFunctionBody,
  printParenParams,
  printParenParamsArrow,
} from "./function.ts";
import { printSpaceBeforeIdentifier, printSpaceBeforeOperator } from "./space.ts";
import { printIndent } from "./indent.ts";
import { printJSXElement, printJSXFragment } from "./jsx.ts";
import { printLiteral } from "./literal.ts";
import {
  CTX_FORBID_CALL,
  CTX_FORBID_IN,
  CTX_NONE,
  PADDED_ASSIGN_OPERATORS,
  unaryOperatorCode,
  updateOperatorCode,
} from "./operators.ts";
import { withoutParens } from "./parens.ts";
import {
  PREC_ASSIGN,
  PREC_CALL,
  PREC_COMMA,
  PREC_COMPARE,
  PREC_CONDITIONAL,
  PREC_EQUALS,
  PREC_EXPONENTIATION,
  PREC_LOWEST,
  PREC_NEW,
  PREC_POSTFIX,
  PREC_PREFIX,
  PREC_YIELD,
} from "./precedence.ts";
import {
  debugAssertLastFresh,
  markWithMap,
  markWithMapAfter,
  markWithMapAtStartOffset,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapEnd,
  writeWithMapNoLast,
} from "./write.ts";
import { escapeScriptCloseTag } from "./string.ts";
import {
  printTSAsOrSatisfiesExpression,
  printTSTypeAssertion,
  printTypeAnnotation,
  printTypeArguments,
  printTypeParameters,
} from "./typescript.ts";

import type { State } from "../state.ts";
import type { UnknownNode } from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Dispatch point for expressions. The switch writes the one-token constructs inline
 * and hands everything else to the printer for that node type, throwing on a type it does not know.
 *
 * Parens are not preserved from the input - every printer re-derives its own from `precedence`,
 * so a `ParenthesizedExpression` mostly prints straight through.
 *
 * @param precedence - Precedence of the position this expression sits in.
 *   A printer compares its own precedence against it and parenthesizes itself when it would otherwise bind too loosely.
 * @param ctx - `CTX_FORBID_IN` / `CTX_FORBID_CALL` flags describing the enclosing position, so that an `in` operator
 *   inside a `for` head, or a call under a `new`, knows to parenthesize itself.
 */
export function printExpression(
  node: ESTree.Expression | UnknownNode,
  state: State,
  precedence: number,
  ctx: number,
): void {
  // Arms are ordered roughly in order of most common nodes.
  // V8 turns this into (essentially) as chain of `if ... else if ... else if...`,
  // so making common nodes short-circuit early is a large perf boost.
  switch (node.type) {
    case "Identifier":
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, node.name, CAT_IDENT, node);
      break;
    case "MemberExpression":
      printMemberExpression(node, state, ctx);
      break;
    case "CallExpression":
      printCallExpression(node, state, precedence, ctx);
      break;
    case "Literal":
      printLiteral(node, state, precedence, ctx);
      break;
    case "BinaryExpression":
      if (node.left.type === "PrivateIdentifier") {
        typeAssertIs<ESTree.PrivateInExpression>(node);
        printPrivateInExpression(node, state, precedence);
      } else {
        typeAssertIs<ESTree.BinaryExpression>(node);
        printBinaryish(node, state, precedence, ctx);
      }
      break;
    case "LogicalExpression":
      printBinaryish(node, state, precedence, ctx);
      break;
    case "ObjectExpression":
      printObjectExpression(node, state);
      break;
    case "ArrayExpression":
      printArrayExpression(node, state);
      break;
    case "AssignmentExpression":
      printAssignmentExpression(node, state, precedence, ctx);
      break;
    case "UpdateExpression":
      printUpdateExpression(node, state, precedence, ctx);
      break;
    case "UnaryExpression":
      printUnaryExpression(node, state, precedence, ctx);
      break;
    case "ConditionalExpression":
      printConditionalExpression(node, state, precedence, ctx);
      break;
    case "SequenceExpression":
      printSequenceExpression(node, state, precedence, ctx);
      break;
    case "ArrowFunctionExpression":
      printArrowFunctionExpression(node, state, precedence, ctx);
      break;
    case "FunctionExpression":
      printFunction(node, state);
      break;
    case "ThisExpression":
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, "this", CAT_IDENT, node);
      break;
    case "Super":
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, "super", CAT_IDENT, node);
      break;
    case "NewExpression":
      printNewExpression(node, state, precedence);
      break;
    case "TemplateLiteral":
      printTemplateLiteral(node, state);
      break;
    case "TaggedTemplateExpression":
      markWithMap(state, node);
      printExpression(node.tag, state, PREC_POSTFIX, ctx & CTX_FORBID_CALL);
      if (TS) printTypeArguments(node.typeArguments, state);
      printTemplateLiteral(node.quasi, state);
      break;
    case "ClassExpression":
      printClass(node, state);
      break;
    case "AwaitExpression":
      printAwaitExpression(node, state, precedence, ctx);
      break;
    case "YieldExpression":
      printYieldExpression(node, state, precedence);
      break;
    case "ImportExpression":
      printImportExpression(node, state, precedence, ctx);
      break;
    case "MetaProperty":
      printSpaceBeforeIdentifier(state);
      writeWithMapNoLast(state, node.meta.name, node);
      writeNoLast(state, ".");
      write(state, node.property.name, CAT_IDENT);
      break;
    case "ChainExpression":
      printChainExpression(node, state, precedence, ctx);
      break;
    case "ParenthesizedExpression": {
      // Parens around function/arrow expressions are preserved (Oxc `pife`)
      const { expression } = node;
      const inner = withoutParens(expression);
      if (inner.type === "FunctionExpression" || inner.type === "ArrowFunctionExpression") {
        write(state, "(", CAT_OTHER);
        printExpression(inner, state, PREC_LOWEST, CTX_NONE);
        write(state, ")", CAT_CLOSE_BRACKET);
        if (SOURCEMAPS && precedence === PREC_POSTFIX) {
          markWithMapAfter(state, inner);
          const wrappers: ESTree.ParenthesizedExpression[] = [];
          let wrapper = expression;
          while (wrapper.type === "ParenthesizedExpression") {
            wrappers.push(wrapper);
            wrapper = wrapper.expression;
          }
          for (let index = wrappers.length - 1; index >= 0; index--) {
            markWithMapAfter(state, wrappers[index]);
          }
        }
      } else {
        printExpression(expression, state, precedence, ctx);
      }
      break;
    }
    case "JSXElement":
      printJSXElement(node, state);
      break;
    case "JSXFragment":
      printJSXFragment(node, state);
      break;
    /* IF TS */
    case "TSAsExpression":
    case "TSSatisfiesExpression":
      printTSAsOrSatisfiesExpression(node, state, precedence, ctx);
      break;
    case "TSNonNullExpression":
      printExpression(node.expression, state, PREC_POSTFIX, ctx);
      write(state, "!", CAT_OP_UN_NOT);
      break;
    case "TSInstantiationExpression":
      printExpression(node.expression, state, PREC_PREFIX, ctx);
      printTypeArguments(node.typeArguments, state);
      break;
    case "TSTypeAssertion":
      printTSTypeAssertion(node, state, precedence, ctx);
      break;
    /* END_IF */
    default:
      throw new Error(`Unknown expression type: ${node.type}`);
  }

  // Rust adds an exclusive-end mapping after a postfix operand which ended in `)` or `]`,
  // so the punctuation of a following call/member/tag chain resolves to the operand,
  // rather than one character to its left
  debugAssertLastFresh(state);
  if (SOURCEMAPS && precedence === PREC_POSTFIX && state.last === CAT_CLOSE_BRACKET) {
    markWithMapAfter(state, node);
  }
}

/**
 * The object is printed at `PREC_POSTFIX` and only `CTX_FORBID_CALL` is carried into it, so a member chain
 * beneath a `new` keeps that restriction while a computed property, printed from `PREC_LOWEST`, starts clean.
 *
 * Two shapes need more than precedence to stay valid:
 * 1. A bare `let` as the object.
 * 2. Member access straight onto an integer literal.
 */
export function printMemberExpression(
  node: ESTree.MemberExpression,
  state: State,
  ctx: number,
): void {
  const { object } = node;
  if (node.computed) {
    // `(let)[0]` - a bare `let` object must be wrapped
    const inner = withoutParens(object);
    const wrap = inner.type === "Identifier" && inner.name === "let";

    if (wrap) write(state, "(", CAT_OTHER);
    printExpression(object, state, PREC_POSTFIX, ctx & CTX_FORBID_CALL);
    if (wrap) write(state, ")", CAT_CLOSE_BRACKET);

    if (node.optional) write(state, "?.", CAT_OTHER);

    write(state, "[", CAT_OTHER);
    printExpression(node.property, state, PREC_LOWEST, CTX_NONE);
    write(state, "]", CAT_CLOSE_BRACKET);
  } else {
    printExpression(object, state, PREC_POSTFIX, ctx & CTX_FORBID_CALL);

    if (node.optional) {
      write(state, "?", CAT_QUESTION);
    } else {
      debugAssertLastFresh(state);
      // `0.toExponential()` is invalid; `0 .toExponential()` is valid
      if (state.last === CAT_INT_DIGIT) write(state, " ", CAT_OTHER);
    }

    write(state, ".", CAT_OTHER);

    const { property } = node;
    if (property.type === "PrivateIdentifier") {
      writeWithMapNoLast(state, "#", property);
      write(state, property.name, CAT_IDENT);
    } else {
      writeWithMap(state, property.name, CAT_IDENT, property);
    }
  }
}

/**
 * A call parenthesizes itself from `PREC_NEW` upwards, or whenever `CTX_FORBID_CALL` is set,
 * both of which mean an unparenthesized argument list would be read as belonging to an enclosing `new`.
 *
 * Writing that paren also re-marks the start of a statement or of an `export default`,
 * so the callee inside the parens is still read as the leftmost token of the construct.
 */
function printCallExpression(
  node: ESTree.CallExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_NEW || (ctx & CTX_FORBID_CALL) !== 0;

  if (wrap) {
    // The mark read below is the one from before the paren, so it is checked here -
    // `writeNoLast` marks `last` stale in between, and the last line clears that again
    debugAssertLastFresh(state);

    // A statement or an `export default` mark is left intact.
    // A concise arrow body's mark is deliberately left to die at the paren, as `oxc_codegen` does.
    writeNoLast(state, "(");

    // `CAT_START_OF_STMT` or `CAT_START_OF_DEFAULT_EXPORT`, which are adjacent - see `write.ts`
    if ((state.last | 1) !== CAT_START_OF_STMT) state.last = CAT_OTHER;
    if (DEBUG) state.lastIsStale = false;
  }

  printExpression(node.callee, state, PREC_POSTFIX, CTX_NONE);

  if (node.optional) write(state, "?.", CAT_OTHER);

  if (TS) printTypeArguments(node.typeArguments, state);

  printArguments(node, node.arguments, state);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * The argument list shared by `printCallExpression` and `printNewExpression`.
 *
 * Arguments print at `PREC_COMMA`, so a sequence expression parenthesizes itself instead of reading as more arguments.
 */
function printArguments(
  node: ESTree.CallExpression | ESTree.NewExpression,
  args: ESTree.Argument[],
  state: State,
): void {
  const { length } = args;
  if (length === 0) {
    writeNoLast(state, "(");
    writeWithMapEnd(state, ")", CAT_CLOSE_BRACKET, node);
    return;
  }

  write(state, "(", CAT_OTHER);

  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);

    const arg = args[i];
    if (arg.type === "SpreadElement") {
      writeWithMap(state, "...", CAT_OTHER, arg);
      printExpression(arg.argument, state, PREC_COMMA, CTX_NONE);
    } else {
      printExpression(arg, state, PREC_COMMA, CTX_NONE);
    }
  }

  writeWithMapEnd(state, ")", CAT_CLOSE_BRACKET, node);
}

/**
 * `#field in obj`, which arrives as a `BinaryExpression` with a `PrivateIdentifier` on the left
 * rather than as a node type of its own - hence the extra test in `printExpression`.
 *
 * It sits at the `in` operator's own level, so it wraps from `PREC_COMPARE` upwards, and the right
 * operand prints one level tighter with `CTX_FORBID_IN` set.
 */
export function printPrivateInExpression(
  node: ESTree.PrivateInExpression,
  state: State,
  precedence: number,
): void {
  const wrap = precedence >= PREC_COMPARE;
  if (wrap) write(state, "(", CAT_OTHER);

  markWithMap(state, node);
  writeWithMapNoLast(state, "#", node.left);
  write(state, node.left.name, CAT_IDENT);
  write(state, " in ", CAT_OTHER);
  printExpression(node.right, state, PREC_EQUALS, CTX_FORBID_IN);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * An object literal parenthesizes itself when it starts a statement or a concise arrow body,
 * where a leading `{` would otherwise be read as a block.
 *
 * Layout follows the property count alone - two or more break across lines, exactly one prints
 * inside spaces on a single line, and none gives `{}`.
 */
function printObjectExpression(node: ESTree.ObjectExpression, state: State): void {
  debugAssertLastFresh(state);
  // `CAT_START_OF_STMT` or `CAT_START_OF_ARROW_EXPR`, which are adjacent - see `write.ts`
  const wrap = ((state.last - 1) | 1) === CAT_START_OF_STMT;

  if (wrap) write(state, "(", CAT_OTHER);

  const { properties } = node;
  const { length } = properties;
  const isMultiLine = length > 1;

  writeWithMap(state, "{", CAT_OTHER, node);

  if (isMultiLine) {
    state.indentLevel++;

    for (let i = 0; i < length; i++) {
      write(state, i > 0 ? ",\n" : "\n", CAT_OTHER);
      printIndent(state);
      printObjectProperty(properties[i], state);
    }

    write(state, "\n", CAT_OTHER);
    state.indentLevel--;
    printIndent(state);
  } else if (length === 1) {
    write(state, " ", CAT_OTHER);
    printObjectProperty(properties[0], state);
    write(state, " ", CAT_OTHER);
  }

  writeWithMapEnd(state, "}", CAT_OTHER, node);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * One entry of an object literal - a spread, a method or accessor, or a plain `key: value` pair.
 *
 * Shorthand and computed-ness are both worked out here rather than taken from the AST,
 * so the output stays valid however a transform left those flags.
 */
function printObjectProperty(node: ESTree.ObjectPropertyKind, state: State): void {
  if (node.type === "SpreadElement") {
    writeWithMap(state, "...", CAT_OTHER, node);
    printExpression(node.argument, state, PREC_COMMA, CTX_NONE);
    return;
  }

  const { key, value } = node;
  if (
    value.type === "FunctionExpression" ||
    (TS && value.type === "TSEmptyBodyFunctionExpression")
  ) {
    markWithMap(state, node);

    const { kind } = node;
    const isGetter = kind === "get";
    const isAccessor = isGetter || kind === "set";
    if (isAccessor) write(state, isGetter ? "get " : "set ", CAT_OTHER);

    if (node.method || isAccessor) {
      if (value.async) {
        printSpaceBeforeIdentifier(state);
        write(state, "async ", CAT_OTHER);
      }
      if (value.generator) write(state, "*", CAT_OTHER);

      if (node.computed) {
        write(state, "[", CAT_OTHER);
        typeAssertIs<ESTree.Expression>(key);
        printExpression(key, state, PREC_COMMA, CTX_NONE);
        write(state, "]", CAT_CLOSE_BRACKET);
      } else {
        printPropertyKey(key, state);
      }

      if (TS) printTypeParameters(value.typeParameters, state);

      printParenParams(value.params, state);

      if (TS && value.returnType != null) printTypeAnnotation(value.returnType, state);

      if (value.body != null) {
        write(state, " ", CAT_OTHER);
        printFunctionBody(value.body, state);
      }

      return;
    }
  }

  // Shorthand is re-derived from names (matching Oxc), except `__proto__`
  // which keeps its flag (`{ __proto__ }` and `{ __proto__: x }` differ).
  let shorthand = false;
  // The identifier a name-derived shorthand prints, which is known here without asking
  // the expression printer what it is.
  // A `__proto__` shorthand leaves this null, since its flag says nothing about the value it carries.
  let shorthandIdentifier: ESTree.IdentifierReference | null = null;
  if (!node.computed && key.type === "Identifier") {
    if (key.name === "__proto__") {
      shorthand = node.shorthand;
    } else {
      const inner = withoutParens(value);
      if (inner.type === "Identifier" && key.name === inner.name) {
        shorthand = true;
        shorthandIdentifier = inner;
      }
    }
  }

  let { computed } = node;
  // `{ -1: 0 }` and `{ 1/0: 0 }` must print as `{ [-1]: 0 }` / `{ [1 / 0]: 0 }`
  if (
    !computed &&
    key.type === "Literal" &&
    typeof key.value === "number" &&
    (key.value < 0 || Object.is(key.value, -0) || !Number.isFinite(key.value))
  ) {
    computed = true;
  }

  if (shorthand) {
    if (shorthandIdentifier !== null) {
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, shorthandIdentifier.name, CAT_IDENT, shorthandIdentifier);
    } else {
      // `__proto__` shorthand, whose value can be anything. Print through any parens around it.
      printExpression(withoutParens(value), state, PREC_COMMA, CTX_NONE);
    }
  } else {
    if (computed) {
      write(state, "[", CAT_OTHER);
      typeAssertIs<ESTree.Expression>(key);
      printExpression(key, state, PREC_COMMA, CTX_NONE);
      write(state, "]", CAT_CLOSE_BRACKET);
    } else {
      printPropertyKey(key, state);
    }

    write(state, ": ", CAT_OTHER);

    printExpression(value, state, PREC_COMMA, CTX_NONE);
  }
}

/**
 * Arrays break across lines from three elements upwards, where objects break from two.
 *
 * Holes print as nothing between the commas, and a hole in final position needs one further comma
 * written after it, since `[a,]` holds one element and `[a,,]` holds two.
 */
function printArrayExpression(node: ESTree.ArrayExpression, state: State): void {
  const { elements } = node;
  const { length } = elements;
  const isMultiLine = length > 2;

  writeWithMap(state, "[", CAT_OTHER, node);

  if (isMultiLine) state.indentLevel++;

  for (let i = 0; i < length; i++) {
    if (isMultiLine) {
      write(state, i !== 0 ? ",\n" : "\n", CAT_OTHER);
      printIndent(state);
    } else if (i !== 0) {
      write(state, ", ", CAT_OTHER);
    }

    const element = elements[i];
    if (element != null) {
      if (element.type === "SpreadElement") {
        writeWithMap(state, "...", CAT_OTHER, element);
        printExpression(element.argument, state, PREC_COMMA, CTX_NONE);
      } else {
        printExpression(element, state, PREC_COMMA, CTX_NONE);
      }
    }

    if (i === length - 1 && element == null) {
      write(state, ",", CAT_OTHER);
    }
  }

  if (isMultiLine) {
    write(state, "\n", CAT_OTHER);
    state.indentLevel--;
    printIndent(state);
  }

  writeWithMapEnd(state, "]", CAT_CLOSE_BRACKET, node);
}

/**
 * Wraps from `PREC_ASSIGN` upwards. An object destructuring target forces parens as well at the start of
 * a statement or a concise arrow body, where the leading `{` would be read as a block.
 *
 * @param ctx - Handed to the right operand unchanged, so a `for` head's `CTX_FORBID_IN`
 *   still reaches an `in` operator there.
 */
function printAssignmentExpression(
  node: ESTree.AssignmentExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const { left } = node;

  let wrap = precedence >= PREC_ASSIGN;
  if (!wrap && left.type === "ObjectPattern") {
    debugAssertLastFresh(state);
    // `CAT_START_OF_STMT` or `CAT_START_OF_ARROW_EXPR`, which are adjacent - see `write.ts`
    wrap = ((state.last - 1) | 1) === CAT_START_OF_STMT;
  }

  if (wrap) write(state, "(", CAT_OTHER);

  markWithMap(state, node);

  printAssignmentTarget(left, state);
  write(state, PADDED_ASSIGN_OPERATORS[node.operator], CAT_OTHER);
  printExpression(node.right, state, PREC_COMMA, ctx);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Which precedence decides the parens depends on `node.prefix` - a prefix `++`/`--` binds at `PREC_PREFIX`,
 * a postfix one at `PREC_POSTFIX`.
 *
 * The operator's own category is recorded as it is written, which is how an adjacent `+` or `-`
 * knows to leave a space rather than glue into `---x`.
 */
function printUpdateExpression(
  node: ESTree.UpdateExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const selfPrecedence = node.prefix ? PREC_PREFIX : PREC_POSTFIX;
  const wrap = precedence >= selfPrecedence;
  if (wrap) write(state, "(", CAT_OTHER);

  const operatorCode = updateOperatorCode(node.operator);

  if (node.prefix) {
    printSpaceBeforeOperator(state, operatorCode);
    writeWithMap(state, node.operator, operatorCode, node);
    printExpression(node.argument, state, PREC_PREFIX, ctx);
  } else {
    markWithMap(state, node);
    printExpression(node.argument, state, PREC_POSTFIX, ctx);
    printSpaceBeforeOperator(state, operatorCode);
    write(state, node.operator, operatorCode);
  }

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Wraps from `PREC_PREFIX` upwards and prints its argument at `PREC_EXPONENTIATION`,
 * so a `**` operand takes parens of its own - `-a ** b` does not parse.
 *
 * A `!` written straight after a `<` records a category of its own, so that a `--`
 * printed next is spaced off it and cannot complete `<!--`.
 */
function printUnaryExpression(
  node: ESTree.UnaryExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_PREFIX;
  if (wrap) write(state, "(", CAT_OTHER);

  const { operator } = node;
  let isDeleteInfinity = false;
  if (operator.length > 1) {
    // typeof, void, delete
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, operator, CAT_IDENT, node);
    write(state, " ", CAT_OTHER);
    // `delete Infinity` is a syntax error in strict mode
    isDeleteInfinity =
      operator === "delete" && node.argument.type === "Literal" && node.argument.value === Infinity;
  } else {
    let operatorCode = unaryOperatorCode(operator);
    printSpaceBeforeOperator(state, operatorCode);
    debugAssertLastFresh(state);
    if (operatorCode === CAT_OP_UN_NOT && state.last === CAT_LT) {
      operatorCode = CAT_OP_UN_NOT_AFTER_LT;
    }
    writeWithMap(state, operator, operatorCode, node);
  }

  if (isDeleteInfinity) write(state, "(0, ", CAT_OTHER);
  printExpression(node.argument, state, PREC_EXPONENTIATION, ctx);
  if (isDeleteInfinity) write(state, ")", CAT_CLOSE_BRACKET);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Wraps from `PREC_CONDITIONAL` upwards. Both branches print at `PREC_YIELD`, which admits
 * a bare assignment or `yield` but still parenthesizes a comma sequence.
 *
 * The consequent is fenced by the `?` and `:` either side of it, so it alone never needs
 * the `in` restriction passed down.
 */
function printConditionalExpression(
  node: ESTree.ConditionalExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_CONDITIONAL;

  // The parens the wrap adds are themselves enough to escape a `for` head,
  // so `in` is only forbidden further down when the expression is printed bare
  let innerCtx = 0;
  if (wrap) {
    write(state, "(", CAT_OTHER);
  } else {
    innerCtx = ctx & CTX_FORBID_IN;
  }

  // Keep an `as` or `satisfies` test grouped, by printing it at the precedence which makes it
  // parenthesize itself. Without those parentheses a regexp consequent does not survive a
  // round trip: `value as Type ? /x/ : y` re-lexes the `/` as a division.
  let testPrecedence = PREC_CONDITIONAL;
  if (TS) {
    const testType = withoutParens(node.test).type;
    if (testType === "TSAsExpression" || testType === "TSSatisfiesExpression") {
      testPrecedence = PREC_COMPARE;
    }
  }

  printExpression(node.test, state, testPrecedence, innerCtx);
  write(state, " ? ", CAT_OTHER);
  printExpression(node.consequent, state, PREC_YIELD, CTX_NONE);
  write(state, " : ", CAT_OTHER);
  printExpression(node.alternate, state, PREC_YIELD, innerCtx);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Wraps from `PREC_COMMA` upwards.
 * `CTX_FORBID_CALL` is cleared for the elements, since every position which sets that flag
 * also passes a precedence at or above `PREC_COMMA`, so the parens are already written by then.
 */
function printSequenceExpression(
  node: ESTree.SequenceExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_COMMA;
  if (wrap) write(state, "(", CAT_OTHER);

  const innerCtx = ctx & ~CTX_FORBID_CALL;
  const { expressions } = node;
  const { length } = expressions;
  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);
    printExpression(expressions[i], state, PREC_LOWEST, innerCtx);
  }

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Wraps from `PREC_ASSIGN` upwards, and when it does the parens fence the body, so `CTX_FORBID_IN`
 * is dropped on the way in.
 *
 * A concise body marks `last`, which is how an object literal body knows to parenthesize itself.
 * A TS return type has to sit between the parameters and the arrow, so those are printed separately,
 * instead of through `printParenParamsArrow`.
 */
function printArrowFunctionExpression(
  node: ESTree.ArrowFunctionExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_ASSIGN;
  const bodyCtx = wrap ? ctx & ~CTX_FORBID_IN : ctx;

  if (wrap) write(state, "(", CAT_OTHER);

  if (node.async) {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, "async ", CAT_OTHER, node);
  }

  if (TS) printTypeParameters(node.typeParameters, state);

  const { returnType } = node;
  if (TS && returnType != null) {
    printParenParams(node.params, state);
    printTypeAnnotation(returnType, state);
    write(state, " => ", CAT_OTHER);
  } else {
    printParenParamsArrow(node.params, state);
  }

  const { body } = node;
  if (body.type === "BlockStatement") {
    printFunctionBody(body, state);
  } else {
    state.last = CAT_START_OF_ARROW_EXPR;
    printExpression(body, state, PREC_COMMA, bodyCtx);
  }

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Only a call position binds tighter than `new`, so this wraps from `PREC_CALL` upwards.
 *
 * The callee prints at `PREC_NEW` with `CTX_FORBID_CALL`, so a call inside it parenthesizes itself,
 * rather than being taken for this `new`'s own argument list.
 */
function printNewExpression(node: ESTree.NewExpression, state: State, precedence: number): void {
  const wrap = precedence >= PREC_CALL;
  if (wrap) write(state, "(", CAT_OTHER);

  printSpaceBeforeIdentifier(state);
  writeWithMap(state, "new ", CAT_OTHER, node);
  printExpression(node.callee, state, PREC_NEW, CTX_FORBID_CALL);
  if (TS) printTypeArguments(node.typeArguments, state);
  printArguments(node, node.arguments, state);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Quasis and expressions interleave, with one more quasi than there are expressions, so the first chunk
 * is written before the loop and each iteration ends with the chunk after its substitution.
 *
 * Substitutions print from `PREC_LOWEST` with no context flags, since `${` and `}` fence them from everything around.
 */
function printTemplateLiteral(node: ESTree.TemplateLiteral, state: State): void {
  writeWithMapNoLast(state, "`", node);

  const { quasis, expressions } = node;
  const { length } = expressions;
  const firstQuasi = quasis[0];
  writeNoLast(state, templateQuasiRaw(firstQuasi));

  for (let i = 0; i < length; i++) {
    write(state, "${", CAT_OTHER);
    printExpression(expressions[i], state, PREC_LOWEST, CTX_NONE);
    writeNoLast(state, "}");

    const quasi = quasis[i + 1];
    const raw = templateQuasiRaw(quasi);
    // A TS-shaped Oxc ESTree quasi includes the substitution's closing `}` in its span.
    // The JS-shaped ESTree quasi and Oxc's Rust AST both start at the raw template text.
    if (raw.length > 0) markWithMapAtStartOffset(state, quasi, TS ? 1 : 0);
    writeNoLast(state, raw);
  }

  write(state, "`", CAT_OTHER);
}

/**
 * The raw text of one template chunk, ready to be written between the backticks.
 *
 * It also runs the `</script` escape, since a template can hold that sequence verbatim
 * and would otherwise close a surrounding script element.
 */
function templateQuasiRaw(quasi: ESTree.TemplateElement): string {
  // Line terminators in template raws are normalized to LF at parse time.
  // TS-ESLint's `raw` keeps the source bytes, so normalize here.
  let { raw } = quasi.value;
  if (raw.includes("\r")) raw = raw.replace(/\r\n?/g, "\n");
  return escapeScriptCloseTag(raw);
}

/**
 * Wraps from `PREC_PREFIX` upwards like the other unary operators, and prints its argument
 * at `PREC_EXPONENTIATION` - `await a ** b` does not parse, so the `**` takes parens.
 */
function printAwaitExpression(
  node: ESTree.AwaitExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_PREFIX;
  if (wrap) write(state, "(", CAT_OTHER);

  printSpaceBeforeIdentifier(state);
  writeWithMap(state, "await ", CAT_OTHER, node);
  printExpression(node.argument, state, PREC_EXPONENTIATION, ctx);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * Wraps from `PREC_ASSIGN` upwards. The argument prints at `PREC_YIELD`, one level below assignment,
 * so a nested `yield` or an assignment stays bare while a comma sequence parenthesizes.
 */
function printYieldExpression(
  node: ESTree.YieldExpression,
  state: State,
  precedence: number,
): void {
  const wrap = precedence >= PREC_ASSIGN;
  if (wrap) write(state, "(", CAT_OTHER);

  printSpaceBeforeIdentifier(state);
  writeWithMap(state, "yield", CAT_IDENT, node);

  if (node.delegate) write(state, "*", CAT_OTHER);

  if (node.argument != null) {
    write(state, " ", CAT_OTHER);
    printExpression(node.argument, state, PREC_YIELD, CTX_NONE);
  }

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * `import(...)` takes the same wrap rule as a call, since `new import("x")` would otherwise
 * attach the argument list to the `new`.
 *
 * A `phase` prints as `import.<phase>(...)`, and import options follow the source as a second argument.
 */
function printImportExpression(
  node: ESTree.ImportExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_NEW || (ctx & CTX_FORBID_CALL) !== 0;
  if (wrap) write(state, "(", CAT_OTHER);

  printSpaceBeforeIdentifier(state);
  writeWithMap(state, "import", CAT_IDENT, node);

  if (node.phase != null) {
    writeNoLast(state, ".");
    write(state, node.phase, CAT_IDENT);
  }

  write(state, "(", CAT_OTHER);
  printExpression(node.source, state, PREC_COMMA, CTX_NONE);
  if (node.options != null) {
    write(state, ", ", CAT_OTHER);
    printExpression(node.options, state, PREC_COMMA, CTX_NONE);
  }
  write(state, ")", CAT_CLOSE_BRACKET);

  if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
}

/**
 * A `ChainExpression` only records how far an optional chain reaches, so with nothing pressing on it
 * the node prints straight through with the outer precedence and context.
 *
 * From `PREC_POSTFIX` upwards, or under `CTX_FORBID_CALL`, the whole chain is parenthesized
 * and printed from `PREC_LOWEST` - the chain has to stop at the paren, instead of running on
 * into the enclosing member access or `new`.
 */
function printChainExpression(
  node: ESTree.ChainExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const wrap = precedence >= PREC_POSTFIX || (ctx & CTX_FORBID_CALL) !== 0;
  if (wrap) {
    write(state, "(", CAT_OTHER);
    printExpression(node.expression, state, PREC_LOWEST, CTX_NONE);
    write(state, ")", CAT_CLOSE_BRACKET);
  } else {
    printExpression(node.expression, state, precedence, ctx);
  }
}
