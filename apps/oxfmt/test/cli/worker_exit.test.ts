import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { pathToFileURL } from "node:url";
import { execa } from "execa";
import { describe, expect, it } from "vitest";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");
const FIXTURE_DIR = join(import.meta.dirname, "worker_exit");

function isAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

describe("oxfmt CLI", () => {
  it("leaves no Prettier worker running after it exits, even if a worker keeps a timer", async () => {
    const tmp = await fs.mkdtemp(join(tmpdir(), "oxfmt-worker-exit-"));
    const pidFile = join(tmp, "pids");
    const preload = pathToFileURL(join(FIXTURE_DIR, "keep-alive.mjs")).href;

    try {
      await execa("node", [CLI_PATH, "--check", "--threads", "2", "fixtures"], {
        cwd: FIXTURE_DIR,
        env: {
          NODE_OPTIONS: `--import="${preload}"`,
          OXFMT_TEST_WORKER_PID_FILE: pidFile,
        },
        reject: false,
      });

      const pids = (await fs.readFile(pidFile, "utf8")).trim().split("\n").map(Number);
      // Workers exit after the CLI is gone, so poll for up to 2 s.
      let alive = pids;
      for (let i = 0; i < 40 && alive.length > 0; i++) {
        // oxlint-disable-next-line no-await-in-loop -- polling is sequential by design
        await sleep(50);
        alive = alive.filter(isAlive);
      }
      for (const pid of alive) process.kill(pid, "SIGKILL");

      expect(alive).toEqual([]);
    } finally {
      await fs.rm(tmp, { recursive: true, force: true });
    }
  });
});
