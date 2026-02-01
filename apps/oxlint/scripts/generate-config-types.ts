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

const bannerComment =
  "/*\n" +
  " * This file is generated from npm/oxlint/configuration_schema.json.\n" +
  " * Run `just linter-config-ts` to regenerate.\n" +
  " */";

const ts = await compile(schema, "OxlintConfig", { bannerComment });

writeFileSync(outputPath, ts);
console.log(`Wrote ${outputPath}`);
