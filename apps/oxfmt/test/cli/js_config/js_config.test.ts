import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("js_config", () => {
  it("basic", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("error: no default export", async () => {
    const cwd = join(fixturesDir, "error_no_default");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("error: default export is not an object", async () => {
    const cwd = join(fixturesDir, "error_not_object");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("priority: JSON config takes precedence over JS config", async () => {
    // `.oxfmtrc.json` has `semi: false`, `oxfmt.config.ts` has `semi: true`
    // JSON should win, so `const a = 1;` (with semicolon) should be flagged
    const cwd = join(fixturesDir, "priority");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });
});
