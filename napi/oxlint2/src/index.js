import { lint } from './bindings.js';

class Linter {
  run() {
    return lint(this.loadPlugin.bind(this), this.lint.bind(this));
  }

  loadPlugin = async (_pluginName) => {
    throw new Error('unimplemented');
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
