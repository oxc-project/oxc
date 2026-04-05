import { describe, expect, it } from "vitest";
import { formatFixture, formatFixtureAfterConfigChange, formatFixtureContent } from "./utils";
import {
  REAL_SVELTE_EXPECTED_OUTPUT,
  REAL_SVELTE_INPUT,
  RUN_REAL_SVELTE_PACKAGE_SUITE,
  assertRealSveltePackagesAvailable,
  getMissingRealSveltePackages,
  getRealSvelteFixtureDirPath,
} from "../svelte_real_package_utils.ts";

const fixturesDir = getRealSvelteFixtureDirPath();
const missingPackages = getMissingRealSveltePackages(fixturesDir);

if (RUN_REAL_SVELTE_PACKAGE_SUITE) {
  assertRealSveltePackagesAvailable(fixturesDir);
}

const realPackageDescribe = missingPackages.length === 0 ? describe : describe.skip;

realPackageDescribe("LSP runtime real prettier-plugin-svelte integration", () => {
  it("should format .svelte files through the npm-installed package from local node_modules", async () => {
    const snapshot = await formatFixture(fixturesDir, "App.svelte", "svelte");

    expect(snapshot).toContain(`--- BEFORE ---------
${REAL_SVELTE_INPUT}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });

  it("should discover override-scoped npm-installed prettier-plugin-svelte config in LSP", async () => {
    const snapshot = await formatFixture(fixturesDir, "subdir/config/App.svelte", "svelte");

    expect(snapshot).toContain(`--- BEFORE ---------
${REAL_SVELTE_INPUT}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });

  it.each([
    "untitled://Untitled-svelte",
    "vscode-userdata://svelte",
    "ccsettings://svelte",
  ])("should format in-memory Svelte documents for %s", async (uri) => {
    const snapshot = await formatFixtureContent(fixturesDir, "App.svelte", uri, "svelte");

    expect(snapshot).toContain(`--- URI -----------
${uri}`);
    expect(snapshot).toContain(`--- BEFORE ---------
${REAL_SVELTE_INPUT}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER ----------
${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });

  it("should restart formatter plugin support after LSP config changes", async () => {
    const snapshot = await formatFixtureAfterConfigChange(
      fixturesDir,
      "App.svelte",
      "svelte",
      {
        "fmt.configPath": "./empty.json",
      },
      {
        "fmt.configPath": null,
      },
    );

    expect(snapshot).toContain(`--- BEFORE ---------
${REAL_SVELTE_INPUT}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER FIRST FORMAT ----------
${REAL_SVELTE_INPUT}`.trimEnd());
    expect(snapshot).toContain(`--- AFTER SECOND FORMAT ----------
${REAL_SVELTE_EXPECTED_OUTPUT}`.trimEnd());
  });
});
