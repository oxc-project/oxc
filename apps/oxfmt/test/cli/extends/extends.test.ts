import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("extends", () => {
  // .oxfmtrc.json:
  //   extends: ["./base.json"]   (base: tabWidth=4, semi=false)
  //   semi: true                 (child overrides semi)
  //
  // Expected: tabWidth=4 (inherited from base), semi=true (child wins)
  it("basic extends - child overrides parent", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.js"]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json:
  //   extends: ["./a.json", "./b.json"]
  //   a.json: tabWidth=4, semi=false
  //   b.json: tabWidth=8, singleQuote=true
  //   child: semi=true
  //
  // Expected: tabWidth=8 (b wins over a), singleQuote=true (from b), semi=true (child wins)
  it("multiple extends - later extends take precedence", async () => {
    const cwd = join(fixturesDir, "multiple");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.js"]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json extends parent.json which extends grandparent.json
  //   grandparent: tabWidth=4
  //   parent: semi=false
  //   child: singleQuote=true
  //
  // Expected: tabWidth=4 (grandparent), semi=false (parent), singleQuote=true (child)
  it("nested extends - A extends B extends C", async () => {
    const cwd = join(fixturesDir, "nested");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.js"]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json extends a.json which extends b.json which extends a.json (cycle)
  //
  // Expected: error about circular extends
  it("circular extends - detected and reported", async () => {
    const cwd = join(fixturesDir, "circular");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.js"]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json:
  //   extends: ["./base.json"]
  //   base.json: tabWidth=4, overrides: [{ files: ["*.test.js"], options: { semi: false } }]
  //   child: overrides: [{ files: ["*.test.js"], options: { singleQuote: true } }]
  //
  // Expected for app.js: tabWidth=4 (base), no overrides applied
  // Expected for app.test.js: tabWidth=4 (base), semi=false (base override), singleQuote=true (child override)
  it("extends with overrides - both parent and child overrides applied", async () => {
    const cwd = join(fixturesDir, "with_overrides");
    const snapshot = await runAndSnapshot(cwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });
});
