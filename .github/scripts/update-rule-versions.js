#!/usr/bin/env node
/* eslint-disable no-console */

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const { parseArgs } = require("node:util");

const DEFAULT_RULES_ROOT = path.join("crates", "oxc_linter", "src", "rules");
const DECLARE_RULE_MACRO = "declare_oxc_lint!(";
const NEXT_VERSION_TEXT = 'version = "next"';
const NEXT_VERSION_REGEX = /version\s*=\s*"next"/;

function collectRuleFiles(rulesRoot) {
  try {
    const output = execFileSync("grep", ["-rl", NEXT_VERSION_TEXT, "--include=*.rs", rulesRoot], {
      encoding: "utf8",
    });
    return output.trim().split("\n").filter(Boolean).sort();
  } catch {
    // grep exits with code 1 when no matches are found
    return [];
  }
}

// Finds the next "word," field in body starting from `from`, skipping comments and whitespace.
// Returns { word, end } where end is the index after the comma, or null if not found.
function findNextField(body, from) {
  const match = body.slice(from).match(/(\w+)\s*,/);
  if (!match) return null;
  return { word: match[1], end: from + match.index + match[0].length };
}

// Skips past any /// doc comment lines at the start of body.
function skipDocComments(body) {
  const match = body.match(/^(?:\s*\/\/\/.*\n)*/);
  return match ? match[0].length : 0;
}

function analyzeRuleFile(source, filePath, releaseVersion, repoRoot) {
  const relativeFile = path.relative(repoRoot, filePath).replaceAll(path.sep, "/");
  const updatedRules = [];
  const skippedNurseryRules = [];

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
    const macroEnd = bodyStart + macroEndMatch.index;
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
      skippedNurseryRules.push({ file: relativeFile, ruleName });
      searchFrom = macroEndFull;
      continue;
    }

    // Step 4: Replace only "next" with the release version, preserving spacing
    const nextLiteral = '"next"';
    const versionStart = bodyStart + versionMatch.index;
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
    skippedNurseryRules,
  };
}

function rewriteNextRuleVersions({ root, releaseVersion }) {
  const repoRoot = path.resolve(root);
  const rulesRoot = path.join(repoRoot, DEFAULT_RULES_ROOT);
  if (!fs.existsSync(rulesRoot)) {
    throw new Error(`rules root does not exist: ${rulesRoot}`);
  }

  const report = { updatedRules: [], skippedNurseryRules: [], pendingWrites: [] };

  for (const filePath of collectRuleFiles(rulesRoot)) {
    const source = fs.readFileSync(filePath, "utf8");
    if (!NEXT_VERSION_REGEX.test(source)) {
      continue;
    }

    const fileReport = analyzeRuleFile(source, filePath, releaseVersion, repoRoot);
    report.updatedRules.push(...fileReport.updatedRules);
    report.skippedNurseryRules.push(...fileReport.skippedNurseryRules);

    if (fileReport.updatedRules.length > 0) {
      report.pendingWrites.push({ filePath, updatedSource: fileReport.updatedSource });
    }
  }

  return report;
}

function printReport(report, dryRun) {
  if (report.updatedRules.length === 0) {
    console.log("No stable rule versions needed updating.");
  } else {
    console.log(
      `${dryRun ? "Would update" : "Updated"} ${report.updatedRules.length} rule version(s):`,
    );
    for (const change of report.updatedRules) {
      console.log(
        `- ${change.file}: ${change.ruleName} ${NEXT_VERSION_TEXT} -> version = "${change.to}"`,
      );
    }
  }

  if (report.skippedNurseryRules.length > 0) {
    console.log(`Skipped ${report.skippedNurseryRules.length} nursery rule(s):`);
    for (const skippedRule of report.skippedNurseryRules) {
      console.log(`- ${skippedRule.file}: ${skippedRule.ruleName}`);
    }
  }
}

function main(argv = process.argv.slice(2)) {
  const { values } = parseArgs({
    args: argv,
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
  node .github/scripts/update-rule-versions.js --release-version <x.y.z> [--root <path>] [--write]

Options:
  --release-version, -r  Version to replace \`version = "next"\` with
  --root, -C             Repository root (defaults to current working directory)
  --write, -w            Write changes to files (default: dry-run)
  --help, -h             Show this help
`);
    return;
  }

  if (!releaseVersion) {
    throw new Error("missing required `--release-version <x.y.z>`");
  }
  if (!/^\d+\.\d+\.\d+$/.test(releaseVersion)) {
    throw new Error(`release version must be x.y.z, got \`${releaseVersion}\``);
  }

  const report = rewriteNextRuleVersions({ root, releaseVersion });
  if (write) {
    for (const { filePath, updatedSource } of report.pendingWrites) {
      fs.writeFileSync(filePath, updatedSource);
    }
  }
  printReport(report, !write);
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = {
  analyzeRuleFile,
};
