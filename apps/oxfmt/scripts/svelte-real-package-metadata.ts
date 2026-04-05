import { join as pathJoin, resolve as pathResolve } from "node:path";
import process from "node:process";

export interface RealSveltePackageProfileEntry {
  packageName: string;
  dependencySpecifier: string;
  expectedVersion: string | null;
}

export type RealSveltePackageProfileName = "pinned" | "latest-svelte";
export type RealSvelteManagedSuiteName = "api" | "cli" | "lsp" | "smoke";
export type RealSvelteReportFormat = "json" | "markdown";

export const REAL_SVELTE_HELPER_BASENAME = ".real-svelte-packages";
export const PACKAGE_ROOT_PATH = pathResolve(import.meta.dirname, "..");
export const TEST_ROOT_PATH = pathJoin(PACKAGE_ROOT_PATH, "test");
export const REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH = pathJoin(
  TEST_ROOT_PATH,
  "cli",
  "plugin_languages_runtime_real_package",
  "fixtures",
);
export const REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH = pathJoin(
  REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH,
  "subdir",
  "config",
);
export const REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH = pathJoin(
  REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH,
  "node_modules",
);

const PINNED_REAL_SVELTE_PACKAGE_SPECS = [
  "prettier@3.8.1",
  "svelte@5.55.1",
  "prettier-plugin-svelte@3.5.1",
] as const;

const LATEST_SVELTE_FLOATING_PACKAGE_NAMES = ["svelte", "prettier-plugin-svelte"] as const;

export const DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME = "pinned" satisfies RealSveltePackageProfileName;

function parsePinnedPackageSpec(packageSpec: string): { packageName: string; version: string } {
  const separatorIndex = packageSpec.lastIndexOf("@");
  if (separatorIndex <= 0) {
    throw new Error(`Expected a pinned package spec, got ${packageSpec}.`);
  }

  return {
    packageName: packageSpec.slice(0, separatorIndex),
    version: packageSpec.slice(separatorIndex + 1),
  };
}

const PINNED_REAL_SVELTE_PACKAGE_ENTRIES = PINNED_REAL_SVELTE_PACKAGE_SPECS.map((packageSpec) => {
  const { packageName, version } = parsePinnedPackageSpec(packageSpec);
  return {
    packageName,
    dependencySpecifier: version,
    expectedVersion: version,
  } satisfies RealSveltePackageProfileEntry;
});

const LATEST_SVELTE_FLOATING_PACKAGE_NAME_SET = new Set(LATEST_SVELTE_FLOATING_PACKAGE_NAMES);

const LATEST_SVELTE_REAL_SVELTE_PACKAGE_ENTRIES = PINNED_REAL_SVELTE_PACKAGE_ENTRIES.map((entry) => ({
  packageName: entry.packageName,
  dependencySpecifier: LATEST_SVELTE_FLOATING_PACKAGE_NAME_SET.has(entry.packageName)
    ? "latest"
    : entry.dependencySpecifier,
  expectedVersion: LATEST_SVELTE_FLOATING_PACKAGE_NAME_SET.has(entry.packageName)
    ? null
    : entry.expectedVersion,
})) satisfies readonly RealSveltePackageProfileEntry[];

export const REAL_SVELTE_PACKAGE_PROFILES = {
  pinned: PINNED_REAL_SVELTE_PACKAGE_ENTRIES,
  "latest-svelte": LATEST_SVELTE_REAL_SVELTE_PACKAGE_ENTRIES,
} as const satisfies Readonly<Record<RealSveltePackageProfileName, readonly RealSveltePackageProfileEntry[]>>;

export const REAL_SVELTE_PACKAGE_PROFILE_NAMES = Object.freeze(
  Object.keys(REAL_SVELTE_PACKAGE_PROFILES) as RealSveltePackageProfileName[],
);

export const REAL_SVELTE_REQUIRED_PACKAGES = Object.freeze(
  PINNED_REAL_SVELTE_PACKAGE_ENTRIES.map(({ packageName }) => packageName),
);

export const REAL_SVELTE_TEST_FILES = [
  "./test/api/plugin_languages_runtime_real_package.test.ts",
  "./test/cli/plugin_languages_runtime_real_package/plugin_languages_runtime_real_package.test.ts",
  "./test/lsp/plugin_languages_runtime_real_package.test.ts",
  "./test/svelte_real_package_smoke.test.ts",
] as const;

export const REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES = {
  api: "test:svelte-real-packages:api",
  cli: "test:svelte-real-packages:cli",
  lsp: "test:svelte-real-packages:lsp",
  smoke: "test:svelte-real-packages:smoke",
} as const satisfies Readonly<Record<RealSvelteManagedSuiteName, string>>;

export const REAL_SVELTE_MANAGED_SUITE_NAMES = Object.freeze(
  Object.keys(REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES) as RealSvelteManagedSuiteName[],
);

export function isRealSveltePackageProfileName(value: string): value is RealSveltePackageProfileName {
  return REAL_SVELTE_PACKAGE_PROFILE_NAMES.includes(value as RealSveltePackageProfileName);
}

export function resolveRealSveltePackageProfileName(
  explicitValue: string | null | undefined,
): RealSveltePackageProfileName {
  const profileName =
    explicitValue ??
    process.env.OXFMT_SVELTE_REAL_PACKAGES_PROFILE ??
    DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME;

  if (!isRealSveltePackageProfileName(profileName)) {
    throw new Error(
      `Unknown real-package Svelte profile: ${profileName}. ` +
        `Expected one of: ${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join(", ")}.`,
    );
  }

  return profileName;
}

export function getRealSveltePackageEntries(
  profileName: RealSveltePackageProfileName,
): readonly RealSveltePackageProfileEntry[] {
  return REAL_SVELTE_PACKAGE_PROFILES[profileName];
}

export function getRealSvelteDependencySpecifierMap(
  profileName: RealSveltePackageProfileName,
): Readonly<Record<string, string>> {
  return Object.freeze(
    Object.fromEntries(
      getRealSveltePackageEntries(profileName).map(({ packageName, dependencySpecifier }) => [
        packageName,
        dependencySpecifier,
      ]),
    ),
  ) as Readonly<Record<string, string>>;
}

export function getRealSvelteExpectedVersionMap(
  profileName: RealSveltePackageProfileName,
): Readonly<Record<string, string>> {
  return Object.freeze(
    Object.fromEntries(
      getRealSveltePackageEntries(profileName)
        .filter((entry): entry is RealSveltePackageProfileEntry & { expectedVersion: string } =>
          typeof entry.expectedVersion === "string",
        )
        .map(({ packageName, expectedVersion }) => [packageName, expectedVersion]),
    ),
  ) as Readonly<Record<string, string>>;
}

export function getRealSvelteInstallRootPath(profileName: RealSveltePackageProfileName): string {
  return pathJoin(
    TEST_ROOT_PATH,
    `${REAL_SVELTE_HELPER_BASENAME}${profileName === DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME ? "" : `.${profileName}`}`,
  );
}

export function getRealSvelteInstallNodeModulesPath(profileName: RealSveltePackageProfileName): string {
  return pathJoin(getRealSvelteInstallRootPath(profileName), "node_modules");
}

export function getRealSvelteInstallPackageJsonPath(profileName: RealSveltePackageProfileName): string {
  return pathJoin(getRealSvelteInstallRootPath(profileName), "package.json");
}

export function getRealSvelteDefaultReportPath(
  profileName: RealSveltePackageProfileName,
  format: RealSvelteReportFormat,
): string {
  return `${getRealSvelteInstallRootPath(profileName)}-report.${format === "markdown" ? "md" : "json"}`;
}

export function getRealSvelteDefaultRunStatePath(profileName: RealSveltePackageProfileName): string {
  return `${getRealSvelteInstallRootPath(profileName)}-state.json`;
}

export const REAL_SVELTE_RUN_STATE_PATH = getRealSvelteDefaultRunStatePath(
  DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME,
);
