import { createRequire } from 'node:module';
import { lint } from './bindings.js';
import { DATA_POINTER_POS_32, SOURCE_LEN_OFFSET } from './generated/constants.cjs';
import { getErrorMessage } from './utils.js';
import { addVisitorToCompiled, compiledVisitor, finalizeCompiledVisitor, initCompiledVisitor } from './visitor.js';

// Import methods and objects from `oxc-parser`.
// Use `require` not `import` as `oxc-parser` uses `require` internally,
// and need to make sure get same instance of modules as it uses internally,
// otherwise `TOKEN` here won't be same `TOKEN` as used within `oxc-parser`.
const require = createRequire(import.meta.url);
const { TOKEN } = require('../../parser/raw-transfer/lazy-common.js'),
  walkProgram = require('../../parser/generated/lazy/walk.js');

// --------------------
// Plugin loading
// --------------------

// Absolute paths of plugins which have been loaded
const registeredPluginPaths = new Set();

// Rule objects for loaded rules.
// Indexed by `ruleId`, passed to `lintFile`.
const registeredRules = [];

/**
 * Load a plugin.
 *
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param {string} path - Absolute path of plugin file
 * @returns {string} - JSON result
 */
async function loadPlugin(path) {
  try {
    return await loadPluginImpl(path);
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

async function loadPluginImpl(path) {
  if (registeredPluginPaths.has(path)) {
    return JSON.stringify({ Failure: 'This plugin has already been registered' });
  }

  const { default: plugin } = await import(path);

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules
  const pluginName = plugin.meta.name;
  const offset = registeredRules.length;
  const ruleNames = [];

  for (const [ruleName, rule] of Object.entries(plugin.rules)) {
    ruleNames.push(ruleName);
    registeredRules.push({ rule, context: new Context(`${pluginName}/${ruleName}`) });
  }

  return JSON.stringify({ Success: { name: pluginName, offset, ruleNames } });
}

let setupContextForFile;

/**
 * Context class.
 *
 * Each rule has its own `Context` object. It is passed to that rule's `create` function.
 */
class Context {
  // Full rule name, including plugin name e.g. `my-plugin/my-rule`.
  id;
  // Index into `ruleIds` sent from Rust. Set before calling `rule`'s `create` method.
  #ruleIndex;
  // Absolute path of file being linted. Set before calling `rule`'s `create` method.
  filename;
  // Absolute path of file being linted. Set before calling `rule`'s `create` method.
  physicalFilename;

  /**
   * @constructor
   * @param {string} fullRuleName - Rule name, in form `<plugin>/<rule>`
   */
  constructor(fullRuleName) {
    this.id = fullRuleName;
  }

  /**
   * Report error.
   * @param {Object} diagnostic - Diagnostic object
   * @param {string} diagnostic.message - Error message
   * @param {Object} diagnostic.loc - Node or loc object
   * @param {number} diagnostic.loc.start - Start range of diagnostic
   * @param {number} diagnostic.loc.end - End range of diagnostic
   * @returns {undefined}
   */
  report(diagnostic) {
    diagnostics.push({
      message: diagnostic.message,
      loc: { start: diagnostic.node.start, end: diagnostic.node.end },
      ruleIndex: this.#ruleIndex,
    });
  }

  static {
    /**
     * Update a `Context` with file-specific data.
     *
     * We have to define this function within class body, as it's not possible to set private property
     * `#ruleIndex` from outside the class.
     * We don't use a normal class method, because we don't want to expose this to user.
     *
     * @param {Context} context - `Context` object
     * @param {number} ruleIndex - Index of this rule within `ruleIds` passed from Rust
     * @param {string} filePath - Absolute path of file being linted
     * @returns {undefined}
     */
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

// Buffers cache.
//
// All buffers sent from Rust are stored in this array, indexed by `bufferId` (also sent from Rust).
// Buffers are only added to this array, never removed, so no buffers will be garbage collected
// until the process exits.
const buffers = [];

// Diagnostics array. Reused for every file.
const diagnostics = [];

// Text decoder, for decoding source text from buffer
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

// Run rules on a file.
//
// TODO(camc314): why do we have to destructure here?
// In `./bindings.d.ts`, it doesn't indicate that we have to
// (typed as `(filePath: string, bufferId: number, buffer: Uint8Array | undefined | null, ruleIds: number[])`).
function lintFile([filePath, bufferId, buffer, ruleIds]) {
  // If new buffer, add it to `buffers` array. Otherwise, get existing buffer from array.
  // Do this before checks below, to make sure buffer doesn't get garbage collected when not expected
  // if there's an error.
  // TODO: Is this enough to guarantee soundness?
  if (buffer !== null) {
    const { buffer: arrayBuffer, byteOffset } = buffer;
    buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
    buffer.float64 = new Float64Array(arrayBuffer, byteOffset);

    while (buffers.length <= bufferId) {
      buffers.push(null);
    }
    buffers[bufferId] = buffer;
  } else {
    buffer = buffers[bufferId];
  }

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
    const { rule: { create }, context } = registeredRules[ruleId];
    setupContextForFile(context, i, filePath);
    const visitor = create(context);
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
    const ast = { buffer, sourceText, sourceByteLen, sourceIsAscii, nodes: new Map(), token: TOKEN };

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
