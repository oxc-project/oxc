import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

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
      let prefixedName = line
        .replaceAll(',', '')
        .replaceAll('::', '/')
        .replaceAll('_', '-');

      // Ignore no reference rules
      if (prefixedName.startsWith('oxc/')) continue;
      if (prefixedName.startsWith('node/')) {
        prefixedName = prefixedName.replace(/^node/, 'n');
      }

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
  'eslint/no-undef-init', // #6456 unicorn/no-useless-undefined covers this case
  'import/no-unresolved', // Will always contain false positives due to module resolution complexity,
  'react/jsx-equals-spacing', // stylistic rule
  'react/jsx-curly-spacing', // stylistic rule
  'react/jsx-indent', // stylistic rule
  'react/jsx-indent-props', // stylistic rule
  'react/jsx-props-no-multi-spaces', // stylistic rule
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
export const createRuleEntries = (loadedAllRules) => {
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
      isDeprecated: !!isDeprecated,
      isRecommended,
      // Will be updated later
      isImplemented: false,
      isNotSupported: false,
    });
  }

  return rulesEntry;
};

/** @param {RuleEntries} ruleEntries */
export const updateImplementedStatus = async (ruleEntries) => {
  const implementedRuleNames = await readAllImplementedRuleNames();

  for (const name of implementedRuleNames) {
    const rule = ruleEntries.get(name);
    if (rule) rule.isImplemented = true;
    else console.log(`👀 ${name} is implemented but not found in their rules`);
  }
};

/** @param {RuleEntries} ruleEntries */
export const updateNotSupportedStatus = (ruleEntries) => {
  for (const name of NOT_SUPPORTED_RULE_NAMES) {
    const rule = ruleEntries.get(name);
    if (rule) rule.isNotSupported = true;
  }
};

/**
 * @param {string} constName
 * @param {string} fileContent
 */
const getArrayEntries = (constName, fileContent) => {
  // Find the start of the list
  // ```
  // const VITEST_COMPATIBLE_JEST_RULES: [&str; 34] = [
  //   "consistent-test-it",
  //   "expect-expect",
  //   ...
  // ];
  // ```
  const regSearch = new RegExp(`const ${constName}[^=]+= \\[([^\\]]+)`, 's');

  const vitestCompatibleRules = fileContent.match(regSearch)?.[1];
  if (!vitestCompatibleRules) {
    throw new Error('Failed to find the list of vitest-compatible rules');
  }

  return new Set(
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
};

/**
 * Some typescript-eslint rules are re-implemented version of eslint rules.
 * e.g. no-array-constructor, max-params, etc...
 * Since oxlint supports these rules under eslint/* and it also supports TS,
 * we should override these to make implementation status up-to-date.
 *
 * @param {RuleEntries} ruleEntries
 */
export const overrideTypeScriptPluginStatusWithEslintPluginStatus = async (
  ruleEntries,
) => {
  const typescriptCompatibleRulesFile = await readFile(
    'crates/oxc_linter/src/utils/mod.rs',
    'utf8',
  );
  const rules = getArrayEntries('TYPESCRIPT_COMPATIBLE_ESLINT_RULES', typescriptCompatibleRulesFile);

  for (const rule of rules) {
    const typescriptRuleEntry = ruleEntries.get(`typescript/${rule}`);
    const eslintRuleEntry = ruleEntries.get(`eslint/${rule}`);
    if (typescriptRuleEntry && eslintRuleEntry) {
      ruleEntries.set(`typescript/${rule}`, {
        ...typescriptRuleEntry,
        isImplemented: eslintRuleEntry.isImplemented,
      });
    }
  }
};

/**
 * Some Jest rules are written to be compatible with Vitest, so we should
 * override the status of the Vitest rules to match the Jest rules.
 * @param {RuleEntries} ruleEntries
 */
export const syncVitestPluginStatusWithJestPluginStatus = async (ruleEntries) => {
  const vitestCompatibleRulesFile = await readFile(
    'crates/oxc_linter/src/utils/mod.rs',
    'utf8',
  );
  const rules = getArrayEntries('VITEST_COMPATIBLE_JEST_RULES', vitestCompatibleRulesFile);

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

/**
 * Some Unicorn rules rules are re-implemented version of eslint rules.
 * We should override these to make implementation status up-to-date.
 * @param {RuleEntries} ruleEntries
 */
export const syncUnicornPluginStatusWithEslintPluginStatus = (ruleEntries) => {
  const rules = new Set(['no-negated-condition']);

  for (const rule of rules) {
    const unicornRuleEntry = ruleEntries.get(`unicorn/${rule}`);
    const eslintRuleEntry = ruleEntries.get(`eslint/${rule}`);
    if (unicornRuleEntry && eslintRuleEntry) {
      ruleEntries.set(`unicorn/${rule}`, {
        ...unicornRuleEntry,
        isImplemented: eslintRuleEntry.isImplemented,
      });
    }
  }
};
