import { join as pathJoin } from "node:path";

import { execa } from "execa";
import { describe, expect, it } from "vitest";

import { PACKAGE_ROOT_PATH } from "../utils.ts";

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");
// A fixture with a JS plugin, so that `lintFile` actually reaches the JS side.
const FIXTURE_PATH = pathJoin(PACKAGE_ROOT_PATH, "test/fixtures/js_config_js_plugins");
// A fixture whose plugin kills its own worker isolate part way through linting.
const CRASH_FIXTURE_PATH = pathJoin(import.meta.dirname, "fixtures/worker-crash");

async function runOxlint(args: string[], cwd = FIXTURE_PATH) {
  const { stdout, stderr, exitCode } = await execa("node", [CLI_PATH, ...args], {
    cwd,
    reject: false,
    env: { NO_COLOR: "1" },
  });
  return { stdout, stderr, exitCode };
}

// Rust prints `worker_boot_ms=<n>` once per run: `0` when JS plugins run on the main JS thread,
// and the time spent booting worker isolates otherwise.
function workerBootMs(stderr: string): number {
  const match = stderr.match(/^worker_boot_ms=(\d+) /m);
  expect(match, `no \`worker_boot_ms\` line in stderr:\n${stderr}`).not.toBeNull();
  return Number(match![1]);
}

describe("JS plugin worker startup", () => {
  it("does not start workers when there is only one thread", async () => {
    const { stdout, stderr } = await runOxlint(["--threads=1"]);
    expect(workerBootMs(stderr)).toBe(0);
    // Diagnostics still come from the JS plugin, just on the main thread
    expect(stdout).toContain("basic-custom-plugin(no-debugger)");
  });

  it("starts workers for the probed worker count, and lints through them", async () => {
    const { stdout, stderr } = await runOxlint(["--threads=4"]);
    // Booting real `Worker`s always takes at least a millisecond
    expect(workerBootMs(stderr)).toBeGreaterThan(0);
    expect(stdout).toContain("basic-custom-plugin(no-debugger)");
  });

  it("starts workers only once for a run", async () => {
    const { stderr } = await runOxlint(["--threads=4"]);
    expect(stderr.match(/^worker_boot_ms=/gm)).toHaveLength(1);
  });

  it("does not report a worker death when workers are terminated on shutdown", async () => {
    const { stderr } = await runOxlint(["--threads=4"]);
    // `terminateJsWorkers` kills every worker once `lint` returns. That is intentional, so it must
    // not be mistaken for a crash.
    expect(stderr).not.toContain("died");
  });
});

describe("JS plugin worker death", () => {
  // The whole point is that a dead worker must not hang the run, so cap how long this may take.
  // Startup plus 13 small files is tens of milliseconds when nothing goes wrong.
  it("finishes instead of hanging when a worker dies mid-lint", async () => {
    const { stdout, stderr, exitCode } = await runOxlint(["--threads=4"], CRASH_FIXTURE_PATH);

    expect(workerBootMs(stderr)).toBeGreaterThan(0);
    // The isolate died while linting this file, so the run reports a plugin error for it
    expect(stdout).toContain("crash.js");
    expect(stdout).toContain("Error running JS plugin");
    expect(exitCode).toBe(1);
  }, 20_000);
});
