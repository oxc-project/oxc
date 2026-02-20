import Tinypool from "tinypool";
import { resolvePlugins } from "../libs/apis";
import type {
  FormatEmbeddedCodeParam,
  FormatFileParam,
  SortTailwindClassesArgs,
} from "../libs/apis";

// Worker pool for parallel Prettier formatting
let pool: Tinypool | null = null;

export async function initExternalFormatter(numThreads: number): Promise<string[]> {
  pool = new Tinypool({
    filename: new URL("./cli-worker.js", import.meta.url).href,
    minThreads: numThreads,
    maxThreads: numThreads,
    // XXX: Use `child_process` instead of `worker_threads`.
    // Not sure why, but when using `worker_threads`,
    // calls from NAPI (CLI) -> worker threads -> NAPI (prettier-plugin-oxfmt) causes a hang...
    runtime: "child_process",
    // When setting the `runtime: child_process`,
    // `process.env` is not inherited (likely a bug), so it needs to be explicitly specified.
    env: process.env as Record<string, string>,
  });

  return resolvePlugins();
}

export async function disposeExternalFormatter(): Promise<void> {
  await pool?.destroy();
  pool = null;
}

export async function formatEmbeddedCode(
  options: FormatEmbeddedCodeParam["options"],
  code: string,
): Promise<string> {
  return pool!.run({ options, code } satisfies FormatEmbeddedCodeParam, {
    name: "formatEmbeddedCode",
  });
}

export async function formatFile(
  options: FormatFileParam["options"],
  code: string,
): Promise<string> {
  return (
    pool!
      .run({ options, code } satisfies FormatFileParam, { name: "formatFile" })
      // `tinypool` with `runtime: "child_process"` serializes Error as plain objects via IPC.
      // (e.g. `{ name, message, stack, ... }`)
      // And napi-rs converts unknown JS values to Rust Error by calling `String()` on them,
      // which yields `"[object Object]"` for plain objects...
      // So, this function reconstructs a proper `Error` instance so napi-rs can extract the message.
      .catch((err) => {
        if (err instanceof Error) throw err;
        if (err !== null && typeof err === "object") {
          const obj = err as { name: string; message: string };
          const newErr = new Error(obj.message);
          newErr.name = obj.name;
          throw newErr;
        }
        throw new Error(String(err));
      })
  );
}

export async function sortTailwindClasses(
  options: SortTailwindClassesArgs["options"],
  classes: string[],
): Promise<string[]> {
  return pool!.run({ classes, options } satisfies SortTailwindClassesArgs, {
    name: "sortTailwindClasses",
  });
}
