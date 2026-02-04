import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { parseAstroSync } from "./src-js/index.js";

// usage:
// node napi/parser/example_astro.js test.astro

process.chdir(path.join(import.meta.dirname, "../.."));

const args = parseArgs({
  args: process.argv.slice(2),
  allowPositionals: true,
  options: {
    range: {
      type: "boolean",
    },
  },
});

const file = args.positionals[0] ?? "test.astro";

const code = fs.readFileSync(file, "utf-8");
const result = parseAstroSync(code, { range: args.values.range });
// oxlint-disable-next-line no-console, typescript-eslint/no-misused-spread
console.dir({ ...result }, { depth: Infinity });
