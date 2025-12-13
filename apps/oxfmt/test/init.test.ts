import { join } from "node:path";
import { tmpdir } from "node:os";
import fs from "node:fs/promises";
import { describe, expect, it } from "vitest";
import { runCli } from "./utils";

describe("init", () => {
  it("should create .oxfmtrc.json", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-init-test-"));

    try {
      const result = await runCli(tempDir, ["--init"]);

      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain("Created `.oxfmtrc.json`.");

      const configPath = join(tempDir, ".oxfmtrc.json");
      const content = await fs.readFile(configPath, "utf8");
      const config = JSON.parse(content);

      expect(config.ignorePatterns).toEqual([]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should add $schema when node_modules/oxfmt exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-init-test-"));

    try {
      // Create fake node_modules/oxfmt/configuration_schema.json
      const schemaDir = join(tempDir, "node_modules", "oxfmt");
      await fs.mkdir(schemaDir, { recursive: true });
      await fs.writeFile(join(schemaDir, "configuration_schema.json"), "{}");

      const result = await runCli(tempDir, ["--init"]);

      expect(result.exitCode).toBe(0);

      const configPath = join(tempDir, ".oxfmtrc.json");
      const content = await fs.readFile(configPath, "utf8");
      const config = JSON.parse(content);

      expect(config.$schema).toBe("./node_modules/oxfmt/configuration_schema.json");
      expect(Object.keys(config)[0]).toBe("$schema"); // $schema should be first
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should abort if .oxfmtrc.json already exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-init-test-"));

    try {
      // Create existing config file
      await fs.writeFile(join(tempDir, ".oxfmtrc.json"), "{}");

      const result = await runCli(tempDir, ["--init"]);

      expect(result.exitCode).toBe(1);
      expect(result.stderr).toContain("Configuration file already exists.");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should abort if .oxfmtrc.jsonc already exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-init-test-"));

    try {
      // Create existing config file
      await fs.writeFile(join(tempDir, ".oxfmtrc.jsonc"), "{}");

      const result = await runCli(tempDir, ["--init"]);

      expect(result.exitCode).toBe(1);
      expect(result.stderr).toContain("Configuration file already exists.");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });
});
