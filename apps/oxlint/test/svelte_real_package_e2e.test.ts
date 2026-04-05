import { join as pathJoin } from "node:path";

import { describe, expect, it } from "vitest";

import { PACKAGE_ROOT_PATH, getMissingPackagesForFixture, testFixtureWithCommand } from "./utils.ts";
import { getRealSvelteFixtures } from "./real_svelte_fixtures.ts";
import { resolveRealSveltePackageProfileName } from "../scripts/svelte-real-package-metadata.ts";

import type { ExpectStatic } from "vitest";
import type { Fixture } from "./utils.ts";

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");
const NODE_BIN_PATH = process.execPath;
const RUN_REAL_SVELTE_FIXTURE_SUITE = process.env.OXLINT_SVELTE_REAL_PACKAGES_CI === "1";

const svelteRealPackageFixtures = getRealSvelteFixtures();
const realSveltePackageProfileName = resolveRealSveltePackageProfileName(undefined);
const realSvelteFixtureTimeoutMs = realSveltePackageProfileName === "latest-svelte" ? 20_000 : 5_000;
const realSvelteDescribe = RUN_REAL_SVELTE_FIXTURE_SUITE ? describe : describe.skip;

// oxlint-disable-next-line valid-describe-callback
realSvelteDescribe("oxlint CLI real-package Svelte fixtures", {
  concurrent: process.platform !== "win32",
}, () => {
  it("collects the focused fixture set", () => {
    expect(svelteRealPackageFixtures.length).toBeGreaterThan(0);
  });

  for (const fixture of svelteRealPackageFixtures) {
    it(
      `fixture: ${fixture.name}`,
      {
        concurrent: process.platform !== "win32",
        retry: process.platform === "win32" ? 2 : 0,
        timeout: realSvelteFixtureTimeoutMs,
      },
      async ({ expect }) => {
        const missingPackages = getMissingPackagesForFixture(fixture);
        expect(
          missingPackages,
          `fixture ${fixture.name} is missing required packages: ${missingPackages.join(", ")}`,
        ).toEqual([]);

        await runFixture(fixture, expect);
      },
    );
  }
});

function getSnapshotName(_fixtureName: string, baseSnapshotName: string): string {
  return realSveltePackageProfileName === "latest-svelte"
    ? `${baseSnapshotName}.latest-svelte`
    : baseSnapshotName;
}

async function runFixture(fixture: Fixture, expect: ExpectStatic): Promise<void> {
  const { options } = fixture;
  const args = [
    CLI_PATH,
    ...(options.singleThread ? ["--threads", "1"] : []),
    "files",
    ...options.args,
  ];
  const testOptions = {
    command: NODE_BIN_PATH,
    args,
    fixture,
    snapshotName: getSnapshotName(fixture.name, "output"),
    isESLint: false,
    expect,
  };

  await testFixtureWithCommand(testOptions);

  if (options.fix) {
    const fixArgs = [args[0], "--fix", ...args.slice(1)];
    await testFixtureWithCommand({ ...testOptions, args: fixArgs, snapshotName: getSnapshotName(fixture.name, "fix") });
  }

  if (options.fixSuggestions) {
    const fixSuggestionsArgs = [args[0], "--fix-suggestions", ...args.slice(1)];
    await testFixtureWithCommand({
      ...testOptions,
      args: fixSuggestionsArgs,
      snapshotName: getSnapshotName(fixture.name, "fix-suggestions"),
    });
  }
}
