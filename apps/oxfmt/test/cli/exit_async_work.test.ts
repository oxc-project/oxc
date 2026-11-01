import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execa } from "execa";
import { describe, expect, it } from "vitest";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");
const FIXTURE_DIR = join(import.meta.dirname, "exit_async_work");

describe("oxfmt CLI", () => {
  it("runs async work that the config leaves before it exits", async () => {
    const tmp = await fs.mkdtemp(join(tmpdir(), "oxfmt-exit-async-work-"));
    const outFile = join(tmp, "out.bin");

    try {
      // `a.vue` is delegated to a Prettier worker, so the exit must also not wait for that worker.
      const result = await execa("node", [CLI_PATH, "--check", "--threads", "2", "fixtures"], {
        cwd: FIXTURE_DIR,
        env: { OXFMT_TEST_OUT_FILE: outFile },
        reject: false,
        timeout: 30_000,
      });

      expect(result.timedOut).toBe(false);
      // `a.vue` is not formatted (exit code 1), but the config's `beforeExit` work sets 7 after that.
      expect(result.exitCode).toBe(7);
      expect((await fs.stat(outFile)).size).toBe(64 * 65_536);
    } finally {
      await fs.rm(tmp, { recursive: true, force: true });
    }
  });
});
