// Bootstrap entry that enables compile cache before loading the CLI.
// This must be a separate file because ESM static imports are resolved
// before any top-level code runs.
import module from "node:module";
module.enableCompileCache?.();

await import("./cli-main.ts");
