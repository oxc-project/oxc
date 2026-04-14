import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");
const VP_ENV = { VP_VERSION: "1" };

describe("vite_plus", () => {
  it("basic: reads fmt field from vite.config.ts", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("error: explicit --config vite.config.ts without fmt field", async () => {
    const cwd = join(fixturesDir, "no_fmt_field");
    const snapshot = await runAndSnapshot(
      cwd,
      [["--check", "--config", "vite.config.ts", "test.ts"]],
      VP_ENV,
    );
    expect(snapshot).toMatchSnapshot();
  });

  it("error: explicit --config vite.config.ts that fails to load", async () => {
    const cwd = join(fixturesDir, "error_load_failure", "child");
    const snapshot = await runAndSnapshot(
      cwd,
      [["--check", "--config", "vite.config.ts", "test.ts"]],
      VP_ENV,
    );
    expect(snapshot).toMatchSnapshot();
  });

  it("error: auto-discovered vite.config.ts that fails to load", async () => {
    const cwd = join(fixturesDir, "error_load_failure", "child");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("skip: auto-discovered vite.config.ts without fmt field uses defaults", async () => {
    const cwd = join(fixturesDir, "no_fmt_field");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("skip: parent oxfmtrc is not found when vite.config.ts without fmt is skipped", async () => {
    // child/ has vite.config.ts without .fmt → skipped
    // parent has .oxfmtrc.json with semi: false, but it's not a candidate in vite-plus mode
    // So defaults (semi: true) apply and `const a = 1;` is OK
    const cwd = join(fixturesDir, "skip_finds_parent", "child");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("skip: auto-discovered vite.config.ts with function export uses defaults", async () => {
    const cwd = join(fixturesDir, "skip_fn_export");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });

  it("priority: oxfmt.config.ts is ignored, vite.config.ts is used", async () => {
    // `oxfmt.config.ts` has `semi: false`, `vite.config.ts` has `semi: true`
    // In vite-plus mode, only vite.config.ts is a candidate
    // So `const a = 1;` (with semicolon) matches and is OK
    const cwd = join(fixturesDir, "priority");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]], VP_ENV);
    expect(snapshot).toMatchSnapshot();
  });
});
