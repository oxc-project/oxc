import { pathToFileURL } from "node:url";
import { isObject } from "../utils.ts";
import { getUnsupportedTypeScriptModuleLoadHintForError } from "./node_version.ts";

/**
 * Import a JS/TS config file and return its `default` export as a plain object.
 *
 * - Bypasses Node.js module cache (uses `?cache=<key>`) so changed files reload (used for LSP).
 * - On `ERR_UNKNOWN_FILE_EXTENSION` for TS specifiers, wraps the error with a Node.js upgrade hint message;
 *   The original error is preserved via `Error.cause`.
 *
 * @param path - Absolute path to the JS/TS config file
 * @param cacheKey - Cache-busting key.
 *   Callers decide whether to use a fresh value per call or share one across a batch.
 * @throws When the file has no `default` export, the export is not a plain object,
 *   or import fails (wrapped with hint message for unsupported TS module load).
 */
export async function importJsConfig(path: string, cacheKey: number): Promise<object> {
  const fileUrl = pathToFileURL(path);
  fileUrl.searchParams.set("cache", cacheKey.toString());

  let module;
  try {
    module = await import(fileUrl.href);
  } catch (err) {
    const hint = getUnsupportedTypeScriptModuleLoadHintForError(err, path);
    if (hint) throw new Error(hint, { cause: err });
    throw err;
  }

  if (module.default === undefined) {
    throw new Error("Configuration file has no default export.");
  }
  if (!isObject(module.default)) {
    throw new Error("Configuration file must have a default export that is an object.");
  }

  return module.default;
}
