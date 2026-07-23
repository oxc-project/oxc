/**
 * Parses disable/enable directive comments (e.g., `eslint-disable`, `oxlint-enable`).
 *
 * Supported patterns match either:
 * - ESLint: `eslint-disable`, `eslint-disable-line`, `eslint-disable-next-line`, `eslint-enable`
 * - Oxlint: `oxlint-disable`, `oxlint-disable-line`, `oxlint-disable-next-line`, `oxlint-enable`
 *
 * The pattern matching mirrors the Rust implementation in
 * `crates/oxc_linter/src/disable_directives.rs` (`match_directive`).
 * This ensures consistent behavior between the Rust linter and JS plugins.
 *
 * @see <https://oxc.rs/docs/guide/usage/linter/ignore-comments.html#inline-ignore-comments>
 */

import {
  BLOCK_COMMENT_KINDS_BITMAP,
  cachedComments,
  COMMENT_KIND_MASK,
  COMMENT_KIND_OFFSET32,
  COMMENT_SIZE32_SHIFT,
  commentsInt32,
  commentsLen,
  initCommentsBuffer,
} from "./comments.ts";
import { COMMENT_SHEBANG_KIND } from "../generated/constants.ts";
import { sourceText } from "./source_code.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Comment } from "./comments.ts";
import type { Location } from "./location.ts";

interface Problem {
  ruleId: string | null;
  message: string;
  loc: Location;
}

type DirectiveType = "disable" | "enable" | "disable-line" | "disable-next-line";

interface Directive {
  type: DirectiveType;
  node: Comment;
  value: string;
  justification: string;
}

// Patterns capturing the whole directive label (e.g. `oxlint-disable-line`) and its type (`disable-line`).
// Both are sticky, so they're matched against source text at the position the comment's text starts,
// without slicing the comment out of source text first. One pattern for each comment kind -
// they differ only in what may precede and follow the label.
//
// The trailing lookahead requires the label to be followed by whitespace, or to end the comment,
// mirroring `is_directive_end` on the Rust side. It stops typos such as `oxlint-disablefoo`
// being treated as directives.
//
// Neither pattern can match past the end of the comment it's anchored in:
//
// * Leading whitespace stops before the comment's end - at `*/` for block comments (`*` is not whitespace),
//   and at the line break for line comments (which the `[^\S...]` class excludes).
// * Labels contain none of those characters, so cannot span the end either.
//
// So `lastIndex` after a match is always within the comment, and no bounds check against its end is needed.

// Line comments end at a line break, so the label can be followed by whitespace or the end of the file.
// The leading whitespace class is every whitespace character *except* line breaks - `\s*` would run past
// the end of the comment, through the indentation of the lines below, which is wasted work on the
// (not uncommon) empty `//` comment in a run of line comments.
const LINE_LABEL_PATTERN =
  /[^\S\n\r\u2028\u2029]*((?:eslint|oxlint)-(disable(?:(?:-next)?-line)?|enable))(?=\s|$)/uy;

// Block comments always end `*/` (an unterminated one is a parse error, so never reaches here),
// so that's the only non-whitespace which can follow the label.
const BLOCK_LABEL_PATTERN =
  /\s*((?:eslint|oxlint)-(disable(?:(?:-next)?-line)?|enable))(?=\s|\*\/)/uy;

const JUSTIFICATION_SEP_PATTERN = /\s-{2,}\s/u;

export function getDisableDirectives(): { problems: Problem[]; directives: Directive[] } {
  const problems: Problem[] = [];
  const directives: Directive[] = [];

  if (commentsInt32 === null) initCommentsBuffer();
  debugAssertIsNonNull(commentsInt32);
  debugAssertIsNonNull(sourceText);

  // Early exit for files with no comments
  if (commentsLen === 0) return { problems, directives };

  // Skip `Shebang` comment. Only the first comment can be one.
  let index =
    (commentsInt32[COMMENT_KIND_OFFSET32] & COMMENT_KIND_MASK) === COMMENT_SHEBANG_KIND ? 1 : 0;

  for (; index < commentsLen; index++) {
    const pos32 = index << COMMENT_SIZE32_SHIFT;

    // Select the pattern for this comment's kind.
    // `endSubtract` is how much shorter than the comment its text is - 2 for block comments (`*/`),
    // 0 for line comments. It's used as the "is a block comment" test below too.
    const kind = commentsInt32[pos32 + COMMENT_KIND_OFFSET32] & COMMENT_KIND_MASK;
    const endSubtract = (BLOCK_COMMENT_KINDS_BITMAP >> kind) & 2;
    const pattern = endSubtract === 0 ? LINE_LABEL_PATTERN : BLOCK_LABEL_PATTERN;

    // Match the directive label against source text directly, anchored at the start of the comment's text.
    // Comments which aren't directives - the vast majority - are rejected without slicing them out of source text.
    pattern.lastIndex = commentsInt32[pos32] + 2;
    const match = pattern.exec(sourceText);
    if (match === null) continue;

    const labelEnd = pattern.lastIndex;
    const textEnd = commentsInt32[pos32 + 1] - endSubtract;
    debugAssert(labelEnd <= textEnd, "Directive label matched beyond the end of the comment");

    const type = match[2] as DirectiveType;
    const comment = cachedComments[index];

    // Validate `disable-line` directive does not span multiple lines.
    // Only block comments can - `Shebang` comments are skipped above, and line comments end at the line's end.
    // `endSubtract` is 2 for block comments, 0 for line comments.
    if (endSubtract !== 0 && type === "disable-line") {
      const { loc } = comment;
      if (loc.start.line !== loc.end.line) {
        problems.push({
          ruleId: null,
          message: `${match[1]} comment should not span multiple lines.`,
          loc,
        });
        continue;
      }
    }

    // Split text after the directive into rule list and justification (`rules -- justification`)
    let value = sourceText.slice(labelEnd, textEnd).trim();
    let justification = "";

    const sepMatch = JUSTIFICATION_SEP_PATTERN.exec(value);
    if (sepMatch !== null) {
      const splitPoint = sepMatch.index;
      justification = value.slice(splitPoint + sepMatch[0].length).trimStart();
      value = value.slice(0, splitPoint).trimEnd();
    }

    directives.push({ type, node: comment, value, justification });
  }

  return { problems, directives };
}
