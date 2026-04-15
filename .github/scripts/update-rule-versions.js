#!/usr/bin/env node
/* eslint-disable no-console */

const fs = require("node:fs");
const path = require("node:path");

const DEFAULT_RULES_ROOT = path.join("crates", "oxc_linter", "src", "rules");
const DECLARE_RULE_MACRO = "declare_oxc_lint!(";
const NEXT_VERSION_TEXT = 'version = "next"';
const NEXT_VERSION_REGEX = /version\s*=\s*"next"/;
const VALID_CATEGORIES = new Set([
  "correctness",
  "suspicious",
  "pedantic",
  "perf",
  "style",
  "restriction",
  "nursery",
]);

function validateReleaseVersion(releaseVersion) {
  if (!/^\d+\.\d+\.\d+$/.test(releaseVersion)) {
    throw new Error(`release version must be x.y.z, got \`${releaseVersion}\``);
  }
}

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

function replaceVersionLiteral(line, releaseVersion) {
  const match = line.match(/^(\s*version\s*=\s*)"next"(.*)$/);
  if (!match) {
    throw new Error(`could not rewrite version line: ${line.trim()}`);
  }

  return `${match[1]}"${releaseVersion}"${match[2]}`;
}

function isCommentOnlyLine(trimmedLine) {
  return (
    trimmedLine.startsWith("///") ||
    trimmedLine.startsWith("//") ||
    trimmedLine.startsWith("/*") ||
    trimmedLine.startsWith("*") ||
    trimmedLine.startsWith("*/")
  );
}

function stripTrailingComments(line) {
  return line
    .replace(/\s*\/\*.*\*\/\s*$/, "")
    .replace(/\s*\/\/.*$/, "")
    .trim();
}

function normalizeMetadataLine(line) {
  const trimmedLine = line.trim();
  if (!trimmedLine || isCommentOnlyLine(trimmedLine)) {
    return null;
  }

  const strippedLine = stripTrailingComments(trimmedLine);
  return strippedLine || null;
}

function analyzeRuleFile(source, filePath, releaseVersion, repoRoot) {
  const relativeFile = normalizePath(path.relative(repoRoot, filePath));
  const lines = source.split("\n");
  const updatedLines = [...lines];
  const coveredNextVersionLines = new Set();
  const declareRuleBlocks = [];
  const updatedRules = [];
  const skippedNurseryRules = [];

  for (let startLine = 0; startLine < lines.length; startLine++) {
    if (!lines[startLine].includes(DECLARE_RULE_MACRO)) {
      continue;
    }

    let endLine = startLine + 1;
    while (endLine < lines.length && stripTrailingComments(lines[endLine]) !== ");") {
      endLine += 1;
    }

    if (endLine >= lines.length) {
      throw new Error(`${relativeFile}: unterminated declare_oxc_lint! block`);
    }

    declareRuleBlocks.push({ startLine, endLine });

    const metadataEntries = [];
    for (let lineIndex = startLine + 1; lineIndex < endLine; lineIndex++) {
      const normalizedLine = normalizeMetadataLine(lines[lineIndex]);
      if (!normalizedLine) {
        continue;
      }

      metadataEntries.push({ lineIndex, trimmed: normalizedLine });
    }

    const versionEntry = metadataEntries.find(({ trimmed }) => NEXT_VERSION_REGEX.test(trimmed));
    if (!versionEntry) {
      startLine = endLine;
      continue;
    }

    if (metadataEntries.length < 3) {
      throw new Error(
        `${relativeFile}: could not parse rule category from declare_oxc_lint! block`,
      );
    }

    const ruleName = metadataEntries[0].trimmed.replace(/,$/, "");
    const category = metadataEntries[2].trimmed.replace(/,$/, "");

    if (!VALID_CATEGORIES.has(category)) {
      throw new Error(`${relativeFile}: unknown rule category \`${category}\``);
    }

    coveredNextVersionLines.add(versionEntry.lineIndex);

    if (category === "nursery") {
      skippedNurseryRules.push({ file: relativeFile, ruleName });
      startLine = endLine;
      continue;
    }

    updatedLines[versionEntry.lineIndex] = replaceVersionLiteral(
      lines[versionEntry.lineIndex],
      releaseVersion,
    );
    updatedRules.push({ file: relativeFile, ruleName, from: "next", to: releaseVersion });
    startLine = endLine;
  }

  for (const [lineIndex, line] of lines.entries()) {
    const isCommentLineInsideDeclareRuleBlock = declareRuleBlocks.some(
      ({ startLine, endLine }) =>
        lineIndex > startLine && lineIndex < endLine && isCommentOnlyLine(line.trim()),
    );
    const strippedLine = stripTrailingComments(line);
    if (
      NEXT_VERSION_REGEX.test(strippedLine) &&
      !coveredNextVersionLines.has(lineIndex) &&
      !isCommentLineInsideDeclareRuleBlock
    ) {
      throw new Error(
        `${relativeFile}: found \`${NEXT_VERSION_TEXT}\` outside a declare_oxc_lint! block`,
      );
    }
  }

  return {
    updatedSource: updatedLines.join("\n"),
    updatedRules,
    skippedNurseryRules,
  };
}

function rewriteNextRuleVersions({ root, releaseVersion, dryRun = false }) {
  validateReleaseVersion(releaseVersion);

  const repoRoot = path.resolve(root);
  const rulesRoot = path.join(repoRoot, DEFAULT_RULES_ROOT);
  if (!fs.existsSync(rulesRoot)) {
    throw new Error(`rules root does not exist: ${rulesRoot}`);
  }

  const report = { updatedRules: [], skippedNurseryRules: [] };
  const pendingWrites = [];

  for (const filePath of collectRuleFiles(rulesRoot, repoRoot)) {
    const source = fs.readFileSync(filePath, "utf8");
    if (!NEXT_VERSION_REGEX.test(source)) {
      continue;
    }

    const fileReport = analyzeRuleFile(source, filePath, releaseVersion, repoRoot);
    report.updatedRules.push(...fileReport.updatedRules);
    report.skippedNurseryRules.push(...fileReport.skippedNurseryRules);

    if (!dryRun && fileReport.updatedRules.length > 0) {
      pendingWrites.push({ filePath, updatedSource: fileReport.updatedSource });
    }
  }

  if (!dryRun) {
    for (const pendingWrite of pendingWrites) {
      fs.writeFileSync(pendingWrite.filePath, pendingWrite.updatedSource);
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

function parseArgs(argv) {
  const options = {
    root: process.cwd(),
    releaseVersion: "",
    dryRun: false,
  };

  for (let index = 0; index < argv.length; index++) {
    const arg = argv[index];
    if ((arg === "--release-version" || arg === "-r") && argv[index + 1]) {
      options.releaseVersion = argv[index + 1];
      index += 1;
    } else if ((arg === "--root" || arg === "-C") && argv[index + 1]) {
      options.root = argv[index + 1];
      index += 1;
    } else if (arg === "--dry-run" || arg === "-n") {
      options.dryRun = true;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  return options;
}

function printHelp() {
  console.log(`Usage:
  node .github/scripts/update-rule-versions.js --release-version <x.y.z> [--root <path>] [--dry-run]

Options:
  --release-version, -r  Version to replace \`version = "next"\` with
  --root, -C             Repository root (defaults to current working directory)
  --dry-run, -n          Print the changes without writing files
  --help, -h             Show this help
`);
}

function main(argv = process.argv.slice(2)) {
  const options = parseArgs(argv);
  if (options.help) {
    printHelp();
    return;
  }
  if (!options.releaseVersion) {
    throw new Error("missing required `--release-version <x.y.z>`");
  }

  const report = rewriteNextRuleVersions(options);
  printReport(report, options.dryRun);
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
  rewriteNextRuleVersions,
  validateReleaseVersion,
};
