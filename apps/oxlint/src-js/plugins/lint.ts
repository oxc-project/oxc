import { setupFileContext, resetFileContext } from "./context.ts";
import { registeredRules } from "./load.ts";
import { allOptions, DEFAULT_OPTIONS_ID } from "./options.ts";
import { diagnostics } from "./report.ts";
import { setSettingsForFile, resetSettings } from "./settings.ts";
import { ast, initAst, resetSourceAndAst, setupSourceForFile } from "./source_code.ts";
import { typeAssertIs, debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";
import { getErrorMessage } from "../utils/utils.ts";
import { setGlobalsForFile, resetGlobals } from "./globals.ts";

import {
  addVisitorToCompiled,
  compiledVisitor,
  finalizeCompiledVisitor,
  initCompiledVisitor,
} from "./visitor.ts";

// Lazy implementation
/*
import { TOKEN } from '../../dist/src-js/raw-transfer/lazy-common.js';
import { walkProgram } from '../generated/walk.js';
*/

// @ts-expect-error - TODO: We need to generate `.d.ts` file for this module
import { walkProgram } from "../generated/walk.js";

import type { AfterHook, BufferWithArrays } from "./types.ts";

// Buffers cache.
//
// All buffers sent from Rust are stored in this array, indexed by `bufferId` (also sent from Rust).
// Buffers are only added to this array, never removed, so no buffers will be garbage collected
// until the process exits.
export const buffers: (BufferWithArrays | null)[] = [];

// Array of `after` hooks to run after traversal. This array reused for every file.
const afterHooks: AfterHook[] = [];

// Default parser services object (empty object).
const PARSER_SERVICES_DEFAULT: Record<string, unknown> = Object.freeze({});

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
): string | null {
  try {
    lintFileImpl(filePath, bufferId, buffer, ruleIds, optionsIds, settingsJSON, globalsJSON);

    // Avoid JSON serialization in common case that there are no diagnostics to report
    if (diagnostics.length === 0) return null;

    // Note: `messageId` field of `DiagnosticReport` is not needed on Rust side, but we assume it's cheaper to leave it
    // in place and let `serde` skip over it on Rust side, than to iterate over all diagnostics and remove it here.
    return JSON.stringify({ Success: diagnostics });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  } finally {
    diagnostics.length = 0;
    resetFile();
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
    buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
    buffer.float64 = new Float64Array(arrayBuffer, byteOffset);

    for (let i = bufferId - buffers.length; i >= 0; i--) {
      buffers.push(null);
    }
    buffers[bufferId] = buffer;
  }
  typeAssertIs<BufferWithArrays>(buffer);

  // Debug asserts that input is valid
  debugAssert(typeof filePath === "string" && filePath.length > 0);
  debugAssert(Array.isArray(ruleIds) && ruleIds.length > 0);
  debugAssert(Array.isArray(optionsIds));
  debugAssert(ruleIds.length === optionsIds.length);

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
  const hasBOM = false; // TODO: Set this correctly
  const parserServices = PARSER_SERVICES_DEFAULT; // TODO: Set this correctly
  setupSourceForFile(buffer, hasBOM, parserServices);

  // Pass settings and globals JSON to modules that handle them
  setSettingsForFile(settingsJSON);
  setGlobalsForFile(globalsJSON);

  // Get visitors for this file from all rules
  initCompiledVisitor();

  for (let i = 0, len = ruleIds.length; i < len; i++) {
    const ruleId = ruleIds[i];
    debugAssert(ruleId < registeredRules.length, "Rule ID out of bounds");
    const ruleDetails = registeredRules[ruleId];

    // Set `ruleIndex` for rule. It's used when sending diagnostics back to Rust.
    ruleDetails.ruleIndex = i;

    // Set `options` for rule
    const optionsId = optionsIds[i];
    debugAssertIsNonNull(allOptions);
    debugAssert(optionsId < allOptions.length, "Options ID out of bounds");

    // If the rule has no user-provided options, use the plugin-provided default
    // options (which falls back to `DEFAULT_OPTIONS`)
    ruleDetails.options =
      optionsId === DEFAULT_OPTIONS_ID ? ruleDetails.defaultOptions : allOptions[optionsId];

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

  const needsVisit = finalizeCompiledVisitor();

  // Visit AST.
  // Skip this if no visitors visit any nodes.
  // Some rules seen in the wild return an empty visitor object from `create` if some initial check fails
  // e.g. file extension is not one the rule acts on.
  if (needsVisit) {
    if (ast === null) initAst();
    walkProgram(ast, compiledVisitor);

    // Lazy implementation
    /*
    const sourceIsAscii = sourceText.length === sourceByteLen;
    const ast = {
      buffer,
      sourceText,
      sourceByteLen,
      sourceIsAscii,
      nodes: new Map(),
      token: TOKEN,
    };

    walkProgram(programPos, ast, compiledVisitor);
    */
  }

  // Run `after` hooks
  const afterHooksLen = afterHooks.length;
  if (afterHooksLen !== 0) {
    for (let i = 0; i < afterHooksLen; i++) {
      // Don't call hook with `afterHooks` array as `this`, or user could mess with it
      (0, afterHooks[i])();
    }
    // Reset array, ready for next file
    afterHooks.length = 0;
  }
}

/**
 * Reset file context, source, AST, and settings, to free memory.
 */
export function resetFile() {
  resetFileContext();
  resetSourceAndAst();
  resetSettings();
  resetGlobals();
}
