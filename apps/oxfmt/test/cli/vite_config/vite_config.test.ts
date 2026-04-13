import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("vite_config", () => {
  it("ignored: vite.config.ts is ignored without VP_VERSION", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("ignored: parent vite.config.ts is also ignored without VP_VERSION", async () => {
    const cwd = join(fixturesDir, "basic", "child");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });

  it("explicit: --config vite.config.ts fails because it is treated as a plain .ts file", async () => {
    // Without VP_VERSION, vite.config.ts is just another .ts config file
    // `import { defineConfig } from "vite"` fails because vite is not installed
    const cwd = join(fixturesDir, "monorepo", "packages", "app");
    const snapshot = await runAndSnapshot(cwd, [
      ["--check", "--config", "vite.config.ts", "test.ts"],
    ]);
    expect(snapshot).toMatchSnapshot();
  });

  it("ignored: monorepo child vite.config.ts is ignored, root oxfmtrc is used", async () => {
    // root/.oxfmtrc.json has semi: false, child/vite.config.ts exists but requires vite
    // Without VP_VERSION, vite.config.ts is skipped (no load error) and root config applies
    // So `const a = 1;` (with semicolon) should be flagged as mismatch
    const cwd = join(fixturesDir, "monorepo", "packages", "app");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.ts"]]);
    expect(snapshot).toMatchSnapshot();
  });
});
