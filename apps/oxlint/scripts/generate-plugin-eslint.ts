// oxlint-disable no-console

import { readdirSync, mkdirSync, writeFileSync } from "node:fs";
import { join, basename } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

const oxlintDirPath = join(import.meta.dirname, "..");
const eslintRulesDir = join(
  require.resolve("eslint/package.json"),
  "..",
  "lib",
  "rules",
);
const generatedDirPath = join(
  oxlintDirPath,
  "src-js",
  "generated",
  "plugin-eslint",
);
const generatedRulesDirPath = join(generatedDirPath, "rules");

// Get all ESLint rule names (exclude index.js which is the registry, not a rule)
const ruleNames = readdirSync(eslintRulesDir)
  .filter((f) => f.endsWith(".js") && f !== "index.js")
  .map((f) => basename(f, ".js"))
  .sort();

console.log(`Found ${ruleNames.length} ESLint rules`);

// Create generated directories
mkdirSync(generatedRulesDirPath, { recursive: true });

// Generate a CJS wrapper file for each rule.
// Uses createRequire with eslint's package.json path to bypass ESLint 9's exports map restrictions
// (which block access to `eslint/lib/rules/*` from outside the package).
// createRequire(path) creates a require that resolves relative paths from `path`'s directory,
// so `./lib/rules/...` resolves to `<eslint package root>/lib/rules/...`.
for (const ruleName of ruleNames) {
  const content = [
    `const { createRequire } = require("node:module");`,
    `// createRequire resolves relative paths from eslint's package root, bypassing its exports map.`,
    `const _require = createRequire(require.resolve("eslint/package.json"));`,
    `module.exports = _require("./lib/rules/${ruleName}.js");`,
    ``,
  ].join("\n");
  writeFileSync(join(generatedRulesDirPath, `${ruleName}.cjs`), content);
}

// Generate the plugin rules index (ESM with lazy getters)
const indexLines = [
  `import { createRequire } from "node:module";`,
  `const require = createRequire(import.meta.url);`,
  ``,
  `export default {`,
];
for (const ruleName of ruleNames) {
  indexLines.push(
    `  get ${JSON.stringify(ruleName)}() { return require("./rules/${ruleName}.cjs"); },`,
  );
}
indexLines.push(`};`, ``);

writeFileSync(join(generatedDirPath, "index.js"), indexLines.join("\n"));

// Generate the rule_names.ts file for use in tsdown config
const ruleNamesLines = [
  `export default [`,
  ...ruleNames.map((name) => `  ${JSON.stringify(name)},`),
  `] as const;`,
  ``,
];

writeFileSync(
  join(generatedDirPath, "rule_names.ts"),
  ruleNamesLines.join("\n"),
);

console.log("Generated plugin-eslint files.");
