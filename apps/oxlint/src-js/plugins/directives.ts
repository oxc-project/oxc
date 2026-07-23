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

import { Comment } from "./comments";
import { getAllComments } from "./comments_methods";
import { Location } from "./location";

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

// Captures the whole directive label (e.g. `oxlint-disable-line`) and its type (`disable-line`)
const LABEL_PATTERN = /^\s*((?:eslint|oxlint)-(disable(?:(?:-next)?-line)?|enable))(?:\s|$)/u;
const JUSTIFICATION_SEP_PATTERN = /\s-{2,}\s/u;

export function getDisableDirectives(): { problems: Problem[]; directives: Directive[] } {
  const problems: Problem[] = [];
  const directives: Directive[] = [];

  const comments = getAllComments();

  // Skip `Shebang` comment
  let i = comments.length > 0 && comments[0].type === "Shebang" ? 1 : 0;

  for (; i < comments.length; i++) {
    const comment = comments[i];

    const match = LABEL_PATTERN.exec(comment.value);
    if (match === null) continue;

    const label = match[1];
    const type = match[2] as DirectiveType;

    // Validate directive does not span multiple lines
    if (type === "disable-line" && comment.loc.start.line !== comment.loc.end.line) {
      problems.push({
        ruleId: null,
        message: `${label} comment should not span multiple lines.`,
        loc: comment.loc,
      });
      continue;
    }

    // Split text after the directive into rule list and justification (`rules -- justification`)
    const rest = comment.value.slice(match[0].length).trim();
    const sepMatch = JUSTIFICATION_SEP_PATTERN.exec(rest);

    let value = rest;
    let justification = "";
    if (sepMatch !== null) {
      value = rest.slice(0, sepMatch.index).trim();
      justification = rest.slice(sepMatch.index + sepMatch[0].length).trim();
    }

    directives.push({ type, node: comment, value, justification });
  }

  return { problems, directives };
}
