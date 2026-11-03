// oxlint-disable no-console
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const [label = "final", destination] = process.argv.slice(2);
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const samples = JSON.parse(
  fs.readFileSync(path.join(root, "target/raw-result", label, "samples.json")),
);
const median = (values) => {
  const sorted = values.toSorted((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};
const rows = [];
for (const mode of new Set(samples.map((row) => row.mode))) {
  const variants = [
    ...new Set(samples.filter((row) => row.mode === mode).map((row) => row.variant)),
  ];
  for (const variant of variants) {
    const selected = samples.filter((row) => row.mode === mode && row.variant === variant);
    const elapsed = selected.map((row) => row.elapsedMilliseconds);
    rows.push({
      mode,
      variant,
      repetitions: selected.length,
      medianMilliseconds: median(elapsed),
      minMilliseconds: Math.min(...elapsed),
      maxMilliseconds: Math.max(...elapsed),
      medianCpuMilliseconds: median(selected.map((row) => row.cpuMilliseconds)),
      medianMaxRssMiB: median(selected.map((row) => row.maxRssMiB)),
      medianRetainedHeapMiB: median(selected.map((row) => row.heapAfterGc.heapUsed / 2 ** 20)),
    });
  }
}
const report = {
  label,
  build: JSON.parse(fs.readFileSync(path.join(root, "target/raw-result", label, "build.json"))),
  rows,
  samples,
};
if (destination) fs.writeFileSync(destination, JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report.rows, null, 2));
