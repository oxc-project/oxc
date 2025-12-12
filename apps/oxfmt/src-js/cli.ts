import { format } from "./bindings.js";
import { setupConfig, formatEmbeddedCode, formatFile } from "./prettier-proxy.js";
import { runInit } from "./migration/init.js";

void (async () => {
  const args = process.argv.slice(2);

  // Handle `--init` command in JS
  if (args.includes("--init")) {
    return await runInit();
  }

  // Call the Rust formatter with our JS callback
  const success = await format(args, setupConfig, formatEmbeddedCode, formatFile);

  // NOTE: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
  // `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
  // https://nodejs.org/api/process.html#processexitcode
  if (!success) process.exitCode = 1;
})();
