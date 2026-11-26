// Call ESLint from an untransformed file: its error-location estimator reads the
// caller's source using stack line numbers, which Vitest's transformation changes.
const { RuleTester } = require("eslint");

module.exports = function run(ruleName, rule, tests) {
  RuleTester.describe = (_name, fn) => fn();
  RuleTester.it = (_name, fn) => fn();
  new RuleTester().run(ruleName, rule, tests);
};
