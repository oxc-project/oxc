import repos from "../../repos.json" with { type: "json" };

import type { TestGroup } from "../index.ts";

const group: TestGroup = {
  name: "mocha",
  ...repos.mocha,

  submoduleName: "mocha",
  testFilesDirPath: "source/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    return filename.slice(0, -".test.ts".length);
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],

  parsers: [],
};

export default group;
