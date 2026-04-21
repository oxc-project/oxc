import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { parseSync } from "./src-js/index.js";

// usage:
// node napi/parser/example.js test.ts

process.chdir(path.join(import.meta.dirname, "../.."));

const args = parseArgs({
  args: process.argv.slice(2),
  allowPositionals: true,
  options: {
    lang: {
      type: "string",
    },
    astType: {
      type: "string",
    },
  },
});

const file = args.positionals[0] ?? "test.js";

const code = fs.readFileSync(file, "utf-8");
const result = parseSync(file, code, args.values);
// oxlint-disable-next-line no-console, typescript-eslint/no-misused-spread
console.dir({ ...result }, { depth: Infinity });
