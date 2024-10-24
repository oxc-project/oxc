const { resolve } = require('node:path');
const { readFile } = require('node:fs/promises');

const readAllImplementedRuleNames = async () => {
  const rulesFile = await readFile(
    resolve('crates/oxc_linter/src/rules.rs'),
    'utf8',
  );

  /** @type {Set<string>} */
  const rules = new Set();

  let found = false;
  for (let line of rulesFile.split('\n')) {
    line = line.trim();

    // Skip commented out rules
    if (line.startsWith('//')) continue;

    if (line === 'oxc_macros::declare_all_lint_rules! {') {
      found = true;
      continue;
    }
    if (found && line === '}') {
      return rules;
    }

    if (found) {
      const prefixedName = line
        .replaceAll(',', '')
        .replaceAll('::', '/')
        .replaceAll('_', '-');

      // Ignore no reference rules
      if (prefixedName.startsWith('oxc/')) continue;

      rules.add(prefixedName);
    }
  }

  throw new Error('Failed to find the end of the rules list');
};

const NOT_SUPPORTED_RULE_NAMES = new Set([
  'eslint/no-dupe-args', // superseded by strict mode
  'eslint/no-octal', // superseded by strict mode
  'eslint/no-with', // superseded by strict mode
  'eslint/no-new-symbol', // Deprecated as of ESLint v9, but for a while disable manually
  'import/no-unresolved', // Will always contain false positives due to module resolution complexity
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
    const prefixedName = name.includes('/') ? name : `eslint/${name}`;

    const docsUrl = rule.meta?.docs?.url ?? '';
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

/**
 * Some typescript-eslint rules are re-implemented version of eslint rules.
 * e.g. no-array-constructor, max-params, etc...
 * Since oxlint supports these rules under eslint/* and it also supports TS,
 * we should override these to make implementation status up-to-date.
 *
 * @param {RuleEntries} ruleEntries
 */
exports.overrideTypeScriptPluginStatusWithEslintPluginStatus = (
  ruleEntries,
) => {
  for (const [name, rule] of ruleEntries) {
    if (!name.startsWith('typescript/')) continue;

    // This assumes that if the same name found, it implements the same rule.
    const eslintRule = ruleEntries.get(name.replace('typescript/', 'eslint/'));
    if (!eslintRule) continue;

    rule.isImplemented = eslintRule.isImplemented;
    rule.isNotSupported = eslintRule.isNotSupported;
  }
};

/**
 * Some Jest rules are written to be compatible with Vitest, so we should
 * override the status of the Vitest rules to match the Jest rules.
 * @param {RuleEntries} ruleEntries
 */
exports.syncVitestPluginStatusWithJestPluginStatus = async (ruleEntries) => {
  const vitestCompatibleRulesFile = await readFile(
    'crates/oxc_linter/src/utils/mod.rs',
    'utf8',
  );

  // Find the start of the list of vitest-compatible rules
  // ```
  // const VITEST_COMPATIBLE_JEST_RULES: phf::Set<&'static str> = phf::phf_set! {
  //   "consistent-test-it",
  //   "expect-expect",
  //   ...
  // };
  // ```
  const vitestCompatibleRules = vitestCompatibleRulesFile.match(
    /const VITEST_COMPATIBLE_JEST_RULES.+phf_set! {([^}]+)/s,
  )?.[1];
  if (!vitestCompatibleRules) {
    throw new Error('Failed to find the list of vitest-compatible rules');
  }

  const rules = new Set(
    vitestCompatibleRules
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith('//'))
      .map((line) =>
        line
          .replace(/"/g, '')
          .split(',')
          .filter((s) => s !== '')
      )
      .flat(),
  );

  for (const rule of rules) {
    const vitestRuleEntry = ruleEntries.get(`vitest/${rule}`);
    const jestRuleEntry = ruleEntries.get(`jest/${rule}`);
    if (vitestRuleEntry && jestRuleEntry) {
      ruleEntries.set(`vitest/${rule}`, {
        ...vitestRuleEntry,
        isImplemented: jestRuleEntry.isImplemented,
      });
    }
  }
};
