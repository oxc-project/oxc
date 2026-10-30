import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { transformSync, type TransformOptions, type TransformResult } from "../index";

const fixturesDir = join(import.meta.dirname, "fixtures");
const fixtures = readdirSync(fixturesDir).sort();

function render(result: TransformResult): string {
  const errors = result.errors.map((error) => `${error.severity}: ${error.message}`).join("\n");
  return `## Code\n\n${result.code}\n## Errors\n\n${errors || "(none)"}\n\n## Fatal\n\n${result.fatal}\n`;
}

describe("fixtures", () => {
  it.each(fixtures)("%s", async (name) => {
    const dir = join(fixturesDir, name);
    const input = readdirSync(dir).find((file) => file.startsWith("input."));
    if (input === undefined) throw new Error(`${name} has no input file`);
    const optionsPath = join(dir, "options.json");
    const options: TransformOptions = existsSync(optionsPath)
      ? JSON.parse(readFileSync(optionsPath, "utf8"))
      : {};
    const result = transformSync(input, readFileSync(join(dir, input), "utf8"), options);
    await expect(render(result)).toMatchFileSnapshot(join(dir, "output.snap"));
  });
});
