import { Comment } from "./comments";
import { getAllComments } from "./comments_methods";
import { Location } from "./location";

interface Problem {
  ruleId: string | null;
  message: string;
  loc: Location;
}

type DirectiveType = "disable" | "disable-line" | "disable-next-line";

interface Directive {
  type: DirectiveType;
  node: Comment;
  value: string;
  justification?: string;
}

const LABEL_PATTERN = /^(?<label>(?:eslint|oxlint)-disable(?:(?:-next)?-line)?)(?:\s|$)/u;
const LINE_DIRECTIVE_PATTERN = /^(?:eslint|oxlint)-disable-(?:-next)?-line$/u;
const JUSTIFICATION_SEP_PATTERN = /\s-{2,}\s/u;

export function getDisableDirectives() {
  const problems: Problem[] = [];
  const directives: Directive[] = [];

  getAllComments().forEach((comment) => {
    if (comment.type === "Shebang") return;

    let match = LABEL_PATTERN.exec(comment.value);
    if (!match?.groups?.label) return;

    // Only some comment types are supported as line comments
    if (comment.type === "Line" && LINE_DIRECTIVE_PATTERN.test(match.groups.label)) return;

    const { label } = match.groups;

    // Validate directive does not span multiple lines
    if (
      (label === "eslint-disable-line" || label === "oxlint-disable-line") &&
      comment.loc.start.line !== comment.loc.end.line
    ) {
      problems.push({
        ruleId: null,
        message: `${label} comment should not span multiple lines.`,
        loc: comment.loc,
      });
      return;
    }

    const rest = comment.value.slice(label.length).trim();
    match = JUSTIFICATION_SEP_PATTERN.exec(rest);

    const [value, justification] = match
      ? [rest.slice(0, match.index).trim(), rest.slice(match.index + match[0].length).trim()]
      : [rest, ""];

    directives.push({
      // oxlint- and eslint- are each 7 characters
      type: label.slice(7) as DirectiveType,
      node: comment,
      value,
      justification,
    });
  });

  return { problems, directives };
}
