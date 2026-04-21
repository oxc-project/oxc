// oxlint-disable no-console, no-await-in-loop

import { createTwoFilesPatch } from "diff";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import prettier from "prettier";
import { format } from "../dist/index.js";

const CONFORMANCE_DIR = import.meta.dirname;
const FIXTURES_DIR = join(CONFORMANCE_DIR, "fixtures");
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
};

const categories: Category[] = [
  {
    name: "js-in-vue",
    sources: [
      { dir: join(FIXTURES_DIR, "prettier"), ext: ".vue" },
      { dir: join(FIXTURES_DIR, "vue-vben-admin"), ext: ".vue" },
      { dir: join(FIXTURES_DIR, "edge-cases", "js-in-vue") },
    ],
    optionSets: [
      { printWidth: 80 },
      { printWidth: 100, vueIndentScriptAndStyle: true, singleQuote: true },
    ],
    notes: {
      "prettier/vue/multiparser/lang-tsx.vue": "`lang=tsx` is not supported",
      "vue-vben-admin/effects/common-ui/src/components/api-component/api-component.vue":
        "`<T = any,>() => {}` comma in generic param is removed even in .ts(x) file",
    },
  },
  {
    name: "gql-in-js",
    sources: [
      {
        dir: join(FIXTURES_DIR, "prettier", "js/multiparser-graphql"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "gql-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {},
  },
  {
    name: "css-in-js",
    sources: [
      {
        dir: join(FIXTURES_DIR, "prettier", "js/multiparser-css"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      {
        dir: join(FIXTURES_DIR, "prettier", "jsx/embed"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "css-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
    notes: {
      "prettier/js/multiparser-css/styled-components.js": "`Xxx.extend` not recognized as tag",
    },
  },
  {
    name: "html-in-js",
    sources: [
      {
        dir: join(FIXTURES_DIR, "prettier", "js/multiparser-html"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      {
        dir: join(FIXTURES_DIR, "webawesome"),
        ext: ".ts",
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "html-in-js") },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, htmlWhitespaceSensitivity: "ignore" }],
    notes: {
      "prettier/js/multiparser-html/issue-10691.js":
        "js-in-html(`<script>`)-in-js needs lot more work; Please see oxc_formatter/src/print/template/embed/html.rs",
      "webawesome/relative-time/relative-time.test.ts":
        "html-in-js: Need to solve `label({ embed, hug }))` + `shouldExpandLastArg`",
    },
  },
  {
    name: "angular-in-js",
    sources: [
      {
        dir: join(FIXTURES_DIR, "prettier", "typescript/angular-component-examples"),
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
        dir: join(FIXTURES_DIR, "prettier", "js/multiparser-markdown"),
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
        dir: join(FIXTURES_DIR, "prettier", "js/multiparser-html/language-comment"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      {
        dir: join(FIXTURES_DIR, "prettier", "js/multiparser-comments"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "xxx-in-js-comment") },
    ],
    optionSets: [{ printWidth: 80 }, { printWith: 100 }],
    notes: {},
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

    for (const entry of readdirSync(source.dir, { withFileTypes: true, recursive: true })) {
      if (!entry.isFile()) continue;
      if (source.ext && !entry.name.endsWith(source.ext)) continue;

      const fullPath = join(entry.parentPath, entry.name);
      const relPath = relative(FIXTURES_DIR, fullPath);
      if (source.excludes?.some((s) => relPath.includes(s))) continue;

      results.push({ name: relPath, fullPath });
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
      filepath: fileName,
      ...options,
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
      { optionIndex: number; options: Record<string, unknown>; failure: Failure }[]
    >();
    for (let i = 0; i < result.optionSetResults.length; i++) {
      for (const failure of result.optionSetResults[i].failures) {
        let entries = failuresByFixture.get(failure.name);
        if (!entries) {
          entries = [];
          failuresByFixture.set(failure.name, entries);
        }
        entries.push({ optionIndex: i + 1, options: result.optionSetResults[i].options, failure });
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
          lines.push(`| ${diffLink} | ${failure.note ?? ""} |`);
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
  entries: { optionIndex: number; options: Record<string, unknown>; failure: Failure }[],
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
    lines.push(`> ${note}`);
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
