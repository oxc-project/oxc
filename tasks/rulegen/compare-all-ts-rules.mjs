#!/usr/bin/env node

/**
 * compare-all-ts-rules.mjs
 *
 * Runs compare-tests.mjs on all TypeScript rules and collects results.
 */

import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = path.resolve(__dirname, "../..");

// Get all TypeScript rule files
const tsRulesDir = path.join(PROJECT_ROOT, "crates/oxc_linter/src/rules/typescript");
const ruleFiles = fs.readdirSync(tsRulesDir).filter((f) => f.endsWith(".rs"));

// Convert filename to kebab-case rule name
function filenameToRuleName(filename) {
  return filename.replace(/\.rs$/, "").replace(/_/g, "-");
}

const results = [];

console.error(`Found ${ruleFiles.length} TypeScript rules. Running comparisons...\n`);

for (const [index, file] of ruleFiles.entries()) {
  const ruleName = filenameToRuleName(file);
  console.error(`[${index + 1}/${ruleFiles.length}] Checking ${ruleName}...`);

  try {
    const output = execSync(
      `node ${path.join(__dirname, "compare-tests.mjs")} ${ruleName} typescript --json`,
      {
        encoding: "utf8",
        maxBuffer: 10 * 1024 * 1024,
        stdio: ["pipe", "pipe", "pipe"],
      },
    );

    const result = JSON.parse(output);
    const totalMissing = result.missingValid.length + result.missingInvalid.length;
    const totalOxlint = result.oxlint.valid + result.oxlint.invalid;

    // Only include rules that have some implementation (non-zero oxlint tests)
    if (totalOxlint > 0) {
      results.push({
        rule: ruleName,
        missingTotal: totalMissing,
        missingValid: result.missingValid.length,
        missingInvalid: result.missingInvalid.length,
        upstreamTotal: result.upstream.valid + result.upstream.invalid,
        oxlintTotal: totalOxlint,
        coverage:
          result.upstream.valid + result.upstream.invalid > 0
            ? ((totalOxlint / (result.upstream.valid + result.upstream.invalid)) * 100).toFixed(1)
            : "N/A",
      });
    }
  } catch (err) {
    // Skip rules that fail (likely type-aware or have issues)
    console.error(`  ⚠️  Skipped (${err.message.split("\n")[0]})`);
  }
}

// Sort by most missing tests
results.sort((a, b) => b.missingTotal - a.missingTotal);

// Output results
console.log("\n=== TypeScript Rules Missing Test Cases ===\n");
console.log("Rule Name                                     Missing  Oxlint  Upstream  Coverage");
console.log("─".repeat(85));

for (const r of results) {
  const rulePadded = r.rule.padEnd(45);
  const missingPadded = String(r.missingTotal).padStart(7);
  const oxlintPadded = String(r.oxlintTotal).padStart(7);
  const upstreamPadded = String(r.upstreamTotal).padStart(9);
  const coverage = String(r.coverage).padStart(7) + "%";
  console.log(`${rulePadded}${missingPadded}${oxlintPadded}${upstreamPadded}  ${coverage}`);
}

console.log("\n");
console.log(`Total rules analyzed: ${results.length}`);
console.log(`Total missing test cases: ${results.reduce((sum, r) => sum + r.missingTotal, 0)}`);
