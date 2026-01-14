import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { writeFile, rm } from "node:fs/promises";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

// NOTE: These tests modify `.gitignore` files in the fixtures directory.
// If we commit `.gitignore`, required test fixtures are also ignored by git!

describe("gitignore", () => {
  const rootGitignore = join(fixturesDir, ".gitignore");

  it("should respect root .gitignore", async () => {
    try {
      await writeFile(rootGitignore, ["ignored-by-root.js", "subdir/"].join("\n"));

      const snapshot = await runAndSnapshot(fixturesDir, [["--check"]]);
      expect(snapshot).toMatchSnapshot();
    } finally {
      await rm(rootGitignore, { force: true });
    }
  });

  it("should respect .gitignore in subdirectory", async () => {
    const subdirGitignore = join(fixturesDir, "subdir", ".gitignore");
    try {
      await writeFile(rootGitignore, ["ignored-by-root.js"].join("\n"));
      await writeFile(subdirGitignore, "ignored-by-subdir.js\n");

      const snapshot = await runAndSnapshot(fixturesDir, [["--check"]]);
      expect(snapshot).toMatchSnapshot();
    } finally {
      await rm(rootGitignore, { force: true });
      await rm(subdirGitignore, { force: true });
    }
  });
});
