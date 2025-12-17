import { runCli } from "./bindings";
import {
  initExternalFormatter,
  formatEmbeddedCode,
  formatFile,
  processTailwindClasses,
} from "./cli/worker-proxy";

// napi-JS `oxfmt` CLI entry point
// See also `run_cli()` function in `./src/main_napi.rs`

void (async () => {
  const args = process.argv.slice(2);

  // Call the Rust CLI first, to parse args and determine mode
  // NOTE: If the mode is formatter CLI, it will also perform formatting and return an exit code
  const [mode, exitCode] = await runCli(
    args,
    initExternalFormatter,
    formatEmbeddedCode,
    formatFile,
    processTailwindClasses,
  );

  // Migration modes are handled by JS
  if (mode === "init") {
    await import("./cli/migration/init").then((m) => m.runInit());
    return;
  }
  if (mode === "migrate:prettier") {
    await import("./cli/migration/migrate-prettier").then((m) => m.runMigratePrettier());
    return;
  }

  // Other modes are handled by Rust, just need to set `exitCode`

  // NOTE: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
  // `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
  // https://nodejs.org/api/process.html#processexitcode
  process.exitCode = exitCode!;
})();
