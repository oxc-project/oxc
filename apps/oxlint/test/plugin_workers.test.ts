import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execa } from "execa";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { PACKAGE_ROOT_PATH } from "./utils.ts";

const CLI_PATH = join(PACKAGE_ROOT_PATH, "dist/cli.js");

// Use real child processes: worker startup, N-API callbacks, and shutdown must all work together.
describe("JS plugin workers", () => {
  let cwd: string;

  beforeEach(async () => {
    cwd = await mkdtemp(join(tmpdir(), "oxlint-plugin-workers-"));
    await mkdir(join(cwd, "files"));
    await Promise.all(
      Array.from({ length: 32 }, (_, index) =>
        writeFile(join(cwd, "files", `${index}.js`), "debugger;\n"),
      ),
    );
    await writeFile(
      join(cwd, ".oxlintrc.json"),
      JSON.stringify({
        plugins: [],
        categories: { correctness: "off" },
        jsPlugins: ["./plugin.mjs"],
        rules: { "workers/check": ["error", { message: "configured message" }] },
      }),
    );
  });

  afterEach(async () => {
    await rm(cwd, { recursive: true, force: true });
  });

  function lint(threads: number, ...args: string[]) {
    return execa(
      process.execPath,
      [CLI_PATH, "--threads", "4", "--js-plugin-threads", String(threads), "files", ...args],
      { cwd, reject: false, timeout: 10_000 },
    );
  }

  async function plugin(source: string) {
    await writeFile(join(cwd, "plugin.mjs"), source);
  }

  it("replays options on workers and preserves diagnostics and fixes", async () => {
    await plugin(`
      import { threadId } from 'node:worker_threads';
      export default {
        meta: { name: 'workers' },
        rules: {
          check: {
            meta: { schema: [{ type: 'object' }], fixable: 'code' },
            create(context) {
              return {
                DebuggerStatement(node) {
                  context.report({
                    node,
                    message: context.options[0].message + ' host=' + threadId,
                    fix(fixer) { return fixer.remove(node); },
                  });
                },
              };
            },
          },
        },
      };
    `);
    const serial = await lint(1, "--format", "json");
    const parallel = await lint(4, "--format", "json");
    for (const result of [serial, parallel]) {
      expect(result.timedOut).toBe(false);
      expect(result.exitCode).toBe(1);
      expect(result.stderr).toBe("");
    }
    const serialOutput = JSON.parse(serial.stdout);
    const parallelOutput = JSON.parse(parallel.stdout);
    expect(serialOutput.diagnostics).toHaveLength(32);
    expect(parallelOutput.diagnostics).toHaveLength(32);
    const normalize = (diagnostics: { message: string }[]) =>
      diagnostics
        .map((diagnostic) =>
          JSON.stringify({
            ...diagnostic,
            message: diagnostic.message.replace(/ host=\d+$/, ""),
          }),
        )
        .sort();
    expect(normalize(parallelOutput.diagnostics)).toEqual(normalize(serialOutput.diagnostics));
    const messages = parallelOutput.diagnostics.map(
      (diagnostic: { message: string }) => diagnostic.message,
    );
    expect(
      messages.every((message: string) => message.startsWith("configured message host=")),
    ).toBe(true);
    // A serial fallback can produce correct diagnostics too; require actual worker execution.
    expect(messages.some((message: string) => /host=[1-9]\d*$/.test(message))).toBe(true);

    const fixed = await lint(4, "--fix");
    expect(fixed.timedOut).toBe(false);
    expect(fixed.exitCode).toBe(0);
    const files = await Promise.all(
      Array.from({ length: 32 }, (_, index) => readFile(join(cwd, "files", `${index}.js`), "utf8")),
    );
    expect(files).toEqual(Array.from({ length: 32 }, () => "\n"));
  });

  it.each([
    [
      "import failure",
      "if (!isMainThread) throw new Error('worker import failed');",
      "worker import failed",
    ],
    ["registration mismatch", "", "Plugin registration did not match the main isolate"],
  ])("reports worker startup %s and shuts down", async (kind, initialization, message) => {
    await plugin(`
      import { isMainThread } from 'node:worker_threads';
      ${initialization}
      export default {
        meta: { name: 'workers' },
        rules: {
          [${kind === "registration mismatch" ? "isMainThread ? 'check' : 'different'" : "'check'"}]: {
            meta: { schema: [{ type: 'object' }] },
            create() { return {}; },
          },
        },
      };
    `);
    const serial = await lint(1);
    expect(serial.exitCode).toBe(0);
    const result = await lint(4);
    expect(result.timedOut).toBe(false);
    expect(result.exitCode).toBe(1);
    expect(result.stdout + result.stderr).toContain(message);
  });

  it("reports a worker exiting inside a visitor instead of waiting forever", async () => {
    await plugin(`
      import { isMainThread } from 'node:worker_threads';
      export default {
        meta: { name: 'workers' },
        rules: {
          check: {
            meta: { schema: [{ type: 'object' }] },
            create() {
              return { DebuggerStatement() { if (!isMainThread) process.exit(23); } };
            },
          },
        },
      };
    `);
    const serial = await lint(1);
    expect(serial.exitCode).toBe(0);
    const result = await lint(4);
    expect(result.timedOut).toBe(false);
    expect(result.exitCode).toBe(1);
    expect(result.stdout + result.stderr).toContain("exited unexpectedly with code 23");
  });

  it("does not start extra workers when only one file is selected", async () => {
    await plugin(`
      import { isMainThread } from 'node:worker_threads';
      if (!isMainThread) throw new Error('unnecessary worker');
      export default {
        meta: { name: 'workers' },
        rules: {
          check: {
            meta: { schema: [{ type: 'object' }] },
            create() { return {}; },
          },
        },
      };
    `);
    await rm(join(cwd, "files"), { recursive: true });
    await mkdir(join(cwd, "files"));
    await writeFile(join(cwd, "files", "only.js"), "debugger;\n");
    const result = await lint(4, "--format", "json");
    expect(result.timedOut).toBe(false);
    expect(result.exitCode).toBe(0);
    expect(JSON.parse(result.stdout).number_of_files).toBe(1);
  });
});
