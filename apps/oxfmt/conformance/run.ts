// oxlint-disable no-console, no-await-in-loop

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
const NOTE_FILL_BREAK_POSITION =
  "fill break position (Prettier breaks inside the wide chunk, ours at the separator). See crates/oxc_formatter_css/DIVERGENCES.md#fill-break-position";
const NOTE_MQ_OP_SPACING =
  "media-query operator spacing. See crates/oxc_formatter_css/DIVERGENCES.md#media-query-operator-spacing";
const NOTE_EOL_LINE_COMMENT_WIDTH =
  "trailing `//` comment never counts toward print width. See crates/oxc_formatter_css/DIVERGENCES.md#trailing-line-comment-print-width";

const NOTE_EMBEDDED_EXPRESSION_INDENT =
  "embedded `${expr}` re-indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#template-expression-indent";

const NOTE_UNION_ANNOTATION_FLAT =
  "union out of its `:`/`as` position expands to leading-`|` right away. See crates/oxc_formatter/DIVERGENCES.md#union-annotation-flat-retry";

const NOTE_BLOCK_SCALAR_TRAILING_WS =
  "block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace";

const NOTE_BROKEN_TEMPLATE_COMMENT_INDENT =
  "broken `${}` holding comments indents to the placeholder. See apps/oxfmt/DIVERGENCES.md#broken-template-comment-indent";
const NOTE_TS_IN_VUE_GENERIC_COMMA =
  "`<T = any,>` comma removed like plain `.ts`. See apps/oxfmt/DIVERGENCES.md#ts-in-vue-generic-trailing-comma";
const NOTE_STYLED_EXTEND_TAG =
  "`Xxx.extend` not recognized as tag. See apps/oxfmt/DIVERGENCES.md#styled-extend-tag";

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
      "externals/prettier/js/multiparser-graphql/graphql-tag.js":
        "`{ # c` comment after an opening delimiter stays inline. See crates/oxc_formatter_graphql/DIVERGENCES.md#comment-after-opening-delimiter",
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
}
