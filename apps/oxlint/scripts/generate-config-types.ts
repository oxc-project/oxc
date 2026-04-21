// oxlint-disable no-console

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { compile } from "json-schema-to-typescript";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const oxlintDir = resolve(scriptDir, "..");
const repoRoot = resolve(oxlintDir, "..", "..");

const schemaPath = resolve(repoRoot, "npm/oxlint/configuration_schema.json");
const outputPath = resolve(oxlintDir, "src-js/package/config.generated.ts");

if (!existsSync(schemaPath)) {
  throw new Error(`Missing schema at ${schemaPath}. Run just linter-schema-json first.`);
}

const schema = JSON.parse(readFileSync(schemaPath, "utf8"));

// Sanitize and format JSON code blocks so they are indented properly.
function sanitizeJsonCodeBlocks(text: string): string {
  const marker = "```json";
  let result = text;
  let searchFrom = 0;

  while (true) {
    const start = result.indexOf(marker, searchFrom);
    if (start === -1) break;

    const contentStart = start + marker.length;
    const end = result.indexOf("```", contentStart);
    if (end === -1) break;

    const jsonStr = result.slice(contentStart, end).trim();
    let json: unknown;
    try {
      json = JSON.parse(jsonStr);
    } catch {
      searchFrom = end + 3;
      continue;
    }

    const formatted = "\n" + JSON.stringify(json, null, 2) + "\n";
    result = result.slice(0, contentStart) + formatted + result.slice(end);
    searchFrom = contentStart + formatted.length;
  }

  return result;
}

function sanitizeSchema(value: any): void {
  if (typeof value !== "object" || value === null) return;
  if (Array.isArray(value)) {
    value.forEach(sanitizeSchema);
    return;
  }
  for (const key of Object.keys(value)) {
    if (
      (key === "description" || key === "markdownDescription") &&
      typeof value[key] === "string"
    ) {
      value[key] = sanitizeJsonCodeBlocks(value[key]);
    } else {
      sanitizeSchema(value[key]);
    }
  }
}

sanitizeSchema(schema);

const bannerComment =
  "/*\n" +
  " * This file is generated from npm/oxlint/configuration_schema.json.\n" +
  " * Run `just linter-config-ts` to regenerate.\n" +
  " */";

const ts = await compile(schema, "OxlintConfig", { bannerComment });

writeFileSync(outputPath, ts);
console.log(`Wrote ${outputPath}`);
