// `oxfmt` CLI - Worker Process Entry Point

// Exit when the CLI process is gone, even if Prettier or a plugin leaves a timer or another handle.
// `process.connected` is already `false` if the IPC channel closed before this module loaded.
if (process.send !== undefined) {
  process.once("disconnect", () => process.exit(0));
  if (!process.connected) process.exit(0);
}

// Re-exports core functions for use as Tinypool tasks
export { formatEmbeddedDoc, formatFile, sortTailwindClasses } from "./libs/apis";
