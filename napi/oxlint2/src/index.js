import { lint } from './bindings.js';

class Linter {
  run() {
    return lint();
  }
}

function main() {
  const linter = new Linter();

  const result = linter.run();

  if (!result) {
    process.exit(1);
  }
}

main();
