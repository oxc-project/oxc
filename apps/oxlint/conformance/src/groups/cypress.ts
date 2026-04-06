import type { TestGroup } from "../index.ts";

const group: TestGroup = {
  name: "cypress",

  submoduleName: "cypress",
  testFilesDirPath: "tests/lib/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".js")) return null;
    return filename.slice(0, -".js".length);
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],

  parsers: [],
};

export default group;
