// oxlint-disable no-console
// Run one oxlint CLI configuration N times and report the median `lint_ms` from stderr.
//
// `lint_ms` is the spike timer around `lint_files`, so it excludes process boot, config load and
// JS plugin import. That isolates the part worker isolates are supposed to change. The first run
// is a discarded warmup.
//
// Usage:
//   node lint-ms.mjs --cli <dist/cli.js> --config <rc.json> --files <dir> \
//     --threads <n> [--runs 11] [--label name] [--json]

import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

function arg(name, fallback) {
  const index = process.argv.indexOf(`--${name}`);
  return index === -1 ? fallback : process.argv[index + 1];
}

const cli = arg("cli");
const config = arg("config");
const files = arg("files");
const threads = arg("threads");
const runs = Number(arg("runs", "11"));
const label = arg("label", "");
const asJson = process.argv.includes("--json");

if (!cli || !config || !files || !threads) {
  console.error("missing --cli / --config / --files / --threads");
  process.exit(1);
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = sorted.length >> 1;
  return sorted.length % 2 === 1 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

const lintMs = [];
const bootMs = [];
const loadMs = [];

for (let run = 0; run < runs + 1; run++) {
  // Runs must be sequential: two linters sharing the cores would measure contention.
  // oxlint-disable-next-line no-await-in-loop
  const { stderr } = await execFileAsync(
    "node",
    [cli, "-c", config, `--threads=${threads}`, "--silent", files],
    { maxBuffer: 64 * 1024 * 1024, env: { ...process.env, NO_COLOR: "1" } },
    // A corpus with a parse error exits non-zero; the timings are still valid.
  ).catch((err) => err);

  const lint = stderr?.match(/^lint_ms=(\d+)$/m);
  const header = stderr?.match(/^worker_boot_ms=(\d+) plugin_load_ms=(\d+)$/m);
  if (!lint || !header) {
    console.error(`run ${run}: missing timer lines. stderr:\n${stderr}`);
    process.exit(1);
  }
  if (run === 0) continue;
  lintMs.push(Number(lint[1]));
  bootMs.push(Number(header[1]));
  loadMs.push(Number(header[2]));
}

const summarize = (values) => ({
  median: median(values),
  min: Math.min(...values),
  max: Math.max(...values),
});

const result = {
  label,
  threads: Number(threads),
  runs,
  lint_ms: summarize(lintMs),
  worker_boot_ms: summarize(bootMs),
  plugin_load_ms: summarize(loadMs),
  samples: lintMs,
};

if (asJson) {
  console.log(JSON.stringify(result));
} else {
  console.log(
    `${label} threads=${threads} lint_ms median=${result.lint_ms.median} ` +
      `min=${result.lint_ms.min} max=${result.lint_ms.max} ` +
      `boot=${result.worker_boot_ms.median} load=${result.plugin_load_ms.median}`,
  );
}
