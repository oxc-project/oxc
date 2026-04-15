import { describe, expect, it } from "vitest";
import { join } from "node:path";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { runCli } from "../utils";

// TODO: Rewrite this tests to use snapshots.

const fixturesDir = join(import.meta.dirname, "fixtures");
const fixturesIgnoreDir = join(fixturesDir, "with-ignore-patterns");

describe("nested_config", () => {
  it("should use nearest config for each file", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(fixturesDir, tempDir, { recursive: true });

      await runCli(tempDir, ["."]);

      // root.js should use root config (tabWidth: 4)
      const rootContent = await fs.readFile(join(tempDir, "root.js"), "utf8");
      expect(rootContent).toContain("    if"); // 4-space indent

      // sub/nested.js should use sub config (tabWidth: 2)
      const nestedContent = await fs.readFile(join(tempDir, "sub", "nested.js"), "utf8");
      expect(nestedContent).toContain("  if"); // 2-space indent
      expect(nestedContent).not.toContain("    if"); // NOT 4-space
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should ignore nested config when --config is specified", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(fixturesDir, tempDir, { recursive: true });

      await runCli(tempDir, ["--config", ".oxfmtrc.json", "."]);

      // Both files should use the explicit config (tabWidth: 4)
      const rootContent = await fs.readFile(join(tempDir, "root.js"), "utf8");
      expect(rootContent).toContain("    if");

      const nestedContent = await fs.readFile(join(tempDir, "sub", "nested.js"), "utf8");
      expect(nestedContent).toContain("    if"); // nested config ignored
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should handle multiple sibling scopes", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(fixturesDir, tempDir, { recursive: true });

      await runCli(tempDir, ["."]);

      // sibling-a: semi: false
      const aContent = await fs.readFile(join(tempDir, "sibling-a", "a.js"), "utf8");
      expect(aContent).not.toContain(";");

      // sibling-b: singleQuote: true
      const bContent = await fs.readFile(join(tempDir, "sibling-b", "b.js"), "utf8");
      expect(bContent).toContain("'hello'");
      expect(bContent).not.toContain('"hello"');
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should handle deeply nested configs (3 levels)", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(fixturesDir, tempDir, { recursive: true });

      await runCli(tempDir, ["."]);

      // root: tabWidth 4
      const rootContent = await fs.readFile(join(tempDir, "root.js"), "utf8");
      expect(rootContent).toContain("    if");

      // sub: tabWidth 2
      const subContent = await fs.readFile(join(tempDir, "sub", "nested.js"), "utf8");
      expect(subContent).toContain("  if");
      expect(subContent).not.toContain("    if");

      // sub/deep: tabWidth 8
      const deepContent = await fs.readFile(join(tempDir, "sub", "deep", "deep.js"), "utf8");
      expect(deepContent).toContain("        if"); // 8-space indent
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should abort on child config parse error", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(fixturesDir, tempDir, { recursive: true });

      // Write invalid JSON to sub config
      await fs.writeFile(join(tempDir, "sub", ".oxfmtrc.json"), "{ invalid json }");

      const result = await runCli(tempDir, ["--check", "."]);

      // Should exit with error
      expect(result.exitCode).not.toBe(0);
      expect(result.stderr).toContain("Failed to parse config");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should respect child scope ignorePatterns", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(fixturesIgnoreDir, tempDir, { recursive: true });

      const beforeIgnored = await fs.readFile(join(tempDir, "sub", "ignored.js"), "utf8");

      await runCli(tempDir, ["."]);

      // root.js should be formatted (root config, tabWidth: 4)
      const rootContent = await fs.readFile(join(tempDir, "root.js"), "utf8");
      expect(rootContent).toContain("    if");

      // sub/formatted.js should be formatted (sub config, tabWidth: 2)
      const formattedContent = await fs.readFile(join(tempDir, "sub", "formatted.js"), "utf8");
      expect(formattedContent).toContain("  if");
      expect(formattedContent).not.toContain("    if");

      // sub/ignored.js should NOT be formatted (ignored by sub config)
      const ignoredContent = await fs.readFile(join(tempDir, "sub", "ignored.js"), "utf8");
      expect(ignoredContent).toBe(beforeIgnored);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should apply glob patterns across nested scopes", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(fixturesDir, tempDir, { recursive: true });

      // Only format .js files via glob
      const result = await runCli(tempDir, ["--check", "**/*.js"]);

      // Should find files in both root and nested scopes
      expect(result.exitCode).not.toBeUndefined();
      // root.js and nested scope files should all be checked
      expect(result.stdout).toContain("root.js");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should use defaults at root when no root config exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(join(fixturesDir, "no-root-config"), tempDir, { recursive: true });

      await runCli(tempDir, ["."]);

      // root.js: no config, uses defaults (tabWidth: 2)
      const rootContent = await fs.readFile(join(tempDir, "root.js"), "utf8");
      expect(rootContent).toContain("  if"); // 2-space (default)
      expect(rootContent).not.toContain("        if"); // NOT 8-space

      // sub/test.js: sub config (tabWidth: 8)
      const subContent = await fs.readFile(join(tempDir, "sub", "test.js"), "utf8");
      expect(subContent).toContain("        if"); // 8-space indent
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should respect both .prettierignore and nested ignorePatterns", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(join(fixturesDir, "prettierignore-with-nested"), tempDir, { recursive: true });

      const beforeRootIgnored = await fs.readFile(join(tempDir, "root-ignored.js"), "utf8");
      const beforeSubIgnored = await fs.readFile(join(tempDir, "sub", "sub-ignored.js"), "utf8");

      await runCli(tempDir, ["."]);

      // root.js: formatted (root config, tabWidth: 4)
      const rootContent = await fs.readFile(join(tempDir, "root.js"), "utf8");
      expect(rootContent).toContain("    if");

      // root-ignored.js: NOT formatted (.prettierignore)
      const rootIgnored = await fs.readFile(join(tempDir, "root-ignored.js"), "utf8");
      expect(rootIgnored).toBe(beforeRootIgnored);

      // sub/formatted.js: formatted (sub config, tabWidth: 2)
      const subFormatted = await fs.readFile(join(tempDir, "sub", "formatted.js"), "utf8");
      expect(subFormatted).toContain("  if");

      // sub/sub-ignored.js: NOT formatted (nested ignorePatterns)
      const subIgnored = await fs.readFile(join(tempDir, "sub", "sub-ignored.js"), "utf8");
      expect(subIgnored).toBe(beforeSubIgnored);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should handle empty child config (use defaults)", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-nested-"));
    try {
      await fs.cp(join(fixturesDir, "empty-child-config"), tempDir, { recursive: true });

      await runCli(tempDir, ["."]);

      // root.js: root config (tabWidth: 4)
      const rootContent = await fs.readFile(join(tempDir, "root.js"), "utf8");
      expect(rootContent).toContain("    if");

      // sub/test.js: empty config = defaults (tabWidth: 2)
      const subContent = await fs.readFile(join(tempDir, "sub", "test.js"), "utf8");
      expect(subContent).toContain("  if");
      expect(subContent).not.toContain("    if");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });
});
