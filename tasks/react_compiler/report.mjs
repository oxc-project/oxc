/**
 * Turns a `compare.mjs --report` file into a readable summary: how the
 * mismatches break down by category, which repositories each category comes
 * from, and a representative diff for each one.
 *
 *   node report.mjs <report.jsonl> [--examples=<count>]
 */

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const DEFAULT_EXAMPLES = 2;
/** Repositories named per category before the rest are counted as "other". */
const TOP_REPOSITORIES = 6;
/** Exact-combination rows listed before the tail is summarized in one row. */
const CATEGORY_TABLE_ROWS = 20;
/** Distinct failure messages listed per side before the tail is summarized. */
const FAILURE_MESSAGE_ROWS = 12;
/** Categories that stand on their own rather than being a set of causes. */
const STANDALONE_CATEGORIES = [
  "failed",
  "structural",
  "memoization-scope",
  "memoization-slots",
  "statement-order",
  "pure-annotation",
];

const { reportPath, exampleCount } = parseArguments(process.argv.slice(2));
const entries = (await readFile(reportPath, "utf8"))
  .split("\n")
  .filter((line) => line !== "")
  .map((line) => JSON.parse(line));

const summary = entries.find(({ status }) => status === "summary") ?? null;
const mismatches = entries.filter(({ status }) => status !== "summary");
console.log(renderReport(summary, mismatches));

function parseArguments(args) {
  let reportPath = null;
  let exampleCount = DEFAULT_EXAMPLES;

  for (const arg of args) {
    const match = arg.match(/^--examples=(\d+)$/);
    if (match !== null) {
      exampleCount = Number.parseInt(match[1], 10);
    } else {
      assert(!arg.startsWith("--"), `Unknown option: ${arg}`);
      assert(reportPath === null, "Pass exactly one report file");
      reportPath = resolve(process.env.INIT_CWD ?? process.cwd(), arg);
    }
  }

  assert(reportPath !== null, "Usage: node report.mjs <report.jsonl> [--examples=<count>]");
  return { reportPath, exampleCount };
}

function renderReport(summary, mismatches) {
  const groups = groupByCategory(mismatches);
  const sections = [
    renderSummary(summary, mismatches),
    renderCauseTable(mismatches),
    renderCategoryTable(groups, mismatches),
  ];

  // A file with several causes belongs in each of their sections, so the
  // sections are keyed by cause rather than by the exact combination.
  for (const category of STANDALONE_CATEGORIES) {
    const files = groups.get(category);
    if (files === undefined || category === "pure-annotation") {
      continue;
    }
    sections.push(category === "failed" ? renderFailures(files) : renderCategory(category, files));
  }
  for (const cause of causeCounts(mismatches).keys()) {
    sections.push(
      renderCategory(
        cause,
        mismatches.filter(({ causes }) => causes?.includes(cause)),
      ),
    );
  }

  return sections.join("\n\n");
}

/** Counts how many files each individual cause contributes to. */
function causeCounts(mismatches) {
  const counts = new Map();
  for (const { causes = [] } of mismatches) {
    for (const cause of causes) {
      counts.set(cause, (counts.get(cause) ?? 0) + 1);
    }
  }
  return new Map([...counts].sort(([, left], [, right]) => right - left));
}

function renderCauseTable(mismatches) {
  const standalone = groupByCategory(mismatches);
  const rows = [
    ...STANDALONE_CATEGORIES.filter((category) => standalone.has(category)).map((category) => [
      category,
      standalone.get(category).length,
    ]),
    ...causeCounts(mismatches),
  ].sort(([, left], [, right]) => right - left);

  return [
    "## Causes",
    "",
    "One file can have several causes, so these overlap.",
    "",
    "| Cause | Files | Share of mismatches |",
    "| --- | ---: | ---: |",
    ...rows.map(
      ([cause, total]) =>
        `| \`${cause}\` | ${total.toLocaleString()} | ${percentage(total, mismatches.length)} |`,
    ),
  ].join("\n");
}

function groupByCategory(mismatches) {
  const groups = new Map();
  for (const mismatch of mismatches) {
    const category = mismatch.status === "different" ? mismatch.category : mismatch.status;
    groups.set(category, [...(groups.get(category) ?? []), mismatch]);
  }
  return new Map([...groups].sort(([, left], [, right]) => right.length - left.length));
}

function renderSummary(summary, mismatches) {
  const lines = ["# React Compiler comparison report", ""];
  if (summary === null) {
    lines.push(`${count(mismatches.length, "mismatching file")}.`);
    return lines.join("\n");
  }

  const matching = summary.counts.same;
  lines.push(
    `Compared ${summary.compared.toLocaleString()} files. ` +
      `${matching.toLocaleString()} matched exactly ` +
      `(${percentage(matching, summary.compared)}), ` +
      `${mismatches.length.toLocaleString()} did not.`,
  );
  return lines.join("\n");
}

function renderCategoryTable(groups, mismatches) {
  const shown = [...groups].slice(0, CATEGORY_TABLE_ROWS);
  const remaining = [...groups].slice(CATEGORY_TABLE_ROWS);
  const rows = shown.map(
    ([category, files]) =>
      `| \`${category}\` | ${files.length.toLocaleString()} | ${percentage(files.length, mismatches.length)} |`,
  );
  if (remaining.length > 0) {
    const total = remaining.reduce((sum, [, files]) => sum + files.length, 0);
    rows.push(
      `| *${remaining.length} rarer combinations* | ${total.toLocaleString()} | ${percentage(total, mismatches.length)} |`,
    );
  }

  return [
    "## Exact categories",
    "",
    "| Category | Files | Share of mismatches |",
    "| --- | ---: | ---: |",
    ...rows,
  ].join("\n");
}

function renderFailures(failures) {
  const lines = [`## \`failed\` — ${count(failures.length, "file")}`, ""];

  // Which pipeline refused the file matters more than the message: only the
  // Oxc side points at Oxc, while the Babel side is a limit of this harness.
  for (const side of ["oxc", "babel"]) {
    const onThisSide = failures.filter((failure) => failingSide(failure.message) === side);
    if (onThisSide.length === 0) {
      continue;
    }
    lines.push(`### ${side} — ${count(onThisSide.length, "file")}`, "");

    const byMessage = new Map();
    for (const failure of onThisSide) {
      const message = generalizeMessage(failure.message);
      byMessage.set(message, [...(byMessage.get(message) ?? []), failure]);
    }
    const sorted = [...byMessage].sort(([, left], [, right]) => right.length - left.length);
    for (const [message, files] of sorted.slice(0, FAILURE_MESSAGE_ROWS)) {
      lines.push(`- **${files.length.toLocaleString()}×** ${message}`);
      lines.push(`  - e.g. \`${files[0].path}\``);
    }
    const tail = sorted.slice(FAILURE_MESSAGE_ROWS);
    if (tail.length > 0) {
      const total = tail.reduce((sum, [, files]) => sum + files.length, 0);
      lines.push(`- *${count(total, "file")} across ${count(tail.length, "rarer message")}*`);
    }
    lines.push("");
  }

  return lines.join("\n").trimEnd();
}

function failingSide(message) {
  return message.startsWith("oxc-transform-react failed") ? "oxc" : "babel";
}

/** The harness prefixes its own assertions, so the detail is on the next line. */
function meaningfulLines(message) {
  const lines = message.split("\n").filter((line) => line.trim() !== "");
  return /failed|Oxc did not|Babel did not/.test(lines[0]) ? lines.slice(0, 2) : lines.slice(0, 1);
}

function renderCategory(category, files) {
  const lines = [
    `## \`${category}\` — ${count(files.length, "file")}`,
    "",
    renderRepositories(files),
  ];

  for (const file of pickExamples(files)) {
    lines.push("", `\`${file.path}\`${file.hunkCount > 1 ? ` (${file.hunkCount} hunks)` : ""}`, "");
    lines.push("```diff");
    for (const hunk of file.hunks.slice(0, 2)) {
      lines.push(`@@ line ${hunk.line} @@`);
      lines.push(...hunk.babel.map((line) => `-${line}`));
      lines.push(...hunk.oxc.map((line) => `+${line}`));
    }
    lines.push("```");
  }

  return lines.join("\n");
}

function renderRepositories(files) {
  const counts = new Map();
  for (const { path } of files) {
    const repository = path.split("/")[0];
    counts.set(repository, (counts.get(repository) ?? 0) + 1);
  }

  const sorted = [...counts].sort(([, left], [, right]) => right - left);
  const named = sorted.slice(0, TOP_REPOSITORIES).map(([name, total]) => `${name} (${total})`);
  const remaining = sorted.length - named.length;
  return `Repositories: ${named.join(", ")}${remaining > 0 ? `, and ${remaining} more` : ""}.`;
}

/** Prefers small diffs as examples, since they show the category most plainly. */
function pickExamples(files) {
  return files
    .filter(({ hunks }) => hunks !== undefined && hunks.length > 0)
    .sort((left, right) => hunkSize(left) - hunkSize(right))
    .slice(0, exampleCount);
}

function hunkSize(file) {
  return file.hunks.reduce((total, hunk) => total + hunk.babel.length + hunk.oxc.length, 0);
}

/** Collapses the varying parts of an error so like failures group together. */
function generalizeMessage(message) {
  return (
    meaningfulLines(message)
      .join(" — ")
      .replace(/[^\s:]*[/\\][^\s:]*/g, "<path>")
      .replace(/\(\d+[:,]\d+\)/g, "(<loc>)")
      // The identifier a message names varies per file while the defect does not.
      .replace(/\bimport\s+[A-Za-z_$][\w$]*\s*=\s*require/g, "import <name> = require")
      .replace(/(['"`])[A-Za-z_$][\w$]*\1/g, "$1<name>$1")
      .replace(/\b\d+\b/g, "<n>")
      .slice(0, 200)
  );
}

function percentage(part, total) {
  return total === 0 ? "0%" : `${((part / total) * 100).toFixed(1)}%`;
}

function count(total, noun) {
  return `${total.toLocaleString()} ${noun}${total === 1 ? "" : "s"}`;
}
