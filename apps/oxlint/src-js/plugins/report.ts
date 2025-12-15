/*
 * `report` function to report errors + diagnostic types.
 */

import { filePath } from "./context.ts";
import { getFixes } from "./fix.ts";
import { getOffsetFromLineColumn } from "./location.ts";
import { typeAssertIs } from "../utils/asserts.ts";

import type { RequireAtLeastOne } from "type-fest";
import type { Fix, FixFn } from "./fix.ts";
import type { RuleDetails } from "./load.ts";
import type { LineColumn, Ranged } from "./location.ts";

const { hasOwn, keys: ObjectKeys } = Object;

/**
 * Diagnostic object.
 * Passed to `Context#report()`.
 *
 * - Either `message` or `messageId` property must be provided.
 * - Either `node` or `loc` property must be provided.
 */
// This is the type of the value passed to `Context#report()` by user.
// `DiagnosticReport` (see below) is the type of diagnostics used internally on JS side, and sent to Rust.
export type Diagnostic = RequireAtLeastOne<
  RequireAtLeastOne<DiagnosticBase, "node" | "loc">,
  "message" | "messageId"
>;

interface DiagnosticBase {
  message?: string | null | undefined;
  messageId?: string | null | undefined;
  node?: Ranged;
  loc?: LocationWithOptionalEnd | LineColumn;
  data?: DiagnosticData | null | undefined;
  fix?: FixFn;
  suggest?: Suggestion[];
}

/**
 * Location with `end` property optional.
 */
interface LocationWithOptionalEnd {
  start: LineColumn;
  end?: LineColumn | null | undefined;
}

/**
 * Data to interpolate into a diagnostic message.
 */
export type DiagnosticData = Record<string, string | number>;

/**
 * Suggested fix.
 * NOT IMPLEMENTED YET.
 */
export type Suggestion = RequireAtLeastOne<SuggestionBase, "desc" | "messageId">;

interface SuggestionBase {
  desc?: string;
  messageId?: string;
  fix: FixFn;
  data?: DiagnosticData | null | undefined;
}

// Diagnostic in form sent to Rust.
// Actually, the `messageId` field is removed before sending to Rust.
export interface DiagnosticReport {
  message: string;
  start: number;
  end: number;
  ruleIndex: number;
  fixes: Fix[] | null;
  messageId: string | null;
}

// Diagnostics array. Reused for every file.
export const diagnostics: DiagnosticReport[] = [];

// Regex for message placeholders.
// https://github.com/eslint/eslint/blob/772c9ee9b65b6ad0be3e46462a7f93c37578cfa8/lib/linter/interpolate.js#L16-L18
export const PLACEHOLDER_REGEX = /\{\{([^{}]+)\}\}/gu;

/**
 * Report error.
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific details e.g. `isFixable`
 * @throws {TypeError} If `diagnostic` is invalid
 */
export function report(diagnostic: Diagnostic, ruleDetails: RuleDetails): void {
  if (filePath === null) throw new Error("Cannot report errors in `createOnce`");

  // Get message, resolving message from `messageId` if present
  let { message, messageId } = getMessage(diagnostic, ruleDetails);

  // Interpolate placeholders {{key}} with data values
  if (hasOwn(diagnostic, "data")) {
    const { data } = diagnostic;
    if (data != null) message = replacePlaceholders(message, data);
  }

  // TODO: Validate `diagnostic`
  let start: number, end: number, loc: LocationWithOptionalEnd | LineColumn | undefined;

  if (hasOwn(diagnostic, "loc") && (loc = diagnostic.loc) != null) {
    // `loc`
    // Can be any of:
    // * `{ start: { line, column }, end: { line, column } }`
    // * `{ start: { line, column }, end: null }`
    // * `{ start: { line, column }, end: undefined }`
    // * `{ start: { line, column } }`
    // * `{ line, column }`
    if (typeof loc !== "object") throw new TypeError("`loc` must be an object if provided");

    if (hasOwn(loc, "start")) {
      typeAssertIs<LocationWithOptionalEnd>(loc);
      start = getOffsetFromLineColumn(loc.start);
      end = loc.end == null ? start : getOffsetFromLineColumn(loc.end);
    } else {
      typeAssertIs<LineColumn>(loc);
      start = getOffsetFromLineColumn(loc);
      end = start;
    }
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
    if (range === null || typeof range !== "object") {
      throw new TypeError("`node.range` must be present");
    }
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
    messageId,
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
 * @returns Message string and `messageId`
 * @throws {Error|TypeError} If neither `message` nor `messageId` provided, or of wrong type
 */
function getMessage(
  diagnostic: Diagnostic,
  ruleDetails: RuleDetails,
): { message: string; messageId: string | null } {
  if (hasOwn(diagnostic, "messageId")) {
    const { messageId } = diagnostic;
    if (messageId != null) {
      return {
        message: resolveMessageFromMessageId(messageId, ruleDetails),
        messageId,
      };
    }
  }

  if (hasOwn(diagnostic, "message")) {
    const { message } = diagnostic;
    if (typeof message === "string") return { message, messageId: null };
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

/**
 * Replace placeholders in message with values from `data`.
 * @param message - Message
 * @param data - Data to replace placeholders with
 * @returns Message with placeholders replaced with data values
 */
export function replacePlaceholders(message: string, data: DiagnosticData): string {
  return message.replace(PLACEHOLDER_REGEX, (match, key) => {
    key = key.trim();
    const value = data[key];
    // TS type def for `string.replace` callback is `(substring: string, ...args: any[]) => string`,
    // but actually returning other types e.g. `number` or `boolean` is fine
    return value !== undefined ? (value as string) : match;
  });
}
