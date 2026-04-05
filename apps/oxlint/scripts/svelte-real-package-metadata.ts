import { join as pathJoin, resolve as pathResolve } from "node:path";
import process from "node:process";

export interface RealSvelteFixtureSpec {
  name: string;
  requiredPackages: readonly string[];
}

export interface RealSveltePackageProfileEntry {
  packageName: string;
  dependencySpecifier: string;
  expectedVersion: string | null;
}

export const REAL_SVELTE_HELPER_BASENAME = ".real-svelte-packages";
export const PACKAGE_ROOT_PATH = pathResolve(import.meta.dirname, "..");
export const FIXTURES_DIR_PATH = pathJoin(PACKAGE_ROOT_PATH, "test", "fixtures");
export const TEST_ROOT_PATH = pathJoin(PACKAGE_ROOT_PATH, "test");
export const REAL_SVELTE_REPORT_BASE_PATH = pathJoin(TEST_ROOT_PATH, REAL_SVELTE_HELPER_BASENAME);
export const INSTALL_ROOT_PATH = pathJoin(TEST_ROOT_PATH, REAL_SVELTE_HELPER_BASENAME);
export const INSTALL_NODE_MODULES_PATH = pathJoin(INSTALL_ROOT_PATH, "node_modules");
export const INSTALL_PACKAGE_JSON_PATH = pathJoin(INSTALL_ROOT_PATH, "package.json");
export const TEST_NODE_MODULES_LINK_PATH = pathJoin(TEST_ROOT_PATH, "node_modules");

const PINNED_REAL_SVELTE_PACKAGE_SPECS = [
  "eslint@9.39.4",
  "typescript@5.9.3",
  "@typescript-eslint/parser@8.57.2",
  "svelte@5.55.1",
  "svelte-eslint-parser@1.5.1",
  "eslint-plugin-svelte@3.16.0",
] as const;

const LATEST_SVELTE_FLOATING_PACKAGE_NAMES = [
  "svelte",
  "svelte-eslint-parser",
  "eslint-plugin-svelte",
] as const;

export type RealSveltePackageProfileName = "pinned" | "latest-svelte";

export const DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME = "pinned" satisfies RealSveltePackageProfileName;

interface ParsedPinnedPackageSpec {
  packageName: string;
  version: string;
}

function parsePinnedPackageSpec(packageSpec: string): ParsedPinnedPackageSpec {
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

export const REAL_SVELTE_PACKAGE_ENTRIES = REAL_SVELTE_PACKAGE_PROFILES[DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME];
export const REAL_SVELTE_PACKAGE_VERSION_MAP = getRealSvelteExpectedVersionMap(
  DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME,
);
export const REAL_SVELTE_PACKAGE_SPECIFIER_MAP = getRealSvelteDependencySpecifierMap(
  DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME,
);

export function isRealSveltePackageProfileName(value: string): value is RealSveltePackageProfileName {
  return REAL_SVELTE_PACKAGE_PROFILE_NAMES.includes(value as RealSveltePackageProfileName);
}

export function resolveRealSveltePackageProfileName(
  explicitValue: string | null | undefined,
): RealSveltePackageProfileName {
  const profileName =
    explicitValue ??
    process.env.OXLINT_SVELTE_REAL_PACKAGES_PROFILE ??
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

export function getFloatingRealSveltePackageNames(
  profileName: RealSveltePackageProfileName,
): readonly string[] {
  return getRealSveltePackageEntries(profileName)
    .filter((entry) => entry.expectedVersion === null)
    .map(({ packageName }) => packageName);
}

export function profileHasFloatingRealSveltePackages(profileName: RealSveltePackageProfileName): boolean {
  return getRealSveltePackageEntries(profileName).some((entry) => entry.expectedVersion === null);
}

export function makeRealSvelteProfileSuffix(profileName: RealSveltePackageProfileName): string {
  return profileName === DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME ? "" : `.${profileName}`;
}

export function getRealSvelteInstallRootPath(profileName: RealSveltePackageProfileName): string {
  return pathJoin(TEST_ROOT_PATH, `${REAL_SVELTE_HELPER_BASENAME}${makeRealSvelteProfileSuffix(profileName)}`);
}

export function getRealSvelteInstallNodeModulesPath(profileName: RealSveltePackageProfileName): string {
  return pathJoin(getRealSvelteInstallRootPath(profileName), "node_modules");
}

export function getRealSvelteInstallPackageJsonPath(profileName: RealSveltePackageProfileName): string {
  return pathJoin(getRealSvelteInstallRootPath(profileName), "package.json");
}

export function getRealSvelteDefaultReportPath(
  profileName: RealSveltePackageProfileName,
  format: "json" | "markdown",
): string {
  return `${getRealSvelteInstallRootPath(profileName)}-report.${format === "json" ? "json" : "md"}`;
}

export const REAL_SVELTE_REPORT_MARKDOWN_PATH = getRealSvelteDefaultReportPath(
  DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME,
  "markdown",
);
export const REAL_SVELTE_REPORT_JSON_PATH = getRealSvelteDefaultReportPath(
  DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME,
  "json",
);

export function getRealSvelteDefaultRunStatePath(profileName: RealSveltePackageProfileName): string {
  return `${getRealSvelteInstallRootPath(profileName)}-state.json`;
}

export const REAL_SVELTE_RUN_STATE_PATH = getRealSvelteDefaultRunStatePath(
  DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME,
);

export const REAL_SVELTE_FIXTURE_SPECS = [
  {
    name: "js_config_svelte_real_comments_tokens_whole_file",
    requiredPackages: ["svelte", "svelte-eslint-parser"],
  },
  {
    name: "js_config_svelte_real_disable_directives_whole_file",
    requiredPackages: ["svelte", "svelte-eslint-parser"],
  },
  {
    name: "js_config_svelte_real_fixes_suggestions_whole_file",
    requiredPackages: ["svelte", "svelte-eslint-parser"],
  },
  {
    name: "js_config_svelte_real_parser_whole_file",
    requiredPackages: ["svelte", "svelte-eslint-parser"],
  },
  {
    name: "js_config_svelte_real_recommended_whole_file",
    requiredPackages: ["svelte", "svelte-eslint-parser", "eslint-plugin-svelte"],
  },
  {
    name: "js_config_svelte_real_unused_disable_directives_whole_file",
    requiredPackages: ["svelte", "svelte-eslint-parser"],
  },
  {
    name: "js_config_svelte_type_aware_whole_file",
    requiredPackages: ["svelte", "svelte-eslint-parser", "@typescript-eslint/parser"],
  },
] as const satisfies readonly RealSvelteFixtureSpec[];

export const REAL_SVELTE_FIXTURE_NAME_SET = new Set(
  REAL_SVELTE_FIXTURE_SPECS.map(({ name }) => name),
);

export const REAL_SVELTE_TEST_FILES = [
  "./test/real_svelte_parser_whole_file.test.ts",
  "./test/real_svelte_plugin_whole_file.test.ts",
  "./test/svelte_real_package_manifest.test.ts",
  "./test/svelte_real_package_e2e.test.ts",
  "./test/svelte_real_package_smoke.test.ts",
  "./test/lsp/real_svelte_package.test.ts",
  "./test/lsp/real_svelte_package_smoke.test.ts",
] as const;


export const REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES = {
  runtime: "test:svelte-real-packages:runtime",
  fixtures: "test:svelte-real-packages:fixtures",
  smoke: "test:svelte-real-packages:smoke",
  lsp: "test:svelte-real-packages:lsp",
  "lsp-smoke": "test:svelte-real-packages:lsp-smoke",
} as const;

export type RealSvelteManagedSuiteName = keyof typeof REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES;

export const REAL_SVELTE_MANAGED_SUITE_NAMES = Object.freeze(
  Object.keys(REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES) as RealSvelteManagedSuiteName[],
);

export function getFixtureDirPath(name: string): string {
  return pathJoin(FIXTURES_DIR_PATH, name);
}

export function getFixtureNodeModulesPath(name: string): string {
  return pathJoin(getFixtureDirPath(name), "node_modules");
}

export function isRealSvelteFixtureName(name: string): boolean {
  return REAL_SVELTE_FIXTURE_NAME_SET.has(name);
}
