import { join as pathJoin } from "node:path";
import { describe, it } from "vitest";
import { PACKAGE_ROOT_PATH, getFixtures, testFixtureWithCommand } from "./utils.ts";

import type { Fixture } from "./utils.ts";
import type { ExpectStatic } from "vitest";

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");

// Use current NodeJS executable, rather than `node`, to avoid problems with a Node version manager
// installed on system resulting in using wrong NodeJS version
const NODE_BIN_PATH = process.execPath;

/**
 * Run Oxlint tests for all fixtures in `test/fixtures`.
 *
 * Oxlint is run with:
 * - CWD set to the fixture directory.
 * - `files` as the only argument (so only lints the files in the fixture's `files` directory).
 *
 * Fixtures with an `options.json` file containing `"fix": true` are also run with `--fix` CLI option.
 * The files' contents after fixes are recorded in the snapshot.
 *
 * Fixtures with an `options.json` file containing `"oxlint": false` are skipped.
 */
// oxlint-disable-next-line valid-describe-callback
describe("oxlint CLI", { concurrent: process.platform !== "win32" }, () => {
  const fixtures = getFixtures();
  for (const fixture of fixtures) {
    if (!fixture.options.oxlint) continue;

    // oxlint-disable-next-line jest/expect-expect
    it(`fixture: ${fixture.name}`, { concurrent: process.platform !== "win32" }, ({ expect }) =>
      runFixture(fixture, expect),
    );
  }
});

/**
 * Run Oxlint on a test fixture.
 * @param fixture - Fixture object
 * @param expect - Vitest expect function from test context
 */
async function runFixture(fixture: Fixture, expect: ExpectStatic): Promise<void> {
  // Run Oxlint without `--fix` option
  await testFixtureWithCommand({
    command: NODE_BIN_PATH,
    args: [CLI_PATH, "files"],
    fixture,
    snapshotName: "output",
    isESLint: false,
    expect,
  });

  // Run Oxlint with `--fix` option
  if (fixture.options.fix) {
    await testFixtureWithCommand({
      command: NODE_BIN_PATH,
      args: [CLI_PATH, "--fix", "files"],
      fixture,
      snapshotName: "fix",
      isESLint: false,
      expect,
    });
  }
}
