#!/usr/bin/env node
/* eslint-disable no-console */

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

const DEFAULT_RULES_ROOT = path.join("crates", "oxc_linter", "src", "rules");
const DECLARE_RULE_MACRO = "declare_oxc_lint!(";
const NEXT_VERSION_REGEX = /version\s*=\s*"next"/;

type RuleChange = {
  file: string;
  ruleName: string;
  from: "next";
  to: string;
};

type AnalyzeReport = {
  updatedSource: string;
  updatedRules: RuleChange[];
};

function collectRuleFiles(rulesRoot: string): string[] {
  try {
    const output = execFileSync(
      "grep",
      ["-Erl", 'version[[:space:]]*=[[:space:]]*"next"', "--include=*.rs", rulesRoot],
      { encoding: "utf8" },
    );
    return output.trim().split("\n").filter(Boolean).sort();
  } catch {
    // grep exits with code 1 when no matches are found
    return [];
  }
}

// Finds the next "word," field in body starting from `from`, skipping comments and whitespace.
// Returns { word, end } where end is the index after the comma, or null if not found.
function findNextField(body: string, from: number): { word: string; end: number } | null {
  const match = body.slice(from).match(/(\w+)\s*,/);
  if (!match) return null;

  const matchIndex = match.index;
  if (matchIndex === undefined) return null;

  return { word: match[1], end: from + matchIndex + match[0].length };
}

// Skips past any /// doc comment lines at the start of body.
function skipDocComments(body: string): number {
  const match = body.match(/^(?:\s*\/\/\/.*\n)*/);
  return match ? match[0].length : 0;
}

function analyzeRuleFile(
  source: string,
  filePath: string,
  releaseVersion: string,
  repoRoot: string,
): AnalyzeReport {
  const relativeFile = path.relative(repoRoot, filePath).replaceAll(path.sep, "/");
  const updatedRules: RuleChange[] = [];

  let updatedSource = source;
  let searchFrom = 0;

  while (true) {
    // Step 1: Find the next declare_oxc_lint!( in the source
    const macroStart = updatedSource.indexOf(DECLARE_RULE_MACRO, searchFrom);
    if (macroStart === -1) break;

    const bodyStart = macroStart + DECLARE_RULE_MACRO.length;

    // Find the closing ); on its own line (not inside doc comments)
    const macroEndMatch = updatedSource.slice(bodyStart).match(/^\s*\);/m);
    if (!macroEndMatch) {
      throw new Error(`${relativeFile}: unterminated declare_oxc_lint! block`);
    }
    const macroEndMatchIndex = macroEndMatch.index;
    if (macroEndMatchIndex === undefined) {
      throw new Error(`${relativeFile}: unterminated declare_oxc_lint! block`);
    }
    const macroEnd = bodyStart + macroEndMatchIndex;
    const macroEndFull = macroEnd + macroEndMatch[0].length;

    const body = updatedSource.slice(bodyStart, macroEnd);

    // Step 2: Skip doc comments, then parse fields in order: name, plugin, category
    const pos = skipDocComments(body);

    const nameField = findNextField(body, pos);
    if (!nameField) {
      searchFrom = macroEndFull;
      continue;
    }
    const ruleName = nameField.word;

    const pluginField = findNextField(body, nameField.end);
    if (!pluginField) {
      searchFrom = macroEndFull;
      continue;
    }

    const categoryField = findNextField(body, pluginField.end);
    if (!categoryField) {
      searchFrom = macroEndFull;
      continue;
    }
    const category = categoryField.word;

    // Step 3: Check if version = "next" exists in the macro body
    const versionMatch = body.match(NEXT_VERSION_REGEX);
    if (!versionMatch) {
      searchFrom = macroEndFull;
      continue;
    }

    if (category === "nursery") {
      searchFrom = macroEndFull;
      continue;
    }

    // Step 4: Replace only "next" with the release version, preserving spacing
    const nextLiteral = '"next"';
    const versionMatchIndex = versionMatch.index;
    if (versionMatchIndex === undefined) {
      searchFrom = macroEndFull;
      continue;
    }
    const versionStart = bodyStart + versionMatchIndex;
    const nextIndex = updatedSource.indexOf(nextLiteral, versionStart);
    updatedSource =
      updatedSource.slice(0, nextIndex) +
      `"${releaseVersion}"` +
      updatedSource.slice(nextIndex + nextLiteral.length);

    updatedRules.push({ file: relativeFile, ruleName, from: "next", to: releaseVersion });
    searchFrom = macroEndFull;
  }

  return {
    updatedSource,
    updatedRules,
  };
}

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "release-version": { type: "string", short: "r" },
    root: { type: "string", short: "C", default: process.cwd() },
    write: { type: "boolean", short: "w", default: false },
    help: { type: "boolean", short: "h" },
  },
  strict: true,
});

const releaseVersion = values["release-version"];
const { root, write } = values;

if (values.help) {
  console.log(`Usage:
  node .github/scripts/update-rule-versions.mts --release-version <x.y.z> [--root <path>] [--write]

Options:
  --release-version, -r  Version to replace \`version = "next"\` with
  --root, -C             Repository root (defaults to current working directory)
  --write, -w            Write changes to files (default: dry-run)
  --help, -h             Show this help
`);
  process.exit(0);
}

if (!releaseVersion) {
  throw new Error("missing required `--release-version <x.y.z>`");
}
if (!/^\d+\.\d+\.\d+$/.test(releaseVersion)) {
  throw new Error(`release version must be x.y.z, got \`${releaseVersion}\``);
}

const dryRun = !write;
const repoRoot = path.resolve(root);
const rulesRoot = path.join(repoRoot, DEFAULT_RULES_ROOT);
if (!existsSync(rulesRoot)) {
  throw new Error(`rules root does not exist: ${rulesRoot}`);
}

let updatedCount = 0;

for (const filePath of collectRuleFiles(rulesRoot)) {
  const source = readFileSync(filePath, "utf8");
  const fileReport = analyzeRuleFile(source, filePath, releaseVersion, repoRoot);

  for (const change of fileReport.updatedRules) {
    console.log(
      `${dryRun ? "Would update" : "Updated"} ${change.file}: ${change.ruleName} version = "next" -> version = "${change.to}"`,
    );
  }
  updatedCount += fileReport.updatedRules.length;

  if (!dryRun && fileReport.updatedRules.length > 0) {
    writeFileSync(filePath, fileReport.updatedSource);
  }
}

console.log(`\nTotal: ${updatedCount} rule(s) ${dryRun ? "would be updated" : "updated"}.`);
