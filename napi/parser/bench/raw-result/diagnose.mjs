// oxlint-disable no-console
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import "./prepare.mjs";

const [directory, label = "diagnostics"] = process.argv.slice(2);
assert.ok(directory, "diagnose.mjs CORPUS [LABEL]");
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const output = path.join(root, "target/raw-result", label);
fs.mkdirSync(output, { recursive: true });
fs.copyFileSync(path.join(root, "target/raw-result/build.json"), path.join(output, "build.json"));
const lifetime = [];
for (let repeat = 0; repeat < 3; repeat++) {
  for (const variant of ["closure", "field", "define", "bound", "private", "plain"]) {
    const child = spawnSync(
      process.execPath,
      [
        "--expose-gc",
        "--allow-natives-syntax",
        "--trace-gc-nvp",
        path.join(import.meta.dirname, "lifetime.mjs"),
        variant,
      ],
      { encoding: "utf8" },
    );
    fs.writeFileSync(
      path.join(output, `lifetime-${variant}-${repeat + 1}.log`),
      child.stdout + child.stderr,
    );
    assert.equal(child.status, 0, child.stderr);
    const row = JSON.parse(child.stdout.trim().split("\n").at(-1));
    const measured = child.stdout.split("LIFETIME_START\n")[1].split("LIFETIME_END")[0];
    const collections = [...measured.matchAll(/GC: (\{[^\n]+\})/g)].map(
      (match) => JSON.parse(match[1]).gc,
    );
    assert.deepEqual(collections, ["s", "s", "mc"]);
    lifetime.push({ repeat: repeat + 1, ...row, collections });
  }
}
const gc = [];
for (const flags of [[], ["--no-allocation-site-pretenuring"], ["--no-maglev"]]) {
  for (const variant of ["baseline", "poc"]) {
    const flagLabel = flags[0]?.slice(2) ?? "default";
    const filename = path.join(output, `gc-${variant}-${flagLabel}.json`);
    const child = spawnSync(
      process.execPath,
      [
        "--expose-gc",
        "--max-old-space-size=6144",
        "--trace-gc-nvp",
        ...flags,
        path.join(import.meta.dirname, "worker.mjs"),
        directory,
        variant,
        "raw",
        filename,
      ],
      { encoding: "utf8", maxBuffer: 32 * 2 ** 20 },
    );
    fs.writeFileSync(filename + ".log", child.stdout + child.stderr);
    assert.equal(child.status, 0, child.stderr);
    const measured = child.stdout
      .split("OXC_MEASUREMENT_START\n")[1]
      .split("OXC_MEASUREMENT_END")[0];
    const events = [...measured.matchAll(/GC: (\{[^\n]+\})/g)].map((match) => JSON.parse(match[1]));
    assert.ok(events.length);
    const counts = {};
    const totals = { pause: 0, allocated: 0, promoted: 0 };
    for (const event of events) {
      counts[event.gc] = (counts[event.gc] ?? 0) + 1;
      for (const key of Object.keys(totals)) totals[key] += event[key] ?? 0;
    }
    gc.push({ variant, flags, counts, totals });
    console.log(JSON.stringify(gc.at(-1)));
  }
}
fs.writeFileSync(
  path.join(output, "summary.json"),
  JSON.stringify({ lifetime, gc }, null, 2) + "\n",
);
