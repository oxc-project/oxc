import { join as pathJoin } from "node:path";
import { describe, it } from "vitest";
import { PACKAGE_ROOT_PATH, getFixtures, testFixtureWithCommand } from "./utils.ts";

import type { Fixture } from "./utils.ts";

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
describe("oxlint CLI", () => {
  const fixtures = getFixtures();
  for (const fixture of fixtures) {
    if (!fixture.options.oxlint) continue;

    // oxlint-disable-next-line jest/expect-expect
    it(`fixture: ${fixture.name}`, () => runFixture(fixture));
  }
});

/**
 * Run Oxlint on a test fixture.
 * @param fixture - Fixture object
 */
async function runFixture(fixture: Fixture): Promise<void> {
  // Run Oxlint without `--fix` option
  await testFixtureWithCommand({
    command: NODE_BIN_PATH,
    args: [CLI_PATH, "files"],
    fixture,
    snapshotName: "output",
    isESLint: false,
  });

  // Run Oxlint with `--fix` option
  if (fixture.options.fix) {
    await testFixtureWithCommand({
      command: NODE_BIN_PATH,
      args: [CLI_PATH, "--fix", "files"],
      fixture,
      snapshotName: "fix",
      isESLint: false,
    });
  }
}
