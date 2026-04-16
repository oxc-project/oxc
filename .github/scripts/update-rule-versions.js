#!/usr/bin/env node
/* eslint-disable no-console */

const fs = require("node:fs");
const path = require("node:path");
const { parseArgs } = require("node:util");

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

function stripCommentsFromLine(line, inBlockComment = false) {
  let strippedLine = "";
  let inString = false;
  let stringDelimiter = "";
  let isEscaped = false;

  for (let index = 0; index < line.length; index++) {
    const char = line[index];
    const nextChar = line[index + 1];

    if (inBlockComment) {
      if (char === "*" && nextChar === "/") {
        inBlockComment = false;
        index += 1;
      }
      continue;
    }

    if (inString) {
      strippedLine += char;
      if (isEscaped) {
        isEscaped = false;
        continue;
      }
      if (char === "\\") {
        isEscaped = true;
        continue;
      }
      if (char === stringDelimiter) {
        inString = false;
      }
      continue;
    }

    if (char === '"' || char === "'") {
      inString = true;
      stringDelimiter = char;
      strippedLine += char;
      continue;
    }

    if (char === "/" && nextChar === "*") {
      inBlockComment = true;
      index += 1;
      continue;
    }

    if (char === "/" && nextChar === "/") {
      break;
    }

    strippedLine += char;
  }

  return { strippedLine, inBlockComment };
}

function stripCommentsFromLines(lines) {
  let inBlockComment = false;

  return lines.map((line) => {
    const result = stripCommentsFromLine(line, inBlockComment);
    inBlockComment = result.inBlockComment;
    return result.strippedLine;
  });
}

function normalizeMetadataLine(line) {
  const strippedLine = line.trim();
  return strippedLine || null;
}

function analyzeRuleFile(source, filePath, releaseVersion, repoRoot) {
  const relativeFile = normalizePath(path.relative(repoRoot, filePath));
  const lines = source.split("\n");
  const strippedLines = stripCommentsFromLines(lines);
  const updatedLines = [...lines];
  const coveredNextVersionLines = new Set();
  const updatedRules = [];
  const skippedNurseryRules = [];

  for (let startLine = 0; startLine < lines.length; startLine++) {
    if (!lines[startLine].includes(DECLARE_RULE_MACRO)) {
      continue;
    }

    let endLine = startLine + 1;
    while (endLine < lines.length && strippedLines[endLine].trim() !== ");") {
      endLine += 1;
    }

    if (endLine >= lines.length) {
      throw new Error(`${relativeFile}: unterminated declare_oxc_lint! block`);
    }

    const metadataEntries = [];
    for (let lineIndex = startLine + 1; lineIndex < endLine; lineIndex++) {
      const normalizedLine = normalizeMetadataLine(strippedLines[lineIndex]);
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

  for (const [lineIndex, strippedLine] of strippedLines.entries()) {
    if (NEXT_VERSION_REGEX.test(strippedLine) && !coveredNextVersionLines.has(lineIndex)) {
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
