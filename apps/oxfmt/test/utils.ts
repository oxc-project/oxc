import { join, relative } from "node:path";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { execa } from "execa";
import type { Result } from "execa";

// NOTE: TS's ESNext is not yet reached to ES2025...
declare global {
  interface RegExpConstructor {
    escape(str: string): string;
  }
}

export function runCli(cwd: string, args: string[]) {
  const cliPath = join(import.meta.dirname, "..", "dist", "cli.js");
  return execa("node", [cliPath, ...args], {
    cwd,
    reject: false,
    timeout: 5000,
  });
}

// Test function for running the CLI with various arguments
export async function runAndSnapshot(cwd: string, testCases: string[][]): Promise<string> {
  // Run all CLI calls in parallel
  const results = await Promise.all(testCases.map((args) => runCli(cwd, args)));
  const snapshot = results.map((result, i) => formatSnapshot(cwd, testCases[i], result));
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

    // Read all "before" contents in parallel
    const beforeContents = await Promise.all(
      files.map((file) => fs.readFile(join(tempDir, file), "utf8")),
    );

    // Run CLI once with all files (instead of once per file)
    await runCli(tempDir, [...args, ...files]);

    // Read all "after" contents in parallel
    const afterContents = await Promise.all(
      files.map((file) => fs.readFile(join(tempDir, file), "utf8")),
    );

    // Build snapshot
    const snapshot = files.map(
      (file, i) =>
        `--- FILE -----------
${file}
--- BEFORE ---------
${beforeContents[i]}
--- AFTER ----------
${afterContents[i]}
--------------------`,
    );

    return snapshot.join("\n\n");
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

// ---

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

function formatSnapshot(cwd: string, args: string[], { stdout, stderr, exitCode }: Result): string {
  const testsRoot = join(import.meta.dirname);
  const relativeDir = relative(testsRoot, cwd) || ".";

  return `
--------------------
arguments: ${args.join(" ")}
working directory: ${relativeDir}
exit code: ${exitCode ?? -1}
--- STDOUT ---------
${String(stdout)}
--- STDERR ---------
${String(stderr)}
--------------------
`.trim();
}
