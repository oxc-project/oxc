import { join as pathJoin } from "node:path";
import { describe, it } from "vitest";
import { PACKAGE_ROOT_PATH, getFixtures, testFixtureWithCommand } from "./utils.ts";

import type { ExpectStatic } from "vitest";
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
 * - `files` as the first argument (so only lints the files in the fixture's `files` directory).
 * - Additional arguments from `options.json` file in fixture directory, if it exists.
 *
 * Fixtures with an `options.json` file containing `"fix": true` are also run with `--fix` CLI option.
 * Fixtures with an `options.json` file containing `"fixSuggestions": true` are also run with `--fix-suggestions` CLI option.
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
    it(
      `fixture: ${fixture.name}`,
      {
        concurrent: process.platform !== "win32",
        // Windows can be flaky due to memory allocation failures
        // Ref: https://github.com/oxc-project/oxc/issues/19395
        retry: process.platform === "win32" ? 2 : 0,
      },
      ({ expect }) => runFixture(fixture, expect),
    );
  }
});

/**
 * Run Oxlint on a test fixture.
 * @param fixture - Fixture object
 * @param expect - Vitest expect function from test context
 */
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
    snapshotName: "output",
    isESLint: false,
    expect,
  };

  // Run Oxlint without `--fix` option
  await testFixtureWithCommand(testOptions);

  // Run Oxlint with `--fix` option
  if (options.fix) {
    const fixArgs = [args[0], "--fix", ...args.slice(1)];
    await testFixtureWithCommand({ ...testOptions, args: fixArgs, snapshotName: "fix" });
  }

  // Run Oxlint with `--fix-suggestions` option
  if (options.fixSuggestions) {
    const fixArgs = [args[0], "--fix-suggestions", ...args.slice(1)];
    await testFixtureWithCommand({
      ...testOptions,
      args: fixArgs,
      snapshotName: "fix-suggestions",
    });
  }
}
