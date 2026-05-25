import { expect, it } from "vitest";
import {
  getUnsupportedTypeScriptModuleLoadHintForError,
  isTypeScriptModuleSpecifier,
  NODE_TYPESCRIPT_SUPPORT_RANGE,
} from "../../src-js/js_config/node_version.ts";

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
  ) as TypeError & { code?: string };
  err.code = "ERR_UNKNOWN_FILE_EXTENSION";

  expect(
    getUnsupportedTypeScriptModuleLoadHintForError(err, "/tmp/oxfmt.config.ts", "v22.11.0"),
  ).toBe(
    `${err.message}\n\nTypeScript config files require Node.js ${NODE_TYPESCRIPT_SUPPORT_RANGE}.\nDetected Node.js v22.11.0.\nPlease upgrade Node.js or use a JSON config file instead.`,
  );
});

it("does not add the hint for non-TypeScript specifiers or unrelated errors", () => {
  const err = new Error("Cannot find package");
  expect(getUnsupportedTypeScriptModuleLoadHintForError(err, "/tmp/oxfmt.config.ts")).toBeNull();

  const unknownExtension = new TypeError('Unknown file extension ".ts"');
  expect(
    getUnsupportedTypeScriptModuleLoadHintForError(unknownExtension, "/tmp/oxfmt.config.js"),
  ).toBeNull();
});
