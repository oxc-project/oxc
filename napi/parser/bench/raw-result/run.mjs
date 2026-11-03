// oxlint-disable no-console
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import "./prepare.mjs";

const [
  corpus,
  variantsText = "baseline,helper,define,bound,private,plain",
  modesText = "raw",
  runsText = "1",
  label = "screen",
  ...flags
] = process.argv.slice(2);
assert.ok(corpus, "run.mjs CORPUS [VARIANTS] [MODES] [RUNS] [LABEL] [...NODE_FLAGS]");
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const destination = path.join(root, "target/raw-result", label);
fs.mkdirSync(destination, { recursive: true });
fs.copyFileSync(
  path.join(root, "target/raw-result/build.json"),
  path.join(destination, "build.json"),
);
const variants = variantsText.split(",");
const modes = modesText.split(",");
const results = [];
for (let run = 0; run < Number(runsText); run++) {
  for (const mode of modes) {
    for (let i = 0; i < variants.length; i++) {
      const variant = variants[(i + run) % variants.length];
      const output = path.join(destination, `${mode}-${variant}-${run + 1}.json`);
      const child = spawnSync(
        process.execPath,
        [
          "--expose-gc",
          "--max-old-space-size=6144",
          ...flags,
          path.join(import.meta.dirname, "worker.mjs"),
          corpus,
          variant,
          mode,
          output,
          process.env.OXC_BENCH_LIMIT ?? "all",
          process.env.OXC_BENCH_PROFILE ?? "none",
        ],
        { cwd: root, encoding: "utf8", maxBuffer: 32 * 2 ** 20 },
      );
      fs.writeFileSync(output + ".log", child.stdout + child.stderr);
      if (child.error) throw child.error;
      assert.equal(child.status, 0, `${output}: ${child.stderr}`);
      const result = JSON.parse(fs.readFileSync(output));
      const previous = results.find((row) => row.mode === mode);
      if (previous) {
        assert.deepEqual(result.corpus, previous.corpus);
        assert.equal(result.checksum, previous.checksum);
      }
      results.push(result);
      fs.writeFileSync(
        path.join(destination, "samples.json"),
        JSON.stringify(results, null, 2) + "\n",
      );
      console.log(
        JSON.stringify({
          run: run + 1,
          mode,
          variant,
          ms: result.elapsedMilliseconds,
          rssMiB: result.maxRssMiB,
          checksum: result.checksum,
        }),
      );
    }
  }
}
