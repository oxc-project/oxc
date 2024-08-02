const { resolve } = require("node:path");
const { readFile } = require("node:fs/promises");
const { pluginTypeScriptRulesNames } = require("./eslint-rules.cjs");

/** 
 * Map OXC rules implementation
 * @type {Map<string, string[]>} 
 * */
const MAP_SPECIFIC_RULES = new Map([
  // no-new-native-nonconstructor has already the testcase for eslint/no-new-symbol
  ['eslint/no-new-native-nonconstructor', ['eslint/no-new-symbol']]
]);

const readAllImplementedRuleNames = async () => {
  const rulesFile = await readFile(
    resolve("crates/oxc_linter/src/rules.rs"),
    "utf8",
  );

  /** @type {Set<string>} */
  const rules = new Set();

  let found = false;
  for (let line of rulesFile.split("\n")) {
    line = line.trim();

    // Skip commented out rules
    if (line.startsWith("//")) continue;

    if (line === "oxc_macros::declare_all_lint_rules! {") {
      found = true;
      continue;
    }
    if (found && line === "}") {
      return rules;
    }

    if (found) {
      const prefixedName = line
        .replaceAll(",", "")
        .replaceAll("::", "/")
        .replaceAll("_", "-");

      // Ignore no reference rules
      if (prefixedName.startsWith("oxc/")) continue;

      rules.add(prefixedName);

      // some tyescript rules are extensions of eslint core rules
      if (prefixedName.startsWith("eslint/")) {
        const ruleName = prefixedName.replace('eslint/', '');

        // there is an alias, so we add it with this in mind.
        if (pluginTypeScriptRulesNames.includes(ruleName)) {
          rules.add(`typescript/${ruleName}`);
        }
      }
    }

    for (const [name, alias] of MAP_SPECIFIC_RULES) {
      if (rules.has(name)) {
        alias.forEach(aliasName => rules.add(aliasName));
      }
    }
  }

  throw new Error("Failed to find the end of the rules list");
};

const NOT_SUPPORTED_RULE_NAMES = new Set([
  "eslint/no-dupe-args", // superseded by strict mode
  "eslint/no-octal", // superseded by strict mode
  "eslint/no-with", // superseded by strict mode
  "import/no-unresolved" // Will always contain false positives due to module resolution complexity
]);

/**
 * @typedef {{
 *   docsUrl: string,
 *   isDeprecated: boolean,
 *   isRecommended: boolean,
 *   isImplemented: boolean,
 *   isNotSupported: boolean,
 * }} RuleEntry
 * @typedef {Map<string, RuleEntry>} RuleEntries
 */

/** @param {ReturnType<import("eslint").Linter["getRules"]>} loadedAllRules */
exports.createRuleEntries = (loadedAllRules) => {
  /** @type {RuleEntries} */
  const rulesEntry = new Map();

  for (const [name, rule] of loadedAllRules) {
    // Default eslint rules are not prefixed
    const prefixedName = name.includes("/") ? name : `eslint/${name}`;

    const docsUrl = rule.meta?.docs?.url ?? "";
    const isDeprecated = rule.meta?.deprecated ?? false;
    const isRecommended = rule.meta?.docs?.recommended ?? false;

    rulesEntry.set(prefixedName, {
      docsUrl,
      isDeprecated,
      isRecommended,
      // Will be updated later
      isImplemented: false,
      isNotSupported: false,
    });
  }

  return rulesEntry;
};

/** @param {RuleEntries} ruleEntries */
exports.updateImplementedStatus = async (ruleEntries) => {
  const implementedRuleNames = await readAllImplementedRuleNames();

  for (const name of implementedRuleNames) {
    const rule = ruleEntries.get(name);
    if (rule) rule.isImplemented = true;
    else console.log(`👀 ${name} is implemented but not found in their rules`);
  }
};

/** @param {RuleEntries} ruleEntries */
exports.updateNotSupportedStatus = (ruleEntries) => {
  for (const name of NOT_SUPPORTED_RULE_NAMES) {
    const rule = ruleEntries.get(name);
    if (rule) rule.isNotSupported = true;
  }
};
