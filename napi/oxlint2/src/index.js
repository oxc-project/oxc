import { createRequire } from 'node:module';
import { lint } from './bindings.js';
import { DATA_POINTER_POS_32, SOURCE_LEN_POS_32 } from './generated/constants.cjs';

// Import lazy visitor from `oxc-parser`.
// Use `require` not `import` as `oxc-parser` uses `require` internally,
// and need to make sure get same instance of modules as it uses internally,
// otherwise `TOKEN` here won't be same `TOKEN` as used within `oxc-parser`.
const require = createRequire(import.meta.url);
const { TOKEN } = require('../../parser/raw-transfer/lazy-common.js'),
  { Visitor, getVisitorsArr } = require('../../parser/raw-transfer/visitor.js'),
  walkProgram = require('../../parser/generated/lazy/walk.js');

const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

class PluginRegistry {
  registeredPluginPaths = new Set();

  registeredRules = [];

  isPluginRegistered(path) {
    return this.registeredPluginPaths.has(path);
  }

  registerPlugin(path, plugin) {
    // TODO: use a validation library to assert the shape of the plugin
    this.registeredPluginPaths.add(path);
    const ret = {
      name: plugin.meta.name,
      offset: this.registeredRules.length,
      rules: [],
    };

    for (const [ruleName, rule] of Object.entries(plugin.rules)) {
      ret.rules.push(ruleName);
      this.registeredRules.push(rule);
    }

    return ret;
  }

  *getRules(ruleIds) {
    for (const ruleId of ruleIds) {
      yield { rule: this.registeredRules[ruleId], ruleId };
    }
  }
}

// Buffers cache
const buffers = [];

class Linter {
  pluginRegistry = new PluginRegistry();

  run() {
    return lint(this.loadPlugin.bind(this), this.lint.bind(this));
  }

  /**
   * @param {string} path - The absolute path of the plugin we're loading
   */
  async loadPlugin(path) {
    if (this.pluginRegistry.isPluginRegistered(path)) {
      return JSON.stringify({
        Failure: 'This plugin has already been registered',
      });
    }

    try {
      const { default: plugin } = await import(path);
      const ret = this.pluginRegistry.registerPlugin(path, plugin);
      return JSON.stringify({ Success: ret });
    } catch (error) {
      const errorMessage = 'message' in error && typeof error.message === 'string'
        ? error.message
        : 'An unknown error occurred';
      return JSON.stringify({ Failure: errorMessage });
    }
  }

  // TODO(camc314): why do we have to destructure here?
  // In `./bindings.d.ts`, it doesn't indicate that we have to
  // (typed as `(filePath: string, bufferId: number, buffer: Uint8Array | undefined | null, ruleIds: number[])`).
  lint([filePath, bufferId, buffer, ruleIds]) {
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
    const diagnostics = [];

    const createContext = (ruleId) => ({
      physicalFilename: filePath,
      report: (diagnostic) => {
        diagnostics.push({
          message: diagnostic.message,
          loc: { start: diagnostic.node.start, end: diagnostic.node.end },
          externalRuleId: ruleId,
        });
      },
    });

    const visitors = [];
    for (const { rule, ruleId } of this.pluginRegistry.getRules(ruleIds)) {
      visitors.push(rule.create(createContext(ruleId)));
    }

    // TODO: Combine visitors for multiple rules
    const visitor = new Visitor(visitors[0]);

    // Visit AST
    const programPos = buffer.uint32[DATA_POINTER_POS_32],
      sourceByteLen = buffer.uint32[SOURCE_LEN_POS_32];

    const sourceText = textDecoder.decode(buffer.subarray(0, sourceByteLen));
    const sourceIsAscii = sourceText.length === sourceByteLen;
    const ast = { buffer, sourceText, sourceByteLen, sourceIsAscii, nodes: new Map(), token: TOKEN };

    walkProgram(programPos, ast, getVisitorsArr(visitor));

    // Send diagnostics back to Rust
    return JSON.stringify(diagnostics);
  }
}

async function main() {
  const linter = new Linter();

  const success = await linter.run();

  // Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
  // `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
  // https://nodejs.org/api/process.html#processexitcode
  if (!success) process.exitCode = 1;
}

main();
