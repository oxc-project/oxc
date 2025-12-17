import { runCli } from "./bindings.js";
import { setupConfig, formatEmbeddedCode, formatFile } from "./prettier-proxy.js";
import { runInit, runMigratePrettier } from "./migration/index.js";

void (async () => {
  const args = process.argv.slice(2);

  // Call the Rust CLI to parse args and determine mode
  // NOTE: If the mode is formatter CLI, it will also perform formatting and return an exit code
  const [mode, exitCode] = await runCli(args, setupConfig, formatEmbeddedCode, formatFile);

  switch (mode) {
    // Handle `--init` and `--migrate` command in JS
    case "init":
      await runInit();
      break;
    case "migrate:prettier":
      await runMigratePrettier();
      break;
    // Other modes are handled by Rust, just need to set `exitCode`
    default:
      // NOTE: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
      // `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
      // https://nodejs.org/api/process.html#processexitcode
      if (exitCode) process.exitCode = exitCode;
      break;
  }
})();
