// Statements.

import { typeAssertIs } from "../asserts.ts";
import { printAssignmentTarget } from "./assignment_target.ts";
import { printBindingPattern } from "./binding_pattern.ts";
import {
  CAT_CLOSE_BRACKET,
  CAT_IDENT,
  CAT_OP_UN_NOT,
  CAT_OTHER,
  CAT_START_OF_STMT,
  markWithMap,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapEnd,
  writeWithMapNoLast,
} from "./write.ts";
import { printClass } from "./class.ts";
import { printExpression } from "./expression.ts";
import { printFunction } from "./function.ts";
import { printSpaceBeforeIdentifier } from "./space.ts";
import { printIndent } from "./indent.ts";
import {
  printExportAllDeclaration,
  printExportDefaultDeclaration,
  printExportNamedDeclaration,
  printImportDeclaration,
} from "./module.ts";
import { CTX_FORBID_IN, CTX_NONE } from "./operators.ts";
import { withoutParens } from "./parens.ts";
import { PREC_COMMA, PREC_LOWEST } from "./precedence.ts";
import { printString } from "./string.ts";
import {
  printTSEnumDeclaration,
  printTSImportEqualsDeclaration,
  printTSInterfaceDeclaration,
  printTSModuleDeclaration,
  printTSTypeAliasDeclaration,
  printTypeAnnotation,
} from "./typescript.ts";

import type { State } from "../state.ts";
import type { LiteralExtras, UnknownNode } from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Entry point for a whole file.
 * A hashbang is written ahead of everything else, since it is only legal on the first line.
 */
export function printProgram(node: ESTree.Program, state: State): void {
  if (node.hashbang != null) {
    writeNoLast(state, "#!");
    writeNoLast(state, node.hashbang.value);
    write(state, "\n", CAT_OTHER);
  }

  printDirectivesAndStatements(node.body, state);
}

/**
 * A directive prologue followed by ordinary statements. Directives are taken from the front for as long
 * as they keep coming, and everything from the first non-directive onwards prints normally.
 *
 * The handover is the delicate part - a string literal statement arriving right after the prologue
 * has to be kept from being read back as one more directive.
 */
export function printDirectivesAndStatements(
  body: Array<ESTree.Directive | ESTree.Statement>,
  state: State,
): void {
  const { length } = body;
  if (length === 0) return;

  let i = 0;
  let stmt = body[0];
  while (stmt.type === "ExpressionStatement") {
    if (stmt.directive != null) {
      typeAssertIs<ESTree.Directive>(stmt);
      printDirective(stmt, state);
      if (++i >= length) return;
      stmt = body[i];
    } else {
      // Ensure a string literal (only possible via parentheses, since a bare one would be a directive)
      // as 1st statement or after other real directives, is not re-parsed as a directive
      const inner = withoutParens(stmt.expression);
      typeAssertIs<LiteralExtras>(inner);
      if (inner.type === "Literal" && typeof inner.value === "string") {
        const mapsIndent = state.indentLevel > 0 || state.pendingIndentAsSpace;
        printIndent(state);
        if (mapsIndent) markWithMap(state, stmt);
        writeNoLast(state, "(");
        printString(state, inner.value, inner);
        write(state, ");\n", CAT_OTHER);
        i++;
      }
      break;
    }
  }

  for (; i < length; i++) {
    printStatement(body[i], state);
  }
}

/**
 * Directives print from their raw source text, so the quote has to be chosen to suit the text
 * rather than the other way round - the scan takes the first unescaped `"` or `'` and quotes with
 * the other one.
 */
function printDirective(stmt: ESTree.Directive, state: State): void {
  printIndent(state);

  const { directive } = stmt;

  // A directive may not contain an escape sequence or line continuation,
  // so print the raw `directive` value with a quote character it doesn't contain.
  let quote = '"';
  const { length } = directive;
  for (let i = 0; i < length; i++) {
    const char = directive[i];
    if (char === '"') {
      quote = "'";
      break;
    } else if (char === "'") {
      break;
    } else if (char === "\\") {
      i++;
    }
  }

  writeWithMapNoLast(state, quote, stmt);
  writeNoLast(state, directive);
  writeNoLast(state, quote);
  write(state, ";\n", CAT_OTHER);
}

/**
 * Dispatch point for statements, and the counterpart to `printExpression`. Unknown types throw.
 *
 * Each case owns its own leading indent and its trailing newline, so a statement always leaves
 * the output at the start of a fresh line for the next one.
 */
export function printStatement(node: ESTree.Statement | UnknownNode, state: State): void {
  // Arms are ordered roughly in order of most common nodes.
  // V8 turns this into (essentially) as chain of `if ... else if ... else if...`,
  // so making common nodes short-circuit early is a large perf boost.
  switch (node.type) {
    case "ExpressionStatement":
      printExpressionStatement(node, state);
      break;
    case "VariableDeclaration":
      printIndent(state);
      printVariableDeclaration(node, state, CTX_NONE);
      write(state, ";\n", CAT_OTHER);
      break;
    case "BlockStatement":
      printIndent(state);
      printBlockStatement(node, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "IfStatement":
      printIndent(state);
      printIf(node, state);
      break;
    case "ReturnStatement":
      printReturnStatement(node, state);
      break;
    case "FunctionDeclaration":
      printIndent(state);
      printFunction(node, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "ForStatement":
      printForStatement(node, state);
      break;
    case "WhileStatement":
      printWhileStatement(node, state);
      break;
    case "DoWhileStatement":
      printDoWhileStatement(node, state);
      break;
    case "SwitchStatement":
      printSwitchStatement(node, state);
      break;
    case "BreakStatement":
      printIndent(state);
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, "break", CAT_IDENT, node);
      if (node.label != null) {
        write(state, " ", CAT_OTHER);
        writeWithMap(state, node.label.name, CAT_IDENT, node.label);
      }
      write(state, ";\n", CAT_OTHER);
      break;
    case "ContinueStatement":
      printIndent(state);
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, "continue", CAT_IDENT, node);
      if (node.label != null) {
        write(state, " ", CAT_OTHER);
        writeWithMap(state, node.label.name, CAT_IDENT, node.label);
      }
      write(state, ";\n", CAT_OTHER);
      break;
    case "TryStatement":
      printTryStatement(node, state);
      break;
    case "ThrowStatement":
      printIndent(state);
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, "throw ", CAT_OTHER, node);
      printExpression(node.argument, state, PREC_LOWEST, CTX_NONE);
      write(state, ";\n", CAT_OTHER);
      break;
    case "ForInStatement":
      printForInStatement(node, state);
      break;
    case "ForOfStatement":
      printForOfStatement(node, state);
      break;
    case "ClassDeclaration":
      printIndent(state);
      printClass(node, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "LabeledStatement":
      printIndent(state);
      printSpaceBeforeIdentifier(state);
      markWithMap(state, node);
      writeWithMap(state, node.label.name, CAT_IDENT, node.label);
      write(state, ":", CAT_OTHER);
      printBody(node.body, state);
      break;
    case "EmptyStatement":
      printIndent(state);
      writeWithMap(state, ";\n", CAT_OTHER, node);
      break;
    case "ImportDeclaration":
      printImportDeclaration(node, state);
      break;
    case "ExportNamedDeclaration":
      printExportNamedDeclaration(node, state);
      break;
    case "ExportDefaultDeclaration":
      printExportDefaultDeclaration(node, state);
      break;
    case "ExportAllDeclaration":
      printExportAllDeclaration(node, state);
      break;
    case "WithStatement":
      printIndent(state);
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, "with(", CAT_OTHER, node);
      printExpression(node.object, state, PREC_LOWEST, CTX_NONE);
      write(state, ")", CAT_CLOSE_BRACKET);
      printBody(node.body, state);
      break;
    case "DebuggerStatement":
      printIndent(state);
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, "debugger;\n", CAT_OTHER, node);
      break;
    /* IF TS */
    case "TSModuleDeclaration":
      printIndent(state);
      printTSModuleDeclaration(node, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "TSInterfaceDeclaration":
      printIndent(state);
      printTSInterfaceDeclaration(node, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "TSTypeAliasDeclaration":
      printIndent(state);
      printTSTypeAliasDeclaration(node, state);
      write(state, ";\n", CAT_OTHER);
      break;
    case "TSEnumDeclaration":
      printIndent(state);
      printTSEnumDeclaration(node, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "TSImportEqualsDeclaration":
      printIndent(state);
      printTSImportEqualsDeclaration(node, state);
      write(state, ";\n", CAT_OTHER);
      break;
    case "TSDeclareFunction":
      printIndent(state);
      printFunction(node, state);
      write(state, "\n", CAT_OTHER);
      break;
    case "TSExportAssignment":
      printIndent(state);
      write(state, "export = ", CAT_OTHER);
      printExpression(node.expression, state, PREC_LOWEST, CTX_NONE);
      write(state, ";\n", CAT_OTHER);
      break;
    case "TSNamespaceExportDeclaration":
      printIndent(state);
      write(state, "export as namespace ", CAT_OTHER);
      writeWithMap(state, node.id.name, CAT_IDENT, node.id);
      write(state, ";\n", CAT_OTHER);
      break;
    /* END_IF */
    default:
      throw new Error(`Unknown statement type: ${node.type}`);
  }
}

/**
 * Marks `last` as the start of a statement, which is how an object literal or an object destructuring assignment
 * printed there knows to parenthesize itself.
 *
 * The indent is written first, so the mark is the last thing to touch `last` before the expression.
 */
function printExpressionStatement(node: ESTree.ExpressionStatement, state: State): void {
  const mapsIndent = state.indentLevel > 0 || state.pendingIndentAsSpace;
  printIndent(state);
  if (mapsIndent) markWithMap(state, node);
  state.last = CAT_START_OF_STMT;
  printExpression(node.expression, state, PREC_LOWEST, CTX_NONE);
  write(state, ";\n", CAT_OTHER);
}

/**
 * Shared by the standalone declaration and the `for` and `for-in` heads, which is why it writes no
 * indent and no terminator - the caller supplies both.
 *
 * @param ctx - Passed to every initializer, so a `for` head can hand down `CTX_FORBID_IN` and stop an
 *   `in` operator there being read as the head's own.
 */
export function printVariableDeclaration(
  node: ESTree.VariableDeclaration,
  state: State,
  ctx: number,
): void {
  printSpaceBeforeIdentifier(state);

  // The node's mapping goes on whichever of these is written first
  const declare = TS && node.declare;
  if (declare) {
    writeWithMap(state, "declare ", CAT_OTHER, node);
    write(state, node.kind, CAT_IDENT);
  } else {
    writeWithMap(state, node.kind, CAT_IDENT, node);
  }

  const { declarations } = node;
  const { length } = declarations;
  if (length > 0) write(state, " ", CAT_OTHER);

  for (let i = 0; i < length; i++) {
    if (i > 0) write(state, ", ", CAT_OTHER);

    const declarator = declarations[i];
    const { id } = declarator;
    if (TS && declarator.definite) {
      // `let x!: T` - the `!` sits between the name and its annotation
      typeAssertIs<ESTree.BindingIdentifier>(id);
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, id.name, CAT_IDENT, id);
      write(state, "!", CAT_OP_UN_NOT);
      if (id.typeAnnotation != null) {
        printTypeAnnotation(id.typeAnnotation, state);
      }
    } else {
      printBindingPattern(id, state);
    }

    if (declarator.init != null) {
      write(state, " = ", CAT_OTHER);
      printExpression(declarator.init, state, PREC_COMMA, ctx);
    }
  }
}

/**
 * An empty block prints as `{}` with no line break inside. Otherwise the body prints one statement
 * per line, a level deeper, with the closing brace back at the caller's level.
 *
 * Nothing is written before the opening brace or after the closing one, so callers can attach
 * ` else`, ` catch` or a newline themselves.
 */
function printBlockStatement(block: ESTree.BlockStatement, state: State): void {
  const { body } = block;
  const { length } = body;
  if (length === 0) {
    writeWithMapNoLast(state, "{", block);
    writeWithMapEnd(state, "}", CAT_OTHER, block);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, block);
  state.indentLevel++;

  for (let i = 0; i < length; i++) {
    printStatement(body[i], state);
  }

  state.indentLevel--;
  printIndent(state);
  writeWithMapEnd(state, "}", CAT_OTHER, block);
}

/**
 * An `else if` chain prints flat, by recursing on an `IfStatement` alternate rather than treating
 * it as a nested body.
 *
 * A non-block consequent which trails off into a dangling `if` is given braces, so a following
 * `else` cannot attach to the wrong one - see `wrapToAvoidAmbiguousElse`.
 */
function printIf(node: ESTree.IfStatement, state: State): void {
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "if (", CAT_OTHER, node);

  printExpression(node.test, state, PREC_LOWEST, CTX_NONE);

  const { consequent, alternate } = node;
  if (consequent.type === "BlockStatement") {
    write(state, ") ", CAT_OTHER);
    printBlockStatement(consequent, state);
    write(state, alternate != null ? " " : "\n", CAT_OTHER);
  } else if (wrapToAvoidAmbiguousElse(consequent)) {
    writeNoLast(state, ") ");
    writeWithMap(state, "{\n", CAT_OTHER, consequent);
    state.indentLevel++;
    printStatement(consequent, state);
    state.indentLevel--;
    printIndent(state);
    writeWithMapEnd(state, "}", CAT_OTHER, consequent);
    write(state, alternate != null ? " " : "\n", CAT_OTHER);
  } else {
    write(state, ")", CAT_CLOSE_BRACKET);
    printBody(consequent, state);
    if (alternate != null) printIndent(state);
  }

  if (alternate != null) {
    printSpaceBeforeIdentifier(state);

    write(state, "else", CAT_IDENT);

    if (alternate.type === "BlockStatement") {
      write(state, " ", CAT_OTHER);
      printBlockStatement(alternate, state);
      write(state, "\n", CAT_OTHER);
    } else if (alternate.type === "IfStatement") {
      write(state, " ", CAT_OTHER);
      printIf(alternate, state);
    } else {
      printBody(alternate, state);
    }
  }
}

/**
 * Whether a statement trails off into an `if` with no `else`, which would swallow an `else` written after it.
 *
 * Only the trailing position is followed - a loop, `with` or label body, and an `if` alternate -
 * since that is where a following `else` would land.
 */
function wrapToAvoidAmbiguousElse(stmt: ESTree.Statement): boolean {
  for (;;) {
    switch (stmt.type) {
      case "IfStatement":
        if (stmt.alternate == null) return true;
        stmt = stmt.alternate;
        break;
      case "ForStatement":
      case "ForOfStatement":
      case "ForInStatement":
      case "WhileStatement":
      case "WithStatement":
      case "LabeledStatement":
        stmt = stmt.body;
        break;
      default:
        return false;
    }
  }
}

/**
 * With an argument the space after `return` is part of the same write.
 * With none, the keyword is recorded as an identifier, so anything printed after it keeps a separating space.
 */
function printReturnStatement(node: ESTree.ReturnStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  const { argument } = node;
  if (argument != null) {
    writeWithMap(state, "return ", CAT_OTHER, node);
    printExpression(argument, state, PREC_LOWEST, CTX_NONE);
  } else {
    writeWithMap(state, "return", CAT_IDENT, node);
  }

  write(state, ";\n", CAT_OTHER);
}

/**
 * The catch binding is optional, so an absent `param` prints a bare `catch` block rather than an empty pair of parens.
 * The handler and the finalizer are each written only if present.
 */
function printTryStatement(node: ESTree.TryStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "try ", CAT_OTHER, node);

  printBlockStatement(node.block, state);

  const { handler } = node;
  if (handler != null) {
    write(state, " catch", CAT_IDENT);

    if (handler.param != null) {
      write(state, " (", CAT_OTHER);
      printBindingPattern(handler.param, state);
      write(state, ")", CAT_CLOSE_BRACKET);
    }

    write(state, " ", CAT_OTHER);
    printBlockStatement(handler.body, state);
  }

  if (node.finalizer != null) {
    write(state, " finally ", CAT_OTHER);
    printBlockStatement(node.finalizer, state);
  }

  write(state, "\n", CAT_OTHER);
}

/**
 * A switch with no cases prints as `{}` on the same line. Otherwise the cases sit one level in,
 * and each indents its own statements again - see `printSwitchCase`.
 */
function printSwitchStatement(node: ESTree.SwitchStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "switch (", CAT_OTHER, node);
  printExpression(node.discriminant, state, PREC_LOWEST, CTX_NONE);
  write(state, ") ", CAT_OTHER);

  const { cases } = node;
  const { length } = cases;
  if (length === 0) {
    writeWithMapNoLast(state, "{", node);
    writeWithMapEnd(state, "}\n", CAT_OTHER, node);
    return;
  }

  writeWithMap(state, "{\n", CAT_OTHER, node);
  state.indentLevel++;

  for (let i = 0; i < length; i++) {
    printSwitchCase(cases[i], state);
  }

  state.indentLevel--;
  printIndent(state);
  writeWithMapEnd(state, "}\n", CAT_OTHER, node);
}

/**
 * A case with exactly one statement hands it to `printBody`, which keeps it on the line of the `case`.
 * Any other count goes onto separate lines a level deeper, so an empty case prints as just its label.
 */
function printSwitchCase(node: ESTree.SwitchCase, state: State): void {
  printIndent(state);

  if (node.test != null) {
    writeWithMap(state, "case ", CAT_OTHER, node);
    printExpression(node.test, state, PREC_LOWEST, CTX_NONE);
  } else {
    writeWithMap(state, "default", CAT_IDENT, node);
  }

  write(state, ":", CAT_OTHER);

  const { consequent } = node;
  const { length } = consequent;
  if (length === 1) {
    printBody(consequent[0], state);
    return;
  }

  write(state, "\n", CAT_OTHER);
  state.indentLevel++;

  for (let i = 0; i < length; i++) {
    printStatement(consequent[i], state);
  }

  state.indentLevel--;
}

/**
 * The test is fenced by its own parens, so it prints from `PREC_LOWEST` with no context flags,
 * and the body goes through `printBody` like every other loop here.
 */
function printWhileStatement(node: ESTree.WhileStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "while (", CAT_OTHER, node);
  printExpression(node.test, state, PREC_LOWEST, CTX_NONE);
  write(state, ")", CAT_CLOSE_BRACKET);

  printBody(node.body, state);
}

/**
 * Three body shapes with their own spacing - a block sits inline between `do` and `while`,
 * an empty body prints as a bare `;`, and anything else takes its own indented line
 * with the `while` returned to the outer level.
 *
 * The closing `);` is always written out rather than left to ASI.
 */
function printDoWhileStatement(node: ESTree.DoWhileStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "do", CAT_IDENT, node);

  const { body } = node;
  if (body.type === "BlockStatement") {
    write(state, " ", CAT_OTHER);
    printBlockStatement(body, state);
    write(state, " ", CAT_OTHER);
  } else if (body.type === "EmptyStatement") {
    printIndent(state);
    writeWithMap(state, ";\n", CAT_OTHER, body);
  } else {
    write(state, "\n", CAT_OTHER);
    state.indentLevel++;
    printStatement(body, state);
    state.indentLevel--;
    printIndent(state);
  }

  write(state, "while (", CAT_OTHER);
  printExpression(node.test, state, PREC_LOWEST, CTX_NONE);
  write(state, ");\n", CAT_OTHER);
}

/**
 * The init prints with `CTX_FORBID_IN`, so an `in` operator there parenthesizes itself instead of
 * turning the head into a `for-in`.
 *
 * An absent test or update still writes its `;`, which is why those cases fold the separator,
 * and for the update the closing paren too, into a single write.
 */
function printForStatement(node: ESTree.ForStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "for (", CAT_OTHER, node);

  const { init } = node;
  if (init != null) {
    if (init.type === "VariableDeclaration") {
      printVariableDeclaration(init, state, CTX_FORBID_IN);
    } else {
      printExpression(init, state, PREC_LOWEST, CTX_FORBID_IN);
    }
  }

  const { test, update } = node;
  if (test != null) {
    write(state, "; ", CAT_OTHER);
    printExpression(test, state, PREC_LOWEST, CTX_NONE);
  } else {
    write(state, ";", CAT_OTHER);
  }

  if (update != null) {
    write(state, "; ", CAT_OTHER);
    printExpression(update, state, PREC_LOWEST, CTX_NONE);
    write(state, ")", CAT_CLOSE_BRACKET);
  } else {
    write(state, ";)", CAT_CLOSE_BRACKET);
  }

  printBody(node.body, state);
}

/**
 * The left side is either a declaration or an assignment target. A declaration prints with `CTX_FORBID_IN`,
 * so an `in` inside its initializer parenthesizes itself instead of ending the head early.
 */
function printForInStatement(node: ESTree.ForInStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "for (", CAT_OTHER, node);

  const { left } = node;
  if (left.type === "VariableDeclaration") {
    printVariableDeclaration(left, state, CTX_FORBID_IN);
  } else {
    printAssignmentTarget(left, state);
  }

  write(state, " in ", CAT_OTHER);
  printExpression(node.right, state, PREC_LOWEST, CTX_NONE);
  write(state, ")", CAT_CLOSE_BRACKET);

  printBody(node.body, state);
}

/**
 * Two targets have to be parenthesized in a `for-of` head - one beginning with `let`,
 * which would read as a declaration, and a bare `async` in a loop without `await`,
 * which would start `async of => {}`.
 *
 * Nothing here needs `CTX_FORBID_IN`, unlike the `for` and `for-in` heads, and the right side
 * prints at `PREC_COMMA` so a sequence takes parens.
 */
function printForOfStatement(node: ESTree.ForOfStatement, state: State): void {
  printIndent(state);
  printSpaceBeforeIdentifier(state);

  writeWithMap(state, "for", CAT_IDENT, node);

  if (node.await) write(state, " await", CAT_IDENT);

  write(state, " (", CAT_OTHER);

  const { left } = node;
  if (left.type === "VariableDeclaration") {
    printVariableDeclaration(left, state, CTX_NONE);
  } else {
    typeAssertIs<ESTree.Expression>(left);
    const bare = withoutParens(left);
    const wrap =
      forOfHeadStartsWithLet(left) ||
      (!node.await && bare.type === "Identifier" && bare.name === "async");
    if (wrap) write(state, "(", CAT_OTHER);
    printAssignmentTarget(left, state);
    if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
  }

  write(state, " of ", CAT_OTHER);
  printExpression(node.right, state, PREC_COMMA, CTX_NONE);
  write(state, ")", CAT_CLOSE_BRACKET);

  printBody(node.body, state);
}

/**
 * The restriction is on the first token of the head rather than on the target as a whole,
 * so this walks down the leftmost spine - through member objects and parens -
 * to whatever prints first.
 */
function forOfHeadStartsWithLet(left: ESTree.Expression): boolean {
  // Whether the leading token would be `let`.
  for (;;) {
    switch (left.type) {
      case "Identifier":
        return left.name === "let";
      case "MemberExpression":
        if (left.computed) {
          const inner = withoutParens(left.object);
          if (inner.type === "Identifier" && inner.name === "let") {
            // `let[...]` is emitted as `(let)[...]`, so no longer starts with `let`
            return false;
          }
        }
        left = left.object;
        break;
      case "ParenthesizedExpression":
        left = left.expression;
        break;
      default:
        return false;
    }
  }
}

/**
 * A statement in body position - an `if` branch, a loop or `with` body, a label's statement,
 * or a lone `case` statement.
 *
 * A block is attached with a space and closed with a newline, an empty statement collapses to `;`,
 * and anything else sets `pendingIndentAsSpace`, so its indent prints as a single space and it stays
 * on the current line.
 */
function printBody(stmt: ESTree.Statement, state: State): void {
  if (stmt.type === "BlockStatement") {
    write(state, " ", CAT_OTHER);
    printBlockStatement(stmt, state);
    write(state, "\n", CAT_OTHER);
  } else if (stmt.type === "EmptyStatement") {
    write(state, ";\n", CAT_OTHER);
  } else {
    state.pendingIndentAsSpace = true;
    printStatement(stmt, state);
  }
}
