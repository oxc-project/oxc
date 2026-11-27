// Shared pieces of the cache benchmarks: installing a build of oxfmt next to a project's
// own install, running commands the way `vp fmt` runs oxfmt, and saving results as data.

import { execFileSync, spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
  renameSync,
} from "node:fs";
import { loadavg, cpus, totalmem, release } from "node:os";
import path from "node:path";

export const schemaVersion = 1;

export type Run = { status: number; stdout: string; stderr: string; ms: number };

/** The oxfmt the project installed (through vite-plus), resolved as vite-plus resolves it. */
export function installedOxfmt(project: string): string {
  const vitePlus = realpathSync(path.join(project, "node_modules/vite-plus"));
  // `node_modules/.pnpm/<vite-plus>/node_modules/vite-plus` → its sibling `oxfmt`, as `require.resolve` finds it.
  const oxfmt = path.join(vitePlus, "..", "oxfmt");
  if (existsSync(oxfmt)) return realpathSync(oxfmt);
  return realpathSync(path.join(project, "node_modules/oxfmt"));
}

/**
 * Install a build of oxfmt (`apps/oxfmt` with `dist/` built) as a copy, beside the project's:
 * `<home>/<label>/node_modules/oxfmt`, its `vite-plus` and `tinypool` linked to the project's,
 * so its JS config loader imports the project's vite-plus as the installed oxfmt does.
 * Returns the path of its `bin/oxfmt`.
 */
export function installBuild(app: string, project: string, home: string, label: string): string {
  const installed = installedOxfmt(project);
  const root = path.join(home, label);
  rmSync(root, { recursive: true, force: true });
  const pkg = path.join(root, "node_modules/oxfmt");
  mkdirSync(pkg, { recursive: true });
  cpSync(path.join(installed, "package.json"), path.join(pkg, "package.json"));
  cpSync(path.join(installed, "bin"), path.join(pkg, "bin"), { recursive: true });
  cpSync(path.join(app, "dist"), path.join(pkg, "dist"), { recursive: true });
  for (const dep of ["vite-plus", "tinypool"]) {
    symlinkSync(
      realpathSync(path.join(installed, "..", dep)),
      path.join(root, "node_modules", dep),
    );
  }
  return path.join(pkg, "bin/oxfmt");
}

/**
 * Install a build of oxfmt (`apps/oxfmt` with `dist/` built) as a copy, for a project without
 * vite-plus: `<home>/<label>/node_modules/oxfmt`, with npm's `bin/oxfmt` and `tinypool` linked
 * to the one the oxc workspace installed. Returns the path of its `bin/oxfmt`.
 */
export function installStandalone(app: string, home: string, label: string): string {
  const root = path.join(home, label);
  rmSync(root, { recursive: true, force: true });
  const pkg = path.join(root, "node_modules/oxfmt");
  mkdirSync(pkg, { recursive: true });
  cpSync(path.join(app, "../../npm/oxfmt/package.json"), path.join(pkg, "package.json"));
  cpSync(path.join(app, "../../npm/oxfmt/bin"), path.join(pkg, "bin"), { recursive: true });
  cpSync(path.join(app, "dist"), path.join(pkg, "dist"), { recursive: true });
  symlinkSync(
    realpathSync(path.join(app, "node_modules/tinypool")),
    path.join(root, "node_modules/tinypool"),
  );
  return path.join(pkg, "bin/oxfmt");
}

/**
 * The variables `vp fmt` sets for oxfmt (vite-plus 1.6.0, `resolve-fmt.ts` and the Rust core),
 * on top of the inherited environment. Only these are recorded with the results.
 */
export function vpEnv(project: string): Record<string, string> {
  const vitePlus = path.join(project, "node_modules/vite-plus/package.json");
  // Without vite-plus, oxfmt runs as a user would run it directly.
  if (!existsSync(vitePlus)) return {};
  const vpVersion = JSON.parse(readFileSync(vitePlus, "utf8")).version;
  return {
    JS_RUNTIME_VERSION: process.versions.node,
    JS_RUNTIME_NAME: process.release.name,
    NODE_PACKAGE_MANAGER: "vite-plus",
    VP_RESOLVING_CONFIG_METADATA: "1",
    VP_VERSION: vpVersion,
  };
}

export function exec(
  cwd: string,
  bin: string,
  args: string[],
  extraEnv: Record<string, string> = {},
): Run {
  const env = { ...process.env, ...extraEnv };
  const t0 = performance.now();
  const r = spawnSync(bin, args, { cwd, env, encoding: "utf8", maxBuffer: 512 * 1024 * 1024 });
  return {
    status: r.status ?? 128,
    stdout: r.stdout ?? "",
    stderr: r.stderr ?? "",
    ms: performance.now() - t0,
  };
}

export function git(cwd: string, ...args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 });
}

/** Where a result came from: commits, dirty trees, machine and its load. */
export function provenance(app: string, project: string) {
  const head = (dir: string) => git(dir, "rev-parse", "HEAD").trim();
  const dirty = (dir: string) => git(dir, "status", "--porcelain").trim().length > 0;
  return {
    schemaVersion,
    generatedAt: new Date().toISOString(),
    oxc: {
      commit: head(app),
      branch: git(app, "rev-parse", "--abbrev-ref", "HEAD").trim(),
      dirty: dirty(app),
    },
    project: {
      path: project,
      commit: head(project),
      dirty: dirty(project),
      files: git(project, "ls-files").split("\n").length - 1,
    },
    machine: {
      cpu: cpus()[0]?.model,
      cores: cpus().length,
      memGB: Math.round(totalmem() / 2 ** 30),
      os: release(),
      node: process.version,
    },
  };
}

export const load = () => loadavg().map((x) => Math.round(x * 100) / 100);

/** Write `data` atomically, so an interrupted run leaves the last complete file. */
export function save(file: string, data: unknown) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(`${file}.tmp`, JSON.stringify(data, null, 1));
  renameSync(`${file}.tmp`, file);
}

export const stamp = () => new Date().toISOString().replace(/[:.]/g, "-");

export function stats(xs: number[]) {
  const s = [...xs].sort((a, b) => a - b);
  const q = (p: number) => s[Math.min(s.length - 1, Math.floor(p * s.length))] ?? NaN;
  return { n: s.length, min: s[0], p50: q(0.5), p95: q(0.95), max: s[s.length - 1] };
}
