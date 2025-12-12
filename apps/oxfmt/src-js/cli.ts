import { runCli } from "./bindings.js";
import {
  setupConfig,
  formatEmbeddedCode,
  formatFile,
  processTailwindClasses,
} from "./prettier-proxy.js";
import { runInit } from "./migration/init.js";

void (async () => {
  const args = process.argv.slice(2);

  // Call the Rust CLI to parse args and determine mode
  const [mode, exitCode] = await runCli(
    args,
    setupConfig,
    formatEmbeddedCode,
    formatFile,
    processTailwindClasses,
  );

  switch (mode) {
    // Handle `--init` command in JS
    case "init":
      await runInit();
      break;
    // LSP mode is handled by Rust, nothing to do here
    case "lsp":
      break;
    // CLI mode also handled by Rust, just need to set exit code
    case "cli":
      // NOTE: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
      // `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
      // https://nodejs.org/api/process.html#processexitcode
      if (exitCode) process.exitCode = exitCode;
      break;
  }
})();
