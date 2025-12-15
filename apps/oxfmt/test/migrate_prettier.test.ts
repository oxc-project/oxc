import { join } from "node:path";
import { tmpdir } from "node:os";
import fs from "node:fs/promises";
import { describe, expect, it } from "vitest";
import { runCli } from "./utils";

describe("--migrate prettier", () => {
  it("should create .oxfmtrc.json when no prettier config exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.ignorePatterns).toEqual([]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should abort if .oxfmtrc.json already exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create existing config file
      await fs.writeFile(join(tempDir, ".oxfmtrc.json"), "{}");

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(1);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate prettier config to .oxfmtrc.json", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          semi: false,
          singleQuote: true,
          tabWidth: 4,
          printWidth: 120,
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.semi).toBe(false);
      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.tabWidth).toBe(4);
      expect(oxfmtrc.printWidth).toBe(120);
      expect(oxfmtrc.ignorePatterns).toEqual([]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should default printWidth to 80 when not set in prettier config", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config without printWidth
      await fs.writeFile(join(tempDir, ".prettierrc"), JSON.stringify({ semi: false }));

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Prettier default is 80, Oxfmt default is 100
      // So we explicitly set 80 to match Prettier behavior
      expect(oxfmtrc.printWidth).toBe(80);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate .prettierignore to ignorePatterns", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config
      await fs.writeFile(join(tempDir, ".prettierrc"), JSON.stringify({ semi: true }));
      // Create .prettierignore
      await fs.writeFile(
        join(tempDir, ".prettierignore"),
        `# Comment line
dist
node_modules
*.min.js
`,
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.ignorePatterns).toEqual(["dist", "node_modules", "*.min.js"]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });
});
