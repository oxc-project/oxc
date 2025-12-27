// `oxfmt` CLI - Worker Thread Entry Point

// Re-exports core functions for use in `worker_threads`
export { formatEmbeddedCode, formatFile, processTailwindClasses } from "./libs/prettier";
