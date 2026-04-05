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

const FIXTURES_DIR = join(import.meta.dirname, "..", "fixtures");
const realSveltePackageProfileName = resolveRealSveltePackageProfileName(undefined);
const expectedTypeAwareTsNodes = realSveltePackageProfileName === "latest-svelte" ? 0 : 1;
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

realSvelteDescribe("real-package Svelte LSP", () => {
  it("reports recommended-rule diagnostics for .svelte files", async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_recommended_whole_file");

    const snapshot = await lintFixture(
      FIXTURES_DIR,
      "js_config_svelte_real_recommended_whole_file/files/App.svelte",
      "svelte",
    );

    expect(snapshot).toContain("js_config_svelte_real_recommended_whole_file/files/App.svelte");
    expect(snapshot).toContain(
      "Unexpected mustache interpolation with a string literal value.",
    );
  });

  it("preserves type-aware Svelte parser services through the LSP path", async () => {
    await expectFixturePackagesAvailable("js_config_svelte_type_aware_whole_file");

    const snapshot = await lintFixture(
      FIXTURES_DIR,
      "js_config_svelte_type_aware_whole_file/files/App.svelte",
      "svelte",
    );

    expect(snapshot).toContain("js_config_svelte_type_aware_whole_file/files/App.svelte");
    expect(snapshot).toContain(
      "parser: true; services: true; projectService: true; extraFileExtensions: true",
    );
    expect(snapshot).toContain(
      "nestedParserFn: true; svelteRunes: true; preprocessFn: true; mergedBaseOption: true",
    );
    expect(snapshot).toContain(
      `scriptCount: 2; styleCount: 1; tsMaps: true; tsNodes: ${expectedTypeAwareTsNodes}; element: h1`,
    );
  });

  it("offers Svelte code actions from real JS plugins", async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_fixes_suggestions_whole_file");

    const snapshot = await fixFixture(
      FIXTURES_DIR,
      "js_config_svelte_real_fixes_suggestions_whole_file/files/App.svelte",
      "svelte",
    );

    expect(snapshot).toContain("js_config_svelte_real_fixes_suggestions_whole_file/files/App.svelte");
    expect(snapshot).toContain("Use Hi");
    expect(snapshot).toContain("Disable real-svelte-fixes/whole-file-edits for this line");
    expect(snapshot).toContain("Disable real-svelte-fixes/whole-file-edits for this whole file");
    expect(snapshot).toContain('<h1 class="welcome">Hello{name}</h1>');
    expect(snapshot).toContain('<h1 class="greeting">Hi{name}</h1>');
  });

  it("refreshes diagnostics after config changes for real-package .svelte files", async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_recommended_whole_file");

    const snapshot = await lintFixtureWithWorkspaceFileContentChange(
      FIXTURES_DIR,
      "js_config_svelte_real_recommended_whole_file",
      "js_config_svelte_real_recommended_whole_file/files/App.svelte",
      "svelte",
      "oxlint.config.ts",
      "oxlint.config.disabled.ts",
    );

    const parts = snapshot.split("=== After Config Change ===");
    expect(parts).toHaveLength(2);
    expect(parts[0]).toContain(
      "Unexpected mustache interpolation with a string literal value.",
    );
    expect(parts[1]).not.toContain(
      "Unexpected mustache interpolation with a string literal value.",
    );
  });

  it("keeps multiple Svelte workspaces isolated in one LSP session", async () => {
    await expectFixturePackagesAvailable("js_config_svelte_real_recommended_whole_file");
    await expectFixturePackagesAvailable("js_config_svelte_type_aware_whole_file");

    const snapshot = await lintMultiWorkspaceFixture(FIXTURES_DIR, [
      {
        path: "js_config_svelte_real_recommended_whole_file/files/App.svelte",
        languageId: "svelte",
      },
      {
        path: "js_config_svelte_type_aware_whole_file/files/App.svelte",
        languageId: "svelte",
      },
    ]);

    expect(snapshot).toContain("js_config_svelte_real_recommended_whole_file/files/App.svelte");
    expect(snapshot).toContain("js_config_svelte_type_aware_whole_file/files/App.svelte");
    expect(snapshot).toContain(
      "Unexpected mustache interpolation with a string literal value.",
    );
    expect(snapshot).toContain(
      "parser: true; services: true; projectService: true; extraFileExtensions: true",
    );
  });
});
