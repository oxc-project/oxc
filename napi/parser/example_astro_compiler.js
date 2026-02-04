import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { parse } from "@astrojs/compiler";

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

const result = await parse(code, {});

console.dir({ ...result }, { depth: Infinity });
