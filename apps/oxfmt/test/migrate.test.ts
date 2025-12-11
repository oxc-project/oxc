import { describe, expect, it, beforeEach, afterEach } from "vitest";
import { join } from "node:path";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { runAndSnapshot } from "./utils";

describe("migrate", () => {
  let tempDir: string;

  beforeEach(async () => {
    tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-migrate-test-"));
  });

  afterEach(async () => {
    await fs.rm(tempDir, { recursive: true, force: true });
  });

  it("should migrate basic prettier config", async () => {
    // Create .prettierrc
    await fs.writeFile(
      join(tempDir, ".prettierrc"),
      JSON.stringify({
        semi: false,
        singleQuote: true,
        tabWidth: 4,
        printWidth: 100,
      }),
    );

    const snapshot = await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);
    expect(snapshot).toMatchSnapshot();

    // Verify .oxfmtrc.jsonc was created
    const configContent = await fs.readFile(join(tempDir, ".oxfmtrc.jsonc"), "utf8");
    expect(configContent).toContain('"semi": false');
    expect(configContent).toContain('"singleQuote": true');
    expect(configContent).toContain('"tabWidth": 4');
    expect(configContent).toContain('"printWidth": 100');
  });

  it("should migrate prettier config with .prettierignore", async () => {
    // Create .prettierrc
    await fs.writeFile(
      join(tempDir, ".prettierrc"),
      JSON.stringify({
        semi: false,
      }),
    );

    // Create .prettierignore
    await fs.writeFile(
      join(tempDir, ".prettierignore"),
      `# Comment line
dist/
build/

node_modules/
*.min.js
`,
    );

    const snapshot = await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);
    expect(snapshot).toMatchSnapshot();

    // Verify .oxfmtrc.jsonc contains ignore patterns
    const configContent = await fs.readFile(join(tempDir, ".oxfmtrc.jsonc"), "utf8");
    expect(configContent).toContain('"ignorePatterns"');
    expect(configContent).toContain('"dist/"');
    expect(configContent).toContain('"build/"');
    expect(configContent).toContain('"node_modules/"');
    expect(configContent).toContain('"*.min.js"');
    expect(configContent).not.toContain("# Comment");
  });

  it("should migrate all supported prettier options", async () => {
    await fs.writeFile(
      join(tempDir, ".prettierrc"),
      JSON.stringify({
        useTabs: true,
        tabWidth: 2,
        printWidth: 120,
        semi: true,
        singleQuote: false,
        jsxSingleQuote: true,
        trailingComma: "all",
        bracketSpacing: false,
        bracketSameLine: true,
        arrowParens: "always",
        endOfLine: "lf",
        quoteProps: "consistent",
        singleAttributePerLine: true,
        embeddedLanguageFormatting: "off",
      }),
    );

    await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);

    const configContent = await fs.readFile(join(tempDir, ".oxfmtrc.jsonc"), "utf8");
    expect(configContent).toContain('"useTabs": true');
    expect(configContent).toContain('"tabWidth": 2');
    expect(configContent).toContain('"printWidth": 120');
    expect(configContent).toContain('"semi": true');
    expect(configContent).toContain('"singleQuote": false');
    expect(configContent).toContain('"jsxSingleQuote": true');
    expect(configContent).toContain('"trailingComma": "all"');
    expect(configContent).toContain('"bracketSpacing": false');
    expect(configContent).toContain('"bracketSameLine": true');
    expect(configContent).toContain('"arrowParens": "always"');
    expect(configContent).toContain('"endOfLine": "lf"');
    expect(configContent).toContain('"quoteProps": "consistent"');
    expect(configContent).toContain('"singleAttributePerLine": true');
    expect(configContent).toContain('"embeddedLanguageFormatting": "off"');
  });

  it("should warn about unsupported options", async () => {
    await fs.writeFile(
      join(tempDir, ".prettierrc"),
      JSON.stringify({
        semi: false,
        experimentalTernaries: true,
        proseWrap: "always",
        htmlWhitespaceSensitivity: "css",
      }),
    );

    const snapshot = await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);
    expect(snapshot).toMatchSnapshot();

    // Check that stderr contains warnings
    expect(snapshot).toContain("experimentalTernaries");
    expect(snapshot).toContain("proseWrap");
    expect(snapshot).toContain("htmlWhitespaceSensitivity");
  });

  it("should error when .oxfmtrc.jsonc already exists", async () => {
    // Create .prettierrc
    await fs.writeFile(join(tempDir, ".prettierrc"), JSON.stringify({ semi: false }));

    // Create .oxfmtrc.jsonc
    await fs.writeFile(join(tempDir, ".oxfmtrc.jsonc"), "{}");

    const snapshot = await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);
    expect(snapshot).toMatchSnapshot();

    // Check that it's an error
    expect(snapshot).toContain("Configuration file already exists");
    expect(snapshot).toContain("exit code: 1");
  });

  it("should error when no prettier config found", async () => {
    const snapshot = await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);
    expect(snapshot).toMatchSnapshot();

    expect(snapshot).toContain("No Prettier configuration found");
    expect(snapshot).toContain("exit code: 1");
  });

  it("should error for unknown migration source", async () => {
    const snapshot = await runAndSnapshot(tempDir, [["--migrate", "eslint"]]);
    expect(snapshot).toMatchSnapshot();

    expect(snapshot).toContain("Unknown migration source");
    expect(snapshot).toContain("exit code: 1");
  });

  it("should handle package.json prettier config", async () => {
    // Create package.json with prettier config
    await fs.writeFile(
      join(tempDir, "package.json"),
      JSON.stringify({
        name: "test",
        prettier: {
          semi: false,
          tabWidth: 8,
        },
      }),
    );

    const snapshot = await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);
    expect(snapshot).toMatchSnapshot();

    const configContent = await fs.readFile(join(tempDir, ".oxfmtrc.jsonc"), "utf8");
    expect(configContent).toContain('"semi": false');
    expect(configContent).toContain('"tabWidth": 8');
  });

  it("should handle empty .prettierignore", async () => {
    await fs.writeFile(join(tempDir, ".prettierrc"), JSON.stringify({ semi: false }));

    // Create empty .prettierignore
    await fs.writeFile(join(tempDir, ".prettierignore"), "");

    await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);

    const configContent = await fs.readFile(join(tempDir, ".oxfmtrc.jsonc"), "utf8");
    expect(configContent).toContain('"ignorePatterns": []');
  });

  it("should handle .prettierignore with only comments", async () => {
    await fs.writeFile(join(tempDir, ".prettierrc"), JSON.stringify({ semi: false }));

    await fs.writeFile(
      join(tempDir, ".prettierignore"),
      `# Just comments
# No actual patterns
`,
    );

    await runAndSnapshot(tempDir, [["--migrate", "prettier"]]);

    const configContent = await fs.readFile(join(tempDir, ".oxfmtrc.jsonc"), "utf8");
    expect(configContent).toContain('"ignorePatterns": []');
  });
});
