import { printIndent } from "./indent.ts";
import { CAT_OTHER, write, writeNoLast } from "./write.ts";

import type { State } from "../state.ts";
import type { Comment } from "./options.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

export function printLeadingComments(node: ESTree.Node, state: State): void {
  const comments = state.commentItems;
  if (comments === null || typeof node.start !== "number") return;

  while (state.commentIndex < comments.length) {
    const comment = comments[state.commentIndex];
    if (comment.end > node.start) return;
    state.commentIndex++;
    if (isSyntheticHashbang(comment, state)) continue;
    printComment(comment, node.start, state);
  }
}

export function printRemainingComments(state: State): void {
  const comments = state.commentItems;
  if (comments === null) return;
  while (state.commentIndex < comments.length) {
    const comment = comments[state.commentIndex++];
    if (!isSyntheticHashbang(comment, state)) printComment(comment, comment.end, state);
  }
}

function isSyntheticHashbang(comment: Comment, state: State): boolean {
  return comment.start === 0 && state.sourceText?.startsWith("#!") === true;
}

function printComment(comment: Comment, nextStart: number, state: State): void {
  const sourceText = state.sourceText;
  const raw =
    sourceText === null
      ? comment.type === "Line"
        ? `//${comment.value}`
        : `/*${comment.value}*/`
      : sourceText.slice(comment.start, comment.end);

  if (state.output.length !== 0 && !state.output.endsWith("\n")) write(state, " ", CAT_OTHER);
  writeNoLast(state, raw);

  const gap = sourceText?.slice(comment.end, nextStart);
  if (comment.type === "Line" || gap?.includes("\n") === true || gap?.includes("\r") === true) {
    write(state, "\n", CAT_OTHER);
    printIndent(state);
  } else {
    write(state, " ", CAT_OTHER);
  }
}
