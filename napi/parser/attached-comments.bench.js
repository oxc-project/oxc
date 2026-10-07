// oxlint-disable vitest/expect-expect -- Benchmark tests measure runtime.
import { describe, test } from "vitest";
import { parseSync } from "./src-js/index.js";
import { commentFixtures } from "./bench/fixtures.js";

for (const { filename, source } of await commentFixtures()) {
  describe(`${filename}`, () => {
    for (const experimentalRawTransfer of [false, true]) {
      for (const attachComments of [false, true]) {
        const name = `${experimentalRawTransfer ? "eager" : "json"}/${attachComments ? "attached" : "disabled"}`;
        test(`${name}`, { timeout: 60_000 }, async ({ bench }) => {
          await bench(name, () => {
            const result = parseSync(filename, source, {
              experimentalRawTransfer,
              attachComments,
            });
            // Force JSON's lazy getters as well as eager deserialization.
            return [result.program, result.comments, result.module, result.errors];
          }).run({ time: 3000, warmupTime: 1000 });
        });
      }
    }
  });
}
