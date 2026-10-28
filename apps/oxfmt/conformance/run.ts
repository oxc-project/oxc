// oxlint-disable no-console, no-await-in-loop

import { createTwoFilesPatch } from "diff";
import {
  existsSync,
  globSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join, relative } from "node:path";
import prettier from "prettier";
import * as astroPlugin from "prettier-plugin-astro";
import * as sveltePlugin from "prettier-plugin-svelte";
import { format } from "../dist/index.js";

const CONFORMANCE_DIR = import.meta.dirname;
const FIXTURES_DIR = join(CONFORMANCE_DIR, "fixtures");
const EXTERNALS_DIR = join(FIXTURES_DIR, "externals");
const SNAPSHOTS_DIR = join(CONFORMANCE_DIR, "snapshots");
const REPO_ROOT = join(CONFORMANCE_DIR, "..", "..", "..");
const DIVERGENCES_FILES = globSync("{apps/oxfmt,crates/oxc_formatter*}/DIVERGENCES.md", {
  cwd: REPO_ROOT,
});

type Category = {
  name: string;
  /** Test suites (Prettier's own, the plugins', `edge-cases/`): compared as written. */
  fixtures: Source[];
  /** Real-world projects: compared on Prettier's output of the file, as written when it has no stable one. */
  projects: Source[];
  optionSets: Record<string, unknown>[];
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
    fixtures: [
      {
        dir: join(EXTERNALS_DIR, "prettier"),
        ext: ".vue",
        // Parser error messages and range formatting are not formatter concerns
        excludes: ["/_errors_/", "vue/range/"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "js-in-vue") },
    ],
    projects: [{ dir: join(EXTERNALS_DIR, "vue-vben-admin"), ext: ".vue" }],
    optionSets: [
      { printWidth: 80 },
      { printWidth: 100, vueIndentScriptAndStyle: true, singleQuote: true },
    ],
  },
  {
    name: "gql-in-js",
    fixtures: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-graphql"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "gql-in-js") },
    ],
    projects: [],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
  },
  {
    name: "css-in-js",
    fixtures: [
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
    projects: [],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
  },
  {
    name: "html-in-js",
    fixtures: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-html"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/embeded"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "html-in-js") },
    ],
    projects: [{ dir: join(EXTERNALS_DIR, "webawesome"), ext: ".ts" }],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, htmlWhitespaceSensitivity: "ignore" }],
  },
  {
    name: "angular-in-js",
    fixtures: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "typescript/angular-component-examples"),
        ext: ".ts",
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "angular-in-js") },
    ],
    projects: [],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, htmlWhitespaceSensitivity: "ignore" }],
  },
  {
    name: "md-in-js",
    fixtures: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "js/multiparser-markdown"),
        ext: ".js",
        excludes: ["format.test.js"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "md-in-js") },
    ],
    projects: [],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, proseWrap: "always" }],
  },
  {
    name: "xxx-in-js-comment",
    fixtures: [
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
    projects: [],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
  },
  {
    name: "svelte",
    fixtures: [
      {
        dir: join(EXTERNALS_DIR, "plugin-svelte"),
        ext: "input.html",
        excludes: ["syntax-error"],
        resolveFilePath: (name) => name.replace("/input.html", ".svelte"),
      },
    ],
    projects: [],
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
  },
  {
    name: "astro",
    fixtures: [
      { dir: join(EXTERNALS_DIR, "plugin-astro"), ext: "input.astro" },
      { dir: join(FIXTURES_DIR, "edge-cases", "astro") },
    ],
    projects: [],
    optionSets: [
      { printWidth: 80, astro: {} },
      {
        printWidth: 120,
        singleQuote: true,
        semi: false,
        // For prettier
        astroAllowShorthand: true,
        astroCompressHTML: "html",
        // For oxfmt
        astro: { allowShorthand: true, compressHTML: "html" },
      },
    ],
  },
  {
    name: "markdown",
    fixtures: [
      {
        dir: join(EXTERNALS_DIR, "prettier", "markdown"),
        ext: ".md",
        // Cursor offsets and plugin loading are not formatter concerns
        excludes: ["markdown/cursor/", "markdown/broken-plugins/"],
      },
      { dir: join(FIXTURES_DIR, "edge-cases", "xxx-in-md") },
    ],
    projects: [],
    optionSets: [{ printWidth: 80 }, { printWidth: 100, proseWrap: "always" }],
  },
  {
    // Real-world documents, dense with fenced code (css / html / js)
    name: "markdown-mdn",
    fixtures: [],
    projects: [
      { dir: join(EXTERNALS_DIR, "mdn-learn"), ext: ".md" },
      { dir: join(EXTERNALS_DIR, "mdn-css-guides"), ext: ".md" },
    ],
    // mdn's own `.prettierrc` first: the documents are already formatted by it.
    // `proseWrap: always` is left to the `markdown` category,
    // mdn's GitHub alerts and macros would fail on known divergences (`line-shapes`) in bulk.
    optionSets: [{ printWidth: 80, bracketSameLine: true }, { printWidth: 100 }],
  },
  {
    name: "graphql",
    fixtures: [],
    projects: [{ dir: join(EXTERNALS_DIR, "gitlab"), ext: ".graphql" }],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
  },
  {
    name: "less",
    fixtures: [],
    projects: [{ dir: join(EXTERNALS_DIR, "ng-zorro-antd"), ext: ".less" }],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
  },
  {
    name: "css",
    fixtures: [],
    projects: [
      { dir: join(EXTERNALS_DIR, "mantine"), ext: ".css" },
      { dir: join(EXTERNALS_DIR, "docusaurus"), ext: ".css" },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
  },
  {
    name: "yaml",
    fixtures: [],
    projects: [
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
  },
  {
    name: "scss",
    fixtures: [],
    projects: [
      { dir: join(EXTERNALS_DIR, "vue-vben-admin"), ext: ".scss" },
      { dir: join(EXTERNALS_DIR, "gitlab"), ext: ".scss" },
    ],
    optionSets: [{ printWidth: 80 }, { printWidth: 100 }],
  },
  {
    name: "jsdoc",
    fixtures: [],
    projects: [{ dir: join(EXTERNALS_DIR, "svelte"), ext: ".js" }],
    optionSets: [{ printWidth: 100 }],
  },
];

// ---

const divergences = collectDivergences();
const results: CategoryResult[] = [];

for (const category of categories) {
  const fixtures = [
    ...collectFixtures(category.fixtures ?? [], false),
    ...collectFixtures(category.projects ?? [], true),
  ].sort((a, b) => a.name.localeCompare(b.name));

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

writeReport(results, divergences);

const failedNames = new Set(
  results.flatMap((r) => r.optionSetResults.flatMap((o) => o.failures.map((f) => f.name))),
);
for (const name of failedNames) {
  if (!divergences.has(name)) {
    console.warn(`WARNING: "${name}" fails and no DIVERGENCES.md entry lists it (unclassified)`);
  }
}
for (const [name, refs] of divergences) {
  if (!failedNames.has(name)) {
    console.warn(`WARNING: "${name}" passes but ${refs.join(", ")} lists it, remove it?`);
  }
}

// ---

/** Fixture name -> `<file>#<slug>` for every backticked `externals/` / `edge-cases/` path in a DIVERGENCES.md entry. */
function collectDivergences(): Map<string, string[]> {
  const map = new Map<string, string[]>();
  for (const file of DIVERGENCES_FILES) {
    let slug = "";
    for (const line of readFileSync(join(REPO_ROOT, file), "utf8").split("\n")) {
      if (line.startsWith("## ")) slug = line.slice(3).trim();
      for (const [, name] of line.matchAll(
        /`(?:conformance\/fixtures\/)?((?:externals|edge-cases)\/[^`]+)`/g,
      )) {
        map.set(name, [...(map.get(name) ?? []), `${file}#${slug}`]);
      }
    }
  }
  return map;
}

type Fixture = { name: string; fullPath: string; isProject: boolean };

type Failure = {
  name: string;
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

function collectFixtures(sources: Source[], isProject: boolean): Fixture[] {
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
      results.push({ name, fullPath, isProject });
    }
  }

  return results;
}

async function runCategory(category: Category, fixtures: Fixture[]): Promise<CategoryResult> {
  const optionSetResults: OptionSetResult[] = [];

  for (const options of category.optionSets) {
    let passed = 0;
    const failures: Failure[] = [];

    for (const fixture of fixtures) {
      let input = readFileSync(fixture.fullPath, "utf8");
      let expected = await formatWithPrettier(fixture.name, input, options);
      // A project hands over Prettier's output, unless Prettier cannot read it back either
      if (fixture.isProject && expected !== null) {
        const again = await formatWithPrettier(fixture.name, expected, options);
        if (again !== null) [input, expected] = [expected, again];
      }
      const prettierResult = expected ?? "ERROR";
      const oxfmtResult = await formatWithOxfmt(fixture.name, input, options);

      // Both failing to parse is not a pass, the fixture exercised nothing
      if (oxfmtResult === prettierResult && oxfmtResult !== "ERROR") {
        passed++;
      } else {
        failures.push({
          name: fixture.name,
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

/** `null` when Prettier fails. */
async function formatWithPrettier(
  fileName: string,
  content: string,
  options: Record<string, unknown>,
): Promise<string | null> {
  try {
    return await prettier.format(content, {
      ...options,
      filepath: fileName,
      plugins: [sveltePlugin, astroPlugin],
    });
  } catch {
    return null;
  }
}

async function formatWithOxfmt(
  fileName: string,
  content: string,
  options: Record<string, unknown>,
): Promise<string> {
  const res = await format(fileName, content, options);
  return res.errors.length === 0 ? res.code : "ERROR";
}

function writeReport(results: CategoryResult[], divergences: Map<string, string[]>) {
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
        for (const failure of r.failures) {
          const safeName = failure.name.replaceAll("/", "__");
          const diffRelPath = `diffs/${result.name}/${safeName}.md`;
          const refs = divergences.get(failure.name)?.join(", ") ?? "unclassified";
          lines.push(`- [${failure.name}](${diffRelPath})`);
          lines.push(`  - ${refs}`);
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
