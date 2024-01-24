const { resolve } = require("node:path");
const { readFile } = require("node:fs/promises");

/**
 * @typedef {Map<string, {
 *   docsUrl: string,
 *   isDeprecated: boolean,
 *   isRecommended: boolean,
 *   isImplemented: boolean,
 *   isNotSupported: boolean,
 * }>} RuleEntries
 */

/** @param {ReturnType<import("eslint").Linter["getRules"]>} loadedAllRules */
exports.createRuleEntries = (loadedAllRules) => {
  /** @type {RuleEntries} */
  const rulesEntry = new Map();

  for (const [name, rule] of loadedAllRules) {
    // Default eslint rules are not prefixed
    const prefixedName = name.includes("/") ? name : `eslint/${name}`;

    if (rulesEntry.has(prefixedName))
      throw new Error(`Duplicate rule: ${name}`);

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

exports.readAllImplementedRuleNames = async () => {
  const rulesFile = await readFile(
    resolve("crates/oxc_linter/src/rules.rs"),
    "utf8",
  );

  /** @type {Set<string>} */
  const rules = new Set();

  let found = false;
  for (const line of rulesFile.split("\n")) {
    if (line.startsWith("oxc_macros::declare_all_lint_rules!")) {
      found = true;
      continue;
    }
    if (found && line.startsWith("}")) {
      return rules;
    }

    if (found) {
      const prefixedName = line
        .trim()
        .replaceAll(",", "")
        .replaceAll("::", "/")
        .replaceAll("_", "-");

      if (prefixedName.startsWith("oxc/")) continue;

      rules.add(prefixedName);
    }
  }

  throw new Error("Failed to find the end of the rules list");
};

const NOT_SUPPORTED_RULE_NAMES = new Set([]);
/** @param {RuleEntries} ruleEntries */
exports.updateNotSupportedStatus = (ruleEntries) => {
  for (const name of NOT_SUPPORTED_RULE_NAMES) {
    const rule = ruleEntries.get(name);
    if (rule) rule.isNotSupported = true;
  }
};

/**
 * @param {RuleEntries} ruleEntries
 * @param {Set<string>} implementedRuleNames
 */
exports.updateImplementedStatus = (ruleEntries, implementedRuleNames) => {
  for (const name of implementedRuleNames) {
    const rule = ruleEntries.get(name);
    if (rule) rule.isImplemented = true;
  }
};
