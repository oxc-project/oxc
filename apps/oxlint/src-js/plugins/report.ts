/*
 * `report` function to report errors + diagnostic types.
 */

import { filePath } from "./context.ts";
import { getFixes, getSuggestions } from "./fix.ts";
import { initLines, lines, lineStartIndices, debugAssertLinesIsInitialized } from "./location.ts";
import { sourceText } from "./source_code.ts";
import { debugAssertIsNonNull, typeAssertIs } from "../utils/asserts.ts";

import type { RequireAtLeastOne } from "type-fest";
import type { FixFn, FixReport } from "./fix.ts";
import type { RuleDetails } from "./load.ts";
import type { LineColumn, Ranged } from "./location.ts";

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
  suggest?: Suggestion[] | null | undefined;
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
export type DiagnosticData = Record<string, string | number | boolean | bigint | null | undefined>;

/**
 * Suggested fix.
 */
export type Suggestion = RequireAtLeastOne<SuggestionBase, "desc" | "messageId">;

interface SuggestionBase {
  desc?: string;
  messageId?: string;
  data?: DiagnosticData | null | undefined;
  fix: FixFn;
}

/**
 * Suggested fix in form sent to Rust.
 */
export interface SuggestionReport {
  message: string;
  // Not needed on Rust side, but `RuleTester` needs it
  messageId: string | null;
  fixes: FixReport[];
}

/**
 * Diagnostic in form sent to Rust.
 */
export interface DiagnosticReport {
  message: string;
  start: number;
  end: number;
  ruleIndex: number;
  fixes: FixReport[] | null;
  suggestions: SuggestionReport[] | null;
  // Not needed on Rust side, but `RuleTester` needs it
  messageId: string | null;
  // Only used in conformance tests. This field is not present except in conformance build.
  loc?: LocationWithOptionalEnd | null;
}

// Diagnostics array. Reused for every file.
export const diagnostics: DiagnosticReport[] = [];

// Regex for message placeholders.
// https://github.com/eslint/eslint/blob/772c9ee9b65b6ad0be3e46462a7f93c37578cfa8/lib/linter/interpolate.js#L16-L18
export const PLACEHOLDER_REGEX = /\{\{([^{}]+)\}\}/gu;

/**
 * Report error.
 * @param diagnostic - Diagnostic object
 * @param extraArgs - Extra arguments passed to `context.report()` (legacy positional forms)
 * @param ruleDetails - `RuleDetails` object, containing rule-specific details e.g. `isFixable`
 * @throws {TypeError} If `diagnostic` is invalid
 */
export function report(
  diagnostic: Diagnostic,
  extraArgs: unknown[],
  ruleDetails: RuleDetails,
): void {
  if (filePath === null) throw new Error("Cannot report errors in `createOnce`");

  // Handle legacy positional forms
  if (extraArgs.length > 0) diagnostic = convertLegacyCallArgs(diagnostic, extraArgs);

  const { message, messageId } = getMessage(
    Object.hasOwn(diagnostic, "message") ? diagnostic.message : null,
    diagnostic,
    ruleDetails,
  );

  // TODO: Validate `diagnostic`
  let start: number, end: number, loc: LocationWithOptionalEnd | LineColumn | undefined;
  // We need the original location in conformance tests
  let conformedLoc: LocationWithOptionalEnd | null = null;

  if (Object.hasOwn(diagnostic, "loc") && (loc = diagnostic.loc) != null) {
    // `loc`
    // Can be any of:
    // * `{ start: { line, column }, end: { line, column } }`
    // * `{ start: { line, column }, end: null }`
    // * `{ start: { line, column }, end: undefined }`
    // * `{ start: { line, column } }`
    // * `{ line, column }`
    if (typeof loc !== "object") throw new TypeError("`loc` must be an object if provided");

    if (Object.hasOwn(loc, "start")) {
      typeAssertIs<LocationWithOptionalEnd>(loc);
      const { start: startLineCol, end: endLineCol } = loc;

      if (startLineCol === null || typeof startLineCol !== "object") {
        throw new TypeError("`loc.start` must be an object");
      }
      start = getOffsetFromLineColumn(startLineCol);

      if (endLineCol == null) {
        end = start;
      } else if (typeof endLineCol === "object") {
        end = getOffsetFromLineColumn(endLineCol);
      } else {
        throw new TypeError("`loc.end` must be an object or null/undefined");
      }

      if (CONFORMANCE) conformedLoc = loc;
    } else {
      typeAssertIs<LineColumn>(loc);
      start = getOffsetFromLineColumn(loc);
      end = start;

      if (CONFORMANCE) conformedLoc = { start: loc, end: null };
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
    suggestions: getSuggestions(diagnostic, ruleDetails),
  });

  // We need the original location in conformance tests
  if (CONFORMANCE) diagnostics.at(-1)!.loc = conformedLoc;
}

/**
 * Convert legacy `context.report()` arguments to a `Diagnostic` object.
 *
 * Supported:
 * - `context.report(node, message, data?, fix?)`
 * - `context.report(node, loc, message, data?, fix?)`
 *
 * @param node - Node to report (first argument)
 * @param extraArgs - Extra arguments passed to `context.report()`
 * @returns Diagnostic object
 */
function convertLegacyCallArgs(node: unknown, extraArgs: unknown[]): Diagnostic {
  const firstExtraArg = extraArgs[0];
  if (typeof firstExtraArg === "string") {
    // `context.report(node, message, data, fix)`
    return {
      message: firstExtraArg,
      node,
      loc: undefined,
      data: extraArgs[1],
      fix: extraArgs[2],
    } as Diagnostic;
  }

  // `context.report(node, loc, message, data, fix)`
  return {
    message: extraArgs[1],
    node,
    loc: firstExtraArg,
    data: extraArgs[2],
    fix: extraArgs[3],
  } as Diagnostic;
}

/**
 * Get message from a diagnostic or suggestion.
 *
 * Resolve message from `messageId` if present, and interpolate placeholders {{key}} with data values.
 *
 * @param message - Provided message string
 * @param descriptor - `Diagnostic` or `Suggestion` object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific `messages`
 * @returns Object containing message string and message ID (if present in `descriptor`)
 * @throws {Error|TypeError} If neither `message` nor `messageId` provided, or of wrong type
 */
export function getMessage(
  message: string | null | undefined,
  descriptor: Diagnostic | Suggestion,
  ruleDetails: RuleDetails,
): { message: string; messageId: string | null } {
  // Resolve from `messageId` if present, otherwise use `message`
  let messageId: string | null = null;
  if (Object.hasOwn(descriptor, "messageId")) messageId = descriptor.messageId ?? null;

  if (messageId !== null) {
    if (typeof messageId !== "string") throw new TypeError("`messageId` must be a string");
    message = resolveMessageFromMessageId(messageId, ruleDetails);
  } else if (message == null) {
    throw new Error("Either `message` or `messageId` is required");
  } else if (typeof message !== "string") {
    throw new TypeError("`message` must be a string");
  }

  // Interpolate data placeholders
  if (Object.hasOwn(descriptor, "data")) {
    const { data } = descriptor;
    if (data != null) message = replacePlaceholders(message, data);
  }

  return { message, messageId };
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

  if (!Object.hasOwn(messages, messageId)) {
    throw new Error(
      `Unknown messageId '${messageId}'. Available \`messageIds\`: ${Object.keys(messages)
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

/**
 * Convert a `{ line, column }` pair into a range index.
 *
 * Same as `getOffsetFromLineColumn` in `location.ts`, except:
 * 1. Does not check that `lineCol` is an object - caller must do that.
 * 2. Allows `column` to be less than 0 or greater than the length of the line.
 *
 * Relaxing the restriction on `column` is required because some rules (e.g. ESLint's `func-call-spacing`)
 * produce `column: -1` in some cases.
 *
 * @param lineCol - A line/column location.
 * @returns The character index of the location in the file.
 * @throws {TypeError} If `lineCol` is not an object with a integer `line` and `column`.
 * @throws {RangeError} If `line` is less than or equal to 0, or greater than the number of lines in the source text.
 * @throws {RangeError} If computed offset is out of range of the source text.
 */
function getOffsetFromLineColumn(lineCol: LineColumn): number {
  const { line, column } = lineCol;
  if (
    typeof line !== "number" ||
    typeof column !== "number" ||
    (line | 0) !== line ||
    (column | 0) !== column
  ) {
    throw new TypeError("Expected an object with integer `line` and `column` properties");
  }

  // Build `lines` and `lineStartIndices` tables if they haven't been already.
  // This also decodes `sourceText` if it wasn't already.
  if (lines.length === 0) initLines();
  debugAssertIsNonNull(sourceText);
  debugAssertLinesIsInitialized();

  if (line <= 0 || line > lineStartIndices.length) {
    if (column === 0) {
      // Allow `line` to be 0 if `column` is 0
      if (line === 0) return 0;

      // Allow `line` to be 1 greater than the number of lines in the file if `column` is 0
      if (line === lineStartIndices.length + 1) return sourceText.length;
    }

    throw new RangeError(
      `Line number out of range (line ${line} requested). ` +
        `Line numbers should be 1-based, and less than or equal to number of lines in file (${lineStartIndices.length}).`,
    );
  }

  const lineOffset = lineStartIndices[line - 1];
  const offset = lineOffset + column;

  // Ensure offset is within bounds.
  // Do this here on JS side to prevent a NAPI error when converting to `u32` on Rust side.
  if (offset < 0 || offset > sourceText.length) {
    throw new RangeError("Line/column pair translates to an out of range offset");
  }

  return offset;
}
