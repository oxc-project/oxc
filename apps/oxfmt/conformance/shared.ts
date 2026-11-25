// oxlint-disable no-console

import { globSync, readFileSync } from "node:fs";
import { join } from "node:path";

// Shared by the `run_*.ts` runners.

/** A file is named by its path under this directory, as the reports print it and `DIVERGENCES.md` lists it */
export const CONFORMANCE_DIR = import.meta.dirname;
const REPO_ROOT = join(CONFORMANCE_DIR, "..", "..", "..");
/** Oxfmt's own fixtures, recorded by `run_fixture.ts` */
export const FIXTURES_DIR = join(CONFORMANCE_DIR, "fixtures");
/** Downloaded by `download-externals.ts`, compared by `run_prettier.ts` and `run_live.ts` */
export const EXTERNALS_DIR = join(CONFORMANCE_DIR, "externals");
export const SNAPSHOTS_DIR = join(CONFORMANCE_DIR, "snapshots");

const BOM = "\uFEFF";
const NAME_BY_EOL: Record<string, string> = { "\n": "LF", "\r\n": "CRLF", "\r": "CR" };
const EOL_BY_NAME = Object.fromEntries(
  Object.entries(NAME_BY_EOL).map(([eol, name]) => [name, eol]),
);

/** Each line break as `<LF>` / `<CRLF>` / `<CR>` and a `\n`, as Prettier's snapshots show them when `endOfLine` is set. */
export const showLineEndings = (text: string) =>
  text.replace(/\r\n?|\n/g, (eol) => `<${NAME_BY_EOL[eol]}>\n`);

/** The inverse of `showLineEndings`. */
export const readLineEndings = (text: string) =>
  text.replace(/<(CRLF|CR|LF)>\n/g, (_, name: string) => EOL_BY_NAME[name]);

/**
 * The CRLF / CR / BOM variants of `input` and the output each must format to, given `output` of `input`:
 * line endings are normalized and a BOM is kept.
 * Inputs already containing `\r` or a BOM are skipped, their variants would not be equivalent.
 */
export function inputVariants(input: string, output: string) {
  const variants: [name: string, input: string, output: string][] = [];
  if (!input.includes("\r")) {
    variants.push(
      ["CRLF", input.replaceAll("\n", "\r\n"), output],
      ["CR", input.replaceAll("\n", "\r"), output],
    );
  }
  if (!input.startsWith(BOM)) variants.push(["BOM", BOM + input, BOM + output]);
  return variants;
}

/** Prettier's own suite, the `run_prettier.ts` part of the file paths (relative to `CONFORMANCE_DIR`) */
export const PRETTIER_SUITE = "externals/prettier/";

/** A file path (relative to `CONFORMANCE_DIR`) and the `<file>#<slug>` of the DIVERGENCES.md entry listing it */
export type Divergence = [listed: string, ref: string];

/**
 * Every path listed in a `Conformance:` line, or its continuation lines.
 * `Pin:` ones are not, a pin is recorded by `run_fixture.ts`, never compared.
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
      if (line.startsWith("- ")) inField = line.startsWith("- Conformance:");
      else if (!line.startsWith("  ")) inField = false;
      if (!inField) continue;
      for (const [, name] of line.matchAll(
        /`(?:conformance\/)?((?:externals|fixtures)\/[^`]+)`/g,
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
