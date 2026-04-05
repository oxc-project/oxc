import { walkProgramWithCfg, resetCfgWalk } from "./cfg.ts";
import {
  setEcmaVersion,
  setParserForFile,
  setupFileContext,
  resetEcmaVersion,
  resetFileContext,
  resetParserForFile,
} from "./context.ts";
import { resolveLanguageOptionsIds } from "../js_language_options_registry.ts";
import { registeredRules } from "./load.ts";
import { allOptions, DEFAULT_OPTIONS_ID } from "./options.ts";
import { diagnostics } from "./report.ts";
import { setSettingsForFile, resetSettings } from "./settings.ts";
import {
  ast,
  initAst,
  resetSourceAndAst,
  setParserMetadataForFile,
  setupExternalSourceForFile,
  setupSourceForFile,
} from "./source_code.ts";
import { HAS_BOM_FLAG_POS } from "../generated/constants.ts";
import { typeAssertIs, debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";
import { getErrorMessage } from "../utils/utils.ts";
import { createRequiredParserCallOptions } from "./parser_call_options.ts";
import { setGlobalsForFile, resetGlobals } from "./globals.ts";
import { detectExternalSourceFlags, normalizeExternalProgramSourceType } from "./external_parser_utils.ts";
import {
  getInferredExternalChildKeys,
  isExternalNodeLike,
  mergeExternalChildKeys,
  sanitizeExternalVisitorKeysRecord,
} from "./external_ast_utils.ts";
import { comments as currentComments, setupExternalCommentsForFile } from "./comments.ts";
import { setupExternalTokensForFile } from "./tokens.ts";
import { resetWeakMaps } from "./weak_map.ts";
import { switchWorkspace } from "./workspace.ts";
import {
  addVisitorToCompiled,
  compiledVisitor,
  finalizeCompiledVisitor,
  resetCompiledVisitor,
  VISITOR_EMPTY,
  VISITOR_CFG,
} from "./visitor.ts";
import {
  compileExternalVisitors,
  walkExternalProgram,
  walkExternalProgramWithCfg,
} from "./external_traversal.ts";

import { walkProgram, ancestors } from "../generated/walk.js";

import type { VisitFn, EnterExit } from "./visitor.ts";
import type { VisitorObject } from "../generated/visitor.d.ts";
import type { Program } from "../generated/types.d.ts";
import type { ScopeManager } from "./scope.ts";
import type { AfterHook, BufferWithArrays } from "./types.ts";

// Buffers cache.
//
// All buffers sent from Rust are stored in this array, indexed by `bufferId` (also sent from Rust).
// Buffers are only added to this array, never removed, so no buffers will be garbage collected
// until the process exits.
export const buffers: (BufferWithArrays | null)[] = [];

// Array of `after` hooks to run after traversal. This array reused for every file.
const afterHooks: AfterHook[] = [];

// Reusable property descriptor for updating `options` value on rule context objects.
// `value` is updated before each call. Other attributes are omitted to retain existing values.
const OPTIONS_DESCRIPTOR: PropertyDescriptor = { value: null };

type VisitorKeysRecord = Readonly<Record<string, readonly string[]>>;

type ExternalDirectiveCommentReport = {
  type: "Line" | "Block" | "Shebang";
  start: number;
  end: number;
};

type WholeFileCustomParserErrorReport = {
  message: string;
  start: number;
  end: number;
};

class WholeFileCustomParserParseError extends SyntaxError {
  start: number;
  end: number;

  constructor(message: string, start: number, end: number) {
    super(message);
    this.name = "WholeFileCustomParserParseError";
    this.start = start;
    this.end = end;
  }
}

function getExternalDirectiveCommentsForRoundTrip(): ExternalDirectiveCommentReport[] | null {
  if (currentComments === null || currentComments.length === 0) return null;

  return currentComments.map(({ type, start, end }) => ({ type, start, end }));
}

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isNonNegativeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

function getOffsetFromParserErrorLineColumn(
  sourceText: string,
  line: number,
  column: number,
): number | null {
  if (!isNonNegativeInteger(column) || !isNonNegativeInteger(line) || line === 0) return null;

  let currentLine = 1;
  let lineStart = 0;

  for (let i = 0; i < sourceText.length && currentLine < line; i++) {
    const code = sourceText.charCodeAt(i);
    if (code === 0x0d) {
      if (sourceText.charCodeAt(i + 1) === 0x0a) i++;
      currentLine++;
      lineStart = i + 1;
    } else if (code === 0x0a || code === 0x2028 || code === 0x2029) {
      currentLine++;
      lineStart = i + 1;
    }
  }

  if (currentLine !== line) return null;

  let lineEnd = sourceText.length;
  for (let i = lineStart; i < sourceText.length; i++) {
    const code = sourceText.charCodeAt(i);
    if (code === 0x0d || code === 0x0a || code === 0x2028 || code === 0x2029) {
      lineEnd = i;
      break;
    }
  }

  const offset = lineStart + column;
  if (offset < lineStart || offset > lineEnd) return null;
  return offset;
}

function getWholeFileCustomParserErrorReport(
  error: unknown,
  sourceText: string,
): WholeFileCustomParserErrorReport | null {
  if (!isObjectRecord(error) || typeof error.message !== "string") return null;

  let offset: number | null = null;

  if (isNonNegativeInteger(error.index) && error.index <= sourceText.length) {
    offset = error.index;
  } else if (isObjectRecord(error.loc)) {
    const line = error.loc.line;
    const column = error.loc.column;
    if (isNonNegativeInteger(line) && isNonNegativeInteger(column)) {
      offset = getOffsetFromParserErrorLineColumn(sourceText, line, column);
    }
  }

  if (offset === null) {
    const lineNumber = error.lineNumber;
    const column = error.column;
    if (isNonNegativeInteger(lineNumber) && isNonNegativeInteger(column)) {
      offset = getOffsetFromParserErrorLineColumn(sourceText, lineNumber, column);
    }
  }

  if (offset === null) return null;

  return {
    message: error.message,
    start: offset,
    end: offset < sourceText.length ? offset + 1 : offset,
  };
}

function isVisitorKeys(value: unknown): value is VisitorKeysRecord {
  if (!isObjectRecord(value)) return false;
  return Object.values(value).every(
    (keys) => Array.isArray(keys) && keys.every((key) => typeof key === "string"),
  );
}

function pickExternalMetadataArray(
  astMetadata: unknown,
  parserResultMetadata: unknown,
): unknown[] | null {
  const astMetadataArray = Array.isArray(astMetadata) ? astMetadata : null;
  const parserResultMetadataArray = Array.isArray(parserResultMetadata)
    ? parserResultMetadata
    : null;

  if (astMetadataArray !== null && astMetadataArray.length > 0) {
    return astMetadataArray;
  }
  if (parserResultMetadataArray !== null && parserResultMetadataArray.length > 0) {
    return parserResultMetadataArray;
  }
  return astMetadataArray ?? parserResultMetadataArray;
}

function normalizeExternalAst(
  node: unknown,
  parent: (Record<string, unknown> & { type: string }) | null,
  visitorKeys: VisitorKeysRecord | null,
  seen: WeakSet<object>,
): void {
  if (!isExternalNodeLike(node)) return;
  if (seen.has(node)) return;
  seen.add(node);

  (
    node as Record<string, unknown> & { parent?: Record<string, unknown> | null }
  ).parent = parent;

  const rangedNode = node as Record<string, unknown> & {
    range?: [number, number];
    start?: number;
    end?: number;
  };
  if (Array.isArray(rangedNode.range) && rangedNode.range.length === 2) {
    if (typeof rangedNode.start !== "number") rangedNode.start = rangedNode.range[0];
    if (typeof rangedNode.end !== "number") rangedNode.end = rangedNode.range[1];
  }

  const inferredKeys = getInferredExternalChildKeys(node);
  const keys = mergeExternalChildKeys(inferredKeys, visitorKeys?.[node.type]);

  for (let i = 0, len = keys.length; i < len; i++) {
    const child = node[keys[i]!];
    if (Array.isArray(child)) {
      for (let j = 0, childLen = child.length; j < childLen; j++) {
        normalizeExternalAst(child[j], node, visitorKeys, seen);
      }
    } else {
      normalizeExternalAst(child, node, visitorKeys, seen);
    }
  }
}

function serializeLintSuccess(
  sourceText: string | null,
): string | null {
  const externalDirectiveComments =
    sourceText === null ? null : getExternalDirectiveCommentsForRoundTrip();

  // Avoid JSON serialization in the common case where there is nothing to report or round-trip.
  if (diagnostics.length === 0 && externalDirectiveComments === null) {
    return null;
  }

  return JSON.stringify({
    Success:
      sourceText === null
        ? diagnostics
        : { diagnostics, comments: externalDirectiveComments ?? [] },
  });
}

function setupExternalParserSource(
  filePath: string,
  parser: {
    parse?: (code: string, options?: Record<string, unknown>) => unknown;
    parseForESLint?: (code: string, options?: Record<string, unknown>) => unknown;
    VisitorKeys?: VisitorKeysRecord;
  } | null,
  parserOptions: Record<string, unknown> | null,
  sourceType: unknown,
  ecmaVersion: unknown,
  sourceText: string,
  hasBOM: boolean,
): void {
  if (parser === null) {
    throw new Error(
      `Whole-file source linting for ${filePath} requires a configured custom parser`,
    );
  }

  const parserCallOptions = createRequiredParserCallOptions(
    filePath,
    parserOptions,
    sourceType,
    ecmaVersion,
  );
  let parserResult: unknown;
  let astResult: unknown;

  try {
    if (typeof parser.parseForESLint === "function") {
      parserResult = parser.parseForESLint(sourceText, parserCallOptions);
      astResult = isObjectRecord(parserResult) ? parserResult.ast : undefined;
    } else if (typeof parser.parse === "function") {
      parserResult = null;
      astResult = parser.parse(sourceText, parserCallOptions);
    } else {
      throw new TypeError("Custom parser must implement `parseForESLint()` or `parse()`");
    }
  } catch (error) {
    const parserError = getWholeFileCustomParserErrorReport(error, sourceText);
    if (parserError !== null) {
      throw new WholeFileCustomParserParseError(
        parserError.message,
        parserError.start,
        parserError.end,
      );
    }
    throw error;
  }

  if (!isObjectRecord(astResult) || astResult.type !== "Program") {
    throw new TypeError("Custom parser must return an ESTree Program");
  }

  const program = astResult as unknown as Program & {
    body?: unknown[];
    comments?: unknown[];
    tokens?: unknown[];
    sourceType?: Program["sourceType"] | string;
    visitorKeys?: VisitorKeysRecord;
    services?: Record<string, unknown>;
    parserServices?: Record<string, unknown>;
    scopeManager?: ScopeManager | null;
  };
  const parserResultCommentsInput =
    isObjectRecord(parserResult) && Array.isArray(parserResult.comments)
      ? parserResult.comments
      : null;
  const parserResultTokensInput =
    isObjectRecord(parserResult) && Array.isArray(parserResult.tokens)
      ? parserResult.tokens
      : null;
  const commentsInput = pickExternalMetadataArray(program.comments, parserResultCommentsInput);
  const tokensInput = pickExternalMetadataArray(program.tokens, parserResultTokensInput);
  if (!Array.isArray(program.body)) program.body = [];
  if (commentsInput === null) program.comments = [];
  if (tokensInput === null) program.tokens = [];
  program.sourceType = normalizeExternalProgramSourceType(
    program.sourceType,
    parserCallOptions.sourceType,
    program.body,
  );

  const visitorKeys = sanitizeExternalVisitorKeysRecord(
    isObjectRecord(parserResult) && isVisitorKeys(parserResult.visitorKeys)
      ? parserResult.visitorKeys
      : isVisitorKeys(program.visitorKeys)
        ? program.visitorKeys
        : isVisitorKeys(parser.VisitorKeys)
          ? parser.VisitorKeys
          : null,
  );
  const sourceFlags = detectExternalSourceFlags(parserOptions, program, visitorKeys);
  normalizeExternalAst(program, null, visitorKeys, new WeakSet());

  const parserServices =
    isObjectRecord(parserResult) && isObjectRecord(parserResult.services)
      ? parserResult.services
      : isObjectRecord(parserResult) && isObjectRecord(parserResult.parserServices)
        ? parserResult.parserServices
        : isObjectRecord(program.services)
          ? program.services
          : isObjectRecord(program.parserServices)
            ? program.parserServices
            : null;
  const scopeManager = (isObjectRecord(parserResult)
    ? parserResult.scopeManager ?? program.scopeManager ?? null
    : program.scopeManager ?? null) as ScopeManager | null;

  setupExternalSourceForFile(sourceText, program, hasBOM, sourceFlags);
  const normalizedComments = setupExternalCommentsForFile(commentsInput, sourceText);
  const normalizedTokens = setupExternalTokensForFile(tokensInput, sourceText);
  program.comments = normalizedComments;
  program.tokens = normalizedTokens;
  setParserMetadataForFile({ visitorKeys, parserServices, scopeManager });
}

/**
 * Lint a file.
 *
 * Main logic is in separate function `lintFileImpl`, because V8 cannot optimize functions containing try/catch.
 *
 * @param filePath - Absolute path of file being linted
 * @param bufferId - ID of buffer containing file data
 * @param buffer - Buffer containing file data, or `null` if buffer with this ID was previously sent to JS
 * @param ruleIds - IDs of rules to run on this file
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for this file, as JSON string
 * @param globalsJSON - Globals for this file, as JSON string
 * @param languageOptionsIds - Internal JS-side `languageOptions` IDs for this file
 * @param workspaceUri - Workspace URI (`null` in CLI, string in LSP)
 * @param sourceText - Whole-file source text for custom parser runs
 * @returns Diagnostics or error serialized to JSON string
 */
export function lintFile(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  languageOptionsIds: number[],
  workspaceUri: string | null,
  sourceText: string | null,
): string | null {
  try {
    const ret = lintFileImpl(
      filePath,
      bufferId,
      buffer,
      ruleIds,
      optionsIds,
      settingsJSON,
      globalsJSON,
      languageOptionsIds,
      workspaceUri,
      sourceText,
    );

    if (diagnostics.length !== 0) {
      diagnostics.length = 0;
    }
    resetFile();

    return ret;
  } catch (err) {
    const parserError =
      err instanceof WholeFileCustomParserParseError
        ? { message: err.message, start: err.start, end: err.end }
        : null;

    resetStateAfterError();

    if (parserError !== null) {
      return JSON.stringify({
        Success: {
          diagnostics: [],
          comments: [],
          parseError: parserError,
        },
      });
    }

    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Run rules on a file.
 *
 * @param filePath - Absolute path of file being linted
 * @param bufferId - ID of buffer containing file data
 * @param buffer - Buffer containing file data, or `null` if buffer with this ID was previously sent to JS
 * @param ruleIds - IDs of rules to run on this file
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for this file, as JSON string
 * @param globalsJSON - Globals for this file, as JSON string
 * @param languageOptionsIds - Internal JS-side `languageOptions` IDs for this file
 * @param workspaceUri - Workspace URI (`null` in CLI, string in LSP)
 * @param sourceText - Whole-file source text for custom parser runs
 * @throws {Error} If any parameters are invalid
 * @throws {*} If any rule throws
 */
export function lintFileImpl(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  languageOptionsIds: number[],
  workspaceUri: string | null,
  sourceText: string | null,
): string | null {
  // If new buffer, add it to `buffers` array. Otherwise, get existing buffer from array.
  // Do this before checks below, to make sure buffer doesn't get garbage collected when not expected
  // if there's an error.
  // TODO: Is this enough to guarantee soundness?
  if (sourceText === null) {
    if (buffer === null) {
      // Rust will only send a `bufferId` alone, if it previously sent a buffer with this same ID
      buffer = buffers[bufferId]!;
    } else {
      typeAssertIs<BufferWithArrays>(buffer);
      const { buffer: arrayBuffer, byteOffset } = buffer;
      buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
      buffer.float64 = new Float64Array(arrayBuffer, byteOffset);

      for (let i = bufferId - buffers.length; i >= 0; i--) {
        buffers.push(null);
      }
      buffers[bufferId] = buffer;
    }
    typeAssertIs<BufferWithArrays>(buffer);
  }

  // Debug asserts that input is valid
  debugAssert(
    typeof filePath === "string" && filePath.length > 0,
    "`filePath` should be a non-empty string",
  );
  debugAssert(Array.isArray(ruleIds) && ruleIds.length > 0, "`ruleIds` should be non-empty array");
  debugAssert(Array.isArray(optionsIds), "`optionsIds` should be an array");
  debugAssert(Array.isArray(languageOptionsIds), "`languageOptionsIds` should be an array");
  debugAssert(
    ruleIds.length === optionsIds.length,
    "`ruleIds` and `optionsIds` should be same length",
  );

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
  // In LSP, there can be multiple workspaces, so we need to switch if we're not already in the right one.
  if (workspaceUri !== null) switchWorkspace(workspaceUri);
  debugAssertIsNonNull(allOptions, "`allOptions` should be initialized");

  // Pass file path to context module, so `Context`s know what file is being linted
  setupFileContext(filePath);

  const resolvedLanguageOptions = resolveLanguageOptionsIds(languageOptionsIds);
  const parser = resolvedLanguageOptions?.parser ?? null;
  const parserOptions =
    (resolvedLanguageOptions?.parserOptions as Record<string, unknown> | null | undefined) ?? null;
  const resolvedEcmaVersion = resolvedLanguageOptions?.ecmaVersion ?? parserOptions?.ecmaVersion;
  setEcmaVersion(resolvedEcmaVersion);
  setParserForFile(
    parser,
    parserOptions,
  );

  // Pass buffer to source code module, so it can decode source text and deserialize AST on demand.
  //
  // We don't want to do this eagerly, because all rules might return empty visitors,
  // or `createOnce` rules might return `false` from their `before` hooks.
  // In such cases, the AST doesn't need to be walked, so we can skip deserializing it.
  //
  // But... source text and AST can be accessed in body of `create` method, or `before` hook, via `context.sourceCode`.
  // So we pass the buffer to source code module here, so it can decode source text / deserialize AST on demand.
  const hasBOM =
    sourceText === null ? buffer[HAS_BOM_FLAG_POS] === 1 : sourceText.charCodeAt(0) === 0xfeff;
  const normalizedExternalSourceText =
    sourceText !== null && hasBOM ? sourceText.slice(1) : sourceText;
  if (sourceText === null) {
    setupSourceForFile(buffer, hasBOM);
  } else {
    setupExternalParserSource(
      filePath,
      parser,
      parserOptions,
      resolvedLanguageOptions?.sourceType,
      resolvedEcmaVersion,
      normalizedExternalSourceText,
      hasBOM,
    );
  }

  // Pass settings and globals JSON to modules that handle them
  setSettingsForFile(settingsJSON);
  setGlobalsForFile(globalsJSON);

  const isWholeFileCustomParserRun = sourceText !== null;
  const externalVisitors: VisitorObject[] | null = isWholeFileCustomParserRun ? [] : null;

  // Get visitors for this file from all rules
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

    // If the rule has no user-provided options, use the plugin-provided default
    // options (which falls back to `DEFAULT_OPTIONS`).
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

    if (externalVisitors !== null) {
      externalVisitors.push(visitor);
    } else {
      addVisitorToCompiled(visitor);
    }
  }

  if (externalVisitors !== null) {
    const compiledExternalVisitor = compileExternalVisitors(externalVisitors);

    // Visit AST.
    // Skip this if no visitors visit any nodes.
    // Some rules seen in the wild return an empty visitor object from `create` if some initial check fails
    // e.g. file extension is not one the rule acts on.
    if (compiledExternalVisitor !== null) {
      if (ast === null) initAst();
      debugAssertIsNonNull(ast);

      debugAssert(ancestors.length === 0, "`ancestors` should be empty before walking AST");
      if (compiledExternalVisitor.hasCfg) {
        walkExternalProgramWithCfg(ast, compiledExternalVisitor);
      } else {
        walkExternalProgram(ast, compiledExternalVisitor);
      }
      debugAssert(ancestors.length === 0, "`ancestors` should be empty after walking AST");
    }
  } else {
    const visitorState = finalizeCompiledVisitor();

    // Visit AST.
    // Skip this if no visitors visit any nodes.
    // Some rules seen in the wild return an empty visitor object from `create` if some initial check fails
    // e.g. file extension is not one the rule acts on.
    if (visitorState !== VISITOR_EMPTY) {
      if (ast === null) initAst();
      debugAssertIsNonNull(ast);

      debugAssert(ancestors.length === 0, "`ancestors` should be empty before walking AST");

      if (visitorState === VISITOR_CFG) {
        walkProgramWithCfg(ast, compiledVisitor);
      } else {
        walkProgram(ast, compiledVisitor as (VisitFn | EnterExit | null)[]);
      }

      debugAssert(ancestors.length === 0, "`ancestors` should be empty after walking AST");

      // Reset compiled visitor, ready for next file
      resetCompiledVisitor();
    }
  }

  // Run any `after` hooks
  runAfterHooks(true);

  return serializeLintSuccess(sourceText);
}

/**
 * Run any `after` hooks.
 *
 * Rules using `before` and `after` hooks likely maintain some internal state in their `createOnce` method.
 * To keep that state in sync, it's critical that `after` hooks always run, even if an error is thrown during any of:
 *
 * 1. A later rule's `before` hook.
 * 2. AST walk.
 * 3. An earlier rule's `after` hook.
 *
 * So if any `after` hook throws an error, this function continues running remaining hooks, and re-throws the error
 * only at the very end. This ensures an error in one rule does not affect any other rules.
 *
 * This function is called by `resetStateAfterError` to ensure `after` hooks are run no matter where an error occurs.
 *
 * @param shouldThrowIfError - `true` if any errors thrown in after hooks should be re-thrown
 */
function runAfterHooks(shouldThrowIfError: boolean) {
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
 * Reset file context, source, AST, and settings, to free memory.
 */
export function resetFile() {
  resetFileContext();
  resetEcmaVersion();
  resetParserForFile();
  resetSourceAndAst();
  resetSettings();
  resetGlobals();
  resetWeakMaps();
}

/**
 * After an error, reset global state which otherwise may not be left
 * in the correct initial state for linting the next file.
 */
export function resetStateAfterError() {
  // This function must never throw, so call `runAfterHooks` with `false` to swallow any errors
  runAfterHooks(false);

  // In case error occurred during visitor compilation, clear internal state of visitor compilation,
  // so no leftovers bleed into next file.
  // We could have a separate function to reset state which could be simpler and faster, but `resetStateAfterError`
  // should never be called - only happens when rules return an invalid visitor or malfunction.
  // So better to use the existing functions, rather than bloat the package with more code which should never run.
  finalizeCompiledVisitor();
  resetCompiledVisitor();

  diagnostics.length = 0;
  ancestors.length = 0;
  resetFile();
  resetCfgWalk();
}
