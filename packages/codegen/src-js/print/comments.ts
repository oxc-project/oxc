// Attached comment printing. Ownership comes from the node's buckets and survives
// AST moves. The sets prevent aliases and nested dispatchers printing an owner twice.
import {
  CAT_OTHER,
  CAT_REGEX_SLASH,
  CAT_START_OF_DEFAULT_EXPORT,
  CAT_START_OF_ARROW_EXPR,
} from "./categories.ts";
import { printIndent } from "./indent.ts";
import { escapeScriptCloseTag } from "./string.ts";
import { write, writeNoLast } from "./write.ts";
import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

type CommentNode = { comments?: ESTree.NodeComments | null };
type Placement = "leading" | "trailing" | "dangling";

/**
 * Hold the final newline or spaces until the next fragment. Trailing comments
 * can precede a newline, and a source newline can replace inline spacing,
 * without truncating and flattening the growing output rope.
 */
export function writeCommentCode(state: State, code: string): void {
  if (!COMMENTS || code.length === 0) return;
  const last = code[code.length - 1];
  if (last === " " || last === "\t") {
    let end = code.length - 1;
    while (end > 0 && (code[end - 1] === " " || code[end - 1] === "\t")) end--;
    if (end === 0 && !state.commentPendingNewline) {
      if (state.commentPendingWhitespace === "") {
        state.commentBeforeWhitespace = state.commentLastChar;
      }
      state.commentPendingWhitespace += code;
    } else {
      flushCommentWhitespace(state);
      state.output += code.slice(0, end);
      state.commentPendingWhitespace = code.slice(end);
      state.commentBeforeWhitespace = end > 0 ? code[end - 1] : state.commentLastChar;
    }
  } else {
    flushCommentWhitespace(state);
    if (last === "\n") {
      state.output += code.slice(0, -1);
      state.commentPendingNewline = true;
    } else {
      state.output += code;
    }
  }
  trackCommentWrite(state, code);
}

/** Commit held whitespace before the next fragment or final output. */
export function flushCommentWhitespace(state: State): void {
  if (!COMMENTS) return;
  if (state.commentPendingWhitespace !== "") {
    state.output += state.commentPendingWhitespace;
    state.commentPendingWhitespace = "";
  }
  if (state.commentPendingNewline) {
    state.output += "\n";
    state.commentPendingNewline = false;
  }
}

/** Track new fragments and the virtual newline, never reading the output rope. */
function trackCommentWrite(state: State, code: string): void {
  if (!COMMENTS) return;
  if (code.length === 0) return;
  const last = code[code.length - 1];
  if (last === "\n") {
    state.commentBeforeNewline = code.length > 1 ? code[code.length - 2] : state.commentLastChar;
  }
  state.commentLastChar = last;
  if (last === "\n") {
    state.commentLineStart = true;
  } else if (last !== " " && last !== "\t") {
    // A token's final non-whitespace character answers this without scanning it.
    state.commentLineStart = false;
  } else {
    const newline = code.lastIndexOf("\n");
    if (newline !== -1) state.commentLineStart = !/[^ \t]/.test(code.slice(newline + 1));
    else if (/[^ \t]/.test(code)) state.commentLineStart = false;
  }
}

/**
 * Claim an owner only when it actually has comment buckets.
 * Arguments, array elements, property keys, and method functions pass `false`:
 * their Rust wrapper claims leading comments before the inner expression printer.
 */
export function startNodeComments(
  node: CommentNode,
  state: State,
  deferLeading?: boolean,
): boolean {
  if (!COMMENTS) return false;
  if (node.comments == null) return false;
  const owners = (state.commentOwners ??= new Set());
  if (owners.has(node)) return false;
  owners.add(node);
  rememberComments(node, state);
  state.hasAttachedComments = true;
  // These owners print their leading group inside any generated parentheses.
  // Applied PURE annotations use the call/new content site for the same reason.
  // CommentContent::Pure = 4; CommentContent::NoSideEffects = 6.
  const { leading } = node.comments;
  const expression = node as ESTree.Expression | ESTree.Function;
  const { type } = expression;
  // Moving an unapplied annotation before a matching construct must not make
  // it active in the generated source. These values are PureNotApplied and
  // NoSideEffectsNotApplied in the native parser's content classification.
  const unapplied =
    type === "CallExpression" || type === "NewExpression"
      ? 5
      : type === "FunctionExpression"
          || type === "FunctionDeclaration"
          || type === "ArrowFunctionExpression"
        ? 7
        : null;
  if (unapplied !== null) {
    for (const bucket of Object.values(node.comments)) {
      if (bucket !== null) {
        for (const comment of bucket) {
          if (comment.content === unapplied) (state.printedComments ??= new Set()).add(comment);
        }
      }
    }
  }
  const defer =
    leading?.some((c) => c.content === 4)
    || (deferLeading
      ?? (type === "LogicalExpression"
        || type === "ConditionalExpression"
        || type === "AssignmentExpression"
        || type === "ObjectExpression"
        || type === "ClassExpression"
        || type === "SequenceExpression"
        || (type === "BinaryExpression" && expression.left.type !== "PrivateIdentifier")
        || ((type === "FunctionExpression" || type === "ArrowFunctionExpression")
          && !leading?.some((c) => c.content === 6))));
  if (!defer) printLeadingComments(node, state);
  return true;
}

/** Emit a claimed leading group after its printer has written an opening paren. */
export function printDeferredLeadingComments(node: CommentNode, state: State): void {
  if (!COMMENTS) return;
  if (
    !state.commentOwners?.has(node)
    || !node.comments?.leading?.some((c) => !state.printedComments?.has(c))
  ) {
    return;
  }
  printLeadingComments(node, state);
}

/** Keep a claimed expression's trailing group inside its generated parentheses. */
export function printTrailingCommentsInsideParens(node: CommentNode, state: State): void {
  if (!COMMENTS || !state.commentOwners?.has(node)) return;
  printBucket(node, state, "trailing");
}

function printLeadingComments(node: CommentNode, state: State): void {
  const { last } = state;
  printBucket(node, state, "leading");
  if (state.commentLastChar === "\n") printIndent(state);
  else if (state.pendingIndentAsSpace) {
    state.pendingIndentAsSpace = false;
    if (!" \t\n".includes(state.commentLastChar)) write(state, " ", CAT_OTHER);
  }
  // Comments do not change which expression is the leftmost token of a construct.
  if (last >= CAT_START_OF_DEFAULT_EXPORT && last <= CAT_START_OF_ARROW_EXPR) state.last = last;
}

/** Finish trailing comments before the statement's final newline. */
export function finishNodeComments(node: CommentNode, state: State): void {
  if (!COMMENTS) return;
  const trailing = node.comments?.trailing;
  if (trailing != null && trailing.some((c) => !state.printedComments?.has(c))) {
    const newline = state.commentLastChar === "\n";
    if (newline) {
      state.commentPendingNewline = false;
      // Only statement terminators end a complete printer in a newline.
      state.commentLastChar = state.commentBeforeNewline;
      state.commentLineStart = false;
      if (DEBUG) state.lastCharWritten = state.commentBeforeNewline;
    }
    printBucket(node, state, "trailing");
    if (newline && state.commentLastChar !== "\n") write(state, "\n", CAT_OTHER);
  }
  state.pendingIndentAsSpace = false;
}

/** Whether a construct owns unprinted content comments. */
export function hasInsideComments(
  node: CommentNode | undefined,
  state: State,
  container?: string,
): boolean {
  if (!COMMENTS) return false;
  return (
    node?.comments?.dangling?.some(
      (c) =>
        !state.printedComments?.has(c)
        && (container === undefined ? c.container == null : c.container?.kind === container),
    ) ?? false
  );
}

/** Print dangling comments at the owner's content site, including folded containers. */
export function printInsideComments(
  node: CommentNode | undefined,
  state: State,
  container?: string,
  placement: Placement = "dangling",
): void {
  if (!COMMENTS) return;
  if (node === undefined) return;
  printBucket(node, state, "dangling", container, placement);
  state.pendingIndentAsSpace = false;
}

/** Print leading/trailing comments for a native owner folded onto this node. */
export function printContainerComments(
  node: CommentNode,
  state: State,
  kind: string,
  placement: Placement,
): void {
  if (!COMMENTS) return;
  printBucket(node, state, "dangling", kind, placement);
  if (placement === "leading") {
    if (state.commentLastChar === "\n") printIndent(state);
    else if (state.pendingIndentAsSpace) {
      state.pendingIndentAsSpace = false;
      if (!" \t\n".includes(state.commentLastChar)) write(state, " ", CAT_OTHER);
    }
  }
}

function printBucket(
  node: CommentNode,
  state: State,
  placement: Placement,
  container?: string,
  originalPlacement: Placement = "dangling",
): void {
  const comments = node.comments?.[placement];
  if (comments == null) return;
  for (const comment of comments) {
    if (
      container === undefined
        ? comment.container != null
        : comment.container?.kind !== container || comment.container.placement !== originalPlacement
    ) {
      continue;
    }
    const printed = (state.printedComments ??= new Set());
    if (printed.has(comment)) continue;
    printed.add(comment);
    printComment(comment, state);
  }
}

/** Format using transferred newline hints and the whitespace already emitted. */
function printComment(comment: ESTree.Comment, state: State): void {
  state.pendingIndentAsSpace = false;
  const newlines = comment.newlines ?? 0;
  if ((newlines & 1) !== 0 && !state.commentLineStart) {
    if (state.commentPendingWhitespace !== "") {
      if (SOURCEMAPS && state.mapPositions !== null) {
        const { length } = state.commentPendingWhitespace;
        const end = state.spilledOutputLength + state.output.length + length;
        // Rust records generated line/column immediately. Keep mappings in the
        // removed spacing on their original line, including columns past its end.
        for (let index = state.mapPositionsLen - 2; index >= 0; index -= 2) {
          if (state.mapPositions[index] <= end - length) break;
          (state.commentMapCorrections ??= new Map()).set(index, length);
        }
      }
      state.commentPendingWhitespace = "";
      state.commentLastChar = state.commentBeforeWhitespace;
      if (DEBUG) state.lastCharWritten = state.commentBeforeWhitespace;
    }
    write(state, "\n", CAT_OTHER);
  }
  const last = state.commentLastChar;
  if (last === "" || last === "\n") printIndent(state);
  else if (!" \t([{".includes(last)) write(state, " ", CAT_OTHER);

  const kind =
    comment.kind
    ?? (comment.type === "Line" ? 0 : /[\n\r\u2028\u2029]/.test(comment.value) ? 2 : 1);
  let text =
    kind === 0
      ? `//${comment.value}`
      : kind === 4
        ? `<!--${comment.value}`
        : kind === 3
          ? `${state.commentLineStart ? "-->" : "//"}${comment.value}`
          : `/*${comment.value}*/`;
  text = escapeScriptCloseTag(text);
  if (kind === 2) {
    for (const line of text.split(/\r\n|[\n\r\u2028\u2029]/)) {
      if (!line.startsWith("/*")) printIndent(state);
      const trimmed = line.trimStart();
      if (trimmed.length !== 0) writeCommentText(state, trimmed);
      if (!line.endsWith("*/")) write(state, "\n", CAT_OTHER);
    }
  } else {
    // Line comment contents may end in any token category; newline is part of this write.
    writeCommentText(state, kind === 1 ? text : `${text}\n`);
  }
  if (kind === 0 || kind === 3 || kind === 4 || (newlines & 2) !== 0) {
    if (state.commentLastChar !== "\n") write(state, "\n", CAT_OTHER);
  } else {
    state.pendingIndentAsSpace = true;
  }
}

/** Restricted productions need parentheses if the leftmost operand begins with a newline. */
export function expressionStartsWithCommentNewline(node: ESTree.Expression, state: State): boolean {
  if (!COMMENTS) return false;
  for (;;) {
    if (
      node.comments?.leading?.some(
        (c) =>
          !state.printedComments?.has(c)
          && (c.type === "Line" || c.kind === 2 || ((c.newlines ?? 0) & 3) !== 0),
      )
    ) {
      return true;
    }
    switch (node.type) {
      case "BinaryExpression":
      case "LogicalExpression":
        node = node.left as ESTree.Expression;
        break;
      case "ConditionalExpression":
        node = node.test;
        break;
      case "CallExpression":
        node = node.callee;
        break;
      case "MemberExpression":
        node = node.object;
        break;
      case "SequenceExpression":
        if (node.expressions.length === 0) return false;
        node = node.expressions[0];
        break;
      /* IF TS */
      case "TSAsExpression":
      case "TSSatisfiesExpression":
      case "TSNonNullExpression":
      case "TSInstantiationExpression":
        node = node.expression;
        break;
      /* END_IF */
      default:
        return false;
    }
  }
}

/** Comment text is not a JavaScript token; leave a fresh separator category. */
function writeCommentText(state: State, text: string): void {
  writeNoLast(state, text);
  state.last = text.endsWith("/") ? CAT_REGEX_SLASH : CAT_OTHER;
  if (DEBUG) state.lastIsStale = false;
}

/**
 * Rust retains explicit parentheses whenever this print has attached comments.
 * Usually a leading owner has already answered this. Probe lazily at the first
 * parenthesis otherwise, stopping at the first bucket and caching the answer.
 * This only checks presence; placement still comes directly from owners.
 */
export function hasAttachedComments(state: State): boolean {
  if (!COMMENTS) return false;
  if (state.hasAttachedComments !== null) return state.hasAttachedComments;
  const pending: unknown[] = [state.commentRoot];
  const seen = new Set<object>();
  while (pending.length !== 0) {
    const value = pending.pop();
    if (value === null || typeof value !== "object" || seen.has(value)) continue;
    seen.add(value);
    if ("comments" in value && value.comments != null) return (state.hasAttachedComments = true);
    for (const [key, child] of Object.entries(value)) {
      if (key !== "parent" && key !== "comments" && child !== null && typeof child === "object") {
        pending.push(child);
      }
    }
  }
  return (state.hasAttachedComments = false);
}

/** Keep comments on syntax elided by a direct printer available for Rust's EOF fallback. */
export function rememberComments(node: CommentNode, state: State): void {
  if (!COMMENTS) return;
  if (node.comments == null) return;
  const pending = (state.unclaimedComments ??= new Set());
  for (const bucket of Object.values(node.comments)) {
    if (bucket !== null) for (const comment of bucket) pending.add(comment);
  }
}

export function printUnclaimedComments(state: State): void {
  if (!COMMENTS) return;
  if (state.unclaimedComments === null) return;
  const pending = [...state.unclaimedComments].filter(
    (c) =>
      !state.printedComments?.has(c)
      && c.container?.kind !== "Elision"
      && c.content !== 4
      && c.content !== 6,
  );
  pending.sort((a, b) => a.start - b.start);
  for (const comment of pending) {
    (state.printedComments ??= new Set()).add(comment);
    printComment({ ...comment, newlines: 3 }, state);
  }
}
