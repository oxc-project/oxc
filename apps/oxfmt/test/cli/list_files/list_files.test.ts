import { describe, expect, it } from "vitest";
import { join, relative } from "node:path";
import { runCli } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

async function runAndSnapshot(cwd: string, testCases: string[][]): Promise<string> {
  const snapshot = [];
  for (const args of testCases) {
    const result = await runCli(cwd, args);
    snapshot.push(formatSnapshot(cwd, args, result));
  }
  return normalizeOutput(snapshot.join("\n"), cwd);
}

function normalizeOutput(output: string, cwd: string): string {
  const cwdPath = cwd.replace(/\\/g, "/");

  return (
    output
      .replace(/\d+(?:\.\d+)?s|\d+ms/g, "<variable>ms")
      .replace(/\\/g, "/")
      .replace(new RegExp(escapeRegExp(cwdPath), "g"), "<cwd>")
      .replace(/[^\S\n]+$/gm, "")
  );
}

function formatSnapshot(
  cwd: string,
  args: string[],
  result: { stdout: string; stderr: string; exitCode?: number | null },
): string {
  const relativeDir = relative(import.meta.dirname, cwd) || ".";

  return `
--------------------
arguments: ${args.join(" ")}
working directory: ${relativeDir}
exit code: ${result.exitCode ?? -1}
--- STDOUT ---------
${result.stdout}
--- STDERR ---------
${result.stderr}
--------------------
`.trim();
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

describe("list_files", () => {
  it("should list selected target files in sorted order", async () => {
    const testCases = [
      ["--list-files"],
      ["--list-files", "z-last.js", "a-first.ts"],
      ["--list-files", "--ignore-path", "custom.ignore"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should preserve no-file semantics", async () => {
    const testCases = [
      ["--list-files", "__nonexistent__/**/*.js"],
      ["--list-files", "--no-error-on-unmatched-pattern", "__nonexistent__/**/*.js"],
      ["--list-files", "--ignore-path", "custom.ignore", "ignored-by-ignore-path/skip.js"],
      [
        "--list-files",
        "--no-error-on-unmatched-pattern",
        "--ignore-path",
        "custom.ignore",
        "ignored-by-ignore-path/skip.js",
      ],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
