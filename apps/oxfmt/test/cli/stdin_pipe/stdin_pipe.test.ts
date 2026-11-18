import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { execa } from "execa";

const CLI_PATH = join(import.meta.dirname, "..", "..", "..", "dist", "cli.js");
const FIXTURE = join(import.meta.dirname, "fixtures", "parser.ts");

const hasBun = spawnSync("bun", ["--version"], { stdio: "ignore" }).status === 0;

// https://github.com/oxc-project/oxc/issues/17939
describe("stdin pipe", () => {
  // NOTE: This test uses Unix commands (`cat`, `wc`) via shell pipes.
  // On our Windows CI (GitHub Actions), these resolve from Git for Windows in PATH.
  // May fail on some Windows environments without Unix tools in PATH.
  it("should not report WouldBlock error on large file piped to wc", async () => {
    const cmd = `cat "${FIXTURE}" | node "${CLI_PATH}" --stdin-filepath=parser.ts | wc -l`;
    const result = await execa({ shell: true, reject: false })`${cmd}`;

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
  });

  // Formatted output larger than the pipe buffer (64 KiB) must be written to
  // stdout in full, not truncated at the buffer size.
  // https://github.com/oxc-project/oxc/issues/29846
  describe("output larger than the pipe buffer", () => {
    const input = `${JSON.stringify(
      Object.fromEntries(
        Array.from({ length: 4000 }, (_, index) => [`key${index}`, "x".repeat(40)]),
      ),
      null,
      2,
    )}\n`;

    async function expectCompleteOutput(runtimeExecutable: string) {
      const result = await execa(runtimeExecutable, [CLI_PATH, "--stdin-filepath=big.json"], {
        input,
        reject: false,
      });

      expect(result.exitCode).toBe(0);
      expect(result.stderr).toBe("");
      // Truncated output stops at the pipe buffer size and misses the end of the document.
      expect(Buffer.byteLength(result.stdout)).toBeGreaterThan(65_536);
      expect(result.stdout).toContain('"key3999"');
    }

    it("should write the full formatted output to a piped stdout", async () => {
      await expectCompleteOutput("node");
    });

    // `bun` switches piped stdio to non-blocking mode and, unlike Node, provides
    // no `setBlocking()` API for it, so this is the runtime that regresses.
    // Skipped when bun is not installed (for example on Windows CI).
    it.runIf(hasBun)(
      "should write the full formatted output to a piped stdout under bun",
      async () => {
        await expectCompleteOutput("bun");
      },
    );
  });
});
