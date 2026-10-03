import repos from "../../repos.json" with { type: "json" };

import type { TestGroup } from "../index.ts";

const group: TestGroup = {
  name: "tsdoc",
  ...repos.tsdoc,
  submoduleName: "tsdoc",
  testFilesDirPath: "eslint-plugin/src/tests",

  transformTestFilename(filename: string) {
    return filename === "plugin.test.ts" ? "syntax" : null;
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],
  parsers: [],
};

export default group;
