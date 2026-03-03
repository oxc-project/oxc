import { readdirSync, mkdirSync, writeFileSync } from "node:fs";
import { join, basename } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

const oxlintDirPath = join(import.meta.dirname, "..");
const eslintRulesDir = join(require.resolve("eslint/package.json"), "../lib/rules");
const generatedDirPath = join(oxlintDirPath, "src-js/generated/plugin-eslint");
const generatedRulesDirPath = join(generatedDirPath, "rules");

// Get all ESLint rule names (exclude index.js which is the registry, not a rule)
const ruleNames = readdirSync(eslintRulesDir)
  .filter((f) => f.endsWith(".js") && f !== "index.js")
  .map((f) => basename(f, ".js"))
  .sort();

// oxlint-disable-next-line no-console
console.log(`Found ${ruleNames.length} ESLint rules`);

// Create generated directories
mkdirSync(generatedRulesDirPath, { recursive: true });

// Generate a CJS wrapper file for each rule
for (const ruleName of ruleNames) {
  const content = `module.exports = require("../../../../node_modules/eslint/lib/rules/${ruleName}.js");`;
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

writeFileSync(join(generatedDirPath, "index.ts"), indexLines.join("\n"));

// Generate the rule_names.ts file for use in tsdown config
const ruleNamesLines = [
  `export default [`,
  ...ruleNames.map((name) => `  ${JSON.stringify(name)},`),
  `] as const;`,
  ``,
];

writeFileSync(join(generatedDirPath, "rule_names.ts"), ruleNamesLines.join("\n"));

// oxlint-disable-next-line no-console
console.log("Generated plugin-eslint files.");
