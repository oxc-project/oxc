import repos from "../../repos.json" with { type: "json" };

import type { TestGroup } from "../index.ts";

const group: TestGroup = {
  name: "react",
  ...repos.react_plugin,
  submoduleName: "react_plugin",
  testFilesDirPath: "tests/lib/rules",

  transformTestFilename(filename: string) {
    // These suites run core ESLint rules with the React rule enabled by an
    // inline directive. RuleTester does not activate additional rules from
    // directives, so apparent passes would not verify the React rule.
    if (filename === "jsx-uses-react.js" || filename === "jsx-uses-vars.js") return null;
    return filename.endsWith(".js") ? filename.slice(0, -3) : null;
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],
  parsers: [
    { specifier: "babel-eslint", lang: "jsx" },
    { specifier: "@babel/eslint-parser", lang: "jsx" },
    { specifier: "typescript-eslint-parser", lang: "tsx" },
    { specifier: "@typescript-eslint/parser", lang: "tsx" },
  ],
};

export default group;
