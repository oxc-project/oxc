import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { readFile, rm, writeFile } from "node:fs/promises";
import { runCli, runCliStdin, runWriteModeAndSnapshot } from "../utils";
import {
  REAL_SVELTE_EXPECTED_OUTPUT,
  REAL_SVELTE_INPUT,
  RUN_REAL_SVELTE_PACKAGE_SUITE,
  assertRealSveltePackagesAvailable,
  getMissingRealSveltePackages,
  getRealSvelteFixtureDirPath,
} from "../../svelte_real_package_utils.ts";

const fixturesDir = getRealSvelteFixtureDirPath();
const missingPackages = getMissingRealSveltePackages(fixturesDir);

if (RUN_REAL_SVELTE_PACKAGE_SUITE) {
  assertRealSveltePackagesAvailable(fixturesDir);
}

const realPackageDescribe = missingPackages.length === 0 ? describe : describe.skip;

realPackageDescribe("runtime plugin_languages_real_package", () => {
  it("should format .svelte through the npm-installed prettier-plugin-svelte package from local node_modules", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["App.svelte"]);

    expect(snapshot).toContain(`--- FILE -----------
App.svelte`);
    expect(snapshot).toContain(`--- BEFORE ---------
${REAL_SVELTE_INPUT}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });

  it("should format stdin .svelte through the npm-installed prettier-plugin-svelte package from local node_modules", async () => {
    const input = await readFile(join(fixturesDir, "App.svelte"), "utf-8");
    const result = await runCliStdin(input, "App.svelte", { cwd: fixturesDir });

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
  });

  it("should discover override-scoped npm-installed prettier-plugin-svelte config for stdin from the config directory", async () => {
    const input = await readFile(join(fixturesDir, "subdir", "config", "App.svelte"), "utf-8");
    const result = await runCliStdin(input, "subdir/config/App.svelte", {
      cwd: fixturesDir,
      args: ["-c", "./subdir/config/.oxfmtrc.json"],
    });

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
  });

  it("should discover override-scoped npm-installed prettier-plugin-svelte config from the config directory", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["subdir/config/App.svelte"],
      ["-c", "./subdir/config/.oxfmtrc.json"],
    );

    expect(snapshot).toContain(`--- FILE -----------
subdir/config/App.svelte`);
    expect(snapshot).toContain(`--- BEFORE ---------
${REAL_SVELTE_INPUT}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });

  it("should treat a newline-only EOF change from the npm-installed real plugin as clean in --check", async () => {
    const result = await runCli(fixturesDir, ["--check", "AlreadyFormattedNoFinalNewline.svelte"]);

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toContain("All matched files use the correct format.");
  });

  it("should omit a newline-only EOF change from the npm-installed real plugin in --list-different", async () => {
    const result = await runCli(fixturesDir, ["--list-different", "AlreadyFormattedNoFinalNewline.svelte"]);

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toBe("");
  });

  it("should treat an empty newline-only EOF change from the npm-installed real plugin as clean in --check", async () => {
    const tempFile = join(fixturesDir, "__temp_empty_no_final_newline__.svelte");

    try {
      await writeFile(tempFile, "");

      const result = await runCli(fixturesDir, ["--check", "__temp_empty_no_final_newline__.svelte"]);

      expect(result.exitCode).toBe(0);
      expect(result.stderr).toBe("");
      expect(result.stdout).toContain("All matched files use the correct format.");
    } finally {
      await rm(tempFile, { force: true });
    }
  });

  it("should omit an empty newline-only EOF change from the npm-installed real plugin in --list-different", async () => {
    const tempFile = join(fixturesDir, "__temp_empty_no_final_newline__.svelte");

    try {
      await writeFile(tempFile, "");

      const result = await runCli(fixturesDir, ["--list-different", "__temp_empty_no_final_newline__.svelte"]);

      expect(result.exitCode).toBe(0);
      expect(result.stderr).toBe("");
      expect(result.stdout).toBe("");
    } finally {
      await rm(tempFile, { force: true });
    }
  });
});
