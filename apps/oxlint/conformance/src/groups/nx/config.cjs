const path = require("node:path");
const rootDir =
  process.env.NX_CONFORMANCE_ROOT || path.resolve(__dirname, "../../../submodules/nx");
module.exports = {
  rootDir,
  roots: ["<rootDir>/packages/eslint-plugin/src/rules"],
  testEnvironment: "node",
  testMatch: ["<rootDir>/packages/eslint-plugin/src/rules/enforce-module-boundaries.spec.ts"],
  transform: { "^.+\\.[tj]s$": path.join(__dirname, "transform.cjs") },
  setupFilesAfterEnv: [path.join(__dirname, "setup.cjs")],
  modulePaths: ["<rootDir>/node_modules"],
  // This testing helper is not included in the published devkit package.
  moduleNameMapper: {
    "^@nx/devkit/internal-testing-utils/mock-fs$":
      "<rootDir>/packages/nx/src/internal-testing-utils/mock-fs.ts",
  },
  maxWorkers: 1,
  cache: false,
};
