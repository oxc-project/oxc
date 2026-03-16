// oxlint-disable no-console

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { compile } from "json-schema-to-typescript";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const oxfmtDir = resolve(scriptDir, "..");
const repoRoot = resolve(oxfmtDir, "..", "..");

const schemaPath = resolve(repoRoot, "npm/oxfmt/configuration_schema.json");
const outputPath = resolve(oxfmtDir, "src-js/config.generated.ts");

if (!existsSync(schemaPath)) {
  throw new Error(`Missing schema at ${schemaPath}. Run \`just formatter-schema-json\` first.`);
}

const schema = JSON.parse(readFileSync(schemaPath, "utf8"));

const bannerComment =
  "/*\n" +
  " * This file is generated from npm/oxfmt/configuration_schema.json.\n" +
  " * Run `just formatter-config-ts` to regenerate.\n" +
  " */";

const ts = await compile(schema, "OxfmtConfig", { bannerComment });

writeFileSync(outputPath, ts);
console.log(`Wrote ${outputPath}`);
