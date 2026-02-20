import { join } from "node:path";
import { tmpdir } from "node:os";
import fs from "node:fs/promises";
import { describe, expect, it } from "vitest";
import { runCli } from "../utils";

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
      expect(Object.keys(oxfmtrc).at(-1)).toBe("ignorePatterns"); // `ignorePatterns` should be last
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should migrate prettier-plugin-tailwindcss options to sortTailwindcss", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with Tailwind plugin options
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-tailwindcss"],
          tailwindConfig: "./tailwind.config.js",
          tailwindFunctions: ["clsx", "cn"],
          tailwindAttributes: ["myClass"],
          tailwindPreserveWhitespace: true,
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Tailwind options should be migrated to sortTailwindcss
      expect(oxfmtrc.sortTailwindcss).toEqual({
        config: "./tailwind.config.js",
        functions: ["clsx", "cn"],
        attributes: ["myClass"],
        preserveWhitespace: true,
      });
      // Tailwind options should not be at root level
      expect(oxfmtrc.tailwindConfig).toBeUndefined();
      expect(oxfmtrc.tailwindFunctions).toBeUndefined();
      // plugins should not be copied
      expect(oxfmtrc.plugins).toBeUndefined();
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should enable sortTailwindcss when plugin is listed without options", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with Tailwind plugin but no options
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-tailwindcss"],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // sortTailwindcss should be enabled (empty object)
      expect(oxfmtrc.sortTailwindcss).toEqual({});
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should warn about regex values in tailwindFunctions and tailwindAttributes", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with regex values in Tailwind options
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-tailwindcss"],
          tailwindFunctions: ["clsx", "/^tw-/"],
          tailwindAttributes: ["className", "/^data-tw-/"],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      // Check warnings are printed for regex values
      expect(result.stderr).toContain('Do not support regex in "tailwindFunctions"');
      expect(result.stderr).toContain("/^tw-/");
      expect(result.stderr).toContain('Do not support regex in "tailwindAttributes"');
      expect(result.stderr).toContain("/^data-tw-/");

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Non-regex values should still be migrated
      expect(oxfmtrc.sortTailwindcss.functions).toEqual(["clsx", "/^tw-/"]);
      expect(oxfmtrc.sortTailwindcss.attributes).toEqual(["className", "/^data-tw-/"]);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should disable sortPackageJson by default", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config without package.json sorting plugin
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          semi: false,
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      // Prettier does not have package.json sorting by default
      expect(oxfmtrc.sortPackageJson).toBe(false);
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });

  it("should enable sortPackageJson when prettier-plugin-packagejson is used", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test"));

    try {
      // Create prettier config with package.json sorting plugin
      await fs.writeFile(
        join(tempDir, ".prettierrc"),
        JSON.stringify({
          plugins: ["prettier-plugin-packagejson"],
        }),
      );

      const result = await runCli(tempDir, ["--migrate", "prettier"]);
      expect(result.exitCode).toBe(0);

      const content = await fs.readFile(join(tempDir, ".oxfmtrc.json"), "utf8");
      const oxfmtrc = JSON.parse(content);

      expect(oxfmtrc.sortPackageJson).toBeTruthy();
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });
});
