import { lint } from './bindings.js';

class PluginRegistry {
  registeredPlugins = new Set();

  registeredRules = [];

  isPluginRegistered(path) {
    return this.registeredPlugins.has(path);
  }

  registerPlugin(path, plugin) {
    // TODO: use a validation library to assert the shape of the plugin
    this.registeredPlugins.add(path);
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
      yield this.registeredRules[ruleId];
    }
  }
}

class Linter {
  pluginRegistry = new PluginRegistry();

  run() {
    return lint(this.loadPlugin.bind(this), this.lint.bind(this));
  }

  /**
   * @param {string} pluginName The name of the plugin we're loading
   */
  loadPlugin = async (pluginName) => {
    if (this.pluginRegistry.isPluginRegistered(pluginName)) {
      return JSON.stringify({
        Failure: 'This plugin has already been registered',
      });
    }

    try {
      const { default: plugin } = await import(pluginName);
      const ret = this.pluginRegistry.registerPlugin(pluginName, plugin);
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
  };
}

async function main() {
  const linter = new Linter();

  const result = await linter.run();
  if (!result) {
    process.exit(1);
  }
}

main();
