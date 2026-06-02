import { join } from "node:path";
import { describe, it } from "vitest";
import { getFixtures, runFixture } from "./utils";

// oxlint-disable valid-title
describe("oxfmt CLI", () => {
  const fixtures = getFixtures();

  for (const fixture of fixtures) {
    // Concurrent across fixtures, sequential within (file writes may conflict)
    describe.concurrent(fixture.name, () => {
      for (let i = 0; i < fixture.testCases.length; i++) {
        const testCase = fixture.testCases[i];

        it.sequential(`${fixture.name}/${i}`, async ({ expect }) => {
          const snapshotPath = join(fixture.dirPath, `${i}.snap.md`);
          const snapshot = await runFixture(fixture, testCase);
          await expect(snapshot).toMatchFileSnapshot(snapshotPath);
        });
      }
    });
  }
});
