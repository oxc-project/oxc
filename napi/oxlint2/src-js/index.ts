import { createRequire } from 'node:module';
import { lint } from './bindings.js';
import {
  DATA_POINTER_POS_32,
  SOURCE_LEN_OFFSET,
  // TODO(camc314): we need to generate `.d.ts` file for this module.
  // @ts-expect-error
} from './generated/constants.cjs';
import { assertIs, getErrorMessage } from './utils.js';
import { addVisitorToCompiled, compiledVisitor, finalizeCompiledVisitor, initCompiledVisitor } from './visitor.js';

import type { Visitor } from './types.ts';

// Import methods and objects from `oxc-parser`.
// Use `require` not `import` as `oxc-parser` uses `require` internally,
// and need to make sure get same instance of modules as it uses internally,
// otherwise `TOKEN` here won't be same `TOKEN` as used within `oxc-parser`.
const require = createRequire(import.meta.url);
const { TOKEN } = require('../dist/parser/raw-transfer/lazy-common.cjs'),
  walkProgram = require('../dist/parser/generated/lazy/walk.cjs');

// --------------------
// Plugin loading
// --------------------

interface Diagnostic {
  message: string;
  node: {
    start: number;
    end: number;
    [key: string]: unknown;
  };
}

interface DiagnosticReport {
  message: string;
  loc: { start: number; end: number };
  ruleIndex: number;
}

interface Rule {
  create: (context: Context) => Visitor;
}

interface Plugin {
  meta: {
    name: string;
  };
  rules: {
    [key: string]: Rule;
  };
}

// Absolute paths of plugins which have been loaded
const registeredPluginPaths = new Set<string>();

// Rule objects for loaded rules.
// Indexed by `ruleId`, passed to `lintFile`.
const registeredRules: {
  rule: Rule;
  context: Context;
}[] = [];

/**
 * Load a plugin.
 *
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param {string} path - Absolute path of plugin file
 * @returns {string} - JSON result
 */
async function loadPlugin(path: string): Promise<string> {
  try {
    return await loadPluginImpl(path);
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

async function loadPluginImpl(path: string): Promise<string> {
  if (registeredPluginPaths.has(path)) {
    return JSON.stringify({
      Failure: 'This plugin has already been registered',
    });
  }

  const { default: plugin } = (await import(path)) as { default: Plugin };

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules
  const pluginName = plugin.meta.name;
  const offset = registeredRules.length;
  const ruleNames = [];

  for (const [ruleName, rule] of Object.entries(plugin.rules)) {
    ruleNames.push(ruleName);
    registeredRules.push({
      rule,
      context: new Context(`${pluginName}/${ruleName}`),
    });
  }

  return JSON.stringify({ Success: { name: pluginName, offset, ruleNames } });
}

/**
 * Update a `Context` with file-specific data.
 *
 * We have to define this function within class body, as it's not possible to set private property
 * `#ruleIndex` from outside the class.
 * We don't use a normal class method, because we don't want to expose this to user.
 *
 * @param context - `Context` object
 * @param ruleIndex - Index of this rule within `ruleIds` passed from Rust
 * @param filePath - Absolute path of file being linted
 */
let setupContextForFile: (
  context: Context,
  ruleIndex: number,
  filePath: string,
) => void;

/**
 * Context class.
 *
 * Each rule has its own `Context` object. It is passed to that rule's `create` function.
 */
class Context {
  // Full rule name, including plugin name e.g. `my-plugin/my-rule`.
  id: string;
  // Index into `ruleIds` sent from Rust. Set before calling `rule`'s `create` method.
  #ruleIndex: number;
  // Absolute path of file being linted. Set before calling `rule`'s `create` method.
  filename: string;
  // Absolute path of file being linted. Set before calling `rule`'s `create` method.
  physicalFilename: string;

  /**
   * @constructor
   * @param fullRuleName - Rule name, in form `<plugin>/<rule>`
   */
  constructor(fullRuleName: string) {
    this.id = fullRuleName;
  }

  /**
   * Report error.
   * @param diagnostic - Diagnostic object
   */
  report(diagnostic: Diagnostic): void {
    diagnostics.push({
      message: diagnostic.message,
      loc: { start: diagnostic.node.start, end: diagnostic.node.end },
      ruleIndex: this.#ruleIndex,
    });
  }

  static {
    setupContextForFile = (context, ruleIndex, filePath) => {
      context.#ruleIndex = ruleIndex;
      context.filename = filePath;
      context.physicalFilename = filePath;
    };
  }
}

// --------------------
// Running rules
// --------------------

interface BufferWithArrays extends Uint8Array {
  uint32: Uint32Array;
  float64: Float64Array;
}

// Buffers cache.
//
// All buffers sent from Rust are stored in this array, indexed by `bufferId` (also sent from Rust).
// Buffers are only added to this array, never removed, so no buffers will be garbage collected
// until the process exits.
const buffers: (BufferWithArrays | null)[] = [];

// Diagnostics array. Reused for every file.
const diagnostics: DiagnosticReport[] = [];

// Text decoder, for decoding source text from buffer
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

// Run rules on a file.
function lintFile(filePath: string, bufferId: number, buffer: Uint8Array | null, ruleIds: number[]) {
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

  // Get visitors for this file from all rules
  initCompiledVisitor();
  for (let i = 0; i < ruleIds.length; i++) {
    const ruleId = ruleIds[i];
    const { rule, context } = registeredRules[ruleId];
    setupContextForFile(context, i, filePath);
    const visitor = rule.create(context);
    addVisitorToCompiled(visitor);
  }
  const needsVisit = finalizeCompiledVisitor();

  // Visit AST.
  // Skip this if no visitors visit any nodes.
  // Some rules seen in the wild return an empty visitor object from `create` if some initial check fails
  // e.g. file extension is not one the rule acts on.
  if (needsVisit) {
    const { uint32 } = buffer,
      programPos = uint32[DATA_POINTER_POS_32],
      sourceByteLen = uint32[(programPos + SOURCE_LEN_OFFSET) >> 2];

    const sourceText = textDecoder.decode(buffer.subarray(0, sourceByteLen));
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
  }

  // Send diagnostics back to Rust
  const ret = JSON.stringify(diagnostics);
  diagnostics.length = 0;
  return ret;
}

// --------------------
// Run linter
// --------------------

// Call Rust, passing `loadPlugin` and `lintFile` as callbacks
const success = await lint(loadPlugin, lintFile);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
