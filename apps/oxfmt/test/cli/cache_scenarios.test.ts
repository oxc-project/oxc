// oxlint-disable no-await-in-loop, valid-title -- each edit names its test
// The `--cache` scenarios: for each named edit to a small Git project, in `--check`,
// `--list-different` and write mode, over the whole project and one directory, a run with the
// cache prints, exits and writes what the same run without it does.
//
// Two copies of the project are made, and left past the racy window, then every run shape is
// cached on the clean copy, as on a developer's project. Both copies then get the same edit, and
// every shape runs with the cache three times, the last after the racy window, so verdicts
// trusted from a `stat` alone must notice the edit. Each run must match the uncached one. Writes are undone by content, so files they didn't change keep their `stat`.

import { execa } from "execa";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");
// The cache trusts a `stat` only this long after it was taken (`RACY_WINDOW` in `oxc_cache`).
const RACY_WINDOW_MS = 2000;
const SCOPE = "src/app";

type Tree = Record<string, string>;

const PROJECT: Tree = {
  "oxfmt.config.ts": "export default {\n  printWidth: 100,\n  experimentalTailwindcss: {},\n};\n",
  ".editorconfig": "root = true\n\n[*]\nindent_style = space\n",
  ".prettierignore": "dist\n",
  ".gitignore": "node_modules\n",
  [`${SCOPE}/.gitignore`]: "*.log\n",
  [`${SCOPE}/a.ts`]:
    "export function greet(name: string, greeting: string): string {\n  return `${greeting}, ${name}! Welcome back to the application.`;\n}\n",
  [`${SCOPE}/b.ts`]: 'import { greet } from "./a";\n\nexport const message = greet("you", "Hi");\n',
  [`${SCOPE}/view.tsx`]: 'export const View = () => <div className="flex p-2">hi</div>;\n',
  "src/lib/c.ts": "export const c = { a: 1, b: [1, 2, 3] };\n",
  "docs/guide.mdx": "# Guide\n\n- one\n- two\n",
  "README.md": "# Project\n\nSome text.\n",
};

const at = (dir: string, path: string) => join(dir, path);

async function writeFile(path: string, text: string) {
  await fs.mkdir(join(path, ".."), { recursive: true });
  await fs.writeFile(path, text);
}

async function append(path: string, text: string) {
  const was = await fs.readFile(path, "utf8").catch(() => "");
  await writeFile(path, was + text);
}

async function tree(root: string, dir = root): Promise<Tree> {
  const out: Tree = {};
  for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
    if (entry.name === ".git") continue;
    const path = join(dir, entry.name);
    if (entry.isDirectory()) Object.assign(out, await tree(root, path));
    else out[relative(root, path)] = await fs.readFile(path, "utf8");
  }
  return out;
}

/** Make `root` hold `want`, writing only files whose content differs. */
async function reset(root: string, want: Tree) {
  const have = await tree(root);
  for (const path of Object.keys(have)) {
    if (!(path in want)) await fs.rm(at(root, path));
  }
  for (const [path, text] of Object.entries(want)) {
    if (have[path] !== text) await writeFile(at(root, path), text);
  }
}

type Edit = { name: string; apply: (dir: string) => Promise<void> };

// Windows has no `ctime`: a same-size edit that puts the mtime back goes unseen there.
const ONLY_CTIME_TELLS = process.platform === "win32";

const EDITS: Edit[] = [
  { name: "nothing", apply: async () => {} },
  {
    name: "an unformatted new file",
    apply: (dir) => writeFile(at(dir, `${SCOPE}/new.ts`), "export const   o = {a:1}\n"),
  },
  {
    name: "a formatted new file",
    apply: (dir) => writeFile(at(dir, `${SCOPE}/new.ts`), "export const o = { a: 1 };\n"),
  },
  {
    name: "a parse error",
    apply: (dir) => writeFile(at(dir, `${SCOPE}/broken.ts`), "export const = ;\n"),
  },
  {
    name: "a formatted file left unformatted",
    apply: (dir) => append(at(dir, `${SCOPE}/b.ts`), "\nexport const   edited = {a:1}\n"),
  },
  {
    name: "a formatted file left formatted",
    apply: (dir) => append(at(dir, `${SCOPE}/b.ts`), "\nexport const edited = 1;\n"),
  },
  ...(ONLY_CTIME_TELLS
    ? []
    : [
        {
          // The same size and modification time: only `ctime` (or the content) tells.
          name: "a same-size edit with mtime restored",
          apply: async (dir: string) => {
            const file = at(dir, `${SCOPE}/b.ts`);
            const { atime, mtime } = await fs.stat(file);
            await fs.writeFile(file, (await fs.readFile(file, "utf8")).replace(";", " "));
            await fs.utimes(file, atime, mtime);
          },
        },
      ]),
  {
    name: "a file renamed",
    apply: (dir) => fs.rename(at(dir, `${SCOPE}/b.ts`), at(dir, `${SCOPE}/b.renamed.ts`)),
  },
  { name: "a formatted file deleted", apply: (dir) => fs.rm(at(dir, `${SCOPE}/b.ts`)) },
  {
    name: "an ignored unformatted file",
    apply: async (dir) => {
      await writeFile(at(dir, `${SCOPE}/ignored.ts`), "export const   o = {a:1}\n");
      await append(at(dir, ".prettierignore"), `${SCOPE}/ignored.ts\n`);
    },
  },
  {
    name: "an .editorconfig section for the scope",
    apply: (dir) => append(at(dir, ".editorconfig"), `\n[${SCOPE}/**.ts]\nindent_size = 3\n`),
  },
  {
    name: "a nested .gitignore ignoring an unformatted file",
    apply: async (dir) => {
      await writeFile(at(dir, `${SCOPE}/gitignored.ts`), "export const   o = {a:1}\n");
      await append(at(dir, `${SCOPE}/.gitignore`), "gitignored.ts\n");
    },
  },
  {
    name: "the format configuration changed",
    apply: async (dir) => {
      const file = at(dir, "oxfmt.config.ts");
      await fs.writeFile(file, (await fs.readFile(file, "utf8")).replace("100", "60"));
    },
  },
  {
    // Sorted by the Tailwind sorter, in Node.
    name: "unsorted Tailwind classes",
    apply: (dir) =>
      append(
        at(dir, `${SCOPE}/view.tsx`),
        '\nexport const Tw = () => <div className="p-2 flex" />;\n',
      ),
  },
  {
    // Formatted by Prettier, in Node.
    name: "an unformatted MDX file",
    apply: (dir) => append(at(dir, "docs/guide.mdx"), "\n\n*  another   item\n"),
  },
];

const SCOPES = [
  { name: "all", args: [] as string[] },
  { name: "scope", args: [SCOPE] },
];
const MODES = [
  { name: "check", args: ["--check"] },
  { name: "list", args: ["--list-different"] },
  { name: "write", args: [] as string[] },
];

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
  const mask = (s: string) => s.replace(/\d+(\.\d+)?m?s\b/g, "Nms");
  return { exitCode, stdout: mask(stdout), stderr: mask(stderr), tree: await tree(cwd) };
}

// Whole seconds, a minute ago: Node sets times only to the millisecond, so a same-size edit can
// put such an mtime back exactly, and only `ctime` tells.
const PAST = new Date((Math.floor(Date.now() / 1000) - 60) * 1000);

async function project(dir: string) {
  await fs.mkdir(dir);
  await reset(dir, PROJECT);
  for (const path of Object.keys(PROJECT)) await fs.utimes(at(dir, path), PAST, PAST);
  await execa("git", ["init", "-q"], { cwd: dir });
}

const SHAPES = SCOPES.flatMap((scope) =>
  MODES.map((mode) => ({
    name: `${scope.name} / ${mode.name}`,
    args: [...mode.args, ...scope.args],
    write: mode.name === "write",
  })),
);

const pastRacyWindow = () => new Promise((r) => setTimeout(r, RACY_WINDOW_MS + 100));

describe.concurrent("--cache matches an uncached run", () => {
  for (const edit of EDITS) {
    it(edit.name, { timeout: 120_000 }, async () => {
      const base = await fs.mkdtemp(join(tmpdir(), "oxfmt-cache-scenarios-"));
      const cached = join(base, "cached");
      const uncached = join(base, "uncached");
      const cache = ["--cache", `--cache-location=${join(base, "cache")}`];
      try {
        await Promise.all([project(cached), project(uncached)]);

        // Each shape runs on `tree` with and without the cache, which must agree; a write is
        // undone after, so every shape sees `tree`.
        const without = new Map<string, Awaited<ReturnType<typeof run>>>();
        const round = async (label: string, tree: Tree) => {
          for (const shape of SHAPES) {
            const name = `${edit.name} / ${shape.name}, ${label}`;
            const key = `${label === "clean" ? "clean" : "edited"} ${shape.name}`;
            if (!without.has(key)) {
              without.set(key, await run(uncached, shape.args));
              if (shape.write) await reset(uncached, tree);
            }
            const withCache = await run(cached, [...shape.args, ...cache]);
            if (shape.write) await reset(cached, tree);
            // oxlint-disable-next-line vitest/valid-expect -- vitest takes a message; it names the case
            expect(withCache, name).toEqual(without.get(key));
          }
        };

        // A project older than the racy window, cached by every shape, as a developer's is.
        await pastRacyWindow();
        await round("clean", PROJECT);

        await edit.apply(cached);
        await edit.apply(uncached);
        const edited = await tree(cached);
        expect(await tree(uncached)).toEqual(edited);
        await round("first cached run", edited);
        await round("second cached run", edited);
        await pastRacyWindow();
        await round("after the racy window", edited);
      } finally {
        await fs.rm(base, { recursive: true, force: true });
      }
    });
  }
});
