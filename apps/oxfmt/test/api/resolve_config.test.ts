import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { resolveConfig } from "../../dist/index.js";

async function withTempDir<T>(fn: (dir: string) => Promise<T>): Promise<T> {
  const dir = await fs.mkdtemp(join(tmpdir(), "oxfmt-resolve-config-"));
  try {
    return await fn(dir);
  } finally {
    await fs.rm(dir, { recursive: true, force: true });
  }
}

describe("resolveConfig() API", () => {
  it("`resolveConfig()` function exists", () => {
    expect(typeof resolveConfig).toBe("function");
  });

  it("returns null when no config is found", async () => {
    await withTempDir(async (dir) => {
      await fs.mkdir(join(dir, "src"), { recursive: true });
      await fs.writeFile(join(dir, "src", "test.ts"), "export const x=1\n");

      await expect(resolveConfig(join(dir, "src", "test.ts"))).resolves.toBeNull();
    });
  });

  it("resolves .oxfmtrc overrides and .editorconfig fallback", async () => {
    await withTempDir(async (dir) => {
      await fs.mkdir(join(dir, "nested"), { recursive: true });
      await fs.writeFile(
        join(dir, ".oxfmtrc.json"),
        JSON.stringify({
          semi: false,
          sortTailwindcss: { config: "./tailwind.config.js" },
          overrides: [{ files: ["**/*.test.ts"], options: { tabWidth: 4 } }],
        }),
      );
      await fs.writeFile(
        join(dir, ".editorconfig"),
        [
          "root = true",
          "",
          "[*]",
          "indent_style = tab",
          "",
          "[nested/*.test.ts]",
          "indent_size = 6",
          "",
        ].join("\n"),
      );
      await fs.writeFile(join(dir, "nested", "example.test.ts"), "export const x=1\n");

      await expect(resolveConfig(join(dir, "nested", "example.test.ts"))).resolves.toEqual({
        semi: false,
        useTabs: true,
        tabWidth: 4,
        sortTailwindcss: {
          config: join(dir, "tailwind.config.js"),
        },
      });
    });
  });

  it("resolves JavaScript config files during auto-discovery", async () => {
    await withTempDir(async (dir) => {
      await fs.mkdir(join(dir, "src"), { recursive: true });
      await fs.writeFile(
        join(dir, "oxfmt.config.ts"),
        "export default { semi: false, singleQuote: true };\n",
      );
      await fs.writeFile(join(dir, "src", "test.ts"), "export const x=1\n");

      await expect(resolveConfig(join(dir, "src", "test.ts"))).resolves.toEqual({
        semi: false,
        singleQuote: true,
      });
    });
  });
});
