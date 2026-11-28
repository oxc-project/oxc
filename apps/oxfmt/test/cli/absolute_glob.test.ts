import { mkdtemp, mkdir, rm, writeFile, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execa } from "execa";
import { describe, it } from "vitest";
import { expect } from "vitest";
import { CLI_PATH } from "./utils";

// Absolute glob patterns cannot be expressed in the static `options.json` fixtures
// (their paths only exist at runtime), so they get this dedicated test.
// <https://github.com/oxc-project/oxc/issues/30442>
describe("oxfmt CLI absolute glob", () => {
  it("formats files matched by an absolute glob outside the cwd", async () => {
    const base = await mkdtemp(join(tmpdir(), "oxfmt-absolute-glob-"));
    const cwd = join(base, "cwd");
    const target = join(base, "target");
    await mkdir(cwd);
    await mkdir(target);
    const unformatted = '{\n  "b": 1,\n     "a": 2\n}\n';
    await writeFile(join(target, "a.json"), unformatted);
    await writeFile(join(target, "b.json"), unformatted);
    await writeFile(join(target, "c.js"), "");

    try {
      const { stdout, stderr, exitCode } = await execa(
        "node",
        [CLI_PATH, join(target, "*.json"), "--threads=1"],
        { cwd, reject: false, timeout: 10_000 },
      );

      expect(stderr).toContain("No config found, using defaults.");
      expect(stdout).toContain("on 2 files");
      expect(exitCode).toBe(0);

      const formatted = await readFile(join(target, "a.json"), "utf8");
      expect(formatted).toBe('{\n  "b": 1,\n  "a": 2\n}\n');
      // Non-matching extension is untouched
      expect(await readFile(join(target, "c.js"), "utf8")).toBe("");
    } finally {
      await rm(base, { recursive: true, force: true });
    }
  });

  it("matches files when the cwd is the glob's directory", async () => {
    const target = await mkdtemp(join(tmpdir(), "oxfmt-absolute-glob-"));
    await writeFile(join(target, "a.json"), '{\n  "b": 1,\n     "a": 2\n}\n');

    try {
      const { stdout, exitCode } = await execa(
        "node",
        [CLI_PATH, join(target, "*.json"), "--check", "--threads=1"],
        { cwd: target, reject: false, timeout: 10_000 },
      );

      expect(stdout).toContain("a.json");
      expect(stdout).toContain("Format issues found in above 1 files");
      expect(exitCode).toBe(1);
    } finally {
      await rm(target, { recursive: true, force: true });
    }
  });

  it("reports no files found when an absolute glob matches nothing", async () => {
    const base = await mkdtemp(join(tmpdir(), "oxfmt-absolute-glob-"));
    const cwd = join(base, "cwd");
    await mkdir(cwd);

    try {
      const { stderr, exitCode } = await execa(
        "node",
        [CLI_PATH, join(base, "none", "*.json"), "--threads=1"],
        { cwd, reject: false, timeout: 10_000 },
      );

      expect(stderr).toContain("Expected at least one target file.");
      expect(exitCode).toBe(2);
    } finally {
      await rm(base, { recursive: true, force: true });
    }
  });
});
