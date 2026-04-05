import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { getRealSvelteFixture } from "../real_svelte_fixtures.ts";
import { getMissingPackagesForFixture } from "../utils.ts";
import { resolveRealSveltePackageProfileName } from "../../scripts/svelte-real-package-metadata.ts";
import {
  fixFixture,
  lintFixture,
  lintFixtureWithWorkspaceFileContentChange,
  lintMultiWorkspaceFixture,
} from "./utils";

import type { LspLaunchMode } from "./utils";

const FIXTURES_DIR = join(import.meta.dirname, "..", "fixtures");
const realSveltePackageProfileName = resolveRealSveltePackageProfileName(undefined);
const realSvelteLspSmokeTimeoutMs = realSveltePackageProfileName === "latest-svelte" ? 20_000 : 5_000;
const RUN_REAL_SVELTE_LSP_SUITE = process.env.OXLINT_SVELTE_REAL_PACKAGES_CI === "1";
const realSvelteDescribe = RUN_REAL_SVELTE_LSP_SUITE ? describe : describe.skip;

async function expectFixturePackagesAvailable(fixtureName: string): Promise<void> {
  const fixture = getRealSvelteFixture(fixtureName);
  const missingPackages = getMissingPackagesForFixture(fixture);

  expect(
    missingPackages,
    `fixture ${fixture.name} is missing required packages: ${missingPackages.join(", ")}`,
  ).toEqual([]);
}

async function expectBuiltAndRawLspParity(
  snapshotFactory: (launchMode: LspLaunchMode) => Promise<string>,
): Promise<string> {
  const built = await snapshotFactory("built");
  const raw = await snapshotFactory("raw");

  expect(raw).toEqual(built);
  return built;
}

realSvelteDescribe("real-package Svelte LSP built-vs-raw smoke checks", () => {
  it("keeps recommended-rule diagnostics in sync for .svelte files", { timeout: realSvelteLspSmokeTimeoutMs }, async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_recommended_whole_file");

    const snapshot = await expectBuiltAndRawLspParity((launchMode) =>
      lintFixture(
        FIXTURES_DIR,
        "js_config_svelte_real_recommended_whole_file/files/App.svelte",
        "svelte",
        undefined,
        launchMode,
      ),
    );

    expect(snapshot).toContain("js_config_svelte_real_recommended_whole_file/files/App.svelte");
    expect(snapshot).toContain("Unexpected mustache interpolation with a string literal value.");
  });

  it("keeps type-aware .svelte diagnostics in sync", { timeout: realSvelteLspSmokeTimeoutMs }, async () => {
    await expectFixturePackagesAvailable("js_config_svelte_type_aware_whole_file");

    const snapshot = await expectBuiltAndRawLspParity((launchMode) =>
      lintFixture(
        FIXTURES_DIR,
        "js_config_svelte_type_aware_whole_file/files/App.svelte",
        "svelte",
        undefined,
        launchMode,
      ),
    );

    expect(snapshot).toContain("js_config_svelte_type_aware_whole_file/files/App.svelte");
    expect(snapshot).toContain(
      "parser: true; services: true; projectService: true; extraFileExtensions: true",
    );
    expect(snapshot).toContain(
      "nestedParserFn: true; svelteRunes: true; preprocessFn: true; mergedBaseOption: true",
    );
  });

  it("keeps Svelte code actions from real JS plugins in sync", { timeout: realSvelteLspSmokeTimeoutMs }, async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_fixes_suggestions_whole_file");

    const snapshot = await expectBuiltAndRawLspParity((launchMode) =>
      fixFixture(
        FIXTURES_DIR,
        "js_config_svelte_real_fixes_suggestions_whole_file/files/App.svelte",
        "svelte",
        undefined,
        launchMode,
      ),
    );

    expect(snapshot).toContain(
      "js_config_svelte_real_fixes_suggestions_whole_file/files/App.svelte",
    );
    expect(snapshot).toContain("Use Hi");
    expect(snapshot).toContain("Disable real-svelte-fixes/whole-file-edits for this line");
    expect(snapshot).toContain('<h1 class="greeting">Hi{name}</h1>');
  });

  it("keeps config-reload diagnostics in sync for real-package .svelte files", { timeout: realSvelteLspSmokeTimeoutMs }, async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_recommended_whole_file");

    const snapshot = await expectBuiltAndRawLspParity((launchMode) =>
      lintFixtureWithWorkspaceFileContentChange(
        FIXTURES_DIR,
        "js_config_svelte_real_recommended_whole_file",
        "js_config_svelte_real_recommended_whole_file/files/App.svelte",
        "svelte",
        "oxlint.config.ts",
        "oxlint.config.disabled.ts",
        undefined,
        launchMode,
      ),
    );

    const parts = snapshot.split("=== After Config Change ===");
    expect(parts).toHaveLength(2);
    expect(parts[0]).toContain("Unexpected mustache interpolation with a string literal value.");
    expect(parts[1]).not.toContain(
      "Unexpected mustache interpolation with a string literal value.",
    );
  });

  it("keeps multi-workspace Svelte diagnostics in sync in one LSP session", { timeout: realSvelteLspSmokeTimeoutMs }, async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_recommended_whole_file");
    await expectFixturePackagesAvailable("js_config_svelte_type_aware_whole_file");

    const snapshot = await expectBuiltAndRawLspParity((launchMode) =>
      lintMultiWorkspaceFixture(
        FIXTURES_DIR,
        [
          {
            path: "js_config_svelte_real_recommended_whole_file/files/App.svelte",
            languageId: "svelte",
          },
          {
            path: "js_config_svelte_type_aware_whole_file/files/App.svelte",
            languageId: "svelte",
          },
        ],
        undefined,
        launchMode,
      ),
    );

    expect(snapshot).toContain("js_config_svelte_real_recommended_whole_file/files/App.svelte");
    expect(snapshot).toContain("js_config_svelte_type_aware_whole_file/files/App.svelte");
    expect(snapshot).toContain("Unexpected mustache interpolation with a string literal value.");
    expect(snapshot).toContain(
      "parser: true; services: true; projectService: true; extraFileExtensions: true",
    );
  });
});
