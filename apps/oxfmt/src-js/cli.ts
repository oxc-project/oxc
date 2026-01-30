import { runCli } from "./bindings";
import {
  initExternalFormatter,
  formatEmbeddedCode,
  formatFile,
  sortTailwindClasses,
  disposeExternalFormatter,
} from "./cli/worker-proxy";

// napi-JS `oxfmt` CLI entry point
// See also `run_cli()` function in `./src/main_napi.rs`

void (async () => {
  const args = process.argv.slice(2);

  // Node.js sets non-TTY `stdio` to non-blocking mode,
  // which causes `WouldBlock` errors in Rust when writing large output with `--stdin-filepath`.
  // https://github.com/oxc-project/oxc/issues/17939 (issue was on macOS)
  //
  // As a workaround, if used with pipe, set blocking mode before calling NAPI bindings.
  // See: https://github.com/napi-rs/napi-rs/issues/1630
  // @ts-expect-error: `_handle` is an internal API
  if (!process.stdout.isTTY) process.stdout._handle?.setBlocking?.(true);

  // Call the Rust CLI first, to parse args and determine mode
  // NOTE: If the mode is formatter CLI, it will also perform formatting and return an exit code
  const [mode, exitCode] = await runCli(
    args,
    initExternalFormatter,
    formatEmbeddedCode,
    formatFile,
    sortTailwindClasses,
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
  if (mode === "migrate:biome") {
    await import("./cli/migration/migrate-biome").then((m) => m.runMigrateBiome());
    return;
  }

  // Other modes are handled by Rust, just need to set `exitCode`

  // Clean up worker pool to not V8 crashes on process exit
  await disposeExternalFormatter();

  // NOTE: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
  // `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
  // https://nodejs.org/api/process.html#processexitcode
  process.exitCode = exitCode!;
})();
