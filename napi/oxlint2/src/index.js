import { lint } from './bindings.js';

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

class Linter {
  pluginRegistry = new PluginRegistry();

  run() {
    return lint(this.loadPlugin.bind(this), this.lint.bind(this));
  }

  /**
   * @param {string} path - The absolute path of the plugin we're loading
   */
  loadPlugin = async (path) => {
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
  };

  // TODO(camc314): why do we have to destructure here?
  // In `./bindings.d.ts`, it doesn't indicate that we have to (typed as `(filePath: string, ruleIds: number[]))`
  lint = ([filePath, ruleIds]) => {
    if (typeof filePath !== 'string' || filePath.length === 0) {
      throw new Error('expected filePath to be a non-zero length string');
    }
    if (!Array.isArray(ruleIds) || ruleIds.length === 0) {
      throw new Error('Expected `ruleIds` to be a non-zero len array');
    }

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

    const rules = [];
    for (const { rule, ruleId } of this.pluginRegistry.getRules(ruleIds)) {
      rules.push(rule(createContext(ruleId)));
    }

    // TODO: walk the AST

    return JSON.stringify(diagnostics);
  };
}

async function main() {
  const linter = new Linter();

  const success = await linter.run();
  if (!success) {
    process.exit(1);
  }
}

main();
