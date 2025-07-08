import { lint } from './bindings.js';

class Linter {
  pluginRegistry = new Map();

  run() {
    return lint(this.loadPlugin.bind(this), this.lint.bind(this));
  }

  loadPlugin = async (pluginName) => {
    if (this.pluginRegistry.has(pluginName)) {
      return JSON.stringify({ Success: null });
    }

    try {
      const plugin = await import(pluginName);
      this.pluginRegistry.set(pluginName, plugin);
      return JSON.stringify({ Success: null });
    } catch (error) {
      const errorMessage = 'message' in error && typeof error.message === 'string'
        ? error.message
        : 'An unknown error occurred';
      return JSON.stringify({ Failure: errorMessage });
    }
  };

  lint = async () => {
    throw new Error('unimplemented');
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
