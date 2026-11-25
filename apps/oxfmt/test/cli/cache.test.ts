// `--cache` across runs: a cached run prints and exits as an uncached one,
// and skips formatting only contents it knows formatted under the same options.

import { execa } from "execa";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { CLI_PATH, normalizeOutput } from "./utils";

// A `stat` is trusted only once its timestamps are this much older than the run.
const RACY_WINDOW_MS = 2000;

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
  let dir: string;
  let cacheDir: string;
  beforeEach(async () => {
    dir = await fs.mkdtemp(join(tmpdir(), "oxfmt-cache-"));
    cacheDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-cache-dir-"));
  });
  afterEach(async () => {
    await fs.rm(dir, { recursive: true, force: true });
    await fs.rm(cacheDir, { recursive: true, force: true });
  });

  // Runs with the cache, then without, and returns the output after checking they agree.
  const check = async (expected: { hits: number; added: number }) => {
    const { hits, added, ...output } = await run(dir, [
      "--check",
      "--cache",
      `--cache-location=${cacheDir}`,
    ]);
    const { hits: _, added: __, ...uncached } = await run(dir, ["--check"]);
    expect(output).toEqual(uncached);
    expect({ hits, added }).toEqual(expected);
    return output;
  };

  it("skips only what it knows formatted under the same options", async () => {
    // Timestamps in the future are never trusted: only contents are cached here.
    const write = async (name: string, text: string) => {
      await fs.writeFile(join(dir, name), text);
      const future = new Date(Date.now() + 3600_000);
      await fs.utimes(join(dir, name), future, future);
    };
    await write("formatted.ts", "export const a = 1;\n");
    await write("unformatted.ts", "export const   b=1\n");
    // Formatted by Prettier in Node: never cached.
    await write("doc.mdx", "# Title\n");

    // Only `formatted.ts` is cached: `unformatted.ts` isn't formatted, `doc.mdx` went through Node.
    expect((await check({ hits: 0, added: 1 })).exitCode).toBe(1);
    await check({ hits: 1, added: 0 });

    // Other options: the same content is another key (only the config itself is added).
    await write(".oxfmtrc.json", '{ "semi": false }\n');
    expect((await check({ hits: 0, added: 1 })).stdout).toContain("formatted.ts");

    // Back to the first options and content: known again.
    await fs.rm(join(dir, ".oxfmtrc.json"));
    await check({ hits: 1, added: 0 });
  });

  // Whole seconds, so it can be put back exactly.
  const old = Math.floor(Date.now() / 1000) - 3600;
  // A formatted file whose `stat` is known after two runs.
  const cacheByStat = async () => {
    const file = join(dir, "formatted.ts");
    await fs.writeFile(file, "export const a = 1;\n");
    await fs.utimes(file, old, old);
    // Wait until the inode change time `utimes` set is old enough to trust too.
    await sleep(RACY_WINDOW_MS + 100);
    // Its content and its `stat` are added, then the `stat` alone answers.
    await check({ hits: 0, added: 2 });
    await check({ hits: 1, added: 0 });
    return file;
  };

  it("doesn't read a file whose `stat` is known", async () => {
    const file = await cacheByStat();
    await fs.writeFile(file, "export const   a = 1\n");
    expect((await check({ hits: 0, added: 0 })).stdout).toContain("formatted.ts");
  });

  // NOTE: Without an inode change time, Windows can't see this one.
  it.skipIf(process.platform === "win32")(
    "sees a same-size write with `mtime` put back",
    async () => {
      const file = await cacheByStat();
      // Unformatted, the same size, the modification time put back: only the inode change time tells.
      await fs.writeFile(file, "export const a =1 ;\n");
      await fs.utimes(file, old, old);
      expect((await check({ hits: 0, added: 0 })).stdout).toContain("formatted.ts");
    },
  );
});
