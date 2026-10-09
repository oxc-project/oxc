// Binary/logical expressions (port of `binary_expr_visitor.rs`).

import {
  startNodeComments,
  finishNodeComments,
  hasAttachedComments,
  printDeferredLeadingComments,
  printTrailingCommentsInsideParens,
} from "./comments.ts";
import { debugAssert, typeAssertIs } from "../asserts.ts";
import { CAT_CLOSE_BRACKET, CAT_OTHER } from "./categories.ts";
import { write } from "./write.ts";
import { printPrivateInExpression, printExpression } from "./expression.ts";
import { BIN_PRECEDENCE, CTX_FORBID_IN, PADDED_BIN_OPERATORS } from "./operators.ts";
import { PREC_CALL, PREC_EXPONENTIATION, PREC_LOWEST, PREC_PREFIX } from "./precedence.ts";

import type { State } from "../state.ts";
import type { Literal, LiteralExtras } from "./types.ts";
import type {
  BinaryExpression,
  BinaryOperator,
  Expression,
  LogicalExpression,
  LogicalOperator,
  ParenthesizedExpression,
  PrivateInExpression,
} from "../../../../npm/oxc-types/types.d.ts";

/**
 * One level of the binary/logical expression chain.
 *
 * `parent` is the level above, which is waiting for this one to finish before printing
 * its own operator and right operand.
 */
interface BinaryVisitor {
  e: BinaryExpression | LogicalExpression;
  precedence: number;
  ctx: number;
  leftPrecedence: number;
  operator: BinaryOperator | LogicalOperator;
  wrap: boolean;
  rightPrecedence: number;
  parent: BinaryVisitor | null;
}

/**
 * Print a binary or logical expression, and its whole left-leaning chain, without recursing.
 *
 * `a + b + c + d` nests to the left as deep as it is long, so recursion would put the stack
 * at the mercy of the input. This walks down the left spine iteratively instead, and unwinds
 * through each level's `parent` to print the operators and right operands on the way back up.
 *
 * @param node - Binary or logical expression to print
 * @param state - Printer state
 * @param precedence - Precedence of the position this expression sits in, deciding parenthesisation
 * @param ctx - Context flags, carrying whether `in` is forbidden and calls are
 * @param leftType - `node.left.type`, which the caller has already read
 */
export function printBinaryish(
  node: BinaryExpression | LogicalExpression,
  state: State,
  precedence: number,
  ctx: number,
  leftType: string,
): void {
  // The pending outer levels are threaded through `parent` rather than a separate stack array
  let v: BinaryVisitor | null = {
    e: node,
    precedence,
    ctx,
    leftPrecedence: PREC_LOWEST,
    operator: node.operator,
    wrap: false,
    rightPrecedence: PREC_LOWEST,
    parent: null,
  };

  // An operand can be any expression, so reading its `type` is a megamorphic load.
  // Each left operand's `type` is read once, and carried down the left spine to the next level.
  // The root's is read by the caller, for its private-in check, and passed in as `leftType`.
  let { left } = node;
  debugAssert(leftType === left.type, "`leftType` must be the `type` of `node.left`");

  // At the top of each iteration, `left` is `v.e.left`, and `leftType` is its `type`
  for (;;) {
    if (COMMENTS && v.e !== node) startNodeComments(v.e, state);
    const preserveLeftParens =
      COMMENTS && leftType === "ParenthesizedExpression" && hasAttachedComments(state);
    while (leftType === "ParenthesizedExpression") {
      left = (left as ParenthesizedExpression).expression;
      leftType = left.type;
    }

    binCheckAndPrepare(v, state, left, leftType);

    let nextLeft;
    if (!preserveLeftParens && leftType === "BinaryExpression") {
      nextLeft = (left as BinaryExpression | PrivateInExpression).left;
      leftType = nextLeft.type;

      if (leftType === "PrivateIdentifier") {
        // Private-in expression as the left operand
        printPrivateInExpression(left as PrivateInExpression, state, v.leftPrecedence);
        binVisitRightAndFinish(v, state);
        if (COMMENTS && v.e !== node && v.e.comments != null) finishNodeComments(v.e, state);
        break;
      }

      typeAssertIs<BinaryExpression>(left);
      typeAssertIs<Expression>(nextLeft);
    } else if (!preserveLeftParens && leftType === "LogicalExpression") {
      typeAssertIs<LogicalExpression>(left);
      nextLeft = left.left;
      leftType = nextLeft.type;
    } else {
      // `v.e.left` prints, not the unwrapped `left` - a `ParenthesizedExpression` around a function
      // expression is how Oxc's `pife` flag reaches this printer, and the arm for it in `printExpression`
      // writes those parens back. Around anything else the wrapper is transparent, forwarding
      // this precedence and `ctx` unchanged.
      printExpression(v.e.left, state, v.leftPrecedence, v.ctx);
      binVisitRightAndFinish(v, state);
      if (COMMENTS && v.e !== node && v.e.comments != null) finishNodeComments(v.e, state);
      break;
    }

    v = {
      e: left,
      precedence: v.leftPrecedence,
      ctx: v.ctx,
      leftPrecedence: PREC_LOWEST,
      operator: v.operator,
      wrap: false,
      rightPrecedence: PREC_LOWEST,
      parent: v,
    };

    left = nextLeft;
  }

  while ((v = v.parent) !== null) {
    binVisitRightAndFinish(v, state);
    if (COMMENTS && v.e !== node && v.e.comments != null) finishNodeComments(v.e, state);
  }
}

/**
 * Work out whether one level of the chain needs parentheses, write the opening one if so,
 * and settle the precedences its two operands are printed at.
 *
 * `**` is right associative, so its left operand binds tighter, and the rest are the other way round.
 * `??` may not sit unparenthesized beside `&&` or `||`, which is why either operand can be forced up to `PREC_PREFIX`.
 *
 * @param v - The level being prepared. Its `operator` holds the parent level's operator on entry,
 *   and is replaced with this level's own.
 * @param state - Printer state
 * @param left - `v.e.left`, with any parens unwrapped
 * @param leftType - `left.type`
 */
function binCheckAndPrepare(
  v: BinaryVisitor,
  state: State,
  left: Expression,
  leftType: string,
): void {
  const { e } = v;
  const eOperator = e.operator;
  const ePrecedence = BIN_PRECEDENCE[eOperator];

  // No parens if both sides use the same logical operator
  const precedenceCheck =
    v.precedence >= ePrecedence
    && (!isLogicalOperator(v.operator) || v.precedence !== BIN_PRECEDENCE[v.operator]);
  v.operator = eOperator;
  v.wrap = precedenceCheck || (eOperator === "in" && (v.ctx & CTX_FORBID_IN) !== 0);

  if (v.wrap) {
    write(state, "(", CAT_OTHER);
    v.ctx &= ~CTX_FORBID_IN;
  }
  if (COMMENTS) printDeferredLeadingComments(e, state);
  // One level below the operator's own precedence. The precedence scale has no gaps, so this is
  // subtraction rather than a second table - `BinaryOperator::lower_precedence` in `oxc_syntax`
  // names the adjacent variant for each operator, which comes to the same thing.
  const lower = ePrecedence - 1;
  v.leftPrecedence = lower;
  v.rightPrecedence = lower;

  if (ePrecedence === PREC_EXPONENTIATION) {
    // Right-associative
    v.leftPrecedence = ePrecedence;
  } else {
    // All other binary/logical operators are left-associative
    v.rightPrecedence = ePrecedence;
  }

  // The operands can be any expression, so reading their `type` is a megamorphic load.
  // The left operand arrives from `printBinaryish` already unwrapped, with its `type` already read.
  // The right operand's `type` is read only once, while unwrapping parens.
  if (eOperator === "??") {
    // Nullish coalescing cannot mix with && / || unparenthesized
    if (leftType === "LogicalExpression" && (left as LogicalExpression).operator !== "??") {
      v.leftPrecedence = PREC_PREFIX;
    }

    let { right } = e;
    let rightType = right.type;
    while (rightType === "ParenthesizedExpression") {
      right = (right as ParenthesizedExpression).expression;
      rightType = right.type;
    }
    if (rightType === "LogicalExpression" && (right as LogicalExpression).operator !== "??") {
      v.rightPrecedence = PREC_PREFIX;
    }
  } else if (eOperator === "**") {
    // The base of `**` must be an `UpdateExpression`.
    // Unary/await bases and negative-printing literals must be parenthesized.
    if (
      leftType === "UnaryExpression"
      || leftType === "AwaitExpression"
      || (TS && leftType === "TSTypeAssertion")
      || (leftType === "Literal"
        && (typeof (left as Literal).value === "number" || (left as LiteralExtras).bigint != null))
    ) {
      v.leftPrecedence = PREC_CALL;
    }
  }
}

/**
 * Whether an operator is one of the three which `??` may not mix with unparenthesized.
 */
function isLogicalOperator(operator: string): boolean {
  return operator === "&&" || operator === "||" || operator === "??";
}

/**
 * Finish one level of the chain - its operator, its right operand, and its closing parenthesis.
 *
 * The operator is written space-padded as one token, which is why nothing here consults `last` -
 * the spacing checks cannot observe an operator which already has a space either side of it.
 */
function binVisitRightAndFinish(v: BinaryVisitor, state: State): void {
  // The operator is always surrounded by spaces here, which makes the token glue checks
  // (`printSpaceBeforeIdentifier` / `printSpaceBeforeOperator`) unobservable, so the whole token is one write
  write(state, PADDED_BIN_OPERATORS[v.operator], CAT_OTHER);
  // Any `ParenthesizedExpression` wrapper is kept, for the same reason as on the left operand
  printExpression(v.e.right, state, v.rightPrecedence, v.ctx);
  if (v.wrap) {
    if (COMMENTS) printTrailingCommentsInsideParens(v.e, state);
    write(state, ")", CAT_CLOSE_BRACKET);
  }
}
