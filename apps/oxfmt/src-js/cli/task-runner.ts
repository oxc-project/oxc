import { resolve } from "node:path";
import { ignoreInput, ignoreOutput } from "@voidzero-dev/vite-task-client";

/**
 * The directory `--cache` uses, as the Rust side resolves it:
 * `--cache-location` against the current directory, else `node_modules/.cache/oxfmt` in it.
 */
export function cacheLocation(args: string[]): string {
  let location: string | undefined;
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;
    if (arg === "--") break;
    if (arg === "--cache-location") location = args[++i];
    else if (arg.startsWith("--cache-location=")) location = arg.slice("--cache-location=".length);
  }
  return resolve(location ?? "node_modules/.cache/oxfmt");
}

/**
 * Under a task runner's cache (`vp run --cache`), keep oxfmt's own cache out of the task's inputs
 * and outputs: what it holds never changes what oxfmt prints or writes, so a change to it must not
 * miss the task's cache, and it isn't worth archiving. No-op outside a runner.
 */
export function excludeCacheFromTaskRunner(args: string[]): void {
  const dir = cacheLocation(args);
  ignoreInput(dir);
  ignoreOutput(dir);
}
