#!/usr/bin/env node
/* eslint-disable no-console */

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

const DEFAULT_RULES_ROOT = path.join("crates", "oxc_linter", "src", "rules");

type RuleChange = {
  file: string;
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

function analyzeRuleFile(
  source: string,
  filePath: string,
  releaseVersion: string,
  repoRoot: string,
): AnalyzeReport {
  const relativeFile = path.relative(repoRoot, filePath).replaceAll(path.sep, "/");
  const updatedRules: RuleChange[] = [];

  const updatedSource = source.replace(
    /declare_oxc_lint!\(([\s\S]*?\n)\s*\);/g,
    (match, body: string) => {
      // Strip leading doc comments, then extract comma-delimited fields: name, plugin, category
      const stripped = body.replace(/^(?:\s*\/\/\/.*\n)*/, "");
      const fields = [...stripped.matchAll(/(\w+)\s*,/g)];
      if (fields.length < 3) return match;

      const category = fields[2][1];
      if (category === "nursery") return match;
      if (!/version\s*=\s*"next"/.test(body)) return match;

      updatedRules.push({ file: relativeFile, from: "next", to: releaseVersion });
      return match.replace(/(version\s*=\s*)"next"/, `$1"${releaseVersion}"`);
    },
  );

  return { updatedSource, updatedRules };
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
      `${dryRun ? "Would update" : "Updated"} ${change.file}: version = "next" -> version = "${change.to}"`,
    );
  }
  updatedCount += fileReport.updatedRules.length;

  if (!dryRun && fileReport.updatedRules.length > 0) {
    writeFileSync(filePath, fileReport.updatedSource);
  }
}

console.log(`${updatedCount} rule(s) ${dryRun ? "would be updated" : "updated"}.`);
