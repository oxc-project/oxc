import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execa } from "execa";
import { describe, expect, it } from "vitest";
import { format, resolveConfig } from "../../dist/index.js";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");

async function withTempDir(files: Record<string, string>, fn: (dir: string) => Promise<void>) {
  const dir = await fs.mkdtemp(join(tmpdir(), "oxfmt-resolve-config-"));
  try {
    await Promise.all(
      Object.entries(files).map(async ([path, content]) => {
        await fs.mkdir(join(dir, path, ".."), { recursive: true });
        await fs.writeFile(join(dir, path), content);
      }),
    );
    await fn(dir);
  } finally {
    await fs.rm(dir, { recursive: true, force: true });
  }
}

describe("resolveConfig() API", () => {
  it("`resolveConfig()` function exists", () => {
    expect(typeof resolveConfig).toBe("function");
  });

  it("returns an empty config when no config is found", async () => {
    await withTempDir({}, async (dir) => {
      await expect(resolveConfig("src/a.ts", { cwd: dir })).resolves.toStrictEqual({
        config: {},
        ignored: false,
      });
    });
  });

  it("applies matching `overrides` and `.editorconfig`", async () => {
    await withTempDir(
      {
        ".oxfmtrc.json": JSON.stringify({
          semi: false,
          sortTailwindcss: { config: "./tailwind.config.js" },
          overrides: [{ files: ["**/*.test.ts"], options: { tabWidth: 4 } }],
        }),
        ".editorconfig": "root = true\n\n[*]\nindent_style = tab\n",
      },
      async (dir) => {
        await expect(resolveConfig("src/a.ts", { cwd: dir })).resolves.toStrictEqual({
          config: {
            semi: false,
            useTabs: true,
            sortTailwindcss: { config: join(dir, "tailwind.config.js") },
          },
          ignored: false,
        });
        await expect(resolveConfig("src/a.test.ts", { cwd: dir })).resolves.toStrictEqual({
          config: {
            semi: false,
            useTabs: true,
            tabWidth: 4,
            sortTailwindcss: { config: join(dir, "tailwind.config.js") },
          },
          ignored: false,
        });
      },
    );
  });

  it("discovers JS/TS config files", async () => {
    await withTempDir(
      { "oxfmt.config.ts": "export default { semi: false, singleQuote: true };\n" },
      async (dir) => {
        const { config } = await resolveConfig(join(dir, "src", "a.ts"), { cwd: dir });
        expect(config).toStrictEqual({ semi: false, singleQuote: true });
      },
    );
  });

  it("uses the nearest nested config", async () => {
    await withTempDir(
      {
        ".oxfmtrc.json": JSON.stringify({ semi: false }),
        "packages/a/.oxfmtrc.json": JSON.stringify({ singleQuote: true }),
      },
      async (dir) => {
        const root = await resolveConfig("src/a.ts", { cwd: dir });
        expect(root.config).toStrictEqual({ semi: false });
        const nested = await resolveConfig("packages/a/src/a.ts", { cwd: dir });
        expect(nested.config).toStrictEqual({ singleQuote: true });
      },
    );
  });

  it("reports files ignored by `ignorePatterns` and `.prettierignore`", async () => {
    await withTempDir(
      {
        ".oxfmtrc.json": JSON.stringify({ ignorePatterns: ["generated/**"] }),
        ".prettierignore": "vendor/\n",
      },
      async (dir) => {
        const check = async (fileName: string) =>
          (await resolveConfig(fileName, { cwd: dir })).ignored;
        expect(await check("src/a.ts")).toBe(false);
        expect(await check("generated/a.ts")).toBe(true);
        expect(await check("vendor/a.ts")).toBe(true);
      },
    );
  });

  it("rejects invalid config", async () => {
    await withTempDir({ ".oxfmtrc.json": JSON.stringify({ printWidth: 1000 }) }, async (dir) => {
      await expect(resolveConfig("a.ts", { cwd: dir })).rejects.toThrow("printWidth");
    });
  });

  it("formats the same as `--stdin-filepath`", async () => {
    await withTempDir(
      {
        ".oxfmtrc.json": JSON.stringify({
          semi: false,
          overrides: [{ files: ["*.test.ts"], options: { singleQuote: true } }],
        }),
        ".editorconfig": "root = true\n\n[*]\nindent_style = tab\n",
        "packages/a/.oxfmtrc.json": JSON.stringify({ printWidth: 40 }),
      },
      async (dir) => {
        const source = 'function f(){return {first:"a",second:"b",third:"c"}}\n';
        await Promise.all(
          ["a.ts", "a.test.ts", "packages/a/a.ts"].map(async (fileName) => {
            const { config } = await resolveConfig(fileName, { cwd: dir });
            const { code } = await format(join(dir, fileName), source, config);
            const { stdout } = await execa("node", [CLI_PATH, "--stdin-filepath", fileName], {
              cwd: dir,
              input: source,
              stripFinalNewline: false,
            });
            expect(code).toBe(stdout);
          }),
        );
      },
    );
  });
});
