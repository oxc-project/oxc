// oxlint-disable no-await-in-loop
import { join, relative } from "node:path";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { execa } from "execa";

// NOTE: TS's ESNext is not yet reached to ES2025...
declare global {
  interface RegExpConstructor {
    escape(str: string): string;
  }
}

// Test function for running the CLI with various arguments
export async function runAndSnapshot(cwd: string, testCases: string[][]): Promise<string> {
  const snapshot = [];
  for (const args of testCases) {
    const result = await runCli(cwd, args);
    snapshot.push(formatSnapshot(cwd, args, result));
  }
  return normalizeOutput(snapshot.join("\n"), cwd);
}

// Test function for write mode
export async function runWriteModeAndSnapshot(
  fixtureDir: string,
  files: string[],
  args: string[] = [],
): Promise<string> {
  const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-test-"));

  try {
    await fs.cp(fixtureDir, tempDir, { recursive: true });

    const snapshot = [];
    for (const file of files) {
      const filePath = join(tempDir, file);

      const beforeContent = await fs.readFile(filePath, "utf8");

      await runCli(tempDir, [...args, file]); // Write by default
      const afterContent = await fs.readFile(filePath, "utf8");

      snapshot.push(
        `
--- FILE -----------
${file}
--- BEFORE ---------
${beforeContent}
--- AFTER ----------
${afterContent}
--------------------
`.trim(),
      );
    }

    return snapshot.join("\n\n");
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

// ---

type RunResult = {
  stdout: string;
  stderr: string;
  exitCode: number;
};

async function runCli(cwd: string, args: string[]): Promise<RunResult> {
  const cliPath = join(import.meta.dirname, "..", "dist", "cli.js");

  const result = await execa("node", [cliPath, ...args], {
    cwd,
    reject: false,
  });

  return {
    stdout: result.stdout,
    stderr: result.stderr,
    exitCode: result.exitCode ?? -1,
  };
}

function normalizeOutput(output: string, cwd: string): string {
  let normalized = output;

  // Normalize timing information
  normalized = normalized.replace(/\d+(?:\.\d+)?s|\d+ms/g, "<variable>ms");
  // Normalize thread count (e.g., "using 8 threads" -> "using 1 threads")
  normalized = normalized.replace(/using \d+ threads/g, "using 1 threads");
  // Normalize path separators (Windows compatibility)
  normalized = normalized.replace(/\\/g, "/");
  // Replace absolute paths
  const cwdPath = cwd.replace(/\\/g, "/");
  normalized = normalized.replace(new RegExp(RegExp.escape(cwdPath), "g"), "<cwd>");
  // Replace repo root path
  const repoRoot = join(import.meta.dirname, "..", "..", "..");
  const rootPath = repoRoot.replace(/\\/g, "/");
  normalized = normalized.replace(new RegExp(RegExp.escape(rootPath), "g"), "<cwd>");

  return normalized;
}

function formatSnapshot(
  cwd: string,
  args: string[],
  { stdout, stderr, exitCode }: RunResult,
): string {
  const testsRoot = join(import.meta.dirname);
  const relativeDir = relative(testsRoot, cwd) || ".";

  return `
--------------------
arguments: ${args.join(" ")}
working directory: ${relativeDir}
exit code: ${exitCode}
--- STDOUT ---------
${stdout}
--- STDERR ---------
${stderr}
--------------------
`.trim();
}
