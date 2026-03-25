/**
 * Generates the `oxlint-plugin-eslint` package source files.
 *
 * This script produces:
 *
 * 1. `rules/<name>.cjs` - One file for each ESLint core rule, that re-exports the rule's `create`
 *    function.
 * 2. `index.ts` - Exports all rules as a `Record<string, CreateRule>`. This is the `rules` property of
 *    the `oxlint-plugin-eslint` plugin.
 * 3. `rule_names.ts` - Exports a list of all rule names, which is used in TSDown config.
 *
 * `index.ts` uses a split eager/lazy strategy so that `registerPlugin` can read each rule's `meta`
 * without loading the rule module itself:
 *
 * - `meta` is serialized and inlined at build time. `registerPlugin` needs it at plugin registration
 *   time (for `fixable`, `hasSuggestions`, `schema`, `defaultOptions`, `messages`), so it must be
 *   available immediately without requiring the rule module.
 * - `create` is deferred via a cached `require` call. The rule module is only loaded the first time
 *   `create` is called (i.e. when the rule actually runs at lint time). A top-level variable per
 *   rule caches the loaded function so subsequent calls skip the `require` call.
 *
 * Build-time validations:
 *
 * - Each rule object must only have `meta` and `create` properties.
 * - `meta` values are walked to ensure they contain no functions (which would be serialized as
 *   executable code by `serialize-javascript`).
 */

import { readdirSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { join as pathJoin, basename, relative as pathRelative } from "node:path";
import { createRequire } from "node:module";
import { execSync } from "node:child_process";
import serialize from "serialize-javascript";

import type { CreateRule } from "../src-js/plugins/load.ts";
import type { RuleMeta } from "../src-js/plugins/rule_meta.ts";

const require = createRequire(import.meta.url);

const oxlintDirPath = pathJoin(import.meta.dirname, "..");
const rootDirPath = pathJoin(oxlintDirPath, "../..");
const eslintRulesDir = pathJoin(require.resolve("eslint/package.json"), "../lib/rules");
const generatedDirPath = pathJoin(oxlintDirPath, "src-js/generated/plugin-eslint");
const generatedRulesDirPath = pathJoin(generatedDirPath, "rules");

export default function generatePluginEslint(): void {
  // Get all ESLint rule names (exclude `index.js` which is the registry, not a rule)
  const ruleNames = readdirSync(eslintRulesDir)
    .filter((filename) => filename.endsWith(".js") && filename !== "index.js")
    .map((filename) => basename(filename, ".js"))
    .sort();

  // oxlint-disable-next-line no-console
  console.log(`Found ${ruleNames.length} ESLint rules`);

  // Wipe and recreate generated directories
  rmSync(generatedDirPath, { recursive: true, force: true });
  mkdirSync(generatedRulesDirPath, { recursive: true });

  // Generate a CJS wrapper file for each rule
  for (const ruleName of ruleNames) {
    const relPath = pathRelative(generatedRulesDirPath, pathJoin(eslintRulesDir, `${ruleName}.js`));
    const content = `module.exports = require(${JSON.stringify(relPath)}).create;\n`;
    writeFileSync(pathJoin(generatedRulesDirPath, `${ruleName}.cjs`), content);
  }

  // Generate the plugin rules index.
  // `meta` is inlined so it's available at registration time without loading the rule module.
  // `create` is deferred via a cached `require` so the rule module is only loaded on first use.
  const indexLines = [
    `
      import { createRequire } from "node:module";

      import type { CreateRule } from "../../plugins/load.ts";

      type CreateFn = CreateRule["create"];

      var require = createRequire(import.meta.url);
    `,
  ];

  // Generate a `let` declaration for each rule's cached `create` function.
  // These are initially `null` and populated on first call.
  for (let i = 0; i < ruleNames.length; i++) {
    indexLines.push(`var create${i}: CreateFn | null = null;`);
  }

  indexLines.push("", "export default {");

  for (let i = 0; i < ruleNames.length; i++) {
    const ruleName = ruleNames[i];
    const rulePath = pathJoin(eslintRulesDir, `${ruleName}.js`);
    const rule: CreateRule = require(rulePath);

    // Validate that the rule only has expected top-level properties.
    // If ESLint adds new properties in a future version, we want to find out at build time.
    const unexpectedKeys = Object.keys(rule).filter((key) => key !== "meta" && key !== "create");
    if (unexpectedKeys.length > 0) {
      throw new Error(
        `Unexpected properties on rule \`${ruleName}\`: ${unexpectedKeys.join(", ")}. ` +
          "Expected only `meta` and `create`.",
      );
    }

    // Reduce `meta` to only the properties Oxlint uses, with consistent shape and property order.
    // We discard e.g. `deprecated` and `docs` properties. This reduces code size.
    // Default values match what `registerPlugin` assumes when a property is absent.
    const { meta } = rule;
    const reducedMeta: RuleMeta = {
      messages: meta?.messages ?? undefined,
      fixable: meta?.fixable ?? null,
      hasSuggestions: meta?.hasSuggestions ?? false,
      schema: meta?.schema ?? undefined,
      defaultOptions: meta?.defaultOptions ?? undefined,
    };

    // Check for function values in `reducedMeta`, which would be unexpected and likely a bug.
    // `serialize-javascript` would serialize them as executable code, so catch this at build time.
    assertNoFunctions(reducedMeta, `eslint/lib/rules/${ruleName}.js`, "meta");

    const metaCode = serialize(reducedMeta, { unsafe: true });

    indexLines.push(`
      ${JSON.stringify(ruleName)}: {
        meta: ${metaCode},
        create(context) {
          if (create${i} === null) create${i} = require("./rules/${ruleName}.cjs") as CreateFn;
          return create${i}(context);
        },
      },
    `);
  }
  indexLines.push("} satisfies Record<string, CreateRule>;\n");

  const indexFilePath = pathJoin(generatedDirPath, "index.ts");
  writeFileSync(indexFilePath, indexLines.join("\n"));

  // Format generated index file with oxfmt to clean up unnecessary quotes around property names.
  // This isn't necessary, as it gets minified and bundled anyway, but it makes generated code easier to read
  // when debugging.
  execSync(`pnpm exec oxfmt --write ${JSON.stringify(indexFilePath)}`, { cwd: rootDirPath });

  // Generate the rule_names.ts file for use in tsdown config
  const ruleNamesCode = [
    "export default [",
    ...ruleNames.map((name) => `  ${JSON.stringify(name)},`),
    "] as const;\n",
  ].join("\n");

  writeFileSync(pathJoin(generatedDirPath, "rule_names.ts"), ruleNamesCode);

  // oxlint-disable-next-line no-console
  console.log("Generated plugin-eslint files.");
}

/**
 * Walk an object tree and throw if any function values are found.
 */
function assertNoFunctions(value: unknown, rulePath: string, path: string): void {
  if (typeof value === "function") {
    throw new Error(
      `Unexpected function value in \`${path}\` of rule \`${rulePath}\`. ` +
        "Rule meta objects must be static data.",
    );
  }
  if (typeof value === "object" && value !== null) {
    for (const [key, child] of Object.entries(value)) {
      assertNoFunctions(child, rulePath, `${path}.${key}`);
    }
  }
}
