import { join as pathJoin } from "node:path";
import { describe, it } from "vitest";
import { PACKAGE_ROOT_PATH, getFixtures, testFixtureWithCommand } from "./utils.ts";

import type { Fixture } from "./utils.ts";

const ESLINT_PATH = pathJoin(PACKAGE_ROOT_PATH, "node_modules/.bin/eslint");

/**
 * Run ESLint tests for all fixtures in `test/fixtures` which contain an `options.json` file
 * containing `"eslint": true`.
 *
 * ESLint is run with CWD set to the fixture directory.
 */
// These tests take longer than 5 seconds on CI, so increase timeout to 20 seconds
// oxlint-disable-next-line jest/valid-describe-callback
describe("ESLint compatibility", { timeout: 20_000 }, () => {
  const fixtures = getFixtures();
  for (const fixture of fixtures) {
    if (!fixture.options.eslint) continue;

    // oxlint-disable-next-line jest/expect-expect
    it(`fixture: ${fixture.name}`, () => runFixture(fixture));
  }
});

/**
 * Run ESLint on a test fixture.
 * @param fixture - Fixture object
 */
async function runFixture(fixture: Fixture): Promise<void> {
  await testFixtureWithCommand({
    command: ESLINT_PATH,
    args: [],
    fixture,
    snapshotName: "eslint",
    isESLint: true,
  });
}
