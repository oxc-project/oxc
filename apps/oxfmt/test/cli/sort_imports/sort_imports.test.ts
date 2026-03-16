import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runCli } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("sort_imports", () => {
  it("should sort imports with customGroups", async () => {
    const cwd = join(fixturesDir, "custom_groups");
    const result = await runCli(cwd, ["--check", "input.ts"]);

    expect(result.exitCode).toBe(0);
  });

  it("should sort imports with customGroups using selector and modifiers", async () => {
    const cwd = join(fixturesDir, "custom_groups_selector_modifiers");
    const result = await runCli(cwd, ["--check", "input.ts"]);

    expect(result.exitCode).toBe(0);
  });
});
