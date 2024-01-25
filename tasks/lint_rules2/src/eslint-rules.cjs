const { Linter } = require("eslint");

// NOTICE!
// Plugins do not provide their type definitions, and also `@types/*` do not exist!
// Even worse, every plugin has slightly different types in detail...
//
// So, we need to normalize recommended and depricated properties manually.

// https://github.com/sindresorhus/eslint-plugin-unicorn/blob/main/index.js
const {
  rules: pluginUnicornAllRules,
  configs: pluginUnicornConfigs,
} = require("eslint-plugin-unicorn");
// https://github.com/gajus/eslint-plugin-jsdoc/blob/main/src/index.js
const {
  // @ts-expect-error: Module has no exported member
  rules: pluginJSDocAllRules,
  // @ts-expect-error: Module has no exported member
  configs: pluginJSDocConfigs,
} = require("eslint-plugin-jsdoc");
// https://github.com/import-js/eslint-plugin-import/blob/main/src/index.js
const {
  rules: pluginImportAllRules,
  configs: pluginImportConfigs,
} = require("eslint-plugin-import");
// https://github.com/jest-community/eslint-plugin-jest/blob/main/src/index.ts
const { rules: pluginJestAllRules } = require("eslint-plugin-jest");

// All rules including depricated, recommended, etc.
exports.createESLintLinter = () => new Linter();

/** @param {import("eslint").Linter} linter */
exports.loadPluginUnicornRules = (linter) => {
  const pluginUnicornRecommendedRules = new Map(
    Object.entries(pluginUnicornConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginUnicornAllRules)) {
    const prefixedName = `unicorn/${name}`;

    // Some of the rules do not have its property, so we need to mark it manually
    const recommendedValue = pluginUnicornRecommendedRules.get(prefixedName);
    // @ts-expect-error: `rule.meta.docs` is possibly `undefined`
    rule.meta.docs.recommended = recommendedValue && recommendedValue !== "off";

    linter.defineRule(prefixedName, rule);
  }
};

/** @param {import("eslint").Linter} linter */
exports.loadPluginJSDocRules = (linter) => {
  const pluginJSDocRecommendedRules = new Map(
    Object.entries(pluginJSDocConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginJSDocAllRules)) {
    const prefixedName = `jsdoc/${name}`;

    // Some of the rules do not have its property, so we need to mark it manually
    rule.meta.docs.recommended =
      pluginJSDocRecommendedRules.get(prefixedName) !== "off";

    linter.defineRule(prefixedName, rule);
  }
};

/** @param {import("eslint").Linter} linter */
exports.loadPluginImportRules = (linter) => {
  const pluginImportRecommendedRules = new Set(
    // @ts-expect-error: Property 'rules' does not exist on type 'Object'.
    Object.keys(pluginImportConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginImportAllRules)) {
    const prefixedName = `import/${name}`;

    // Some of the rules do not have its property, so we need to mark it manually
    // @ts-expect-error: Property 'recommended' does not exist on type
    rule.meta.docs.recommended = pluginImportRecommendedRules.has(prefixedName);

    // @ts-expect-error: The types of 'meta.type', 'string' is not assignable to type '"problem" | "suggestion" | "layout" | undefined'.
    linter.defineRule(prefixedName, rule);
  }
};

/** @param {import("eslint").Linter} linter */
exports.loadPluginJestRules = (linter) => {
  for (const [name, rule] of Object.entries(pluginJestAllRules)) {
    const prefixedName = `jest/${name}`;

    // This is `string | false`
    rule.meta.docs.recommended = typeof rule.meta.docs.recommended === "string";

    linter.defineRule(prefixedName, rule);
  }
};
