const { Linter } = require("eslint");

// NOTICE!
// Plugins do not provide their type definitions, and also `@types/*` do not exist!
// Even worse, every plugin has slightly different types, different way of configuration in detail...
//
// So here, we need to list all rules while normalizing recommended and deprecated flags.
// - rule.meta.docs.recommended
// - rule.meta.deprecated

// https://github.com/typescript-eslint/typescript-eslint/blob/main/packages/eslint-plugin/src/index.ts
const {
  rules: pluginTypeScriptAllRules,
  configs: pluginTypeScriptConfigs,
} = require("@typescript-eslint/eslint-plugin");
// https://github.com/eslint-community/eslint-plugin-n/blob/master/lib/index.js
const { rules: pluginNAllRules } = require("eslint-plugin-n");
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

// All rules(including deprecated, recommended) are loaded initially.
exports.createESLintLinter = () => new Linter();

/** @param {import("eslint").Linter} linter */
exports.loadPluginTypeScriptRules = (linter) => {
  // We want to list all rules but not support type-checked rules
  const pluginTypeScriptDisableTypeCheckedRules = new Map(
    Object.entries(pluginTypeScriptConfigs["disable-type-checked"].rules),
  );
  for (const [name, rule] of Object.entries(pluginTypeScriptAllRules)) {
    if (
      pluginTypeScriptDisableTypeCheckedRules.has(`@typescript-eslint/${name}`)
    )
      continue;

    const prefixedName = `typescript/${name}`;

    linter.defineRule(prefixedName, rule);
  }
};

/** @param {import("eslint").Linter} linter */
exports.loadPluginNRules = (linter) => {
  for (const [name, rule] of Object.entries(pluginNAllRules)) {
    const prefixedName = `n/${name}`;

    // @ts-expect-error: The types of 'meta.fixable', 'null' is not assignable to type '"code" | "whitespace" | undefined'.
    linter.defineRule(prefixedName, rule);
  }
};

/** @param {import("eslint").Linter} linter */
exports.loadPluginUnicornRules = (linter) => {
  const pluginUnicornRecommendedRules = new Map(
    Object.entries(pluginUnicornConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginUnicornAllRules)) {
    const prefixedName = `unicorn/${name}`;

    // If name is presented and value is not "off", it is recommended
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

    // If name is presented and value is not "off", it is recommended
    const recommendedValue = pluginJSDocRecommendedRules.get(prefixedName);
    rule.meta.docs.recommended = recommendedValue && recommendedValue !== "off";

    linter.defineRule(prefixedName, rule);
  }
};

/** @param {import("eslint").Linter} linter */
exports.loadPluginImportRules = (linter) => {
  const pluginImportRecommendedRules = new Map(
    // @ts-expect-error: Property 'rules' does not exist on type 'Object'.
    Object.entries(pluginImportConfigs.recommended.rules),
  );
  for (const [name, rule] of Object.entries(pluginImportAllRules)) {
    const prefixedName = `import/${name}`;

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

    // Presented but type is `string | false`
    rule.meta.docs.recommended = typeof rule.meta.docs.recommended === "string";

    linter.defineRule(prefixedName, rule);
  }
};
