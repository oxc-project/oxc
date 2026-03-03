// oxlint-disable no-console, no-await-in-loop

import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import prettier from "prettier";
import * as sveltePlugin from "prettier-plugin-svelte";
import { format } from "../dist/index.js";

const CONFORMANCE_DIR = import.meta.dirname;
const FIXTURES_DIR = join(CONFORMANCE_DIR, "fixtures");
const SNAPSHOTS_DIR = join(CONFORMANCE_DIR, "snapshots");
const PRETTIER_FIXTURES_DIR = join(FIXTURES_DIR, "prettier");
const EDGE_CASES_DIR = join(FIXTURES_DIR, "edge-cases");

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

const categories: Category[] = [
  {
    name: "js-in-vue",
    sources: [
      { dir: PRETTIER_FIXTURES_DIR, ext: ".vue" },
      { dir: join(EDGE_CASES_DIR, "js-in-vue") },
    ],
    optionSets: [
      { printWidth: 80 },
      { printWidth: 100, vueIndentScriptAndStyle: true, singleQuote: true },
    ],
    notes: {
      "vue/multiparser/lang-tsx.vue": "`lang=tsx` is not supported",
    },
  },
  {
    name: "gql-in-js",
    sources: [
      {
        dir: join(PRETTIER_FIXTURES_DIR, "js/multiparser-graphql"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(EDGE_CASES_DIR, "gql-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {
      "comment-tag.js": "`/* GraphQL */` comment tag not yet supported",
    },
  },
  {
    name: "svelte",
    sources: [
      {
        dir: join(FIXTURES_DIR, "plugin-svelte", "samples"),
        ext: "input.html",
        excludes: ["syntax-error"],
        resolveFilePath: (name) => name.replace("/input.html", ".svelte"),
      },
    ],
    optionSets: [
      { printWidth: 80 },
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

  for (let i = 0; i < categoryResult.optionSetResults.length; i++) {
    const r = categoryResult.optionSetResults[i];
    const pct = ((r.passed / r.total) * 100).toFixed(2);
    console.log(`  Option ${i + 1}: ${r.passed}/${r.total} (${pct}%)`);
  }
}

writeReport(results);

// ---

type Fixture = { name: string; fullPath: string };

type Failure = {
  name: string;
  note?: string;
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

    for (const entry of readdirSync(source.dir, { withFileTypes: true, recursive: true })) {
      if (!entry.isFile()) continue;
      if (source.ext && !entry.name.endsWith(source.ext)) continue;

      const fullPath = join(entry.parentPath, entry.name);
      const relPath = relative(source.dir, fullPath);
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
        failures.push({ name: fixture.name, note: category.notes?.[fixture.name] });
      }
    }

    optionSetResults.push({ options, passed, total: fixtures.length, failures });
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

  for (const result of results) {
    lines.push(`## ${result.name}`);
    lines.push("");

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
          lines.push(`| ${failure.name} | ${failure.note ?? ""} |`);
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
