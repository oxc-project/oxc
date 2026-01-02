import { readFile, readdir } from "node:fs/promises";
import { resolve, join } from "node:path";
import unsupportedRules from "./unsupported-rules.json" with { type: "json" };
import { typescriptTypeCheckRules } from "./eslint-rules.mjs";

const readAllImplementedRuleNames = async () => {
  const rulesFile = await readFile(resolve("crates/oxc_linter/src/rules.rs"), "utf8");

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
      let prefixedName = line.replaceAll(",", "").replaceAll("::", "/").replaceAll("_", "-");

      // Ignore no reference rules
      if (prefixedName.startsWith("oxc/")) continue;
      if (prefixedName.startsWith("node/")) {
        prefixedName = prefixedName.replace(/^node/, "n");
      }

      rules.add(prefixedName);
    }
  }

  throw new Error("Failed to find the end of the rules list");
};

/**
 * Read all rule files and find rules with pending fixes.
 * A rule has a pending fix if it's declared with the `pending` keyword in its
 * declare_oxc_lint! macro, like: declare_oxc_lint!(RuleName, plugin, category, pending)
 */
const readAllPendingFixRuleNames = async () => {
  /** @type {Set<string>} */
  const pendingFixRules = new Set();

  const rulesDir = resolve("crates/oxc_linter/src/rules");

  /**
   * Recursively read all .rs files in a directory
   * @param {string} dir
   * @returns {Promise<string[]>}
   */
  const readRustFiles = async (dir) => {
    const entries = await readdir(dir, { withFileTypes: true });
    const files = await Promise.all(
      entries.map((entry) => {
        const fullPath = join(dir, entry.name);
        if (entry.isDirectory()) {
          return readRustFiles(fullPath);
        } else if (entry.name.endsWith(".rs") && entry.name !== "mod.rs") {
          return [fullPath];
        }
        return [];
      }),
    );
    return files.flat();
  };

  const ruleFiles = await readRustFiles(rulesDir);

  for (const filePath of ruleFiles) {
    // oxlint-disable-next-line no-await-in-loop
    const content = await readFile(filePath, "utf8");

    // Look for declare_oxc_lint! macro with pending fix
    // Pattern matches: declare_oxc_lint!( ... , pending, ... )
    // or: declare_oxc_lint!( ... , pending ) at the end
    const declareMacroMatch = content.match(
      /declare_oxc_lint!\s*\(\s*(?:\/\/\/[^\n]*\n\s*)*(\w+)\s*(?:\(tsgolint\))?\s*,\s*(\w+)\s*,\s*(\w+)\s*,\s*([^)]+)\)/s,
    );

    if (declareMacroMatch) {
      const [, ruleName, plugin, , restParams] = declareMacroMatch;

      // Check if 'pending' appears in the remaining parameters
      // It could be standalone or part of fix capabilities like "pending" or "fix = pending"
      if (/\bpending\b/.test(restParams)) {
        // Convert Rust struct name to kebab-case rule name
        const kebabRuleName = ruleName
          .replace(/([a-z])([A-Z])/g, "$1-$2")
          .replace(/([A-Z]+)([A-Z][a-z])/g, "$1-$2")
          .toLowerCase();

        let prefixedName = `${plugin}/${kebabRuleName}`;

        // Handle node -> n rename
        if (prefixedName.startsWith("node/")) {
          prefixedName = prefixedName.replace(/^node/, "n");
        }

        pendingFixRules.add(prefixedName);
      }
    }
  }

  return pendingFixRules;
};

/**
 * These rules will not be supported/implemented in oxlint.
 *
 * oxlint does not intend to cover stylistic lint rules, as oxfmt will handle code formatting.
 * There are some other rules listed here which are difficult to support due to technical limitations,
 * or rules that are deprecated in their source plugins and no longer relevant.
 *
 * @type {Set<string>}
 **/
const NOT_SUPPORTED_RULE_NAMES = new Set(Object.keys(unsupportedRules.unsupportedRules ?? {}));

/**
 * @typedef {{
 *   docsUrl: string,
 *   isDeprecated: boolean,
 *   isRecommended: boolean,
 *   isImplemented: boolean,
 *   isNotSupported: boolean,
 *   isPendingFix: boolean,
 *   unsupportedRationale: string | null
 * }} RuleEntry
 * @typedef {Map<string, RuleEntry>} RuleEntries
 */

/** @param {ReturnType<import("eslint").Linter["getRules"]>} loadedAllRules */
export const createRuleEntries = (loadedAllRules) => {
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
      isDeprecated: !!isDeprecated,
      isRecommended,
      // Will be updated later
      isImplemented: false,
      isNotSupported: false,
      isPendingFix: false,
      unsupportedRationale: null,
    });
  }

  return rulesEntry;
};

/** @param {RuleEntries} ruleEntries */
export const updateImplementedStatus = async (ruleEntries) => {
  const implementedRuleNames = await readAllImplementedRuleNames();

  for (const name of implementedRuleNames) {
    const rule = ruleEntries.get(name);
    if (rule) {
      rule.isImplemented = true;
    } else {
      // Don't print the warning for rules that are type-check only and thus implemented via tsgolint.
      if (name.startsWith("typescript/")) {
        const tsRuleName = name.split("/").at(1);
        if (tsRuleName && typescriptTypeCheckRules.has(`@typescript-eslint/${tsRuleName}`)) {
          continue;
        }
      }

      // oxlint-disable-next-line no-console
      console.log(`👀 ${name} is implemented but not found in their rules`);
    }
  }
};

/** @param {RuleEntries} ruleEntries */
export const updateNotSupportedStatus = (ruleEntries) => {
  for (const name of NOT_SUPPORTED_RULE_NAMES) {
    const rule = ruleEntries.get(name);
    if (rule) {
      rule.isNotSupported = true;
      rule.unsupportedRationale = unsupportedRules.unsupportedRules?.[name] ?? null;
    }
  }
};

/** @param {RuleEntries} ruleEntries */
export const updatePendingFixStatus = async (ruleEntries) => {
  const pendingFixRuleNames = await readAllPendingFixRuleNames();

  for (const name of pendingFixRuleNames) {
    const rule = ruleEntries.get(name);
    if (rule && rule.isImplemented) {
      rule.isPendingFix = true;
    }
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
  const regSearch = new RegExp(`const ${constName}[^=]+= \\[([^\\]]+)`, "s");

  const vitestCompatibleRules = fileContent.match(regSearch)?.[1];
  if (!vitestCompatibleRules) {
    throw new Error("Failed to find the list of vitest-compatible rules");
  }

  return new Set(
    vitestCompatibleRules
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith("//"))
      .flatMap((line) =>
        line
          .replace(/"/g, "")
          .split(",")
          .filter((s) => s !== ""),
      ),
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
export const overrideTypeScriptPluginStatusWithEslintPluginStatus = async (ruleEntries) => {
  const typescriptCompatibleRulesFile = await readFile(
    "crates/oxc_linter/src/utils/mod.rs",
    "utf8",
  );
  const rules = getArrayEntries(
    "TYPESCRIPT_COMPATIBLE_ESLINT_RULES",
    typescriptCompatibleRulesFile,
  );

  for (const rule of rules) {
    const typescriptRuleEntry = ruleEntries.get(`typescript/${rule}`);
    const eslintRuleEntry = ruleEntries.get(`eslint/${rule}`);
    if (typescriptRuleEntry && eslintRuleEntry) {
      ruleEntries.set(`typescript/${rule}`, {
        ...typescriptRuleEntry,
        isImplemented: eslintRuleEntry.isImplemented,
        isPendingFix: eslintRuleEntry.isPendingFix,
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
  const vitestCompatibleRulesFile = await readFile("crates/oxc_linter/src/utils/mod.rs", "utf8");
  const rules = getArrayEntries("VITEST_COMPATIBLE_JEST_RULES", vitestCompatibleRulesFile);

  for (const rule of rules) {
    const vitestRuleEntry = ruleEntries.get(`vitest/${rule}`);
    const jestRuleEntry = ruleEntries.get(`jest/${rule}`);
    if (vitestRuleEntry && jestRuleEntry) {
      ruleEntries.set(`vitest/${rule}`, {
        ...vitestRuleEntry,
        isImplemented: jestRuleEntry.isImplemented,
        isPendingFix: jestRuleEntry.isPendingFix,
      });
    }
  }

  // Special case: vitest/no-restricted-vi-methods is implemented by jest/no-restricted-jest-methods
  const vitestRestrictedViMethodsEntry = ruleEntries.get("vitest/no-restricted-vi-methods");
  const jestRestrictedJestMethodsEntry = ruleEntries.get("jest/no-restricted-jest-methods");
  if (vitestRestrictedViMethodsEntry && jestRestrictedJestMethodsEntry) {
    ruleEntries.set("vitest/no-restricted-vi-methods", {
      ...vitestRestrictedViMethodsEntry,
      isImplemented: jestRestrictedJestMethodsEntry.isImplemented,
      isPendingFix: jestRestrictedJestMethodsEntry.isPendingFix,
    });
  }
};

/**
 * Some Unicorn rules rules are re-implemented version of eslint rules.
 * We should override these to make implementation status up-to-date.
 * @param {RuleEntries} ruleEntries
 */
export const syncUnicornPluginStatusWithEslintPluginStatus = (ruleEntries) => {
  const rules = new Set(["no-negated-condition"]);

  for (const rule of rules) {
    const unicornRuleEntry = ruleEntries.get(`unicorn/${rule}`);
    const eslintRuleEntry = ruleEntries.get(`eslint/${rule}`);
    if (unicornRuleEntry && eslintRuleEntry) {
      ruleEntries.set(`unicorn/${rule}`, {
        ...unicornRuleEntry,
        isImplemented: eslintRuleEntry.isImplemented,
        isPendingFix: eslintRuleEntry.isPendingFix,
      });
    }
  }
};
