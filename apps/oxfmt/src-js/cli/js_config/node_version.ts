import { extname as pathExtname } from "node:path";
import { fileURLToPath } from "node:url";

const NODE_TYPESCRIPT_SUPPORT_RANGE = "^20.19.0 || >=22.12.0";
const TS_MODULE_EXTENSIONS = new Set([".ts", ".mts", ".cts"]);

export function getUnsupportedTypeScriptModuleLoadHint(
  err: unknown,
  specifier: string,
  nodeVersion: string = process.version,
): string | null {
  if (!isTypeScriptModuleSpecifier(specifier) || !isUnknownFileExtensionError(err)) return null;

  return `TypeScript config files require Node.js ${NODE_TYPESCRIPT_SUPPORT_RANGE}.\nDetected Node.js ${nodeVersion}.\nPlease upgrade Node.js or use a JSON config file instead.`;
}

// ---

function isTypeScriptModuleSpecifier(specifier: string): boolean {
  const ext = pathExtname(normalizeModuleSpecifierPath(specifier)).toLowerCase();
  return TS_MODULE_EXTENSIONS.has(ext);
}

function normalizeModuleSpecifierPath(specifier: string): string {
  if (!specifier.startsWith("file:")) return specifier;

  try {
    return fileURLToPath(specifier);
  } catch {
    return specifier;
  }
}

function isUnknownFileExtensionError(err: unknown): boolean {
  if ((err as { code?: unknown })?.code === "ERR_UNKNOWN_FILE_EXTENSION") return true;

  const message = (err as { message?: unknown })?.message;
  return typeof message === "string" && /unknown(?: or unsupported)? file extension/i.test(message);
}

// ---

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it("detects supported TypeScript config specifiers", () => {
    expect(isTypeScriptModuleSpecifier("/tmp/oxfmt.config.ts")).toBe(true);
    expect(isTypeScriptModuleSpecifier("/tmp/oxfmt.config.mts")).toBe(true);
    expect(isTypeScriptModuleSpecifier("/tmp/oxfmt.config.cts")).toBe(true);
    expect(isTypeScriptModuleSpecifier("file:///tmp/oxfmt.config.ts")).toBe(true);
    expect(isTypeScriptModuleSpecifier("/tmp/oxfmt.config.js")).toBe(false);
  });

  it("returns a node version hint for unsupported TypeScript module loading", () => {
    const err = new TypeError(
      'Unknown file extension ".ts" for /tmp/oxfmt.config.ts',
    ) as TypeError & {
      code?: string;
    };
    err.code = "ERR_UNKNOWN_FILE_EXTENSION";

    expect(getUnsupportedTypeScriptModuleLoadHint(err, "/tmp/oxfmt.config.ts", "v22.11.0")).toBe(
      `TypeScript config files require Node.js ${NODE_TYPESCRIPT_SUPPORT_RANGE}.\nDetected Node.js v22.11.0.\nPlease upgrade Node.js or use a JSON config file instead.`,
    );
  });

  it("does not add the hint for non-TypeScript specifiers or unrelated errors", () => {
    const err = new Error("Cannot find package");
    expect(getUnsupportedTypeScriptModuleLoadHint(err, "/tmp/oxfmt.config.ts")).toBeNull();

    const unknownExtension = new TypeError('Unknown file extension ".ts"');
    expect(
      getUnsupportedTypeScriptModuleLoadHint(unknownExtension, "/tmp/oxfmt.config.js"),
    ).toBeNull();
  });
}
