#!/usr/bin/env node
/* eslint-disable no-console */

const fs = require("node:fs");
const path = require("node:path");
const { parseArgs } = require("node:util");

const DEFAULT_RULES_ROOT = path.join("crates", "oxc_linter", "src", "rules");
const DECLARE_RULE_MACRO = "declare_oxc_lint!(";
const NEXT_VERSION_TEXT = 'version = "next"';
const NEXT_VERSION_REGEX = /version\s*=\s*"next"/;

function collectRuleFiles(dir, repoRoot) {
  const files = [];

  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isSymbolicLink()) {
      const relativePath = normalizePath(path.relative(repoRoot, entryPath));
      throw new Error(`${relativePath}: symlinked rule paths are not supported`);
    }

    if (entry.isDirectory()) {
      files.push(...collectRuleFiles(entryPath, repoRoot));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(entryPath);
    }
  }

  files.sort(compareStrings);
  return files;
}

function compareStrings(a, b) {
  if (a < b) {
    return -1;
  }
  if (a > b) {
    return 1;
  }
  return 0;
}

function normalizePath(filePath) {
  return filePath.split(path.sep).join("/");
}

// Regex to match the structured fields after doc comments inside declare_oxc_lint!():
//   RuleName,
//   plugin_name,
//   category,
//   ...optional fields like `fix`, `conditional_fix`, `pending`...
//   version = "next",
const COMMENT_LINE_REGEX = /^[ \t]*\/\/.*\n/gm;
const MACRO_BODY_REGEX = /(?:^[ \t]*\/\/\/.*\n)*[ \t]*(\w+)\s*,\s*\n\s*(\w+)\s*,\s*\n\s*(\w+)\s*,/m;

function analyzeRuleFile(source, filePath, releaseVersion, repoRoot) {
  const relativeFile = normalizePath(path.relative(repoRoot, filePath));
  const updatedRules = [];
  const skippedNurseryRules = [];

  let updatedSource = source;
  let searchFrom = 0;

  while (true) {
    // Step 1: Find the next declare_oxc_lint!( in the source
    const macroStart = updatedSource.indexOf(DECLARE_RULE_MACRO, searchFrom);
    if (macroStart === -1) break;

    const bodyStart = macroStart + DECLARE_RULE_MACRO.length;

    // Find the closing );
    const macroEnd = updatedSource.indexOf(");", bodyStart);
    if (macroEnd === -1) {
      throw new Error(`${relativeFile}: unterminated declare_oxc_lint! block`);
    }

    const body = updatedSource.slice(bodyStart, macroEnd);

    // Step 2: Extract rule name, plugin, and category from the body
    // Strip // comment lines so they don't interfere with field matching
    const strippedBody = body.replace(COMMENT_LINE_REGEX, "");
    const bodyMatch = strippedBody.match(MACRO_BODY_REGEX);
    if (!bodyMatch) {
      searchFrom = macroEnd + 2;
      continue;
    }

    const ruleName = bodyMatch[1];
    const category = bodyMatch[3];

    // Step 3: Check if version = "next" exists in the macro body
    const versionMatch = body.match(NEXT_VERSION_REGEX);
    if (!versionMatch) {
      searchFrom = macroEnd + 2;
      continue;
    }

    if (category === "nursery") {
      skippedNurseryRules.push({ file: relativeFile, ruleName });
      searchFrom = macroEnd + 2;
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
    searchFrom = macroEnd + 2;
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

  for (const filePath of collectRuleFiles(rulesRoot, repoRoot)) {
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
