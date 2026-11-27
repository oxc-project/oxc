// oxlint-disable no-console
// A markdown table of scenarios-*.json results: median (p95) wall time per scenario and target,
// warm-ups excluded, with the project, file count and load average of each file.
//
// usage: node bench/cache/report-scenarios.ts <results.json>...

import { readFileSync } from "node:fs";
import { stats } from "./lib.ts";

type Point = {
  scenario: string;
  target: string;
  warmup: boolean;
  ms: number;
  files: number | null;
  loadavg: number[];
};

const fmt = (ms: number) => (ms >= 1000 ? `${(ms / 1000).toFixed(2)} s` : `${ms.toFixed(0)} ms`);

for (const file of process.argv.slice(2)) {
  const data = JSON.parse(readFileSync(file, "utf8"));
  const points: Point[] = data.points;
  const targets: string[] = data.config.targets;
  const scenarios = [...new Set(points.map((p) => p.scenario))];
  const files = points.find((p) => p.files !== null)?.files;
  const load = points.map((p) => p.loadavg[0]!);
  console.log(
    `\n${data.project.path.split("/").pop()} @ ${data.project.commit.slice(0, 10)}, ${files} files; ${data.machine.cpu}, ${data.machine.cores} cores; load ${Math.min(...load)}-${Math.max(...load)}; ${file.split("/").pop()}\n`,
  );
  console.log(`| scenario | ${targets.join(" | ")} |`);
  console.log(`|---|${targets.map(() => "---").join("|")}|`);
  for (const scenario of scenarios) {
    const cells = targets.map((target) => {
      const ms = points
        .filter((p) => p.scenario === scenario && p.target === target && !p.warmup)
        .map((p) => p.ms);
      if (!ms.length) return "";
      const s = stats(ms);
      return `${fmt(s.p50!)} (${fmt(s.p95!)})`;
    });
    console.log(`| ${scenario} | ${cells.join(" | ")} |`);
  }
}
