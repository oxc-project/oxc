// Binary/logical expressions (port of `binary_expr_visitor.rs`).

import { typeAssertIs } from "../asserts.ts";
import { CAT_CLOSE_BRACKET, CAT_OTHER, write } from "./write.ts";
import { printPrivateInExpression, printExpression } from "./expression.ts";
import { BIN_PRECEDENCE, CTX_FORBID_IN, PADDED_BIN_OPERATORS } from "./operators.ts";
import { withoutParens } from "./parens.ts";
import { PREC_CALL, PREC_EXPONENTIATION, PREC_LOWEST, PREC_PREFIX } from "./precedence.ts";

import type { State } from "../state.ts";
import type { LiteralExtras } from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * One level of the binary/logical expression chain.
 *
 * `parent` is the level above, which is waiting for this one to finish before printing
 * its own operator and right operand.
 */
interface BinaryVisitor {
  e: ESTree.BinaryExpression | ESTree.LogicalExpression;
  precedence: number;
  ctx: number;
  leftPrecedence: number;
  operator: ESTree.BinaryOperator | ESTree.LogicalOperator;
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
 * @param precedence - Precedence of the position this expression sits in, deciding parenthesisation
 * @param ctx - Context flags, carrying whether `in` is forbidden and calls are
 */
export function printBinaryish(
  node: ESTree.BinaryExpression | ESTree.LogicalExpression,
  state: State,
  precedence: number,
  ctx: number,
): void {
  // The pending outer levels are threaded through `parent` rather than a separate stack array.
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

  for (;;) {
    binCheckAndPrepare(v, state);

    const left = withoutParens(v.e.left);
    if (left.type === "BinaryExpression" || left.type === "LogicalExpression") {
      if (left.type === "BinaryExpression" && left.left.type === "PrivateIdentifier") {
        // Private-in expression as the left operand
        typeAssertIs<ESTree.PrivateInExpression>(left);
        printPrivateInExpression(left, state, PREC_LOWEST);
        binVisitRightAndFinish(v, state);
        break;
      }

      typeAssertIs<ESTree.BinaryExpression | ESTree.LogicalExpression>(left);

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
    } else {
      // `v.e.left` prints, not the unwrapped `left` - a `ParenthesizedExpression` around a function
      // expression is how Oxc's `pife` flag reaches this printer, and the arm for it in `printExpression`
      // writes those parens back. Around anything else the wrapper is transparent, forwarding
      // this precedence and `ctx` unchanged.
      printExpression(v.e.left, state, v.leftPrecedence, v.ctx);
      binVisitRightAndFinish(v, state);
      break;
    }
  }

  while ((v = v.parent) !== null) {
    binVisitRightAndFinish(v, state);
  }
}

/**
 * Work out whether one level of the chain needs parentheses, write the opening one if so,
 * and settle the precedences its two operands are printed at.
 *
 * `**` is right associative, so its left operand binds tighter, and the rest are the other way round.
 * `??` may not sit unparenthesized beside `&&` or `||`, which is why either operand can be forced up to `PREC_PREFIX`.
 */
function binCheckAndPrepare(v: BinaryVisitor, state: State): void {
  const { e } = v;
  const eOperator = e.operator;
  const ePrecedence = BIN_PRECEDENCE[eOperator];

  // No parens if both sides use the same logical operator
  const precedenceCheck =
    v.precedence >= ePrecedence &&
    (!isLogicalOperator(v.operator) || v.precedence !== BIN_PRECEDENCE[v.operator]);
  v.operator = eOperator;
  v.wrap = precedenceCheck || (eOperator === "in" && (v.ctx & CTX_FORBID_IN) !== 0);

  if (v.wrap) {
    write(state, "(", CAT_OTHER);
    v.ctx &= ~CTX_FORBID_IN;
  }
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

  if (eOperator === "??") {
    // Nullish coalescing cannot mix with && / || unparenthesized
    const left = withoutParens(e.left);
    if (left.type === "LogicalExpression" && left.operator !== "??") {
      v.leftPrecedence = PREC_PREFIX;
    }

    const right = withoutParens(e.right);
    if (right.type === "LogicalExpression" && right.operator !== "??") {
      v.rightPrecedence = PREC_PREFIX;
    }
  } else if (eOperator === "**") {
    // The base of `**` must be an `UpdateExpression`.
    // Unary/await bases and negative-printing literals must be parenthesized.
    const left = withoutParens(e.left);
    typeAssertIs<LiteralExtras>(left);
    if (
      left.type === "UnaryExpression" ||
      left.type === "AwaitExpression" ||
      (TS && left.type === "TSTypeAssertion") ||
      (left.type === "Literal" && (typeof left.value === "number" || left.bigint != null))
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
  if (v.wrap) write(state, ")", CAT_CLOSE_BRACKET);
}
