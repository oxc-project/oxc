import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("vite_config", () => {
  it("basic: reads fmt field from vite.config.ts", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("missing fmt field in vite.config.ts falls back to default config", async () => {
    const cwd = join(fixturesDir, "missing_fmt_field");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("priority: oxfmt.config.ts takes precedence over vite.config.ts", async () => {
    // `oxfmt.config.ts` has `semi: false`, `vite.config.ts` has `semi: true`
    // oxfmt.config.ts should win, so `const a = 1;` (with semicolon) should be flagged
    const cwd = join(fixturesDir, "priority");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });
});
