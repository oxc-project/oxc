const { resolve } = require("node:path");
const { readFile } = require("node:fs/promises");

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
      if (prefixedName.startsWith("deepscan/")) continue;

      rules.add(prefixedName);
    }
  }

  throw new Error("Failed to find the end of the rules list");
};

const NOT_SUPPORTED_RULE_NAMES = new Set([
  "eslint/no-dupe-args", // superseded by strict mode
  "eslint/no-octal", // superseded by strict mode
  "eslint/no-with" // superseded by strict mode
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
