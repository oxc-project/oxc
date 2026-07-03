/*
 * Lint a file which is parsed by a custom (JS) parser.
 *
 * Mirrors the structure of `lint.ts` `lintFile` / `lintFileImpl`, with these differences:
 *
 * - There is no buffer. The source text is sent from Rust as a string,
 *   and the AST is produced by the custom parser configured in `languageOptions.parser`.
 * - `context.sourceCode` returns `JS_PARSER_SOURCE_CODE` (via `setSourceCodeOverride`),
 *   backed by the parser's output, instead of the buffer-based `SOURCE_CODE`.
 * - Rule visitors are compiled into a string-keyed dispatch (`js_ast_walk.ts`),
 *   which can handle node types unknown to Oxc (e.g. Ember's `GlimmerTemplate`).
 * - The result sent to Rust includes the spans of all comments in the file
 *   (from the parser output), which drive `eslint-disable` directive handling on Rust side.
 */

import { setupFileContext, setSourceCodeOverride } from "./context.ts";
import { setGlobalsForFile } from "./globals.ts";
import { compileJsVisitors, walkParserAst } from "./js_ast_walk.ts";
import {
  getJsParserVisitorKeys,
  JS_PARSER_SOURCE_CODE,
  resetJsParserSourceCode,
  setupJsParserSourceCode,
} from "./js_parser_source_code.ts";
import { resetFile } from "./lint.ts";
import { registeredRules } from "./load.ts";
import { allOptions, DEFAULT_OPTIONS_ID } from "./options.ts";
import { registeredParsers } from "./parsers.ts";
import { diagnostics } from "./report.ts";
import { setSettingsForFile } from "./settings.ts";
import { setSourceTextForJsParser } from "./source_code.ts";
import { switchWorkspace } from "./workspace.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";
import { getErrorMessage } from "../utils/utils.ts";

import type { AfterHook, Visitor } from "./types.ts";
import type { JsParserParseResult, JsParserToken } from "./parsers.ts";
import type { SourceCode } from "./source_code.ts";

// Comment in form sent to Rust (`JsComment` on Rust side). Spans are UTF-16 offsets.
interface CommentReport {
  isBlock: boolean;
  start: number;
  end: number;
}

// Array of `after` hooks to run after traversal. This array reused for every file.
// Same purpose as `afterHooks` in `lint.ts` (which is module-private, so duplicated here).
const afterHooks: AfterHook[] = [];

// Reusable property descriptor for updating `options` value on rule context objects.
const OPTIONS_DESCRIPTOR: PropertyDescriptor = { value: null };

/**
 * Lint a file which is parsed by a custom (JS) parser.
 *
 * Main logic is in separate function `lintFileWithJsParserImpl`,
 * because V8 cannot optimize functions containing try/catch.
 *
 * @param filePath - Absolute path of file being linted
 * @param sourceText - Source text of the file (BOM already stripped on Rust side)
 * @param parserId - ID of parser to parse the file with
 * @param parserOptionsJSON - Parser options as JSON, or `null` if not configured
 * @param ruleIds - IDs of rules to run on this file
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for this file, as JSON string
 * @param globalsJSON - Globals for this file, as JSON string
 * @param workspaceUri - Workspace URI (`null` in CLI, string in LSP)
 * @returns Diagnostics and comments, or error, serialized to JSON string
 */
export function lintFileWithJsParser(
  filePath: string,
  sourceText: string,
  parserId: number,
  parserOptionsJSON: string | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  workspaceUri: string | null,
): string {
  try {
    const comments = lintFileWithJsParserImpl(
      filePath,
      sourceText,
      parserId,
      parserOptionsJSON,
      ruleIds,
      optionsIds,
      settingsJSON,
      globalsJSON,
      workspaceUri,
    );

    // Note: `messageId` field of `DiagnosticReport` is not needed on Rust side - `serde` skips over it
    const ret = JSON.stringify({ Success: { diagnostics, comments } });

    // Empty `diagnostics` array, so it starts empty when linting next file
    diagnostics.length = 0;

    resetJsParserFile();

    return ret;
  } catch (err) {
    resetStateAfterError();

    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Parse a file with the registered custom parser, and run rules on it.
 *
 * @param filePath - Absolute path of file being linted
 * @param sourceText - Source text of the file
 * @param parserId - ID of parser to parse the file with
 * @param parserOptionsJSON - Parser options as JSON, or `null` if not configured
 * @param ruleIds - IDs of rules to run on this file
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for this file, as JSON string
 * @param globalsJSON - Globals for this file, as JSON string
 * @param workspaceUri - Workspace URI (`null` in CLI, string in LSP)
 * @returns Spans of all comments in the file, in form sent to Rust
 * @throws {Error} If any parameters are invalid, or the parser fails to parse the file
 * @throws {*} If any rule throws
 */
function lintFileWithJsParserImpl(
  filePath: string,
  sourceText: string,
  parserId: number,
  parserOptionsJSON: string | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  workspaceUri: string | null,
): CommentReport[] {
  // Debug asserts that input is valid
  debugAssert(
    typeof filePath === "string" && filePath.length > 0,
    "`filePath` should be a non-empty string",
  );
  debugAssert(typeof sourceText === "string", "`sourceText` should be a string");
  debugAssert(Array.isArray(ruleIds) && ruleIds.length > 0, "`ruleIds` should be non-empty array");
  debugAssert(Array.isArray(optionsIds), "`optionsIds` should be an array");
  debugAssert(
    ruleIds.length === optionsIds.length,
    "`ruleIds` and `optionsIds` should be same length",
  );
  debugAssert(parserId < registeredParsers.length, "Parser ID out of bounds");

  // The order rules run in is indeterminate.
  // To make order predictable in tests, in debug builds, sort rules by ID in ascending order.
  // i.e. rules run in same order as they're defined in plugin.
  let ruleIndexes: number[] | undefined;
  if (DEBUG) {
    const rules = ruleIds.map((ruleId, index) => ({ ruleId, optionsId: optionsIds[index], index }));
    rules.sort((rule1, rule2) => rule1.ruleId - rule2.ruleId);
    ruleIds = rules.map((rule) => rule.ruleId);
    optionsIds = rules.map((rule) => rule.optionsId);
    ruleIndexes = rules.map((rule) => rule.index);
  }

  // Switch to requested workspace.
  // In CLI, `workspaceUri` is `null`, and there's only 1 workspace, so no need to switch.
  if (workspaceUri !== null) switchWorkspace(workspaceUri);
  debugAssertIsNonNull(allOptions, "`allOptions` should be initialized");

  // Pass file path to context module, so `Context`s know what file is being linted
  setupFileContext(filePath);

  // Pass source text to source code module.
  // There is no buffer for this path, but shared code paths (`report.ts` `loc`-based reporting)
  // read `sourceText` from `source_code.ts` and lazily build line tables from it.
  setSourceTextForJsParser(sourceText);

  // Pass settings and globals JSON to modules that handle them
  setSettingsForFile(settingsJSON);
  setGlobalsForFile(globalsJSON);

  // Parse the file with the custom parser
  const parseResult = parseWithJsParser(filePath, sourceText, parserId, parserOptionsJSON);

  // Set up `SourceCode` backed by the parse result,
  // and make `context.sourceCode` return it instead of the buffer-based `SOURCE_CODE`
  setupJsParserSourceCode(parseResult, sourceText);
  setSourceCodeOverride(JS_PARSER_SOURCE_CODE as unknown as SourceCode);

  // Get visitors for this file from all rules
  const visitors: Visitor[] = [];
  for (let i = 0, len = ruleIds.length; i < len; i++) {
    const ruleId = ruleIds[i];
    debugAssert(ruleId < registeredRules.length, "Rule ID out of bounds");
    const ruleDetails = registeredRules[ruleId];

    // Set `ruleIndex` for rule. It's used when sending diagnostics back to Rust.
    // In debug build, use `ruleIndexes`, because `ruleIds` has been re-ordered.
    ruleDetails.ruleIndex = DEBUG ? ruleIndexes![i] : i;

    // Set `options` for rule
    const optionsId = optionsIds[i];
    debugAssert(optionsId < allOptions.length, "Options ID out of bounds");

    // If the rule has no user-provided options, use the plugin-provided default options.
    // Reuse `OPTIONS_DESCRIPTOR` object to avoid unnecessarily creating a temporary object each time.
    OPTIONS_DESCRIPTOR.value =
      optionsId === DEFAULT_OPTIONS_ID ? ruleDetails.defaultOptions : allOptions[optionsId];
    Object.defineProperty(ruleDetails.context, "options", OPTIONS_DESCRIPTOR);

    let { visitor } = ruleDetails;
    if (visitor === null) {
      // Rule defined with `create` method
      debugAssertIsNonNull(ruleDetails.rule.create);
      visitor = ruleDetails.rule.create(ruleDetails.context);
    } else {
      // Rule defined with `createOnce` method
      const { beforeHook, afterHook } = ruleDetails;
      if (beforeHook !== null) {
        // If `before` hook returns `false`, skip this rule
        const shouldRun = beforeHook();
        if (shouldRun === false) continue;
      }
      // Note: If `before` hook returned `false`, `after` hook is not called
      if (afterHook !== null) afterHooks.push(afterHook);
    }

    visitors.push(visitor);
  }

  // Compile visitors into string-keyed dispatch, and walk the AST.
  // Skip the walk if no visitors visit any nodes.
  const compiled = compileJsVisitors(visitors);
  if (compiled.hasVisitors) {
    walkParserAst(parseResult.ast, getJsParserVisitorKeys(), compiled);
  }

  // Run any `after` hooks
  runAfterHooks(true);

  // Extract comments from parse result, to send to Rust.
  // They drive `eslint-disable` directive handling on Rust side.
  return getComments(parseResult.ast.comments, sourceText);
}

/**
 * Parse a file with a registered custom parser.
 *
 * The parser's `parseForESLint` method is used if present, falling back to `parse`.
 * Base options match what ESLint passes to parsers, so parsers emit ranges, locations,
 * tokens, and comments. User-configured `parserOptions` are spread over the base options,
 * but `filePath` always wins.
 *
 * @param filePath - Absolute path of file being parsed
 * @param sourceText - Source text of the file
 * @param parserId - ID of parser to parse the file with
 * @param parserOptionsJSON - Parser options as JSON, or `null` if not configured
 * @returns Parse result
 * @throws {Error} If the parser fails to parse the file, or does not return an AST
 */
function parseWithJsParser(
  filePath: string,
  sourceText: string,
  parserId: number,
  parserOptionsJSON: string | null,
): JsParserParseResult {
  const parser = registeredParsers[parserId];
  debugAssertIsNonNull(parser, "Parser should be registered");

  const parserOptions: Record<string, unknown> | null =
    parserOptionsJSON === null ? null : JSON.parse(parserOptionsJSON);

  const options: Record<string, unknown> = {
    range: true,
    loc: true,
    tokens: true,
    comment: true,
    ecmaVersion: "latest",
    ...parserOptions,
    filePath,
  };

  let parseResult: JsParserParseResult;
  if (typeof parser.parseForESLint === "function") {
    parseResult = parser.parseForESLint(sourceText, options);
  } else {
    debugAssert(typeof parser.parse === "function", "Parser should have a `parse` method");
    parseResult = { ast: parser.parse!(sourceText, options) };
  }

  if (
    parseResult === null ||
    typeof parseResult !== "object" ||
    parseResult.ast === null ||
    typeof parseResult.ast !== "object"
  ) {
    throw new Error("Parser did not return an AST");
  }

  return parseResult;
}

/**
 * Convert comments from parser output to the form sent to Rust.
 *
 * Spans are UTF-16 offsets into the original source text. Comments without a valid range
 * are skipped (only possible if the parser misbehaves).
 *
 * Rust extracts each comment's text from source text assuming standard JS comment delimiters
 * (2 chars each side for block comments e.g. `/* *\/`, 2 chars before for line comments
 * e.g. `//`). Parsers for non-JS syntax report comments with other delimiters (e.g. Glimmer's
 * `{{! ... }}` in Ember templates), so spans are realigned using the comment's `value`,
 * such that Rust recovers exactly `value` as the comment's text. For standard JS comments,
 * this leaves the span unchanged.
 *
 * @param parserComments - Comments from parser output (`ast.comments`), or `undefined`
 * @param sourceText - Source text of the file
 * @returns Comments in form sent to Rust
 */
function getComments(
  parserComments: JsParserToken[] | undefined,
  sourceText: string,
): CommentReport[] {
  if (!Array.isArray(parserComments)) return [];

  const comments: CommentReport[] = [];
  for (let i = 0, len = parserComments.length; i < len; i++) {
    const comment = parserComments[i];
    if (comment === null || typeof comment !== "object" || !Array.isArray(comment.range)) continue;

    const isBlock = comment.type === "Block";
    let [start, end] = comment.range;

    // Realign span using the comment's `value` (see doc comment above).
    // Skip if `value` starts less than 2 chars into the comment (impossible for any real
    // comment syntax, whose opening delimiters are at least 2 chars), or isn't found.
    const { value } = comment;
    if (typeof value === "string" && value.length > 0 && start >= 0 && end > start) {
      const valueIndex = sourceText.slice(start, end).indexOf(value);
      if (valueIndex >= 2) {
        const contentStart = start + valueIndex;
        const contentEnd = contentStart + value.length;
        start = contentStart - 2;
        end = isBlock ? contentEnd + 2 : contentEnd;
      }
    }

    comments.push({ isBlock, start, end });
  }
  return comments;
}

/**
 * Run any `after` hooks.
 *
 * Same logic as `runAfterHooks` in `lint.ts` (which is module-private, so duplicated here).
 * See that function for explanation of why hooks always run, and errors are re-thrown at the end.
 *
 * @param shouldThrowIfError - `true` if any errors thrown in after hooks should be re-thrown
 */
function runAfterHooks(shouldThrowIfError: boolean): void {
  const afterHooksLen = afterHooks.length;
  if (afterHooksLen === 0) return;

  // Run `after` hooks
  let error: unknown;
  let didError = false;

  for (let i = 0; i < afterHooksLen; i++) {
    try {
      // Don't call hook with `afterHooks` array as `this`, or user could mess with it
      (0, afterHooks[i])();
    } catch (err) {
      if (didError === false) {
        error = err;
        didError = true;
      }
    }
  }

  // Reset array, ready for next file
  afterHooks.length = 0;

  // If error was thrown in any `after` hooks, re-throw it
  if (didError && shouldThrowIfError) throw error;
}

/**
 * Reset all state after linting a file parsed by a custom (JS) parser.
 */
function resetJsParserFile(): void {
  setSourceCodeOverride(null);
  resetJsParserSourceCode();
  // Also resets file context, source text, settings, and globals (shared with buffer-based path)
  resetFile();
}

/**
 * After an error, reset global state which otherwise may not be left
 * in the correct initial state for linting the next file.
 */
function resetStateAfterError(): void {
  // This function must never throw, so call `runAfterHooks` with `false` to swallow any errors
  runAfterHooks(false);

  diagnostics.length = 0;
  resetJsParserFile();
}
