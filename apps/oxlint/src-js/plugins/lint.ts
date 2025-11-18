import { setupFileContext, resetFileContext } from './context.js';
import { registeredRules } from './load.js';
import { diagnostics } from './report.js';
import { setSettingsForFile, resetSettings } from './settings.js';
import { ast, initAst, resetSourceAndAst, setupSourceForFile } from './source_code.js';
import { assertIs, getErrorMessage } from './utils.js';
import { addVisitorToCompiled, compiledVisitor, finalizeCompiledVisitor, initCompiledVisitor } from './visitor.js';

// Lazy implementation
/*
import { TOKEN } from '../../dist/src-js/raw-transfer/lazy-common.js';
import { walkProgram } from '../generated/walk.js';
*/

// @ts-expect-error we need to generate `.d.ts` file for this module
import { walkProgram } from '../generated/walk.js';

import type { AfterHook, BufferWithArrays } from './types.ts';

// Buffers cache.
//
// All buffers sent from Rust are stored in this array, indexed by `bufferId` (also sent from Rust).
// Buffers are only added to this array, never removed, so no buffers will be garbage collected
// until the process exits.
const buffers: (BufferWithArrays | null)[] = [];

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
 * @param settingsJSON - Settings for file, as JSON
 * @returns Diagnostics or error serialized to JSON string
 */
export function lintFile(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  settingsJSON: string,
): string {
  try {
    lintFileImpl(filePath, bufferId, buffer, ruleIds, settingsJSON);
    return JSON.stringify({ Success: diagnostics });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  } finally {
    diagnostics.length = 0;
  }
}

/**
 * Run rules on a file.
 *
 * @param filePath - Absolute path of file being linted
 * @param bufferId - ID of buffer containing file data
 * @param buffer - Buffer containing file data, or `null` if buffer with this ID was previously sent to JS
 * @param ruleIds - IDs of rules to run on this file
 * @param settingsJSON - Stringified settings for this file
 * @returns Diagnostics to send back to Rust
 * @throws {Error} If any parameters are invalid
 * @throws {*} If any rule throws
 */
function lintFileImpl(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  settingsJSON: string,
) {
  // If new buffer, add it to `buffers` array. Otherwise, get existing buffer from array.
  // Do this before checks below, to make sure buffer doesn't get garbage collected when not expected
  // if there's an error.
  // TODO: Is this enough to guarantee soundness?
  if (buffer === null) {
    // Rust will only send a `bufferId` alone, if it previously sent a buffer with this same ID
    buffer = buffers[bufferId]!;
  } else {
    assertIs<BufferWithArrays>(buffer);
    const { buffer: arrayBuffer, byteOffset } = buffer;
    buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
    buffer.float64 = new Float64Array(arrayBuffer, byteOffset);

    for (let i = bufferId - buffers.length; i >= 0; i--) {
      buffers.push(null);
    }
    buffers[bufferId] = buffer;
  }
  assertIs<BufferWithArrays>(buffer);

  if (typeof filePath !== 'string' || filePath.length === 0) {
    throw new Error('expected filePath to be a non-zero length string');
  }
  if (!Array.isArray(ruleIds) || ruleIds.length === 0) {
    throw new Error('Expected `ruleIds` to be a non-zero len array');
  }

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

  // Pass settings JSON to context module
  setSettingsForFile(settingsJSON);

  // Get visitors for this file from all rules
  initCompiledVisitor();

  for (let i = 0, len = ruleIds.length; i < len; i++) {
    const ruleId = ruleIds[i],
      ruleDetails = registeredRules[ruleId];

    // Set `ruleIndex` for rule. It's used when sending diagnostics back to Rust.
    ruleDetails.ruleIndex = i;

    const { rule, context } = ruleDetails;

    let { visitor } = ruleDetails;
    if (visitor === null) {
      // Rule defined with `create` method
      visitor = rule.create(context);
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

  // Reset file context, source, AST, and settings, to free memory
  resetFileContext();
  resetSourceAndAst();
  resetSettings();
}
