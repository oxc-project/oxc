// `--cache` across runs: a cached run prints and exits as an uncached one,
// and skips formatting only contents it knows formatted under the same options.

import { execa } from "execa";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { CLI_PATH, normalizeOutput } from "./utils";

async function run(cwd: string, args: string[]) {
  const { stdout, stderr, exitCode } = await execa("node", [CLI_PATH, "--threads=1", ...args], {
    cwd,
    reject: false,
    env: { OXC_LOG: "oxfmt::cache=debug" },
  });
  // oxlint-disable-next-line no-control-regex
  const log = stderr.replace(/\x1b\[[0-9;]*m/g, "");
  const save = /hits=(\d+) added=(\d+)/.exec(log);
  return {
    exitCode,
    stdout: normalizeOutput(stdout, cwd),
    hits: save ? Number(save[1]) : null,
    added: save ? Number(save[2]) : null,
  };
}

describe("--cache", () => {
  it("skips only what it knows formatted, and prints as an uncached run", async () => {
    const dir = await fs.mkdtemp(join(tmpdir(), "oxfmt-cache-"));
    const cacheDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-cache-dir-"));
    try {
      await fs.writeFile(join(dir, "formatted.ts"), "export const a = 1;\n");
      await fs.writeFile(join(dir, "unformatted.ts"), "export const   b=1\n");
      // Formatted by Prettier in Node: never cached.
      await fs.writeFile(join(dir, "doc.mdx"), "# Title\n");

      const cached = ["--check", "--cache", `--cache-location=${cacheDir}`];
      const check = async (expected: { hits: number; added: number }) => {
        const { hits, added, ...output } = await run(dir, cached);
        const { hits: _, added: __, ...uncached } = await run(dir, ["--check"]);
        expect(output).toEqual(uncached);
        expect({ hits, added }).toEqual(expected);
        return output;
      };

      // Only `formatted.ts` is cached: `unformatted.ts` isn't formatted, `doc.mdx` went through Node.
      expect((await check({ hits: 0, added: 1 })).exitCode).toBe(1);
      await check({ hits: 1, added: 0 });

      // Other options: the same content is another key (only the config itself is added).
      await fs.writeFile(join(dir, ".oxfmtrc.json"), '{ "semi": false }\n');
      expect((await check({ hits: 0, added: 1 })).stdout).toContain("formatted.ts");

      // Back to the first options and content: known again.
      await fs.rm(join(dir, ".oxfmtrc.json"));
      await check({ hits: 1, added: 0 });
    } finally {
      await fs.rm(dir, { recursive: true, force: true });
      await fs.rm(cacheDir, { recursive: true, force: true });
    }
  });
});
