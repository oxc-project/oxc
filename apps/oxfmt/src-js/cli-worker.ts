// `oxfmt` CLI - Worker Process Entry Point
// Runs in each Tinypool `child_process` worker spawned by `./cli/worker-proxy.ts`.

// Exit when the CLI process is gone.
// The CLI calls `process.exit()` without waiting for the workers (see `cli.ts`),
// which closes the IPC channel to each worker.
// An idle worker then exits by itself, but a timer or other handle left by Prettier or a plugin would keep it alive.
// (`process.connected` is already `false` if the channel closed before this module was loaded.)
if (process.send !== undefined) {
  process.once("disconnect", () => process.exit(0));
  if (!process.connected) process.exit(0);
}

// Re-exports core functions for use as Tinypool tasks
export {
  formatEmbeddedCode,
  formatEmbeddedDoc,
  formatFile,
  sortTailwindClasses,
} from "./libs/apis";
