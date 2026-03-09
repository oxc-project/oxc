import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("config_field", () => {
  it("should handle --config-field flag", async () => {
    const snapshot = await runAndSnapshot(fixturesDir, [
      // JSON: `fmt` field has `semi: false`
      ["--check", "-c", "config.json", "--config-field", "fmt", "input.js"],
      // JSON: `other` field has `semi: true` (default behavior)
      ["--check", "-c", "config.json", "--config-field", "other", "input.js"],
      // TS config
      ["--check", "-c", "config.ts", "--config-field", "fmt", "input.js"],
      // ERR: Missing field
      ["--check", "-c", "config.json", "--config-field", "nonexistent", "input.js"],
      // ERR: Without --config
      ["--check", "--config-field", "fmt", "input.js"],
    ]);
    expect(snapshot).toMatchSnapshot();
  });
});
