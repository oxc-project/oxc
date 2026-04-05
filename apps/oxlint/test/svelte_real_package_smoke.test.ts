import {
  cp,
  mkdtemp,
  readdir,
  readFile,
  realpath,
  rm,
  symlink,
} from "node:fs/promises";
import {
  basename as pathBasename,
  join as pathJoin,
  relative as pathRelative,
} from "node:path";

import { execa } from "execa";
import { describe, expect, it } from "vitest";

import { PACKAGE_ROOT_PATH, getMissingPackagesForFixture, normalizeStdout } from "./utils.ts";
import { getRealSvelteFixture, getRealSvelteFixtures } from "./real_svelte_fixtures.ts";
import { resolveRealSveltePackageProfileName } from "../scripts/svelte-real-package-metadata.ts";

import type { Fixture } from "./utils.ts";

const BUILT_CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");
const RAW_CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "src-js/cli.ts");

const TSX_CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "node_modules", "tsx", "dist", "cli.mjs");
const RUN_REAL_SVELTE_FIXTURE_SUITE = process.env.OXLINT_SVELTE_REAL_PACKAGES_CI === "1";
const SHARED_CLI_ENV = {
  ...process.env,
  DEBUG: "true",
  NODE_DISABLE_COLORS: "1",
};

const realSveltePackageProfileName = resolveRealSveltePackageProfileName(undefined);
const realSvelteSmokeTimeoutMs = realSveltePackageProfileName === "latest-svelte" ? 20_000 : 5_000;
const realSvelteDescribe = RUN_REAL_SVELTE_FIXTURE_SUITE ? describe : describe.skip;
const realSvelteFixtures = getRealSvelteFixtures();
const fixCapableFixtures = realSvelteFixtures.filter(
  (fixture) => fixture.options.fix || fixture.options.fixSuggestions,
);

realSvelteDescribe("oxlint CLI real-package Svelte smoke checks", () => {
  const recommendedFixture = getRealSvelteFixture("js_config_svelte_real_recommended_whole_file");
  const typeAwareFixture = getRealSvelteFixture("js_config_svelte_type_aware_whole_file");

  for (const fixture of realSvelteFixtures) {
    it(`keeps built and raw default CLI output in sync for ${fixture.name}`, { timeout: realSvelteSmokeTimeoutMs }, async () => {
      await assertFixturePackagesAvailable(fixture);
      await expectDefaultCliParity(fixture);
    });
  }

  for (const fixture of fixCapableFixtures) {
    if (fixture.options.fix) {
      it(`keeps built and raw --fix output in sync for ${fixture.name}`, async () => {
        await assertFixturePackagesAvailable(fixture);
        await expectMutationCliParity(fixture, "fix");
      });
    }

    if (fixture.options.fixSuggestions) {
      it(`keeps built and raw --fix-suggestions output in sync for ${fixture.name}`, async () => {
        await assertFixturePackagesAvailable(fixture);
        await expectMutationCliParity(fixture, "fix-suggestions");
      });
    }
  }

  it("keeps built and raw print-config output in sync for the recommended Svelte fixture", { timeout: realSvelteSmokeTimeoutMs }, async () => {
    await assertFixturePackagesAvailable(recommendedFixture);
    const built = await runBuiltCli(recommendedFixture, ["--print-config"]);
    const raw = await runRawCli(recommendedFixture, ["--print-config"]);

    expectCliParity(built, raw);
    expect(built.exitCode).toBe(0);
    expect(raw.exitCode).toBe(0);
    expect(built.stderr).toBe("");
    expect(raw.stderr).toBe("");

    const printedConfig = JSON.parse(raw.stdout) as { plugins?: string[] };
    const pluginNames = printedConfig.plugins ?? [];
    expect(Array.isArray(pluginNames)).toBe(true);
    expect(
      pluginNames.some((pluginName) => pluginName.toLowerCase().includes("svelte")),
    ).toBe(true);
    if (realSveltePackageProfileName !== "latest-svelte") {
      expect(pluginNames).toContain("svelte");
    }
  });

  it("keeps built and raw type-aware Svelte lint output in sync", { timeout: realSvelteSmokeTimeoutMs }, async () => {
    await assertFixturePackagesAvailable(typeAwareFixture);
    const built = await runBuiltCli(typeAwareFixture, getFixtureCliArgs(typeAwareFixture, "lint"));
    const raw = await runRawCli(typeAwareFixture, getFixtureCliArgs(typeAwareFixture, "lint"));

    expectCliParity(built, raw);
    expect(built.exitCode).toBe(1);
    expect(raw.exitCode).toBe(1);
    expect(built.stderr).toBe("");
    expect(raw.stderr).toBe("");
    expect(raw.stdout).toContain("projectService: true");
    expect(raw.stdout).toContain("extraFileExtensions: true");
    expect(raw.stdout).toContain("svelteRunes: true");
  });
});

type FixtureCommandMode = "lint" | "fix" | "fix-suggestions";

interface CliResult {
  exitCode: number;
  stdout: string;
  stderr: string;
}

interface FixtureMutation {
  path: string;
  content: string | null;
}

async function assertFixturePackagesAvailable(fixture: Fixture): Promise<void> {
  const missingPackages = getMissingPackagesForFixture(fixture);
  expect(
    missingPackages,
    `fixture ${fixture.name} is missing required packages: ${missingPackages.join(", ")}`,
  ).toEqual([]);
}

function getFixtureCliArgs(fixture: Fixture, mode: FixtureCommandMode): string[] {
  const args = fixture.options.singleThread ? ["--threads", "1"] : [];

  if (mode === "fix") {
    args.push("--fix");
  } else if (mode === "fix-suggestions") {
    args.push("--fix-suggestions");
  }

  return [...args, "files", ...fixture.options.args];
}

function getFixtureCwd(fixture: Fixture): string {
  return fixture.options.cwd === null
    ? fixture.dirPath
    : pathJoin(fixture.dirPath, fixture.options.cwd);
}

async function expectDefaultCliParity(fixture: Fixture): Promise<void> {
  const built = await runBuiltCli(fixture, getFixtureCliArgs(fixture, "lint"));
  const raw = await runRawCli(fixture, getFixtureCliArgs(fixture, "lint"));
  expectCliParity(built, raw);
}

async function expectMutationCliParity(
  fixture: Fixture,
  mode: Exclude<FixtureCommandMode, "lint">,
): Promise<void> {
  const originalFixtureFiles = await readTrackedFixtureFiles(fixture);

  await withTemporaryFixtureCopies(fixture, async ({ builtFixture, rawFixture }) => {
    const built = await runBuiltCli(builtFixture, getFixtureCliArgs(builtFixture, mode));
    const raw = await runRawCli(rawFixture, getFixtureCliArgs(rawFixture, mode));

    const [builtMutations, rawMutations] = await Promise.all([
      getFixtureMutations(builtFixture, originalFixtureFiles),
      getFixtureMutations(rawFixture, originalFixtureFiles),
    ]);

    expectCliParity(built, raw);
    expect(rawMutations).toEqual(builtMutations);
    expect(rawMutations.length).toBeGreaterThan(0);
  });
}

async function runBuiltCli(fixture: Fixture, args: string[]): Promise<CliResult> {
  const { stdout, stderr, exitCode } = await execa(process.execPath, [BUILT_CLI_PATH, ...args], {
    cwd: getFixtureCwd(fixture),
    reject: false,
    env: SHARED_CLI_ENV,
  });

  return normalizeCliResult(fixture, exitCode, stdout, stderr);
}

async function runRawCli(fixture: Fixture, args: string[]): Promise<CliResult> {
  const { stdout, stderr, exitCode } = await execa(process.execPath, [TSX_CLI_PATH, "--conditions=code", RAW_CLI_PATH, ...args], {
    cwd: getFixtureCwd(fixture),
    reject: false,
    env: SHARED_CLI_ENV,
  });

  return normalizeCliResult(fixture, exitCode, stdout, stderr);
}

function normalizeCliResult(fixture: Fixture, exitCode: number, stdout: string, stderr: string): CliResult {
  const fixtureName = pathBasename(fixture.dirPath);
  return {
    exitCode,
    stdout: normalizeStdout(stdout, fixtureName, false),
    stderr: normalizeStdout(stderr, fixtureName, false),
  };
}

function normalizeCliParityText(text: string): string {
  return text
    .replace(
      /(?:\/tmp\/oxlint-svelte-fixture-[^/\s]+|[^\n\r\t ]*\.tmp-svelte-fixture-[^/\s]+)\/(?:built|raw)\//g,
      "<fixture-copy>/",
    )
    .replace(
      new RegExp(
        String.raw`TypeError \[ERR_PACKAGE_IMPORT_NOT_DEFINED\]: Package import specifier "#oxlint" is not defined imported from <fixture-copy>/oxlint\.config\.ts`,
        "g",
      ),
      "Error: Cannot resolve '#oxlint' from <fixture-copy>/oxlint.config.ts",
    )
    .replace(
      new RegExp(
        String.raw`[ 	]*\| Error: Cannot find module '#oxlint'
[ 	]*\| Require stack:
[ 	]*\| - <fixture-copy>/oxlint\.config\.ts`,
        "g",
      ),
      "| Error: Cannot resolve '#oxlint' from <fixture-copy>/oxlint.config.ts",
    )
    .replace(
      /^\s*\| Error: Cannot resolve '#oxlint' from <fixture-copy>\/oxlint\.config\.ts$/gm,
      "| Error: Cannot resolve '#oxlint' from <fixture-copy>/oxlint.config.ts",
    );
}

function normalizeCliParityResult(result: CliResult): CliResult {
  return {
    ...result,
    stdout: normalizeCliParityText(result.stdout),
    stderr: normalizeCliParityText(result.stderr),
  };
}

function expectCliParity(built: CliResult, raw: CliResult): void {
  expect(normalizeCliParityResult(raw)).toEqual(normalizeCliParityResult(built));
}

async function readTrackedFixtureFiles(fixture: Fixture): Promise<Map<string, string>> {
  const fileEntries = await readdir(fixture.dirPath, {
    recursive: true,
    withFileTypes: true,
  });
  const trackedFiles = new Map<string, string>();

  for (const fileEntry of fileEntries) {
    if (!fileEntry.isFile()) {
      continue;
    }

    const filePath = pathJoin(fileEntry.parentPath, fileEntry.name);
    const relativePath = normalizeFixturePath(pathRelative(fixture.dirPath, filePath));
    if (!shouldTrackFixtureFile(relativePath)) {
      continue;
    }

    trackedFiles.set(relativePath, await readFile(filePath, "utf8"));
  }

  return trackedFiles;
}

async function getFixtureMutations(
  fixture: Fixture,
  originalFixtureFiles: ReadonlyMap<string, string>,
): Promise<FixtureMutation[]> {
  const currentFixtureFiles = await readTrackedFixtureFiles(fixture);
  const trackedPaths = new Set<string>([
    ...originalFixtureFiles.keys(),
    ...currentFixtureFiles.keys(),
  ]);
  const mutations: FixtureMutation[] = [];

  for (const trackedPath of [...trackedPaths].sort()) {
    const before = originalFixtureFiles.get(trackedPath);
    const after = currentFixtureFiles.get(trackedPath);
    if (before === after) {
      continue;
    }

    mutations.push({
      path: trackedPath,
      content: after ?? null,
    });
  }

  return mutations;
}

async function withTemporaryFixtureCopies<T>(
  fixture: Fixture,
  run: (fixtures: { builtFixture: Fixture; rawFixture: Fixture }) => Promise<T>,
): Promise<T> {
  const tempRootDirPath = await mkdtemp(pathJoin(PACKAGE_ROOT_PATH, ".tmp-svelte-fixture-"));
  const builtDirPath = pathJoin(tempRootDirPath, "built");
  const rawDirPath = pathJoin(tempRootDirPath, "raw");

  await Promise.all([
    copyFixtureToTempDir(fixture, builtDirPath),
    copyFixtureToTempDir(fixture, rawDirPath),
  ]);

  try {
    return await run({
      builtFixture: { ...fixture, dirPath: builtDirPath },
      rawFixture: { ...fixture, dirPath: rawDirPath },
    });
  } finally {
    await rm(tempRootDirPath, { recursive: true, force: true });
  }
}

async function copyFixtureToTempDir(fixture: Fixture, destinationDirPath: string): Promise<void> {
  await cp(fixture.dirPath, destinationDirPath, {
    recursive: true,
    filter(sourcePath) {
      return pathBasename(sourcePath) !== "node_modules";
    },
  });

  const sourceNodeModulesPath = pathJoin(fixture.dirPath, "node_modules");

  try {
    const resolvedNodeModulesPath = await realpath(sourceNodeModulesPath);
    await symlink(
      resolvedNodeModulesPath,
      pathJoin(destinationDirPath, "node_modules"),
      process.platform === "win32" ? "junction" : "dir",
    );
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") {
      throw error;
    }
  }
}

function shouldTrackFixtureFile(relativePath: string): boolean {
  return !relativePath.endsWith(".snap.md") && !relativePath.startsWith("node_modules/");
}

function normalizeFixturePath(path: string): string {
  return path.replaceAll("\\", "/");
}
