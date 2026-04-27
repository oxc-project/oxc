import { runCli } from "./bindings";
import {
  initExternalFormatter,
  disposeExternalFormatter,
  formatFile,
  formatEmbeddedCode,
  formatEmbeddedDoc,
  sortTailwindClasses,
} from "./cli/worker-proxy";
import { loadJsConfig, loadVitePlusConfig } from "./cli/js_config/index";

// napi-JS `oxfmt` CLI entry point
// See also `run_cli()` function in `./src/main_napi.rs`

void (async () => {
  const args = process.argv.slice(2);

  // Node.js sets non-TTY `stdio` to non-blocking mode,
  // which causes `WouldBlock` errors in Rust.
  // As a workaround, if used with pipe, set blocking mode before calling NAPI bindings.
  // See: https://github.com/napi-rs/napi-rs/issues/1630
  //
  // stdout: Writing large formatted output via `--stdin-filepath` can overflow the pipe buffer.
  // https://github.com/oxc-project/oxc/issues/17939 (observed on macOS)
  // @ts-expect-error: `_handle` is an internal API
  if (!process.stdout.isTTY) process.stdout._handle?.setBlocking?.(true);
  // stdin: In LSP mode (`--lsp`), VSCode communicates via stdin/stdout pipes.
  // Rust reads stdin expecting blocking I/O, but non-blocking mode returns `EAGAIN` (os error 11).
  // https://github.com/oxc-project/oxc/issues/20285
  // @ts-expect-error: `_handle` is an internal API
  if (!process.stdin.isTTY) process.stdin._handle?.setBlocking?.(true);

  // LSP uses stdout for communication, so write logs to stderr to avoid breaking the protocol.
  // Since LSP is handled on the Rust side, we have to check the flag here. (`runCli()` starts the server and waits)
  // Also, for Oxfmt, this only actually affects loading JS/TS config files.
  // The call to Prettier is currently done via `child_process`, so it won't break LSP.
  if (args.includes("--lsp")) process.stdout.write = process.stderr.write.bind(process.stderr);

  // Call the Rust CLI first, to parse args and determine mode
  // NOTE: If the mode is formatter CLI, it will also perform formatting and return an exit code
  const [mode, exitCode] = await runCli(
    args,
    process.env.VP_VERSION ? loadVitePlusConfig : loadJsConfig,
    initExternalFormatter,
    formatFile,
    formatEmbeddedCode,
    formatEmbeddedDoc,
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

  // Node.js < 25.4.0 has a race condition with ThreadsafeFunction cleanup that causes
  // crashes on large codebases. Add a small delay to allow pending NAPI operations
  // to complete before exit. Fixed in Node.js 25.4.0+.
  // See: https://github.com/nodejs/node/issues/55706
  const [major, minor] = process.versions.node.split(".").map(Number);
  if (major < 25 || (major === 25 && minor < 4)) {
    setTimeout(() => process.exit(), 50);
  }
})();
