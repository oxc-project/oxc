// oxlint-disable no-console

import { execFileSync, execSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

interface RuleMetadata {
  scope: string;
  value: string;
  recommended: boolean;
}

interface PluginPresets {
  recommended: {
    plugins: string[];
    rules: Record<string, "error">;
  };
}

function isRuleMetadata(rule: unknown): rule is RuleMetadata {
  return (
    typeof rule === "object"
    && rule !== null
    && "scope" in rule
    && typeof rule.scope === "string"
    && "value" in rule
    && typeof rule.value === "string"
    && "recommended" in rule
    && typeof rule.recommended === "boolean"
  );
}

export function generatePresets(metadata: unknown): Record<string, PluginPresets> {
  if (!Array.isArray(metadata) || !metadata.every(isRuleMetadata)) {
    throw new Error("Expected rules JSON with scope, value, and recommended fields.");
  }

  const rules = metadata.toSorted((a, b) => {
    if (a.scope !== b.scope) return a.scope < b.scope ? -1 : 1;
    if (a.value !== b.value) return a.value < b.value ? -1 : 1;
    return 0;
  });
  const presets: Record<string, PluginPresets> = {};

  for (const { scope, value, recommended } of rules) {
    if (!recommended) continue;

    const plugin = scope.replaceAll("_", "-");
    const { rules } = (presets[plugin] ??= {
      recommended: { plugins: [plugin], rules: {} },
    }).recommended;
    rules[plugin === "eslint" ? value : `${plugin}/${value}`] = "error";
  }

  return presets;
}

if (process.argv[1] === import.meta.filename) {
  const repoRoot = resolve(import.meta.dirname, "../../..");
  const outputPath = resolve(repoRoot, "apps/oxlint/src-js/package/presets.generated.ts");
  const rulesJson = execFileSync(
    "cargo",
    ["run", "-p", "oxlint", "--no-default-features", "--", "--rules", "-f=json"],
    { cwd: repoRoot, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] },
  );
  const presets = generatePresets(JSON.parse(rulesJson));

  writeFileSync(
    outputPath,
    "/*\n"
      + " * This file is generated from Oxlint's Rust rule metadata.\n"
      + " * Run `just linter-presets` to regenerate.\n"
      + " */\n\n"
      + 'import type { OxlintConfig } from "./config.ts";\n\n'
      + `export const presets = ${JSON.stringify(presets, null, 2)} satisfies Record<string, { recommended: OxlintConfig }>;\n`,
  );
  execSync(`pnpm exec oxfmt --write ${JSON.stringify(outputPath)}`, {
    cwd: repoRoot,
    stdio: "inherit",
  });
  console.log(`Wrote ${outputPath}`);
}
