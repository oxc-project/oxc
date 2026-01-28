import { join } from "node:path";
import { tmpdir } from "node:os";
import fs from "node:fs/promises";
import { describe, expect, it } from "vitest";
import { runCli } from "../utils";

describe("--migrate biome", () => {
  it("should create .oxfmtrc.json when no biome config exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.ignorePatterns).toEqual([]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should abort if .oxfmtrc.json already exists", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(join(tempDir, ".oxfmtrc.json"), "{}");

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(1);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate biome.json config to .oxfmtrc.json", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          formatter: {
            lineWidth: 120,
            indentStyle: "space",
            indentWidth: 4,
          },
          javascript: {
            formatter: {
              quoteStyle: "single",
              semicolons: "asNeeded",
            },
          },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.printWidth).toBe(120);
      expect(oxfmtrc.useTabs).toBe(false);
      expect(oxfmtrc.tabWidth).toBe(4);
      expect(oxfmtrc.singleQuote).toBe(true);
      expect(oxfmtrc.semi).toBe(false);
      expect(oxfmtrc.ignorePatterns).toEqual([]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate biome.jsonc config with comments", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.jsonc"),
        `{
          // This is a comment
          "formatter": {
            "lineWidth": 100,
            /* Multi-line
               comment */
            "indentStyle": "tab"
          }
        }`,
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.printWidth).toBe(100);
      expect(oxfmtrc.useTabs).toBe(true);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should prefer biome.json over biome.jsonc when both exist", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          formatter: { lineWidth: 80 },
        }),
      );
      await fs.writeFile(
        join(tempDir, "biome.jsonc"),
        JSON.stringify({
          formatter: { lineWidth: 120 },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.printWidth).toBe(80);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should use Biome defaults when options are not specified", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(join(tempDir, "biome.json"), JSON.stringify({}));

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.printWidth).toBe(80);
      expect(oxfmtrc.useTabs).toBe(true);
      expect(oxfmtrc.tabWidth).toBe(2);
      expect(oxfmtrc.singleQuote).toBe(false);
      expect(oxfmtrc.jsxSingleQuote).toBe(false);
      expect(oxfmtrc.quoteProps).toBe("as-needed");
      expect(oxfmtrc.trailingComma).toBe("all");
      expect(oxfmtrc.semi).toBe(true);
      expect(oxfmtrc.arrowParens).toBe("always");
      expect(oxfmtrc.bracketSameLine).toBe(false);
      expect(oxfmtrc.bracketSpacing).toBe(true);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should prefer javascript.formatter options over formatter options", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          formatter: {
            lineWidth: 80,
            indentWidth: 2,
          },
          javascript: {
            formatter: {
              lineWidth: 120,
              indentWidth: 4,
            },
          },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.printWidth).toBe(120);
      expect(oxfmtrc.tabWidth).toBe(4);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate arrowParentheses correctly", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          javascript: {
            formatter: {
              arrowParentheses: "asNeeded",
            },
          },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.arrowParens).toBe("avoid");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate attributePosition to singleAttributePerLine", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          formatter: {
            attributePosition: "multiline",
          },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.singleAttributePerLine).toBe(true);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate ignore patterns from files.includes", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          files: {
            includes: ["**", "!dist", "!node_modules", "!*.min.js"],
          },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.ignorePatterns).toEqual(["dist", "node_modules", "*.min.js"]);
      expect(Object.keys(oxfmtrc).at(-1)).toBe("ignorePatterns");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should not include force-ignore patterns (starting with !!) in ignorePatterns", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          files: {
            includes: ["**", "!dist", "!!build"],
          },
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.ignorePatterns).toEqual(["dist"]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should warn about overrides that cannot be migrated", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-biome-test"));

    try {
      await fs.writeFile(
        join(tempDir, "biome.json"),
        JSON.stringify({
          overrides: [
            {
              includes: ["generated/**"],
              formatter: {
                lineWidth: 160,
              },
            },
          ],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "biome"]);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("overrides");
      expect(result.stderr).toContain("cannot be migrated automatically");
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });
});
