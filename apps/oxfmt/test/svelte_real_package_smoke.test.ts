import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import * as builtApi from "../dist/index.js";
import * as rawApi from "../src-js/index.ts";
import { runCliStdinWithLaunchMode, runCliWithLaunchMode } from "./cli/utils";
import { formatFixture, formatFixtureAfterConfigChange, formatFixtureContent } from "./lsp/utils";
import {
  REAL_SVELTE_EXPECTED_OUTPUT,
  REAL_SVELTE_INPUT,
  RUN_REAL_SVELTE_PACKAGE_SUITE,
  assertRealSveltePackagesAvailable,
  getMissingRealSveltePackages,
  getRealSvelteFixtureDirPath,
  getRealSveltePluginEntryPath,
} from "./svelte_real_package_utils.ts";

const fixturesDir = getRealSvelteFixtureDirPath();
const missingPackages = getMissingRealSveltePackages(fixturesDir);

if (RUN_REAL_SVELTE_PACKAGE_SUITE) {
  assertRealSveltePackagesAvailable(fixturesDir);
}

const realPackageDescribe = missingPackages.length === 0 ? describe : describe.skip;

realPackageDescribe("real-package Svelte formatter built-vs-raw smoke checks", () => {
  it("keeps built and raw API string-plugin formatting in sync", async () => {
    const previousCwd = process.cwd();

    try {
      process.chdir(fixturesDir);

      const [built, raw] = await Promise.all([
        builtApi.format("App.svelte", REAL_SVELTE_INPUT, {
          plugins: ["prettier-plugin-svelte"],
          svelteSortOrder: "scripts-markup-styles-options",
          tabWidth: 2,
        } as any),
        rawApi.format("App.svelte", REAL_SVELTE_INPUT, {
          plugins: ["prettier-plugin-svelte"],
          svelteSortOrder: "scripts-markup-styles-options",
          tabWidth: 2,
        } as any),
      ]);

      expect(raw).toStrictEqual(built);
      expect(built.code).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
      expect(built.errors).toStrictEqual([]);
    } finally {
      process.chdir(previousCwd);
    }
  });

  it("keeps built and raw API direct-plugin formatting in sync", async () => {
    const realPluginPath = getRealSveltePluginEntryPath();
    const { default: sveltePlugin } = await import(pathToFileURL(realPluginPath).href);

    const [built, raw] = await Promise.all([
      builtApi.format("App.svelte", REAL_SVELTE_INPUT, {
        plugins: [sveltePlugin],
        svelteSortOrder: "scripts-markup-styles-options",
        tabWidth: 2,
      } as any),
      rawApi.format("App.svelte", REAL_SVELTE_INPUT, {
        plugins: [sveltePlugin],
        svelteSortOrder: "scripts-markup-styles-options",
        tabWidth: 2,
      } as any),
    ]);

    expect(raw).toStrictEqual(built);
    expect(built.code).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
  });

  it("keeps built and raw CLI stdin formatting in sync", async () => {
    const [built, raw] = await Promise.all([
      runCliStdinWithLaunchMode(REAL_SVELTE_INPUT, "App.svelte", "built", { cwd: fixturesDir }),
      runCliStdinWithLaunchMode(REAL_SVELTE_INPUT, "App.svelte", "raw", { cwd: fixturesDir }),
    ]);

    expectCliParity(built, raw);
    expect(built.exitCode).toBe(0);
    expect(built.stderr).toBe("");
    expect(built.stdout).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
  });

  it("keeps built and raw CLI write-mode formatting in sync", async () => {
    const tempDir = await mkdtemp(join(fixturesDir, "__temp-smoke-"));
    const builtFileName = "BuiltWrite.svelte";
    const rawFileName = "RawWrite.svelte";
    const builtFilePath = join(tempDir, builtFileName);
    const rawFilePath = join(tempDir, rawFileName);

    try {
      await Promise.all([
        writeFile(builtFilePath, REAL_SVELTE_INPUT),
        writeFile(rawFilePath, REAL_SVELTE_INPUT),
      ]);

      const [built, raw] = await Promise.all([
        runCliWithLaunchMode(tempDir, [builtFileName], "built"),
        runCliWithLaunchMode(tempDir, [rawFileName], "raw"),
      ]);

      const [builtOutput, rawOutput] = await Promise.all([
        readFile(builtFilePath, "utf8"),
        readFile(rawFilePath, "utf8"),
      ]);

      expectCliParity(built, raw);
      expect(builtOutput).toBe(REAL_SVELTE_EXPECTED_OUTPUT);
      expect(rawOutput).toBe(builtOutput);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it("keeps built and raw CLI --check output in sync for newline-only EOF differences", async () => {
    const [built, raw] = await Promise.all([
      runCliWithLaunchMode(fixturesDir, ["--check", "AlreadyFormattedNoFinalNewline.svelte"], "built"),
      runCliWithLaunchMode(fixturesDir, ["--check", "AlreadyFormattedNoFinalNewline.svelte"], "raw"),
    ]);

    expectCliParity(built, raw);
    expect(built.exitCode).toBe(0);
    expect(built.stderr).toBe("");
    expect(built.stdout).toContain("All matched files use the correct format.");
  });

  it("keeps built and raw CLI --list-different output in sync for newline-only EOF differences", async () => {
    const [built, raw] = await Promise.all([
      runCliWithLaunchMode(fixturesDir, ["--list-different", "AlreadyFormattedNoFinalNewline.svelte"], "built"),
      runCliWithLaunchMode(fixturesDir, ["--list-different", "AlreadyFormattedNoFinalNewline.svelte"], "raw"),
    ]);

    expectCliParity(built, raw);
    expect(built.exitCode).toBe(0);
    expect(built.stderr).toBe("");
    expect(built.stdout).toBe("");
  });

  it("keeps built and raw LSP file formatting in sync", async () => {
    const [built, raw] = await Promise.all([
      formatFixture(fixturesDir, "App.svelte", "svelte", undefined, "built"),
      formatFixture(fixturesDir, "App.svelte", "svelte", undefined, "raw"),
    ]);

    expect(raw).toBe(built);
    expect(built).toContain(`--- AFTER ----------\n${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });

  it("keeps built and raw LSP in-memory formatting in sync", async () => {
    const uri = "untitled://Untitled-svelte-smoke";
    const [built, raw] = await Promise.all([
      formatFixtureContent(fixturesDir, "App.svelte", uri, "svelte", undefined, "built"),
      formatFixtureContent(fixturesDir, "App.svelte", uri, "svelte", undefined, "raw"),
    ]);

    expect(raw).toBe(built);
    expect(built).toContain(`--- URI -----------\n${uri}`);
  });

  it("keeps built and raw LSP config reload behavior in sync", async () => {
    const [built, raw] = await Promise.all([
      formatFixtureAfterConfigChange(
        fixturesDir,
        "App.svelte",
        "svelte",
        {
          "fmt.configPath": "./empty.json",
        },
        {
          "fmt.configPath": null,
        },
        "built",
      ),
      formatFixtureAfterConfigChange(
        fixturesDir,
        "App.svelte",
        "svelte",
        {
          "fmt.configPath": "./empty.json",
        },
        {
          "fmt.configPath": null,
        },
        "raw",
      ),
    ]);

    expect(raw).toBe(built);
    expect(built).toContain(`--- AFTER FIRST FORMAT ----------\n${REAL_SVELTE_INPUT}`.trimEnd());
    expect(built).toContain(`--- AFTER SECOND FORMAT ----------\n${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });
});

function expectCliParity(
  built: { exitCode: number | null; stdout: string; stderr: string },
  raw: { exitCode: number | null; stdout: string; stderr: string },
): void {
  expect(raw.exitCode).toBe(built.exitCode);
  expect(raw.stdout).toBe(built.stdout);
  expect(raw.stderr).toBe(built.stderr);
}
