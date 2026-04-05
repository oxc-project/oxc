import { readFile } from "node:fs/promises";
import { describe, expect, it } from "vitest";
import {
  REAL_SVELTE_MANAGED_SUITE_NAMES,
  REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  REAL_SVELTE_TEST_FILES,
  getRealSvelteDefaultReportPath,
  getRealSvelteDefaultRunStatePath,
} from "../scripts/svelte-real-package-metadata.ts";

const packageJsonPath = new URL("../package.json", import.meta.url);

describe("oxfmt real-package svelte manifest", () => {
  it("keeps the profile list explicit", () => {
    expect([...REAL_SVELTE_PACKAGE_PROFILE_NAMES].sort()).toEqual(["latest-svelte", "pinned"]);
  });

  it("keeps the focused real-package Svelte test file list explicit", () => {
    expect([...REAL_SVELTE_TEST_FILES].sort()).toEqual([
      "./test/api/plugin_languages_runtime_real_package.test.ts",
      "./test/cli/plugin_languages_runtime_real_package/plugin_languages_runtime_real_package.test.ts",
      "./test/lsp/plugin_languages_runtime_real_package.test.ts",
      "./test/svelte_real_package_smoke.test.ts",
    ]);
  });

  it("keeps the managed suite list explicit", () => {
    expect([...REAL_SVELTE_MANAGED_SUITE_NAMES].sort()).toEqual(["api", "cli", "lsp", "smoke"]);
  });

  it("keeps diagnostics path naming profile-aware", () => {
    expect(getRealSvelteDefaultReportPath("pinned", "markdown")).toContain(".real-svelte-packages-report.md");
    expect(getRealSvelteDefaultReportPath("latest-svelte", "json")).toContain(
      ".real-svelte-packages.latest-svelte-report.json",
    );
    expect(getRealSvelteDefaultRunStatePath("pinned")).toContain(".real-svelte-packages-state.json");
    expect(getRealSvelteDefaultRunStatePath("latest-svelte")).toContain(
      ".real-svelte-packages.latest-svelte-state.json",
    );
  });

  it("keeps package scripts in sync with the managed suite list", async () => {
    const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8")) as {
      scripts: Record<string, string>;
    };

    for (const suiteName of REAL_SVELTE_MANAGED_SUITE_NAMES) {
      expect(packageJson.scripts[REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES[suiteName]]).toBeTruthy();
    }

    expect(packageJson.scripts["report:svelte-real-packages"]).toBeTruthy();
    expect(packageJson.scripts["report:svelte-real-packages:latest-svelte"]).toBeTruthy();
    expect(packageJson.scripts["annotate:svelte-real-packages"]).toBeTruthy();
    expect(packageJson.scripts["annotate:svelte-real-packages:latest-svelte"]).toBeTruthy();

    const aggregateScript = packageJson.scripts["test:svelte-real-packages"];
    expect(aggregateScript).toContain("setup:svelte-real-packages");
    expect(aggregateScript).toContain("check:svelte-real-packages");

    for (const scriptName of Object.values(REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES)) {
      expect(aggregateScript).toContain(scriptName);
    }
  });
});
