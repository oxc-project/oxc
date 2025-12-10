import { format } from "./bindings.js";
import { setupConfig, formatEmbeddedCode, formatFile } from "./prettier-proxy.js";

const args = process.argv.slice(2);

// Call the Rust formatter with our JS callback
const success = await format(args, setupConfig, formatEmbeddedCode, formatFile);

// NOTE: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
