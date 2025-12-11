/*
 * ESLint Rule Tester Conformance Script
 *
 * This script runs ESLint's rule tests using Oxlint's RuleTester to verify API compatibility.
 *
 * It works by:
 * 1. Patching NodeJS's CommonJS loader to substitute ESLint's `RuleTester` with Oxlint's.
 * 2. Hooking `describe` and `it` to capture test results.
 * 3. Loading each ESLint rule test file.
 * 4. Recording success/failure of each test.
 * 5. Outputting results to a markdown file.
 */

// NodeJS's `assert` produces coloured output unsuitable for logging to file when colors are enabled.
// Need to set `process.env` before the `assert` module is loaded for it to reliably take effect.
// So set it here, and use dynamic import to load the rest of the code.
process.env.NODE_DISABLE_COLORS = "1";

await import("./main.ts");

export {};
