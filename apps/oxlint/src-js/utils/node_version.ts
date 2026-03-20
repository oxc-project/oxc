import { extname as pathExtname } from "node:path";
import { fileURLToPath } from "node:url";
import { getErrorMessage } from "./utils.ts";

export const NODE_TYPESCRIPT_SUPPORT_RANGE = "^20.19.0 || >=22.12.0";

const TS_MODULE_EXTENSIONS = new Set([".ts", ".mts", ".cts"]);

function normalizeModuleSpecifierPath(specifier: string): string {
  if (!specifier.startsWith("file:")) return specifier;

  try {
    return fileURLToPath(specifier);
  } catch {
    return specifier;
  }
}

export function isTypeScriptModuleSpecifier(specifier: string): boolean {
  const ext = pathExtname(normalizeModuleSpecifierPath(specifier)).toLowerCase();
  return TS_MODULE_EXTENSIONS.has(ext);
}

function isUnknownFileExtensionError(err: unknown): boolean {
  if ((err as { code?: unknown })?.code === "ERR_UNKNOWN_FILE_EXTENSION") return true;

  const message = (err as { message?: unknown })?.message;
  return typeof message === "string" && /unknown(?: or unsupported)? file extension/i.test(message);
}

export function getUnsupportedTypeScriptModuleLoadHintForError(
  err: unknown,
  specifier: string,
  nodeVersion: string = process.version,
): string | null {
  if (!isTypeScriptModuleSpecifier(specifier) || !isUnknownFileExtensionError(err)) {
    return null;
  }

  return `${getErrorMessage(err)}\n\nTypeScript config files require Node.js ${NODE_TYPESCRIPT_SUPPORT_RANGE}.\nDetected Node.js ${nodeVersion}.\nPlease upgrade Node.js or use a JSON config file instead.`;
}
