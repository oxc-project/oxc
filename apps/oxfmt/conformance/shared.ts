// oxlint-disable no-console

import { globSync, readFileSync } from "node:fs";
import { join } from "node:path";

// Shared by `run_prettier.ts` and `run_live.ts`.

const REPO_ROOT = join(import.meta.dirname, "..", "..", "..");
export const FIXTURES_DIR = join(import.meta.dirname, "fixtures");
export const SNAPSHOTS_DIR = join(import.meta.dirname, "snapshots");

/** Prettier's own suite, the `run_prettier.ts` part of the fixture paths (relative to `FIXTURES_DIR`) */
export const PRETTIER_SUITE = "externals/prettier/";

/** A fixture path (relative to `FIXTURES_DIR`) and the `<file>#<slug>` of the DIVERGENCES.md entry listing it */
export type Divergence = [listed: string, ref: string];

/**
 * Every path listed in a `Pin:` or `Conformance:` line, or its continuation lines.
 * A path ending in `*` lists every path it starts with.
 */
export function collectDivergences(): Divergence[] {
  const divergences: Divergence[] = [];
  for (const file of globSync("{apps/oxfmt,crates/oxc_formatter*}/DIVERGENCES.md", {
    cwd: REPO_ROOT,
  })) {
    let slug = "";
    let inField = false;
    for (const line of readFileSync(join(REPO_ROOT, file), "utf8").split("\n")) {
      if (line.startsWith("## ")) slug = line.slice(3).trim();
      if (line.startsWith("- ")) inField = /^- (Pin|Conformance):/.test(line);
      else if (!line.startsWith("  ")) inField = false;
      if (!inField) continue;
      for (const [, name] of line.matchAll(
        /`(?:conformance\/fixtures\/)?((?:externals|edge-cases)\/[^`]+)`/g,
      )) {
        divergences.push([name, `${file}#${slug}`]);
      }
    }
  }
  return divergences;
}

const covers = (listed: string, name: string) =>
  listed.endsWith("*") ? name.startsWith(listed.slice(0, -1)) : name === listed;

/** The entries listing `name`. */
export function divergencesOf(divergences: Divergence[], name: string) {
  return divergences.flatMap(([listed, ref]) => (covers(listed, name) ? [ref] : []));
}

/**
 * Report lines of the failing `[name, line]`s, each followed by the entries listing it as sub-items.
 * The ones no entry lists (`❓unclassified`) come first.
 */
export function listDivergences(
  divergences: Divergence[],
  failures: [name: string, line: string][],
) {
  return failures
    .map(([name, line]) => [line, divergencesOf(divergences, name)] as const)
    .sort(([, a], [, b]) => Number(a.length > 0) - Number(b.length > 0))
    .flatMap(([line, refs]) =>
      [line].concat((refs.length > 0 ? refs : ["❓unclassified"]).map((ref) => `  - ${ref}`)),
    );
}

/**
 * Fails the run on every listed path of this runner's part (Prettier's suite or the rest) that no failure is.
 * The report is written anyway, the exit code is what CI sees.
 */
export function checkStaleDivergences(
  divergences: Divergence[],
  failed: string[],
  { prettierSuite }: { prettierSuite: boolean },
) {
  for (const [listed, ref] of divergences) {
    if (
      listed.startsWith(PRETTIER_SUITE) === prettierSuite
      && !failed.some((name) => covers(listed, name))
    ) {
      console.error(`STALE: "${listed}" passes but ${ref} lists it, remove it`);
      process.exitCode = 1;
    }
  }
}
