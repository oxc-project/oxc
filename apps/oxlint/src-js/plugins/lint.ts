import { walkProgramWithCfg, resetCfgWalk } from "./cfg.ts";
import { setupFileContext, resetFileContext } from "./context.ts";
import { registeredRules } from "./load.ts";
import { allOptions, DEFAULT_OPTIONS_ID } from "./options.ts";
import { diagnostics } from "./report.ts";
import { setSettingsForFile, resetSettings } from "./settings.ts";
import { ast, initAst, resetSourceAndAst, setupSourceForFile } from "./source_code.ts";
import { HAS_BOM_FLAG_POS } from "../generated/constants.ts";
import { typeAssertIs, debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";
import { getErrorMessage } from "../utils/utils.ts";
import { setGlobalsForFile, resetGlobals } from "./globals.ts";
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

import { walkProgram, ancestors } from "../generated/walk.js";

import type { VisitFn, EnterExit } from "./visitor.ts";
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
 * @param workspaceUri - Workspace URI (`null` in CLI, string in LSP)
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
  workspaceUri: string | null,
): string | null {
  try {
    lintFileImpl(
      filePath,
      bufferId,
      buffer,
      ruleIds,
      optionsIds,
      settingsJSON,
      globalsJSON,
      workspaceUri,
    );

    let ret: string | null = null;

    // Avoid JSON serialization in common case that there are no diagnostics to report
    if (diagnostics.length !== 0) {
      // Note: `messageId` field of `DiagnosticReport` is not needed on Rust side, but we assume it's cheaper to leave it
      // in place and let `serde` skip over it on Rust side, than to iterate over all diagnostics and remove it here.
      ret = JSON.stringify({ Success: diagnostics });

      // Empty `diagnostics` array, so it starts empty when linting next file
      diagnostics.length = 0;
    }

    resetFile();

    return ret;
  } catch (err) {
    resetStateAfterError();

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
 * @param workspaceUri - Workspace URI (`null` in CLI, string in LSP)
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
  workspaceUri: string | null,
) {
  // If new buffer, add it to `buffers` array. Otherwise, get existing buffer from array.
  // Do this before checks below, to make sure buffer doesn't get garbage collected when not expected
  // if there's an error.
  // TODO: Is this enough to guarantee soundness?
  if (buffer === null) {
    // Rust will only send a `bufferId` alone, if it previously sent a buffer with this same ID
    buffer = buffers[bufferId]!;
  } else {
    typeAssertIs<BufferWithArrays>(buffer);
    const { buffer: arrayBuffer, byteOffset } = buffer;
    buffer.int32 = new Int32Array(arrayBuffer, byteOffset);
    buffer.float64 = new Float64Array(arrayBuffer, byteOffset);

    for (let i = bufferId - buffers.length; i >= 0; i--) {
      buffers.push(null);
    }
    buffers[bufferId] = buffer;
  }
  typeAssertIs<BufferWithArrays>(buffer);

  // Debug asserts that input is valid
  debugAssert(
    typeof filePath === "string" && filePath.length > 0,
    "`filePath` should be a non-empty string",
  );
  debugAssert(Array.isArray(ruleIds) && ruleIds.length > 0, "`ruleIds` should be non-empty array");
  debugAssert(Array.isArray(optionsIds), "`optionsIds` should be an array");
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

  // Pass buffer to source code module, so it can decode source text and deserialize AST on demand.
  //
  // We don't want to do this eagerly, because all rules might return empty visitors,
  // or `createOnce` rules might return `false` from their `before` hooks.
  // In such cases, the AST doesn't need to be walked, so we can skip deserializing it.
  //
  // But... source text and AST can be accessed in body of `create` method, or `before` hook, via `context.sourceCode`.
  // So we pass the buffer to source code module here, so it can decode source text / deserialize AST on demand.
  const hasBOM = buffer[HAS_BOM_FLAG_POS] === 1;
  setupSourceForFile(buffer, hasBOM);

  // Pass settings and globals JSON to modules that handle them
  setSettingsForFile(settingsJSON);
  setGlobalsForFile(globalsJSON);

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

    addVisitorToCompiled(visitor);
  }

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

  // Run any `after` hooks
  runAfterHooks(true);
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
