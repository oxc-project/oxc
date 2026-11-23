// oxlint-disable no-await-in-loop
// The `--cache` property: for any sequence of edits, a run with the cache prints, exits and
// writes what the same run without it does.
//
// Two copies of a small project get the same random edits. After each, one is run with
// `--cache` (its cache kept across the whole sequence), the other without, and their exit codes,
// output and files must agree. Some steps wait out the racy window first, so verdicts trusted
// from a `stat` alone are exercised too.
//
// Reproduce a failure with `OXFMT_CACHE_SEED=<seed>`, printed with it.

import { execa } from "execa";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");
// The cache trusts a `stat` only this long after it was taken (`RACY_WINDOW_NS`).
const RACY_WINDOW_MS = 2000;
const STEPS = 30;

// A small seeded PRNG (mulberry32), so a failing sequence can be replayed.
function prng(seed: number) {
  let a = seed >>> 0;
  const next = () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 2 ** 32;
  };
  const int = (n: number) => Math.floor(next() * n);
  const pick = <T>(xs: readonly T[]): T => xs[int(xs.length)]!;
  return { next, int, pick };
}

type Rng = ReturnType<typeof prng>;

// File contents: formatted, unformatted, broken, and ones formatted in Node (Markdown, CSS).
const SOURCES = {
  ts: [
    "export const a = 1;\n",
    "export const   a = {b:1}\n",
    "export const = ;\n",
    'import { b } from "./b";\nexport const c = b;\n',
  ],
  md: ["# Title\n\n- item\n", "#   Title\n*  item\n"],
  css: ["a {\n  color: red;\n}\n", "a{color:red}\n"],
  json: ['{ "a": 1 }\n', '{"a":1,\n"b":[1,2]}\n'],
} as const;
const DIRS = ["", "src", "src/nested", "lib"] as const;
const NAMES = ["a", "b", "c"] as const;

const randomFile = (rng: Rng) => {
  const ext = rng.pick(Object.keys(SOURCES) as (keyof typeof SOURCES)[]);
  const dir = rng.pick(DIRS);
  return { path: join(dir, `${rng.pick(NAMES)}.${ext}`), text: rng.pick(SOURCES[ext]) };
};

const randomConfig = (rng: Rng) =>
  JSON.stringify({
    ...(rng.next() < 0.5 ? { semi: false } : {}),
    ...(rng.next() < 0.5 ? { printWidth: rng.pick([20, 80]) } : {}),
    ...(rng.next() < 0.3
      ? { ignorePatterns: [rng.pick(["lib/**", "*.md", "src/nested/**"])] }
      : {}),
    ...(rng.next() < 0.3
      ? { overrides: [{ files: ["src/**"], options: { singleQuote: true } }] }
      : {}),
  });

/** A random edit, as a function applied to each copy's root. */
async function randomEdit(
  rng: Rng,
  root: string,
): Promise<{ name: string; apply: (dir: string) => Promise<void> }> {
  const existing = await listFiles(root);
  const choice = rng.int(10);
  switch (choice) {
    case 0:
    case 1:
    case 2: {
      const { path, text } = randomFile(rng);
      return { name: `write ${path}`, apply: (dir) => writeFile(join(dir, path), text) };
    }
    case 3: {
      const path = existing.length ? rng.pick(existing) : "a.ts";
      return { name: `delete ${path}`, apply: (dir) => fs.rm(join(dir, path), { force: true }) };
    }
    case 4: {
      const from = existing.length ? rng.pick(existing) : "a.ts";
      const to = randomFile(rng).path;
      return {
        name: `rename ${from} ${to}`,
        apply: async (dir) => {
          await fs.mkdir(join(dir, to, ".."), { recursive: true });
          await fs.rename(join(dir, from), join(dir, to)).catch(() => {});
        },
      };
    }
    case 5: {
      const config = randomConfig(rng);
      return {
        name: `.oxfmtrc.json ${config}`,
        apply: (dir) => writeFile(join(dir, ".oxfmtrc.json"), config),
      };
    }
    case 6: {
      const lines = rng.pick(["", "*.css\n", "src/nested\n", "b.ts\n"]);
      return {
        name: `.prettierignore ${JSON.stringify(lines)}`,
        apply: (dir) => writeFile(join(dir, ".prettierignore"), lines),
      };
    }
    case 7: {
      const at = rng.pick(DIRS);
      const lines = rng.pick(["", "*.json\n", "c.*\n"]);
      return {
        name: `${join(at, ".gitignore")} ${JSON.stringify(lines)}`,
        apply: (dir) => writeFile(join(dir, at, ".gitignore"), lines),
      };
    }
    case 8: {
      const text = rng.pick(["", "[*]\nindent_style = tab\n", "[*.ts]\nindent_size = 4\n"]);
      return {
        name: `.editorconfig ${JSON.stringify(text)}`,
        apply: (dir) => writeFile(join(dir, ".editorconfig"), text),
      };
    }
    default: {
      // Same size, same modification time: only `ctime` or the content tells. Windows has no
      // `ctime`, so such an edit goes unseen there.
      const path = existing.find((p) => p.endsWith(".ts"));
      if (!path || process.platform === "win32") return { name: "nothing", apply: async () => {} };
      return {
        name: `same-size edit of ${path}`,
        apply: async (dir) => {
          const file = join(dir, path);
          const { atime, mtime } = await fs.stat(file);
          const text = await fs.readFile(file, "utf8");
          await fs.writeFile(
            file,
            text.includes(";") ? text.replace(";", " ") : text.replace(" ", ";"),
          );
          await fs.utimes(file, atime, mtime);
        },
      };
    }
  }
}

async function writeFile(path: string, text: string) {
  await fs.mkdir(join(path, ".."), { recursive: true });
  await fs.writeFile(path, text);
}

async function listFiles(root: string, dir = root): Promise<string[]> {
  const out: string[] = [];
  for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...(await listFiles(root, path)));
    else if (!entry.name.startsWith(".")) out.push(relative(root, path));
  }
  return out.sort();
}

async function tree(root: string): Promise<Record<string, string>> {
  const out: Record<string, string> = {};
  for (const path of await listFiles(root)) out[path] = await fs.readFile(join(root, path), "utf8");
  return out;
}

async function run(cwd: string, args: string[]) {
  const { CI: _ci, FORCE_COLOR: _forceColor, ...env } = process.env;
  // One thread: diagnostics are then reported in the walk's order, run to run.
  const { stdout, stderr, exitCode } = await execa("node", [CLI_PATH, ...args, "--threads=1"], {
    cwd,
    reject: false,
    timeout: 20000,
    env,
    extendEnv: false,
  });
  // Durations differ run to run.
  const mask = (s: string) => s.replace(/\d+m?s\b/g, "Nms");
  return { exitCode, stdout: mask(stdout), stderr: mask(stderr), tree: await tree(cwd) };
}

const MODES = [["--check"], ["--list-different"], []] as const;

describe("--cache", () => {
  // Otherwise every case below would pass with the cache off: say, if the build can't be found.
  it("keeps what it found", async () => {
    const base = await fs.mkdtemp(join(tmpdir(), "oxfmt-cache-"));
    try {
      await writeFile(join(base, "a.ts"), SOURCES.ts[0]);
      await run(base, ["--check", "--cache", "--cache-location=cache"]);
      expect(await fs.readdir(join(base, "cache/v1/files"))).toHaveLength(1);
      expect(await fs.readdir(join(base, "cache/v1"))).toEqual(["builds", "files", "store"]);
    } finally {
      await fs.rm(base, { recursive: true, force: true });
    }
  });

  const seeds = process.env.OXFMT_CACHE_SEED
    ? [Number(process.env.OXFMT_CACHE_SEED)]
    : [Date.now() % 2 ** 31, 1, 2];

  for (const seed of seeds) {
    it(`a cached run matches an uncached one (seed ${seed})`, { timeout: 120_000 }, async () => {
      const rng = prng(seed);
      const base = await fs.mkdtemp(join(tmpdir(), "oxfmt-cache-"));
      const cached = join(base, "cached");
      const uncached = join(base, "uncached");
      const cacheDir = join(base, "cache");
      try {
        for (const dir of [cached, uncached]) {
          await writeFile(join(dir, "a.ts"), SOURCES.ts[0]);
          await writeFile(join(dir, "src/b.ts"), SOURCES.ts[1]);
          await writeFile(join(dir, "src/c.md"), SOURCES.md[0]);
        }
        const history: string[] = [];
        for (let step = 0; step < STEPS; step++) {
          // Several edits a step, sometimes none: a run right after another must agree too.
          for (let n = rng.int(3); n > 0; n--) {
            const edit = await randomEdit(rng, cached);
            history.push(edit.name);
            await edit.apply(cached);
            await edit.apply(uncached);
          }
          if (rng.next() < 0.15) await new Promise((r) => setTimeout(r, RACY_WINDOW_MS + 100));
          const args = [...rng.pick(MODES)];
          history.push(`run ${args.join(" ")}`);
          const withCache = await run(cached, [...args, "--cache", `--cache-location=${cacheDir}`]);
          const without = await run(uncached, args);
          expect(withCache, `seed ${seed}, after:\n${history.join("\n")}`).toEqual(without);
        }
      } finally {
        await fs.rm(base, { recursive: true, force: true });
      }
    });
  }
});
