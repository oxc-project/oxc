// oxlint-disable vitest/expect-expect -- Benchmark tests measure runtime.
// Build release printers and parser before running this file. Parsing stays outside print timings.
import { describe, test } from "vitest";
import { parseSync } from "oxc-parser";
import { printSync } from "./dist/index.js";
import { commentFixtures } from "../../napi/parser/bench/fixtures.js";

let flattenSink = 0;
for (const { filename, source } of await commentFixtures()) {
  const options = { ts: /\.tsx?$/.test(filename), jsx: filename.endsWith("x") };
  const plain = parseSync(filename, source, {
    preserveParens: false,
    experimentalRawTransfer: true,
  });
  const attached = parseSync(filename, source, {
    preserveParens: false,
    experimentalRawTransfer: true,
    attachComments: true,
  });
  if (plain.errors.length || attached.errors.length) throw new Error(`Cannot parse ${filename}`);

  describe.for([false, true])(`${filename} maps=%s`, (sourcemap) => {
    for (const comments of [false, true]) {
      const { program } = comments ? attached : plain;
      const printOptions = {
        ...options,
        comments,
        sourcemap,
        sourceText: source,
        sourceFilename: filename,
      };
      const name = comments ? "attached" : "disabled";
      test(`${name}`, { timeout: 60_000 }, async ({ bench }) => {
        await bench(name, () => {
          flattenSink ^= printSync(program, printOptions).code.charCodeAt(0);
        }).run({ time: 3000, warmupTime: 1000 });
      });
    }
  });
}
