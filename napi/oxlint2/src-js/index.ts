import { lint } from './bindings.js';
import { lintFile, loadPlugin } from './plugins/index.js';

// Call Rust, passing `loadPlugin` and `lintFile` as callbacks
const success = await lint(loadPlugin, lintFile);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
