import { readFile, readdir } from "node:fs/promises";
import { resolve, join } from "node:path";

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
 */
const NOT_SUPPORTED_RULE_NAMES = new Set([
  "eslint/no-dupe-args", // superseded by strict mode
  "eslint/no-octal", // superseded by strict mode
  "eslint/no-new-symbol", // Deprecated as of ESLint v9, but for a while disable manually
  "eslint/no-undef-init", // #6456 unicorn/no-useless-undefined covers this case
  "import/no-unresolved", // Will always contain false positives due to module resolution complexity,
  "promise/no-native", // handled by eslint/no-undef
  "unicorn/no-for-loop", // this rule suggest using `Array.prototype.entries` which is slow https://github.com/oxc-project/oxc/issues/11311, furthermore, `typescript/prefer-for-of` covers most cases
  "eslint/no-negated-in-lhs", // replaced by eslint/no-unsafe-negation, which we support
  "eslint/no-catch-shadow", // replaced by eslint/no-shadow
  "eslint/id-blacklist", // replaced by eslint/id-denylist
  "eslint/no-new-object", // replaced by eslint/no-object-constructor, which we support
  "eslint/no-native-reassign", // replaced by eslint/no-global-assign, which we support
  "n/shebang", // replaced by node/hashbang
  "n/no-hide-core-modules", // this rule is deprecated in eslint-plugin-n for being inherently incorrect, no need for us to implement it
  "unicorn/no-array-push-push", // replaced by unicorn/prefer-single-call
  "import/imports-first", // replaced by import/first, which we support
  "react/jsx-sort-default-props", // replaced by react/sort-default-props
  "vitest/no-done-callback", // [deprecated in eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/issues/158)
  "eslint/no-return-await", // deprecated, not recommended anymore by ESLint
  "eslint/prefer-reflect", // deprecated, not recommended anymore by ESLint
  "jsx-a11y/accessible-emoji", // deprecated
  "jsx-a11y/label-has-for", // deprecated, replaced by jsx-a11y/label-has-associated-control
  "jsx-a11y/no-onchange", // deprecated, based on behavior of very old browsers, and so no longer necessary

  // ESLint rules that are deprecated in ESLint and replaced by rules in eslint-plugin-n:
  "eslint/no-process-env", // replaced by node/no-process-env, which we already support
  "eslint/no-new-require", // replaced by node/no-new-require, which we already support
  "eslint/no-buffer-constructor", // replaced by node/no-deprecated-api
  "eslint/no-path-concat", // replaced by node/no-path-concat
  "eslint/no-sync", // replaced by node/no-sync
  "eslint/no-process-exit", // replaced by node/no-process-exit
  "eslint/no-restricted-modules", // replaced by node/no-restricted-require
  "eslint/no-mixed-requires", // replaced by node/no-mixed-requires
  "eslint/global-require", // replaced by node/global-require
  "eslint/handle-callback-err", // replaced by node/handle-callback-err
  "eslint/callback-return", // replaced by node/callback-return

  // Stylistic rules from eslint-plugin-react:
  "react/jsx-equals-spacing",
  "react/jsx-curly-spacing",
  "react/jsx-indent",
  "react/jsx-indent-props",
  "react/jsx-newline",
  "react/jsx-wrap-multilines",
  "react/jsx-props-no-multi-spaces",
  "react/jsx-tag-spacing",
  "react/jsx-space-before-closing",

  // Deprecated typescript-eslint rules:
  "typescript/sort-type-constituents", // replaced by `perfectionist/sort-intersection-types` and `perfectionist/sort-union-types` rules.
  "typescript/no-type-alias", // replaced by `typescript-eslint/consistent-type-definitions` rule.
  "typescript/typedef", // just generally deprecated

  // The following ESLint rules are deprecated in the main package, and are all stylistic:
  "eslint/array-bracket-newline",
  "eslint/array-bracket-spacing",
  "eslint/array-element-newline",
  "eslint/arrow-parens",
  "eslint/arrow-spacing",
  "eslint/block-spacing",
  "eslint/brace-style",
  "eslint/comma-dangle",
  "eslint/comma-spacing",
  "eslint/comma-style",
  "eslint/computed-property-spacing",
  "eslint/dot-location",
  "eslint/eol-last",
  "eslint/func-call-spacing",
  "eslint/function-call-argument-newline",
  "eslint/function-paren-newline",
  "eslint/generator-star-spacing",
  "eslint/implicit-arrow-linebreak",
  "eslint/indent-legacy",
  "eslint/indent",
  "eslint/jsx-quotes",
  "eslint/key-spacing",
  "eslint/keyword-spacing",
  "eslint/line-comment-position",
  "eslint/linebreak-style",
  "eslint/lines-around-comment",
  "eslint/lines-around-directive",
  "eslint/lines-between-class-members",
  "eslint/max-len",
  "eslint/max-statements-per-line",
  "eslint/multiline-comment-style",
  "eslint/multiline-ternary",
  "eslint/new-parens",
  "eslint/newline-after-var",
  "eslint/newline-before-return",
  "eslint/newline-per-chained-call",
  "eslint/no-confusing-arrow",
  "eslint/no-extra-parens",
  "eslint/no-extra-semi",
  "eslint/no-floating-decimal",
  "eslint/no-mixed-operators",
  "eslint/no-mixed-spaces-and-tabs",
  "eslint/no-multi-spaces",
  "eslint/no-multiple-empty-lines",
  "eslint/no-spaced-func",
  "eslint/no-tabs",
  "eslint/no-trailing-spaces",
  "eslint/no-whitespace-before-property",
  "eslint/nonblock-statement-body-position",
  "eslint/object-curly-newline",
  "eslint/object-curly-spacing",
  "eslint/object-property-newline",
  "eslint/one-var-declaration-per-line",
  "eslint/operator-linebreak",
  "eslint/padded-blocks",
  "eslint/padding-line-between-statements",
  "eslint/quote-props",
  "eslint/quotes",
  "eslint/rest-spread-spacing",
  "eslint/semi-spacing",
  "eslint/semi-style",
  "eslint/semi",
  "eslint/space-before-blocks",
  "eslint/space-before-function-paren",
  "eslint/space-in-parens",
  "eslint/space-infix-ops",
  "eslint/space-unary-ops",
  "eslint/spaced-comment",
  "eslint/switch-colon-spacing",
  "eslint/template-curly-spacing",
  "eslint/template-tag-spacing",
  "eslint/wrap-iife",
  "eslint/wrap-regex",
  "eslint/yield-star-spacing",

  "unicorn/no-named-default", // implemented via import/no-named-default

  // not supported as it requires parsing the vue template
  "vue/no-lone-template",
  "vue/no-v-html",
  "vue/this-in-template",

  "vue/array-bracket-newline",
  "vue/array-bracket-spacing",
  "vue/array-element-newline",
  "vue/arrow-spacing",
  "vue/attribute-hyphenation",
  "vue/attributes-order",
  "vue/block-lang",
  "vue/block-order",
  "vue/block-spacing",
  "vue/block-tag-newline",
  "vue/brace-style",
  "vue/camelcase",
  "vue/comma-dangle",
  "vue/comma-spacing",
  "vue/comma-style",
  "vue/comment-directive",
  "vue/component-name-in-template-casing",
  "vue/custom-event-name-casing",
  "vue/define-macros-order",
  "vue/dot-location",
  "vue/dot-notation",
  "vue/enforce-style-attribute",
  "vue/eqeqeq",
  "vue/first-attribute-linebreak",
  "vue/func-call-spacing",
  "vue/html-button-has-type",
  "vue/html-closing-bracket-newline",
  "vue/html-closing-bracket-spacing",
  "vue/html-comment-content-newline",
  "vue/html-comment-content-spacing",
  "vue/html-comment-indent",
  "vue/html-end-tags",
  "vue/html-indent",
  "vue/html-quotes",
  "vue/html-self-closing",
  "vue/key-spacing",
  "vue/keyword-spacing",
  "vue/max-attributes-per-line",
  "vue/max-len",
  "vue/max-lines-per-block",
  "vue/max-template-depth",
  "vue/multiline-html-element-content-newline",
  "vue/multiline-ternary",
  "vue/mustache-interpolation-spacing",
  "vue/new-line-between-multi-line-property", // stylistic rule
  "vue/no-bare-strings-in-template",
  "vue/no-child-content",
  "vue/no-console",
  "vue/no-constant-condition",
  "vue/no-custom-modifiers-on-v-model",
  "vue/no-deprecated-filter",
  "vue/no-deprecated-functional-template",
  "vue/no-deprecated-html-element-is",
  "vue/no-deprecated-inline-template",
  "vue/no-deprecated-router-link-tag-prop",
  "vue/no-deprecated-scope-attribute",
  "vue/no-deprecated-slot-attribute",
  "vue/no-deprecated-slot-scope-attribute",
  "vue/no-deprecated-v-bind-sync",
  "vue/no-deprecated-v-is",
  "vue/no-deprecated-v-on-native-modifier",
  "vue/no-deprecated-v-on-number-modifiers",
  "vue/no-dupe-v-else-if",
  "vue/no-duplicate-attr-inheritance",
  "vue/no-duplicate-attributes",
  "vue/no-empty-component-block",
  "vue/no-empty-pattern",
  "vue/no-extra-parens", // stylistic rule + template parsing
  "vue/no-implicit-coercion",
  "vue/no-loss-of-precision",
  "vue/no-multi-spaces",
  "vue/no-multiple-objects-in-class",
  "vue/no-multiple-template-root",
  "vue/no-negated-condition",
  "vue/no-negated-v-if-condition",
  "vue/no-parsing-error",
  "vue/no-restricted-block",
  "vue/no-restricted-class",
  "vue/no-restricted-html-elements",
  "vue/no-restricted-static-attribute",
  "vue/no-restricted-syntax",
  "vue/no-restricted-v-bind",
  "vue/no-restricted-v-on",
  "vue/no-root-v-if",
  "vue/no-spaces-around-equal-signs-in-attribute",
  "vue/no-sparse-arrays",
  "vue/no-static-inline-styles",
  "vue/no-template-key",
  "vue/no-template-shadow",
  "vue/no-template-target-blank",
  "vue/no-textarea-mustache",
  "vue/no-undef-components",
  "vue/no-unsupported-features", // can not be up to date with vue versions + template parsing
  "vue/no-unused-components",
  "vue/no-unused-refs",
  "vue/no-unused-vars",
  "vue/no-use-v-else-with-v-for",
  "vue/no-use-v-if-with-v-for",
  "vue/no-useless-concat",
  "vue/no-useless-mustaches",
  "vue/no-useless-template-attributes",
  "vue/no-useless-v-bind",
  "vue/no-v-text-v-html-on-component",
  "vue/no-v-text",
  "vue/no-v-for-template-key",
  "vue/object-curly-newline", // stylistic rule + template parsing
  "vue/object-curly-spacing", // stylistic rule + template parsing
  "vue/object-property-newline", // stylistic rule + template parsing
  "vue/object-shorthand",
  "vue/operator-linebreak", // stylistic rule + template parsing
  "vue/padding-line-between-blocks", // stylistic rule + template parsing
  "vue/padding-line-between-tags", // stylistic rule + template parsing
  "vue/padding-lines-in-component-definition", // stylistic rule
  "vue/prefer-separate-static-class",
  "vue/prefer-template",
  "vue/prefer-true-attribute-shorthand",
  "vue/quote-props",
  "vue/require-component-is",
  "vue/require-explicit-emits",
  "vue/require-explicit-slots",
  "vue/require-toggle-inside-transition",
  "vue/require-v-for-key",
  "vue/restricted-component-names",
  "vue/singleline-html-element-content-newline",
  "vue/slot-name-casing",
  "vue/space-in-parens", // stylistic rule + template parsing
  "vue/space-infix-ops", // stylistic rule + template parsing
  "vue/space-unary-ops", // stylistic rule + template parsing
  "vue/static-class-names-order",
  "vue/template-curly-spacing", // stylistic rule + template parsing
  "vue/use-v-on-exact",
  "vue/v-bind-style",
  "vue/v-for-delimiter-style",
  "vue/v-if-else-key",
  "vue/v-on-event-hyphenation",
  "vue/v-on-handler-style",
  "vue/v-on-style",
  "vue/v-slot-style",
  "vue/valid-attribute-name",
  "vue/valid-template-root",
  "vue/valid-v-bind",
  "vue/valid-v-cloak",
  "vue/valid-v-else-if",
  "vue/valid-v-else",
  "vue/valid-v-for",
  "vue/valid-v-html",
  "vue/valid-v-if",
  "vue/valid-v-is",
  "vue/valid-v-memo",
  "vue/valid-v-model",
  "vue/valid-v-on",
  "vue/valid-v-once",
  "vue/valid-v-pre",
  "vue/valid-v-show",
  "vue/valid-v-slot",
  "vue/valid-v-text",

  "vue/no-v-for-template-key-on-child",
  "vue/no-v-model-argument",
  "vue/valid-v-bind-sync",
  "vue/valid-model-definition", // deprecated
]);

/**
 * @typedef {{
 *   docsUrl: string,
 *   isDeprecated: boolean,
 *   isRecommended: boolean,
 *   isImplemented: boolean,
 *   isNotSupported: boolean,
 *   isPendingFix: boolean,
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
      // oxlint-disable-next-line no-console
      console.log(`👀 ${name} is implemented but not found in their rules`);
    }
  }
};

/** @param {RuleEntries} ruleEntries */
export const updateNotSupportedStatus = (ruleEntries) => {
  for (const name of NOT_SUPPORTED_RULE_NAMES) {
    const rule = ruleEntries.get(name);
    if (rule) rule.isNotSupported = true;
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
