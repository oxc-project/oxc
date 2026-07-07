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

import { setLanguageOptionsOverride, setSourceCodeOverride, setupFileContext } from "./context.ts";
import { setGlobalsForFile } from "./globals.ts";
import { compileJsVisitors, walkParserAst } from "./js_ast_walk.ts";
import { setParentsAndGetMaskedRegions } from "./js_parser_shadow.ts";
import {
  getJsParserVisitorKeys,
  JS_PARSER_SOURCE_CODE,
  resetJsParserSourceCode,
  setupJsParserSourceCode,
} from "./js_parser_source_code.ts";
import { buildRuleVisitors, resetFile, runAfterHooks } from "./lint.ts";
import { registeredParsers } from "./parsers.ts";
import { diagnostics } from "./report.ts";
import { setSettingsForFile } from "./settings.ts";
import { setSourceTextForJsParser } from "./source_code.ts";
import { switchWorkspace } from "./workspace.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";
import { getErrorMessage } from "../utils/utils.ts";

import type { Visitor } from "./types.ts";
import type { MaskedRegionReport } from "./js_parser_shadow.ts";
import type { JsParserParseResult, JsParserToken } from "./parsers.ts";
import type { SourceCode } from "./source_code.ts";
import type { ModuleKind } from "../generated/types.d.ts";

// Comment in form sent to Rust (`JsComment` on Rust side). Spans are UTF-16 offsets.
interface CommentReport {
  isBlock: boolean;
  start: number;
  end: number;
}

// Result of `lintFileWithJsParserImpl`, in form sent to Rust.
interface JsParserLintResult {
  comments: CommentReport[];
  // Masked regions for shadow-source native linting (see `js_parser_shadow.ts`),
  // or `null` if they could not be determined (Rust then skips native linting)
  maskedRegions: MaskedRegionReport[] | null;
}

// `buildRuleVisitors` (rule iteration + hooks) and `runAfterHooks` are shared with the
// buffer-based path (`lint.ts`); the two flows never interleave (both synchronous on the main
// JS thread, one file at a time), so sharing `lint.ts`'s `afterHooks` array is safe.

/**
 * Lint a file which is parsed by a custom (JS) parser.
 *
 * Main logic is in separate function `lintFileWithJsParserImpl`,
 * because V8 cannot optimize functions containing try/catch.
 *
 * @param filePath - Absolute path of file being linted
 * @param sourceText - Source text of the file (BOM already stripped on Rust side)
 * @param hasBOM - `true` if the original file started with a Unicode BOM
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
  hasBOM: boolean,
  parserId: number,
  parserOptionsJSON: string | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  workspaceUri: string | null,
): string {
  try {
    const { comments, maskedRegions } = lintFileWithJsParserImpl(
      filePath,
      sourceText,
      hasBOM,
      parserId,
      parserOptionsJSON,
      ruleIds,
      optionsIds,
      settingsJSON,
      globalsJSON,
      workspaceUri,
    );

    // Note: `messageId` field of `DiagnosticReport` is not needed on Rust side - `serde` skips over it
    const ret = JSON.stringify({ Success: { diagnostics, comments, maskedRegions } });

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
 * @param hasBOM - `true` if the original file started with a Unicode BOM
 * @param parserId - ID of parser to parse the file with
 * @param parserOptionsJSON - Parser options as JSON, or `null` if not configured
 * @param ruleIds - IDs of rules to run on this file
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for this file, as JSON string
 * @param globalsJSON - Globals for this file, as JSON string
 * @param workspaceUri - Workspace URI (`null` in CLI, string in LSP)
 * @returns Comment spans and masked regions, in form sent to Rust
 * @throws {Error} If any parameters are invalid, or the parser fails to parse the file
 * @throws {*} If any rule throws
 */
function lintFileWithJsParserImpl(
  filePath: string,
  sourceText: string,
  hasBOM: boolean,
  parserId: number,
  parserOptionsJSON: string | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  workspaceUri: string | null,
): JsParserLintResult {
  // Debug asserts that input is valid.
  // Note: `ruleIds` may be empty - the file is then only parsed (comments and masked
  // regions are still needed on Rust side for native linting of the shadow source).
  debugAssert(
    typeof filePath === "string" && filePath.length > 0,
    "`filePath` should be a non-empty string",
  );
  debugAssert(typeof sourceText === "string", "`sourceText` should be a string");
  debugAssert(typeof hasBOM === "boolean", "`hasBOM` should be a boolean");
  debugAssert(Array.isArray(ruleIds), "`ruleIds` should be an array");
  debugAssert(Array.isArray(optionsIds), "`optionsIds` should be an array");
  debugAssert(
    ruleIds.length === optionsIds.length,
    "`ruleIds` and `optionsIds` should be same length",
  );
  debugAssert(parserId < registeredParsers.length, "Parser ID out of bounds");

  // Switch to requested workspace.
  // In CLI, `workspaceUri` is `null`, and there's only 1 workspace, so no need to switch.
  if (workspaceUri !== null) switchWorkspace(workspaceUri);

  // Pass file path to context module, so `Context`s know what file is being linted
  setupFileContext(filePath);

  // Pass source text to source code module.
  // There is no buffer for this path, but shared code paths (`report.ts` `loc`-based reporting)
  // read `sourceText` from `source_code.ts` and lazily build line tables from it.
  setSourceTextForJsParser(sourceText);

  // Pass settings and globals JSON to modules that handle them
  setSettingsForFile(settingsJSON);
  setGlobalsForFile(globalsJSON);

  // Parse user-configured parser options.
  // Used both for parsing, and for `context.languageOptions.parserOptions`.
  const parserOptions: Record<string, unknown> | null =
    parserOptionsJSON === null ? null : JSON.parse(parserOptionsJSON);

  // Parse the file with the custom parser
  const parseResult = parseWithJsParser(filePath, sourceText, parserId, parserOptions);

  // Set up `SourceCode` backed by the parse result,
  // and make `context.sourceCode` return it instead of the buffer-based `SOURCE_CODE`.
  // Also override `context.languageOptions` / `context.parserOptions` - the buffer-based
  // singletons read `sourceType` etc. from the buffer, which does not exist for this path.
  setupJsParserSourceCode(parseResult, sourceText, hasBOM);
  setSourceCodeOverride(JS_PARSER_SOURCE_CODE as unknown as SourceCode);
  setLanguageOptionsOverride({
    sourceType: (parseResult.ast.sourceType ?? "module") as ModuleKind,
    parser: registeredParsers[parserId],
    parserOptions: parserOptions ?? {},
  });

  // Set `parent` on all AST nodes, and compute masked regions for shadow-source native
  // linting. Setting parents must happen BEFORE running rules: ESLint pre-computes its
  // traversal, so rules see `parent` on every node - even nodes later in the file than
  // the node currently being visited (see `setParentsAndGetMaskedRegions` docs).
  // A failure computing regions must not prevent JS rules from running, so it degrades
  // to `null` (Rust then skips native linting for the file).
  let maskedRegions: MaskedRegionReport[] | null = null;
  try {
    maskedRegions = setParentsAndGetMaskedRegions(
      parseResult.ast,
      getJsParserVisitorKeys(),
      parseResult.scopeManager ?? null,
      sourceText.length,
    );
  } catch {
    // Leave `maskedRegions` as `null`
  }

  // Get visitors for this file from all rules, collecting them for the string-keyed walk
  const visitors: Visitor[] = [];
  buildRuleVisitors(ruleIds, optionsIds, (visitor) => visitors.push(visitor));

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
  return { comments: getComments(parseResult.ast.comments, sourceText), maskedRegions };
}

/**
 * Parse a file with a registered custom parser.
 *
 * The parser's `parseForESLint` method is used if present, falling back to `parse`.
 * User-configured `parserOptions` are spread over `ecmaVersion`, but the
 * `range` / `loc` / `tokens` / `comment` flags and `filePath` are applied last,
 * so user options can never disable them - same as ESLint.
 *
 * @param filePath - Absolute path of file being parsed
 * @param sourceText - Source text of the file
 * @param parserId - ID of parser to parse the file with
 * @param parserOptions - User-configured parser options, or `null` if not configured
 * @returns Parse result
 * @throws {Error} If the parser fails to parse the file, or does not return an AST
 */
function parseWithJsParser(
  filePath: string,
  sourceText: string,
  parserId: number,
  parserOptions: Record<string, unknown> | null,
): JsParserParseResult {
  const parser = registeredParsers[parserId];
  debugAssertIsNonNull(parser, "Parser should be registered");

  const options: Record<string, unknown> = {
    ecmaVersion: "latest",
    ...parserOptions,
    // These flags must always be set - `SourceCode` and comment extraction depend on them.
    // ESLint also applies them after user options, so they cannot be disabled.
    range: true,
    loc: true,
    tokens: true,
    comment: true,
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
 * Limitation: realignment locates `value` in the source with `indexOf`. A parser that
 * normalizes `value` (e.g. strips `\r` from `\r\n`, or decodes HTML entities) so it no longer
 * occurs verbatim in the source defeats this, and the span is sent unchanged - for a non-JS
 * comment that means its delimiters are miscounted and an `eslint-disable` directive inside it
 * is silently ignored. Parsers oxlint targets (e.g. `ember-eslint-parser`) report verbatim
 * `value`, so this does not arise in practice.
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

    // Skip comments with invalid ranges (only possible if the parser misbehaves).
    // Rust side deserializes spans as `u32`, so a negative / fractional / out-of-range value
    // would fail deserialization of the whole result, losing all diagnostics for the file.
    if (
      !Number.isInteger(start) ||
      !Number.isInteger(end) ||
      start < 0 ||
      end <= start ||
      end > sourceText.length
    ) {
      continue;
    }

    // Realign span using the comment's `value` (see doc comment above).
    // Realignment requires at least 2 chars before the content (for delimiters shorter than
    // 2 chars, e.g. Glimmer / YAML-style comments, the extra chars come from the surrounding
    // source text - Rust strips 2 chars blindly, so their content doesn't matter).
    // If `value` isn't found, or the realigned span isn't representable, the span is sent
    // unchanged (correct for standard JS comments).
    const { value } = comment;
    if (typeof value === "string" && value.length > 0) {
      const contentStart = sourceText.indexOf(value, start);
      if (contentStart !== -1 && contentStart >= 2) {
        const contentEnd = contentStart + value.length;
        if (contentEnd <= end && (!isBlock || contentEnd + 2 <= sourceText.length)) {
          start = contentStart - 2;
          end = isBlock ? contentEnd + 2 : contentEnd;
        }
      }
    }

    // Rust strips comment delimiters blindly when extracting a comment's content
    // (`content_span` = `Span::new(start + 2, end - 2)` for blocks, `start + 2` for line
    // comments). A span too short for its delimiters would invert there and panic, so drop
    // it. Realigned spans above always satisfy this; only an un-realigned span from a
    // misbehaving parser can be too short.
    if (end - start < (isBlock ? 4 : 2)) continue;

    comments.push({ isBlock, start, end });
  }
  return comments;
}

/**
 * Reset all state after linting a file parsed by a custom (JS) parser.
 */
function resetJsParserFile(): void {
  setSourceCodeOverride(null);
  setLanguageOptionsOverride(null);
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
