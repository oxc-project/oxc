import assert from "node:assert";
import repos from "../../repos.json" with { type: "json" };

import type { MockFn, TestGroup } from "../index.ts";

const group: TestGroup = {
  name: "jsx-a11y",
  ...repos.jsx_a11y,
  submoduleName: "jsx_a11y",
  testFilesDirPath: ".oxlint-conformance/__tests__/src/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith("-test.js")) return null;
    return filename.slice(0, -"-test.js".length);
  },

  prepare(_require: NodeJS.Require, mock: MockFn) {
    // Check the one ancillary Tape assertion synchronously, outside RuleTester counts.
    mock(
      "tape",
      (name: string, fn: (test: { equal: typeof assert.equal; end: () => void }) => void) => {
        assert.equal(name, "validityCheck");
        fn({ equal: assert.equal, end() {} });
      },
    );
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],
  parsers: [],
};

export default group;
