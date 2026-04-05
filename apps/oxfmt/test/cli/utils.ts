// oxlint-disable no-await-in-loop

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

const PACKAGE_ROOT_PATH = join(import.meta.dirname, "..", "..");
const BUILT_CLI_PATH = join(PACKAGE_ROOT_PATH, "dist", "cli.js");
const RAW_CLI_PATH = join(PACKAGE_ROOT_PATH, "src-js", "cli.ts");
const TSX_CLI_PATH = join(PACKAGE_ROOT_PATH, "node_modules", "tsx", "dist", "cli.mjs");

export type CliLaunchMode = "built" | "raw";

type RunCliStdinOptions = {
  cwd?: string;
  pipe?: string;
  args?: string[];
};

function getCliCommand(launchMode: CliLaunchMode): { command: string; args: string[] } {
  return launchMode === "built"
    ? { command: "node", args: [BUILT_CLI_PATH] }
    : { command: process.execPath, args: [TSX_CLI_PATH, RAW_CLI_PATH] };
}

export function runCliWithLaunchMode(cwd: string, args: string[], launchMode: CliLaunchMode) {
  const cliCommand = getCliCommand(launchMode);
  return execa(cliCommand.command, [...cliCommand.args, ...args, "--threads=1"], {
    cwd,
    reject: false,
    timeout: 5000,
  });
}

export function runCli(cwd: string, args: string[]) {
  return runCliWithLaunchMode(cwd, args, "built");
}

function quoteShellArg(arg: string): string {
  return `'${arg.replaceAll("'", `'"'"'`)}'`;
}

export function runCliStdinWithLaunchMode(
  input: string,
  filepath: string,
  launchMode: CliLaunchMode,
  pipeOrOptions?: string | RunCliStdinOptions,
) {
  const options =
    typeof pipeOrOptions === "string"
      ? { pipe: pipeOrOptions }
      : pipeOrOptions ?? {};

  const cliArgs = [...(options.args ?? []), `--stdin-filepath=${filepath}`];
  const cliCommand = getCliCommand(launchMode);

  if (options.pipe) {
    const cmd = [cliCommand.command, ...cliCommand.args, ...cliArgs].map(quoteShellArg).join(" ");
    return execa({ shell: true, reject: false, input, cwd: options.cwd, stripFinalNewline: false })`${cmd} | ${options.pipe}`;
  }

  return execa(cliCommand.command, [...cliCommand.args, ...cliArgs], {
    cwd: options.cwd,
    reject: false,
    input,
    stripFinalNewline: false,
  });
}

export function runCliStdin(
  input: string,
  filepath: string,
  pipeOrOptions?: string | RunCliStdinOptions,
) {
  return runCliStdinWithLaunchMode(input, filepath, "built", pipeOrOptions);
}

// Test function for running the CLI with various arguments
export async function runAndSnapshot(
  cwd: string,
  testCases: string[][],
  launchMode: CliLaunchMode = "built",
): Promise<string> {
  const snapshot = [];
  for (const args of testCases) {
    const result = await runCliWithLaunchMode(cwd, args, launchMode);
    snapshot.push(formatSnapshot(cwd, args, result));
  }
  return normalizeOutput(snapshot.join("\n"), cwd);
}

// Test function for write mode
export async function runWriteModeAndSnapshot(
  fixtureDir: string,
  files: string[],
  args: string[] = [],
  launchMode: CliLaunchMode = "built",
): Promise<string> {
  const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-test-"));

  try {
    await fs.cp(fixtureDir, tempDir, { recursive: true });

    const snapshot = [];
    for (const file of files) {
      const filePath = join(tempDir, file);

      const beforeContent = await fs.readFile(filePath, "utf8");

      await runCliWithLaunchMode(tempDir, [...args, file], launchMode); // Write by default
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

function normalizeOutput(output: string, cwd: string): string {
  const cwdPath = cwd.replace(/\\/g, "/");
  const repoRoot = join(import.meta.dirname, "..", "..", "..", "..");
  const rootPath = repoRoot.replace(/\\/g, "/");

  return (
    output
      // Normalize timing information
      .replace(/\d+(?:\.\d+)?s|\d+ms/g, "<variable>ms")
      // Normalize path separators (Windows compatibility)
      .replace(/\\/g, "/")
      // Replace absolute paths
      .replace(new RegExp(RegExp.escape(cwdPath), "g"), "<cwd>")
      .replace(new RegExp(RegExp.escape(rootPath), "g"), "<cwd>")
      // NOTE: These redundant processes are necessary to obtain stable snapshots in CI.
      // In Oxfmt, there are 2 kinds of errors displayed
      // - on the Rust side using `miette`
      // - on the JS side by `prettier`
      // and in order to handle them in one place, it needs to be done outside of Oxfmt.
      // Strip ANSI escape codes (e.g. from Prettier error messages)
      // oxlint-disable-next-line no-control-regex
      .replace(/\x1b\[[0-9;]*m/g, "")
      // Normalize `miette` Unicode theme to ASCII
      .replace(/×/g, "x")
      .replace(/╭/g, ",")
      .replace(/─/g, "-")
      .replace(/│/g, "|")
      .replace(/·/g, ":")
      .replace(/┬/g, "|")
      .replace(/╰/g, "`")
      // Trim trailing whitespace per line for sure
      .replace(/[^\S\n]+$/gm, "")
  );
}

function formatSnapshot(cwd: string, args: string[], { stdout, stderr, exitCode }: Result): string {
  const cliRoot = join(import.meta.dirname);
  const relativeDir = relative(cliRoot, cwd) || ".";

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
