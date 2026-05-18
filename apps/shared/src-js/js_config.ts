import { pathToFileURL } from "node:url";
import { getUnsupportedTypeScriptModuleLoadHintForError } from "./node_version.ts";

/**
 * Import a JS/TS config file and return its `default` export.
 *
 * - Bypasses Node.js module cache (uses `?cache=<key>`) so changed files reload (used for LSP).
 * - On `ERR_UNKNOWN_FILE_EXTENSION` for TS specifiers, overwrites `err.message` with a
 *   Node.js upgrade hint that already includes the original message.
 *
 * @param path - Absolute path to the JS/TS config file
 * @param cacheKey - Cache-busting key.
 *   Callers decide whether to use a fresh value per call or share one across a batch.
 * @throws When the file has no `default` export,
 *   or import fails (with rewritten message for unsupported TS module load).
 */
export async function importJsConfig(path: string, cacheKey: number): Promise<unknown> {
  const fileUrl = pathToFileURL(path);
  fileUrl.searchParams.set("cache", cacheKey.toString());

  let module;
  try {
    module = await import(fileUrl.href);
  } catch (err) {
    const hint = getUnsupportedTypeScriptModuleLoadHintForError(err, path);
    if (hint && err instanceof Error) err.message = hint;
    throw err;
  }

  if (module.default === undefined) {
    throw new Error("Configuration file has no default export.");
  }

  return module.default;
}
