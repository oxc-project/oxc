// oxlint-disable no-console
// What `--cache` buys on a real project: `oxfmt --check --cache` against `oxfmt --check` of the
// same build, bare `vp fmt --check`, and `vp fmt --check` as a `vp run --cache` task, in the
// scenarios a developer meets. Each run is timed from spawn to exit, and saved as it happens, as
// raw data with provenance, to bench/cache/results/scenarios-<time>.json.
//
// Scenarios:
//   fresh     nothing cached: a cache location never used before (`vp cache clean` for vp run) for
//             every run.
//   nothing   nothing changed since the last run.
//   edit      a tracked source file gains a comment before every run (still formatted),
//             undone after: the next run sees an edit too.
//   new-file  a new formatted source file before every run, removed after.
// Targets:
//   oxfmt        the build's `bin/oxfmt --check`, with the environment `vp fmt` gives it.
//   oxfmt-cache  the same, with `--cache --cache-location <dir>`.
//   vp           `vp fmt --check` from the project's node_modules (its own oxfmt).
//   vp-run       `vp run --cache oxfmt-bench:fmt`, a root package.json script `vp fmt --check`,
//                added after the project is found clean and restored after the last run.
//   baseline, baseline-cache
//                oxfmt and oxfmt-cache with another build, `--baseline <bin>`: to compare
//                two builds run by run.
// Targets alternate run by run, so drift in machine load hits all of them, and their order
// rotates round by round: a run right after an uncached one on a large project is slower, so no
// target always follows the same one. Each scenario's first round is a warm-up, marked and
// excluded from statistics.
//
// usage: node bench/cache/scenarios.ts --project <dir> --oxfmt <bin> [--baseline <bin>]
//          [--targets a,b] [--runs N] [--only a,b]

import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { exec, git, load, provenance, save, stamp, stats, vpEnv } from "./lib.ts";

const { values } = parseArgs({
  options: {
    project: { type: "string" },
    oxfmt: { type: "string" },
    baseline: { type: "string" },
    runs: { type: "string", default: "7" },
    only: { type: "string" },
    cacheDir: { type: "string" },
    // Comma-separated: oxfmt, oxfmt-cache, vp, vp-run, baseline, baseline-cache. vp and vp-run
    // need vite-plus in the project; baseline and baseline-cache, `--baseline`.
    targets: { type: "string", default: "oxfmt,oxfmt-cache,vp,vp-run" },
  },
});
if (!values.project || !values.oxfmt) {
  console.error(
    "usage: node bench/cache/scenarios.ts --project <dir> --oxfmt <bin> [--runs N] [--only a,b]",
  );
  process.exit(2);
}
const project = path.resolve(values.project);
const oxfmt = path.resolve(values.oxfmt);
const runs = Number(values.runs);
const only = values.only?.split(",");
const cacheDir = path.resolve(
  values.cacheDir ?? path.join(path.dirname(oxfmt), "../../../cache-dir"),
);
const baseline = values.baseline && path.resolve(values.baseline);
const baselineCacheDir = `${cacheDir}-baseline`;
// On macOS, removing a directory right before a run slows that run (up to 2x on vscode, cached or
// not): each fresh run gets a location never used before instead, all removed after the last run.
const freshDirs: string[] = [];
const freshLocation = (args: string[], i: number) =>
  args.map((arg) => {
    if (!arg.startsWith("--cache-location=")) return arg;
    const dir = `${arg.slice("--cache-location=".length)}-fresh-${process.pid}-${i}`;
    freshDirs.push(dir);
    return `--cache-location=${dir}`;
  });
const env = vpEnv(project);
const vp = path.join(project, "node_modules/.bin/vp");
const at = (rel: string) => path.join(project, rel);

const TASK = "oxfmt-bench:fmt";
type Target = "oxfmt" | "oxfmt-cache" | "vp" | "vp-run" | "baseline" | "baseline-cache";
const targets = values.targets!.split(",") as Target[];
if (targets.some((t) => t.startsWith("baseline")) && !baseline) {
  console.error("the baseline targets need --baseline <bin>");
  process.exit(2);
}
const command: Record<Target, [string, string[], Record<string, string>]> = {
  oxfmt: [oxfmt, ["--check"], env],
  "oxfmt-cache": [oxfmt, ["--check", "--cache", `--cache-location=${cacheDir}`], env],
  vp: [vp, ["fmt", "--check"], {}],
  "vp-run": [vp, ["run", "--cache", TASK], {}],
  baseline: [baseline ?? "", ["--check"], env],
  "baseline-cache": [
    baseline ?? "",
    ["--check", "--cache", `--cache-location=${baselineCacheDir}`],
    env,
  ],
};

// The edited file: a tracked TypeScript source, formatted, the first in Git's order.
const edited = git(project, "ls-files", "-z", "--", "src/*.ts")
  .split("\0")
  .find((f) => f.endsWith(".ts") && !f.endsWith(".d.ts"))!;

type Scenario = {
  name: string;
  /** Before a target's run: returns how to undo it. */
  before: (target: Target, i: number) => () => void;
};
const noop = () => {};
const scenarios: Scenario[] = [
  {
    name: "fresh",
    before: (target) => {
      if (target === "vp-run") exec(project, vp, ["cache", "clean"]);
      return noop;
    },
  },
  { name: "nothing", before: () => noop },
  {
    name: "edit",
    before: (_, i) => {
      const bytes = readFileSync(at(edited));
      writeFileSync(at(edited), Buffer.concat([bytes, Buffer.from(`// oxfmt-bench ${i}\n`)]));
      return () => writeFileSync(at(edited), bytes);
    },
  },
  {
    name: "new-file",
    before: (_, i) => {
      const file = path.join(path.dirname(at(edited)), `oxfmt-bench-${i}.ts`);
      writeFileSync(file, `export const oxfmtBench${i} = ${i};\n`);
      return () => rmSync(file, { force: true });
    },
  },
];

type Point = {
  scenario: string;
  target: Target;
  run: number;
  warmup: boolean;
  ms: number;
  exitCode: number;
  files: number | null;
  /** vp-run: whether vp's task cache was hit. */
  cacheHit?: boolean;
  loadavg: number[];
  at: string;
};

if (git(project, "status", "--porcelain").trim()) {
  console.error(`${project} has uncommitted changes; the benchmark needs a clean tree`);
  process.exit(2);
}
// The vp-run task, a root package.json script, in the file's own indentation.
const pkgFile = at("package.json");
const vpRun = targets.includes("vp-run");
const pkgBytes = vpRun ? readFileSync(pkgFile) : Buffer.alloc(0);
const pkgText = pkgBytes.toString("utf8");
const withTask = pkgText.replace(
  /("scripts":\s*\{\n)([ \t]*)/,
  (_, open: string, indent: string) =>
    `${open}${indent}${JSON.stringify(TASK)}: "vp fmt --check",\n${indent}`,
);
if (vpRun && withTask === pkgText) {
  throw new Error(`${pkgFile}: no "scripts" to add ${TASK} to`);
}

const points: Point[] = [];
const out = path.join(import.meta.dirname, "results", `scenarios-${stamp()}.json`);
const app = path.join(import.meta.dirname, "../..");
const config = { project, oxfmt, baseline, cacheDir, runs, edited, targets, commands: command };
const notes = [
  "Each run is timed with performance.now() from spawn to exit (spawnSync), stdout and stderr captured.",
  "oxfmt and oxfmt-cache are a copy of a release build (pnpm run build) installed beside the project, its vite-plus and tinypool linked to the project's; vp and vp-run are the project's own vite-plus and oxfmt.",
  "oxfmt is run with the environment vite-plus gives it (JS_RUNTIME_*, NODE_PACKAGE_MANAGER, VP_RESOLVING_CONFIG_METADATA, VP_VERSION).",
  "Targets alternate run by run, in an order rotated round by round; each scenario's first round is a warm-up, excluded from statistics.",
  "edit appends a comment line to a tracked formatted source before each run and restores its bytes after; new-file adds a formatted file beside it and removes it after.",
  "fresh gives each run a cache location never used before: removing one right before a run slows it on macOS.",
  "The load average is recorded with every run.",
];
const write = () => save(out, { ...provenance(app, project), config, notes, points });

if (vpRun) writeFileSync(pkgFile, withTask);
try {
  for (const scenario of scenarios) {
    if (only && !only.includes(scenario.name)) continue;
    for (let i = 0; i <= runs; i++) {
      const rotation = i % targets.length;
      for (const target of [...targets.slice(rotation), ...targets.slice(0, rotation)]) {
        const undo = scenario.before(target, i);
        const [bin, args, runEnv] = command[target];
        const runArgs = scenario.name === "fresh" ? freshLocation(args, i) : args;
        const r = exec(project, bin, runArgs, runEnv);
        undo();
        const files = /on (\d+) files/.exec(r.stdout)?.[1];
        const point: Point = {
          scenario: scenario.name,
          target,
          run: i,
          warmup: i === 0,
          ms: r.ms,
          exitCode: r.status,
          files: files ? Number(files) : null,
          ...(target === "vp-run" ? { cacheHit: /cache hit/.test(r.stdout + r.stderr) } : {}),
          loadavg: load(),
          at: new Date().toISOString(),
        };
        points.push(point);
        write();
        console.log(
          `${scenario.name.padEnd(9)} ${String(i).padStart(2)} ${target.padEnd(12)} ${r.ms.toFixed(0).padStart(6)} ms  exit ${r.status}  files ${point.files}  load ${point.loadavg[0]}${point.warmup ? "  (warm-up)" : ""}`,
        );
      }
    }
  }
} finally {
  if (vpRun) writeFileSync(pkgFile, pkgBytes);
}
if (git(project, "status", "--porcelain").trim()) {
  console.error(`warning: ${project} isn't clean after the benchmark`);
}

for (const scenario of scenarios) {
  for (const target of targets) {
    const ms = points
      .filter((p) => p.scenario === scenario.name && p.target === target && !p.warmup)
      .map((p) => p.ms);
    if (ms.length) {
      console.log(`${scenario.name.padEnd(9)} ${target.padEnd(12)} ${JSON.stringify(stats(ms))}`);
    }
  }
}
console.log(`results: ${out}`);
for (const dir of [cacheDir, baselineCacheDir, ...freshDirs]) {
  if (existsSync(dir)) rmSync(dir, { recursive: true, force: true });
}
