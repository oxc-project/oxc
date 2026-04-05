import { describe, expect, it } from "vitest";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { format } from "../../dist/index.js";
import {
  REAL_SVELTE_EXPECTED_OUTPUT,
  REAL_SVELTE_INPUT,
  RUN_REAL_SVELTE_PACKAGE_SUITE,
  assertRealSveltePackagesAvailable,
  getMissingRealSveltePackages,
  getRealSvelteCompilerEntryPath,
  getRealSvelteFixtureDirPath,
  getRealSveltePluginEntryPath,
} from "../svelte_real_package_utils.ts";

const fixturesDir = getRealSvelteFixtureDirPath();
const missingPackages = getMissingRealSveltePackages(fixturesDir);

if (RUN_REAL_SVELTE_PACKAGE_SUITE) {
  assertRealSveltePackagesAvailable(fixturesDir);
}

const realPackageDescribe = missingPackages.length === 0 ? describe : describe.skip;

realPackageDescribe("runtime real prettier-plugin-svelte integration", () => {
  it("should resolve svelte/compiler from the fixture-local node_modules", () => {
    const realPluginPath = getRealSveltePluginEntryPath();
    const compilerPath = getRealSvelteCompilerEntryPath();
    const requireFromPlugin = createRequire(realPluginPath);
    const resolvedCompiler = requireFromPlugin.resolve("svelte/compiler");

    expect(resolvedCompiler).toBe(compilerPath);
  });

  it("should format .svelte through the real package resolved from cwd", async () => {
    const previousCwd = process.cwd();

    try {
      process.chdir(fixturesDir);

      const result = await format("App.svelte", REAL_SVELTE_INPUT, {
        plugins: ["prettier-plugin-svelte"],
        svelteSortOrder: "scripts-markup-styles-options",
        tabWidth: 2,
      } as any);

      expect(result.code).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
      expect(result.errors).toStrictEqual([]);
    } finally {
      process.chdir(previousCwd);
    }
  });

  it("should format .svelte through a direct real plugin object passed to the API", async () => {
    const realPluginPath = getRealSveltePluginEntryPath();
    const { default: sveltePlugin } = await import(pathToFileURL(realPluginPath).href);

    const result = await format("App.svelte", REAL_SVELTE_INPUT, {
      plugins: [sveltePlugin],
      svelteSortOrder: "scripts-markup-styles-options",
      tabWidth: 2,
    } as any);

    expect(result.code).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
    expect(result.errors).toStrictEqual([]);
  });

  it("preserves a missing final newline when it is the only real-plugin difference", async () => {
    const previousCwd = process.cwd();

    try {
      process.chdir(fixturesDir);

      const result = await format("AlreadyFormattedNoFinalNewline.svelte", REAL_SVELTE_EXPECTED_OUTPUT.trimEnd(), {
        plugins: ["prettier-plugin-svelte"],
        svelteSortOrder: "scripts-markup-styles-options",
        tabWidth: 2,
      } as any);

      expect(result.code).toBe(REAL_SVELTE_EXPECTED_OUTPUT.trimEnd());
      expect(result.errors).toStrictEqual([]);
    } finally {
      process.chdir(previousCwd);
    }
  });
});
