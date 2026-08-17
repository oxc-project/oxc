import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join as pathJoin } from "node:path";
import { pathToFileURL } from "node:url";

import { execa } from "execa";
import { describe, expect, it } from "vitest";

import { createLspConnection } from "../lsp/utils.ts";
import { PACKAGE_ROOT_PATH } from "../utils.ts";

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");
// A fixture with a JS plugin, so that `lintFile` actually reaches the JS side.
const FIXTURE_PATH = pathJoin(PACKAGE_ROOT_PATH, "test/fixtures/js_config_js_plugins");
// A fixture whose plugin kills its own worker isolate part way through linting.
const CRASH_FIXTURE_PATH = pathJoin(import.meta.dirname, "fixtures/worker-crash");
// createOnce + create rules, four files. Used for same-diagnostics / createOnce × K / forget.
const THREADS_FIXTURE_PATH = pathJoin(PACKAGE_ROOT_PATH, "test/fixtures/js_plugins_threads");

async function runOxlint(args: string[], cwd = FIXTURE_PATH, env: Record<string, string> = {}) {
  const { stdout, stderr, exitCode } = await execa("node", [CLI_PATH, ...args], {
    cwd,
    reject: false,
    env: { NO_COLOR: "1", ...env },
  });
  return { stdout, stderr, exitCode };
}

// `js_workers=` is only printed when a test hook is set and workers actually start (`K > 1`).
const TEST_WORKER_ENV = { OXLINT_TEST_CREATE_ONCE_COUNTER: "1" };

function matchLine(stderr: string, prefix: string): number | null {
  const match = stderr.match(new RegExp(`^${prefix}(\\d+)$`, "m"));
  return match ? Number(match[1]) : null;
}

function jsWorkers(stderr: string): number | null {
  return matchLine(stderr, "js_workers=");
}

describe("JS plugin worker startup", () => {
  it("does not start workers when there is only one thread", async () => {
    const { stdout, stderr } = await runOxlint(["--threads=1"], FIXTURE_PATH, TEST_WORKER_ENV);
    expect(jsWorkers(stderr)).toBeNull();
    // Diagnostics still come from the JS plugin, just on the main thread
    expect(stdout).toContain("basic-custom-plugin(no-debugger)");
  });

  it("starts workers for the probed worker count, and lints through them", async (ctx) => {
    const { stdout, stderr } = await runOxlint(["--threads=4"], FIXTURE_PATH, TEST_WORKER_ENV);
    const k = jsWorkers(stderr);
    if (k === null) {
      ctx.skip();
      return;
    }
    expect(k).toBeGreaterThan(1);
    expect(stdout).toContain("basic-custom-plugin(no-debugger)");
  });

  it("starts workers only once for a run", async (ctx) => {
    const { stderr } = await runOxlint(["--threads=4"], FIXTURE_PATH, TEST_WORKER_ENV);
    if (jsWorkers(stderr) === null) {
      ctx.skip();
      return;
    }
    expect(stderr.match(/^js_workers=/gm)).toHaveLength(1);
  });

  it("does not report a worker death when workers are terminated on shutdown", async (ctx) => {
    const { stderr } = await runOxlint(["--threads=4"], FIXTURE_PATH, TEST_WORKER_ENV);
    if (jsWorkers(stderr) === null) {
      ctx.skip();
      return;
    }
    // `terminateJsWorkers` kills every worker once `lint` returns. That is intentional, so it must
    // not be mistaken for a crash.
    expect(stderr).not.toContain("died");
  });
});

describe("JS plugin worker death", () => {
  // The whole point is that a dead worker must not hang the run, so cap how long this may take.
  // Startup plus 13 small files is tens of milliseconds when nothing goes wrong.
  it("finishes instead of hanging when a worker dies mid-lint", async (ctx) => {
    const { stdout, stderr, exitCode } = await runOxlint(
      ["--threads=4"],
      CRASH_FIXTURE_PATH,
      TEST_WORKER_ENV,
    );

    if (jsWorkers(stderr) === null) {
      ctx.skip();
      return;
    }
    // The isolate died while linting this file, so the run reports a plugin error for it
    expect(stdout).toContain("crash.js");
    expect(stdout).toContain("Error running JS plugin");
    expect(exitCode).toBe(1);
  }, 20_000);
});

type SortedDiagnostic = {
  file: string;
  start: number;
  end: number;
  message: string;
  rule: string;
};

function parseJsonDiagnostics(stdout: string): SortedDiagnostic[] {
  const json = JSON.parse(stdout) as {
    diagnostics: Array<{
      message: string;
      code?: string;
      filename: string;
      labels?: Array<{ span: { offset: number; length: number } }>;
    }>;
  };
  return json.diagnostics
    .map((diagnostic) => {
      const span = diagnostic.labels?.[0]?.span;
      const start = span?.offset ?? 0;
      const end = start + (span?.length ?? 0);
      return {
        file: diagnostic.filename,
        start,
        end,
        message: diagnostic.message,
        rule: diagnostic.code ?? "",
      };
    })
    .sort((a, b) => {
      if (a.file !== b.file) return a.file < b.file ? -1 : 1;
      if (a.start !== b.start) return a.start - b.start;
      if (a.end !== b.end) return a.end - b.end;
      if (a.message !== b.message) return a.message < b.message ? -1 : 1;
      if (a.rule !== b.rule) return a.rule < b.rule ? -1 : 1;
      return 0;
    });
}

describe("JS plugin worker fixtures", () => {
  it("reports the same sorted diagnostics at --threads=1 and --threads=4", async (ctx) => {
    const args = ["--format=json", "files"];
    const one = await runOxlint(["--threads=1", ...args], THREADS_FIXTURE_PATH, TEST_WORKER_ENV);
    expect(jsWorkers(one.stderr)).toBeNull();

    const four = await runOxlint(["--threads=4", ...args], THREADS_FIXTURE_PATH, TEST_WORKER_ENV);
    if (jsWorkers(four.stderr) === null) {
      ctx.skip();
      return;
    }

    expect(parseJsonDiagnostics(four.stdout)).toEqual(parseJsonDiagnostics(one.stdout));
  });

  it("runs createOnce once per worker isolate", async (ctx) => {
    const { stderr } = await runOxlint(["--threads=4", "files"], THREADS_FIXTURE_PATH, {
      OXLINT_TEST_CREATE_ONCE_COUNTER: "1",
    });
    if (jsWorkers(stderr) === null) {
      ctx.skip();
      return;
    }

    const k = jsWorkers(stderr);
    const count = matchLine(stderr, "create_once_count=");
    expect(k, `no js_workers= line in stderr:\n${stderr}`).not.toBeNull();
    expect(count, `no create_once_count= line in stderr:\n${stderr}`).toBe(k);
  });

  it("drops occupied JS buffers after each LSP folder rebuild", async (ctx) => {
    // Copy so config toggles cannot race the concurrent e2e snapshot of the committed fixture.
    const fixtureDir = await fs.mkdtemp(pathJoin(tmpdir(), "oxlint-js-plugins-threads-"));
    try {
      await fs.cp(THREADS_FIXTURE_PATH, fixtureDir, { recursive: true });

      let stderr = "";
      await using client = createLspConnection(
        { OXLINT_TEST_OCCUPIED_BUFFERS: "1" },
        {
          extraArgs: ["--threads=2"],
          onStderr: (chunk) => {
            stderr += chunk;
          },
        },
      );

      const workspaceUri = pathToFileURL(fixtureDir).href;
      await client.initialize([{ uri: workspaceUri, name: "js-plugins-threads" }], {
        textDocument: { diagnostic: {} },
        workspace: { diagnostics: { refreshSupport: true } },
      });

      // `js_workers=` is the test hook on this path. `startJsWorkers` only runs when `K > 1`,
      // so a missing line is `K == 1`.
      const k = await waitForLine(() => stderr, "js_workers=", 2000);
      if (k === null) {
        ctx.skip();
        return;
      }

      const filePath = pathJoin(fixtureDir, "files/a.js");
      const fileUri = pathToFileURL(filePath).href;
      const content = await fs.readFile(filePath, "utf8");
      await client.didOpen(fileUri, "javascript", content);
      await client.diagnostic(fileUri);

      const configPath = pathJoin(fixtureDir, ".oxlintrc.json");
      const original = await fs.readFile(configPath, "utf8");
      let drops = 0;
      // Each rebuild must finish (and its occupied count be read) before the next starts.
      // oxlint-disable no-await-in-loop
      for (let i = 0; i < 8; i++) {
        const seen = [...stderr.matchAll(/^occupied_buffers=\d+$/gm)].length;
        const severity = i % 2 === 0 ? "warn" : "error";
        await fs.writeFile(
          configPath,
          original.replace(
            /"threads-fixture\/no-todo": "(?:error|warn)"/,
            `"threads-fixture/no-todo": "${severity}"`,
          ),
        );
        const refresh = client.getWorkspaceRefresh();
        await client.didChangeWatchedFiles([pathToFileURL(configPath).href]);
        await refresh;
        await client.diagnostic(fileUri);

        const count = await waitForOccupiedLine(() => stderr, seen);
        expect(count, `occupied_buffers after drop ${i + 1}`).toBeLessThanOrEqual(k);
        drops++;
      }
      // oxlint-enable no-await-in-loop
      expect(drops).toBe(8);
    } finally {
      await fs.rm(fixtureDir, { recursive: true, force: true });
    }
  }, 30_000);
});

function waitForLine(
  getStderr: () => string,
  prefix: string,
  timeoutMs: number,
): Promise<number | null> {
  return new Promise((resolve) => {
    const start = Date.now();
    const poll = () => {
      const value = matchLine(getStderr(), prefix);
      if (value !== null) {
        resolve(value);
        return;
      }
      if (Date.now() - start > timeoutMs) {
        resolve(null);
        return;
      }
      setTimeout(poll, 25);
    };
    poll();
  });
}

function waitForOccupiedLine(
  getStderr: () => string,
  seen: number,
  timeoutMs = 5000,
): Promise<number> {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const poll = () => {
      const text = getStderr();
      if (/^occupied_query_failed=/m.test(text)) {
        reject(new Error(`occupied query failed (stderr:\n${text})`));
        return;
      }
      const matches = [...text.matchAll(/^occupied_buffers=(\d+)$/gm)];
      if (matches.length > seen) {
        resolve(Number(matches[matches.length - 1][1]));
        return;
      }
      if (Date.now() - start > timeoutMs) {
        reject(new Error(`timed out waiting for occupied_buffers (stderr:\n${text})`));
        return;
      }
      setTimeout(poll, 25);
    };
    poll();
  });
}
