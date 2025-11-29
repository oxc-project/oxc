/*
 * `report` function to report errors + diagnostic types.
 */

import { filePath } from "./context.js";
import { getFixes } from "./fix.js";
import { getOffsetFromLineColumn } from "./location.js";

import type { RequireAtLeastOne } from "type-fest";
import type { Fix, FixFn } from "./fix.ts";
import type { RuleDetails } from "./load.ts";
import type { Location, Ranged } from "./location.ts";

const { hasOwn, keys: ObjectKeys } = Object;

/**
 * Diagnostic object.
 * Passed to `Context#report()`.
 *
 * - Either `message` or `messageId` property must be provided.
 * - Either `node` or `loc` property must be provided.
 */
// This is the type of the value passed to `Context#report()` by user.
// `DiagnosticReport` (see below) is the type of diagnostics sent to Rust.
export type Diagnostic = RequireAtLeastOne<
  RequireAtLeastOne<DiagnosticBase, "node" | "loc">,
  "message" | "messageId"
>;

interface DiagnosticBase {
  message?: string | null | undefined;
  messageId?: string | null | undefined;
  node?: Ranged;
  loc?: Location;
  data?: Record<string, string | number> | null | undefined;
  fix?: FixFn;
  suggest?: Suggestion[];
}

/**
 * Suggested fix.
 * NOT IMPLEMENTED YET.
 */
export type Suggestion = RequireAtLeastOne<SuggestionBase, "desc" | "messageId">;

interface SuggestionBase {
  desc?: string;
  messageId?: string;
  fix: FixFn;
  data?: Record<string, string | number> | null | undefined;
}

// Diagnostic in form sent to Rust
interface DiagnosticReport {
  message: string;
  start: number;
  end: number;
  ruleIndex: number;
  fixes: Fix[] | null;
}

// Diagnostics array. Reused for every file.
export const diagnostics: DiagnosticReport[] = [];

/**
 * Report error.
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific details e.g. `isFixable`
 * @throws {TypeError} If `diagnostic` is invalid
 */
export function report(diagnostic: Diagnostic, ruleDetails: RuleDetails): void {
  if (filePath === null) throw new Error("Cannot report errors in `createOnce`");

  // Get message, resolving message from `messageId` if present
  let message = getMessage(diagnostic, ruleDetails);

  // Interpolate placeholders {{key}} with data values
  if (hasOwn(diagnostic, "data")) {
    const { data } = diagnostic;
    if (data != null) {
      message = message.replace(/\{\{([^}]+)\}\}/g, (match, key) => {
        key = key.trim();
        const value = data[key];
        return value !== undefined ? String(value) : match;
      });
    }
  }

  // TODO: Validate `diagnostic`
  let start: number, end: number, loc: Location | undefined;

  if (hasOwn(diagnostic, "loc") && (loc = diagnostic.loc) != null) {
    // `loc`
    if (typeof loc !== "object") throw new TypeError("`loc` must be an object");
    start = getOffsetFromLineColumn(loc.start);
    end = getOffsetFromLineColumn(loc.end);
  } else {
    // `node`
    const { node } = diagnostic;
    if (node == null) throw new TypeError("Either `node` or `loc` is required");
    if (typeof node !== "object") throw new TypeError("`node` must be an object");

    // ESLint uses `loc` here instead of `range`.
    // We can't do that because AST nodes don't have `loc` property yet. In any case, `range` is preferable,
    // as otherwise we have to convert `loc` to `range` which is expensive at present.
    // TODO: Revisit this once we have `loc` support in AST, and a fast translation table to convert `loc` to `range`.
    const { range } = node;
    if (range === null || typeof range !== "object")
      throw new TypeError("`node.range` must be present");
    start = range[0];
    end = range[1];

    // Do type validation checks here, to ensure no error in serialization / deserialization.
    // Range validation happens on Rust side.
    if (
      typeof start !== "number" ||
      typeof end !== "number" ||
      start < 0 ||
      end < 0 ||
      (start | 0) !== start ||
      (end | 0) !== end
    ) {
      throw new TypeError("`node.range[0]` and `node.range[1]` must be non-negative integers");
    }
  }

  diagnostics.push({
    message,
    start,
    end,
    ruleIndex: ruleDetails.ruleIndex,
    fixes: getFixes(diagnostic, ruleDetails),
  });
}

/**
 * Get message from diagnostic.
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific `messages`
 * @returns Message string
 * @throws {Error|TypeError} If neither `message` nor `messageId` provided, or of wrong type
 */
function getMessage(diagnostic: Diagnostic, ruleDetails: RuleDetails): string {
  if (hasOwn(diagnostic, "messageId")) {
    const { messageId } = diagnostic;
    if (messageId != null) return resolveMessageFromMessageId(messageId, ruleDetails);
  }

  if (hasOwn(diagnostic, "message")) {
    const { message } = diagnostic;
    if (typeof message === "string") return message;
    if (message != null) throw new TypeError("`message` must be a string");
  }

  throw new Error("Either `message` or `messageId` is required");
}

/**
 * Resolve a message ID to its message string, with optional data interpolation.
 * @param messageId - The message ID to resolve
 * @param ruleDetails - `RuleDetails` object, containing rule-specific `messages`
 * @returns Resolved message string
 * @throws {Error} If `messageId` is not found in `messages`
 */
function resolveMessageFromMessageId(messageId: string, ruleDetails: RuleDetails): string {
  const { messages } = ruleDetails;
  if (messages === null) {
    throw new Error(
      `Cannot use messageId '${messageId}' - rule does not define any messages in \`meta.messages\``,
    );
  }

  if (!hasOwn(messages, messageId)) {
    throw new Error(
      `Unknown messageId '${messageId}'. Available \`messageIds\`: ${ObjectKeys(messages)
        .map((msg) => `'${msg}'`)
        .join(", ")}`,
    );
  }

  return messages[messageId];
}
