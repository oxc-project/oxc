import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { join as pathJoin } from "node:path";
import process from "node:process";
import {
  REAL_SVELTE_REQUIRED_PACKAGES,
  REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH,
  REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH,
} from "../scripts/svelte-real-package-metadata.ts";

export const RUN_REAL_SVELTE_PACKAGE_SUITE = process.env.OXFMT_SVELTE_REAL_PACKAGES_CI === "1";
export const REAL_SVELTE_INPUT = `<style>h1{color:red}</style>
<h1>Hello {name}</h1>
<script>export let name = "world";</script>
`;
export const REAL_SVELTE_EXPECTED_OUTPUT = `<script>
  export let name = "world";
</script>

<h1>Hello {name}</h1>

<style>
  h1 {
    color: red;
  }
</style>
`;

const packageImportabilityCache = new Map<string, boolean>();

function canImportPackage(packageName: string, fromDirPath: string): boolean {
  const cacheKey = `${fromDirPath}\u0000${packageName}`;
  const cached = packageImportabilityCache.get(cacheKey);
  if (cached !== undefined) {
    return cached;
  }

  const result = spawnSync(
    process.execPath,
    ["--input-type=module", "--eval", `await import(${JSON.stringify(packageName)});`],
    {
      cwd: fromDirPath,
      stdio: "ignore",
    },
  );

  const canImport = result.status === 0;
  packageImportabilityCache.set(cacheKey, canImport);
  return canImport;
}

export function getMissingRealSveltePackages(fromDirPath: string = REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH): string[] {
  return REAL_SVELTE_REQUIRED_PACKAGES.filter((packageName) => !canImportPackage(packageName, fromDirPath));
}

export function assertRealSveltePackagesAvailable(
  fromDirPath: string = REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH,
): void {
  const missingPackages = getMissingRealSveltePackages(fromDirPath);
  if (missingPackages.length === 0) {
    return;
  }

  const message =
    "Real prettier-plugin-svelte tests require loadable `prettier`, `svelte`, and `prettier-plugin-svelte` packages. " +
    `Missing: ${missingPackages.join(", ")}.`;

  if (RUN_REAL_SVELTE_PACKAGE_SUITE) {
    throw new Error(message);
  }
}

export function getRealSvelteFixtureDirPath(): string {
  return REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH;
}

export function getRealSvelteConfigDirPath(): string {
  return REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH;
}

function getFixtureRequire(): NodeRequire {
  return createRequire(pathJoin(REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH, "empty.json"));
}

export function getRealSveltePluginEntryPath(): string {
  return getFixtureRequire().resolve("prettier-plugin-svelte");
}

export function getRealSvelteCompilerEntryPath(): string {
  return getFixtureRequire().resolve("svelte/compiler");
}
