import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME,
  PACKAGE_ROOT_PATH,
  REAL_SVELTE_FIXTURE_SPECS,
  REAL_SVELTE_MANAGED_SUITE_NAMES,
  REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  REAL_SVELTE_TEST_FILES,
  getFloatingRealSveltePackageNames,
  getRealSvelteDefaultRunStatePath,
  getRealSvelteInstallRootPath,
} from "../scripts/svelte-real-package-metadata.ts";
import { getFixtures } from "./utils.ts";

describe("real-package Svelte CI manifest", () => {
  it("keeps the focused fixture list explicit and in sync with fixture metadata", () => {
    const fixturesByName = new Map(getFixtures().map((fixture) => [fixture.name, fixture]));

    expect(REAL_SVELTE_FIXTURE_SPECS.length).toBeGreaterThan(0);

    for (const fixtureSpec of REAL_SVELTE_FIXTURE_SPECS) {
      const fixture = fixturesByName.get(fixtureSpec.name);
      expect(fixture, `missing fixture ${fixtureSpec.name}`).toBeDefined();
      expect(fixture?.options.requiredPackages).toEqual([...fixtureSpec.requiredPackages]);
    }
  });

  it("does not silently add new real-package Svelte fixtures outside the CI manifest", () => {
    const manifestFixtureNames = new Set(
      REAL_SVELTE_FIXTURE_SPECS.map(({ name }) => name),
    );
    const discoveredFixtureNames = getFixtures()
      .filter((fixture) => fixture.options.requiredPackages.includes("svelte-eslint-parser"))
      .map((fixture) => fixture.name)
      .sort();

    expect(discoveredFixtureNames).toEqual([...manifestFixtureNames].sort());
  });

  it("keeps the default profile pinned and limits floating canaries to Svelte ecosystem packages", () => {
    expect(DEFAULT_REAL_SVELTE_PACKAGE_PROFILE_NAME).toBe("pinned");
    expect([...REAL_SVELTE_PACKAGE_PROFILE_NAMES].sort()).toEqual(["latest-svelte", "pinned"]);
    expect([...getFloatingRealSveltePackageNames("pinned")]).toEqual([]);
    expect([...getFloatingRealSveltePackageNames("latest-svelte")].sort()).toEqual([
      "eslint-plugin-svelte",
      "svelte",
      "svelte-eslint-parser",
    ]);
  });


  it("keeps the focused real-package Svelte test files explicit and present", () => {
    expect(REAL_SVELTE_TEST_FILES).toContain("./test/lsp/real_svelte_package_smoke.test.ts");

    for (const testFile of REAL_SVELTE_TEST_FILES) {
      const filePath = join(PACKAGE_ROOT_PATH, testFile);
      expect(existsSync(filePath), `missing focused test file ${testFile}`).toBe(true);
    }
  });

  it("keeps managed suite metadata and package scripts in sync", () => {
    expect([...REAL_SVELTE_MANAGED_SUITE_NAMES]).toEqual([
      "runtime",
      "fixtures",
      "smoke",
      "lsp",
      "lsp-smoke",
    ]);

    const packageJson = JSON.parse(
      readFileSync(join(PACKAGE_ROOT_PATH, "package.json"), "utf8"),
    ) as {
      scripts: Record<string, string>;
    };

    const aggregateScript = packageJson.scripts["test:svelte-real-packages"];
    expect(aggregateScript).toBeTruthy();

    for (const suiteName of REAL_SVELTE_MANAGED_SUITE_NAMES) {
      const scriptName = REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES[suiteName];
      expect(packageJson.scripts[scriptName], `missing package script ${scriptName}`).toBeTruthy();
      expect(aggregateScript).toContain(`pnpm run ${scriptName}`);
    }
  });

  it("keeps managed-run state paths profile-specific and adjacent to diagnostic reports", () => {
    expect(getRealSvelteDefaultRunStatePath("pinned")).toMatch(/\.real-svelte-packages-state\.json$/);
    expect(getRealSvelteDefaultRunStatePath("latest-svelte")).toMatch(
      /\.real-svelte-packages\.latest-svelte-state\.json$/,
    );
  });

  it("keeps helper install roots profile-specific so pinned and canary installs do not overwrite each other", () => {
    expect(getRealSvelteInstallRootPath("pinned")).toMatch(/\.real-svelte-packages$/);
    expect(getRealSvelteInstallRootPath("latest-svelte")).toMatch(/\.real-svelte-packages\.latest-svelte$/);
    expect(getRealSvelteInstallRootPath("pinned")).not.toBe(getRealSvelteInstallRootPath("latest-svelte"));
  });
});
