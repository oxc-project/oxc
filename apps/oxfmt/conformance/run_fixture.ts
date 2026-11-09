// oxlint-disable no-console, no-await-in-loop

import { existsSync, globSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, extname, join, relative } from "node:path";
import { stripVTControlCharacters } from "node:util";
import { format } from "../dist/index.js";
import { FIXTURES_DIR, CONFORMANCE_DIR, inputVariants, showLineEndings } from "./shared.ts";

// Records oxfmt's output of every `fixtures/` fixture under each option set of its nearest `options.json`,
// as `<fixture>.snap.md` next to it, never compared with Prettier.
// Each output is also checked for idempotency and for the CRLF / CR / BOM variants of the input.
// `options.json` is an array of oxfmt options (e.g. `[{ "printWidth": 80 }]`), `[{}]` when none is found.

const SNAP_EXT = ".snap.md";
const FENCE = "`````";

// Rewritten from scratch, so a removed fixture leaves no snapshot behind
for (const snap of globSync(`**/*${SNAP_EXT}`, { cwd: FIXTURES_DIR })) {
  rmSync(join(FIXTURES_DIR, snap));
}

const fixtures = globSync("**/*", { cwd: FIXTURES_DIR, withFileTypes: true })
  .filter((entry) => entry.isFile() && entry.name !== "options.json")
  .map((entry) => join(entry.parentPath, entry.name))
  .sort();

for (const fullPath of fixtures) {
  const name = relative(CONFORMANCE_DIR, fullPath);
  const lang = extname(name).slice(1);
  const input = readFileSync(fullPath, "utf8");

  const lines = ["# Input", "", `${FENCE}${lang}`, input, FENCE, "", "# Output", ""];
  for (const options of resolveOptions(fullPath)) {
    // With `endOfLine` set or a `\r` in the output, its line endings are shown (`<CRLF>` etc.), so the snapshot pins them
    const show = (text: string) =>
      "endOfLine" in options || text.includes("\r") ? showLineEndings(text) : text;
    const block = ({ ok, text }: Awaited<ReturnType<typeof formatWithOxfmt>>) =>
      ok ? [`${FENCE}${lang}`, show(text), FENCE, ""] : ["### Errors", "", FENCE, text, FENCE, ""];

    lines.push(`## \`${JSON.stringify(options)}\``, "");
    const first = await formatWithOxfmt(name, input, options);
    lines.push(...block(first));
    if (!first.ok) continue;

    const second = await formatWithOxfmt(name, first.text, options);
    if (second.text !== first.text) {
      lines.push("### Not idempotent (second pass)", "", ...block(second));
    }
    const differ = [];
    for (const [variant, text, output] of inputVariants(input, first.text)) {
      const res = await formatWithOxfmt(name, text, options);
      if (res.text !== output) differ.push(res.ok ? variant : `${variant} (error)`);
    }
    if (differ.length > 0) lines.push(`### Input variants differ: ${differ.join(", ")}`, "");
  }

  writeFileSync(`${fullPath}${SNAP_EXT}`, lines.join("\n"));
}

console.log(`Recorded ${fixtures.length} fixtures`);

// ---

/** The nearest `options.json` up to `fixtures/`. */
function resolveOptions(fullPath: string): Record<string, unknown>[] {
  for (let dir = dirname(fullPath); dir.startsWith(FIXTURES_DIR); dir = dirname(dir)) {
    const file = join(dir, "options.json");
    if (existsSync(file)) return JSON.parse(readFileSync(file, "utf8"));
  }
  return [{}];
}

async function formatWithOxfmt(name: string, input: string, options: Record<string, unknown>) {
  const res = await format(name, input, options);
  return res.errors.length === 0
    ? { ok: true, text: res.code }
    : // Prettier colors its code frame when the env supports it (TTY, `FORCE_COLOR`), keep the snapshot env-independent
      { ok: false, text: res.errors.map((e) => stripVTControlCharacters(e.message)).join("\n") };
}
