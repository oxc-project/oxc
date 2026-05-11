#!/usr/bin/env node
/* eslint-disable no-console */

import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

const DEFAULT_RULES_ROOT = path.join("crates", "oxc_linter", "src", "rules");

function analyzeRuleFile(source: string, releaseVersion: string): string {
  return source.replace(/declare_oxc_lint!\(([\s\S]*?\n)\s*\);/g, (match, body: string) => {
    // Strip leading doc comments, then extract comma-delimited fields: name, plugin, category
    const stripped = body.replace(/^(?:\s*\/\/\/.*\n)*/, "");
    const fields = [...stripped.matchAll(/(\w+)\s*,/g)];
    if (fields.length < 3) return match;

    const category = fields[2][1];
    if (category === "nursery") return match;
    if (!/version\s*=\s*"next"/.test(body)) return match;

    return match.replace(/(version\s*=\s*)"next"/, `$1"${releaseVersion}"`);
  });
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

const ruleFiles = readdirSync(rulesRoot, { recursive: true })
  .filter((f) => typeof f === "string" && f.endsWith(".rs"))
  .map((f) => path.join(rulesRoot, String(f)))
  .sort();

for (const filePath of ruleFiles) {
  const source = readFileSync(filePath, "utf8");
  const updatedSource = analyzeRuleFile(source, releaseVersion);
  if (updatedSource === source) continue;

  const relativeFile = path.relative(repoRoot, filePath).replaceAll(path.sep, "/");
  console.log(
    `${dryRun ? "Would update" : "Updated"} ${relativeFile}: version = "next" -> version = "${releaseVersion}"`,
  );
  updatedCount++;

  if (!dryRun) {
    writeFileSync(filePath, updatedSource);
  }
}

console.log(`${updatedCount} rule(s) ${dryRun ? "would be updated" : "updated"}.`);
