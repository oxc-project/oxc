// oxlint-disable no-await-in-loop
import { join, relative } from "node:path";
import { readdirSync, readFileSync } from "node:fs";
import fs from "node:fs/promises";
import { execa } from "execa";
import { createPatch } from "diff";

// NOTE: TS's ESNext is not yet reached to ES2025...
declare global {
  interface RegExpConstructor {
    escape(str: string): string;
  }
}

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");
const CLI_TEST_DIR = import.meta.dirname;

// --- Types ---

interface TestCaseOptions {
  args: string[];
  cwd?: string;
  env?: Record<string, string>;
  stdin?: string;
  gitignore?: Record<string, string>;
}

export interface Fixture {
  name: string;
  dirPath: string;
  fixturesPath: string;
  testCases: TestCaseOptions[];
}

// --- Fixture discovery ---

export function getFixtures(): Fixture[] {
  const fixtures: Fixture[] = [];

  for (const entry of readdirSync(CLI_TEST_DIR, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;

    const dirPath = join(CLI_TEST_DIR, entry.name);
    const optionsPath = join(dirPath, "options.json");

    let raw: string;
    try {
      raw = readFileSync(optionsPath, "utf8");
    } catch {
      continue; // No options.json, skip
    }

    const testCases: TestCaseOptions[] = JSON.parse(raw);

    fixtures.push({
      name: entry.name,
      dirPath,
      fixturesPath: join(dirPath, "fixtures"),
      testCases,
    });
  }

  return fixtures;
}

// --- Test execution ---

export async function runFixture(fixture: Fixture, testCase: TestCaseOptions): Promise<string> {
  const cwd = testCase.cwd ? join(fixture.fixturesPath, testCase.cwd) : fixture.fixturesPath;

  // Read all files before execution (for diff detection and tree)
  const filesBefore = await readAllFiles(fixture.fixturesPath);
  const tree = [...filesBefore.keys()];

  // Setup: create .gitignore files if specified
  const gitignoreFiles: string[] = [];
  if (testCase.gitignore) {
    for (const [path, content] of Object.entries(testCase.gitignore)) {
      const fullPath = join(fixture.fixturesPath, path);
      await fs.writeFile(fullPath, content);
      gitignoreFiles.push(fullPath);
    }
  }

  try {
    // Read stdin file if specified
    let input: string | undefined;
    if (testCase.stdin) {
      input = await fs.readFile(join(fixture.fixturesPath, testCase.stdin), "utf8");
    }

    // Execute
    const { stdout, stderr, exitCode } = await execa(
      "node",
      [CLI_PATH, ...testCase.args, "--threads=1"],
      {
        cwd,
        reject: false,
        timeout: 5000,
        input,
        env: { ...process.env, ...testCase.env },
      },
    );

    // Cleanup .gitignore files before detecting changes
    for (const path of gitignoreFiles) {
      await fs.rm(path, { force: true });
    }
    gitignoreFiles.length = 0;

    // Detect file changes and restore
    const changes = await detectAndRestoreChanges(fixture.fixturesPath, filesBefore);

    // Build snapshot
    return buildSnapshot({
      args: testCase.args,
      env: testCase.env,
      cwdRelative: testCase.cwd ?? null,
      tree,
      stdout: normalizeOutput(String(stdout), cwd),
      stderr: normalizeOutput(String(stderr), cwd),
      exitCode: exitCode ?? -1,
      changes,
    });
  } finally {
    // Safety net: cleanup .gitignore files if execa threw before the normal cleanup
    for (const path of gitignoreFiles) {
      await fs.rm(path, { force: true });
    }
  }
}

// --- File reading ---

async function readAllFiles(dir: string): Promise<Map<string, string>> {
  const files = new Map<string, string>();
  const entries = await fs.readdir(dir, { withFileTypes: true, recursive: true });

  for (const entry of entries) {
    if (!entry.isFile()) continue;
    const fullPath = join(entry.parentPath, entry.name);
    const relPath = relative(dir, fullPath).replace(/\\/g, "/");
    files.set(relPath, await fs.readFile(fullPath, "utf8"));
  }

  return files;
}

// --- Diff detection and restore ---

interface FileDiff {
  path: string;
  before: string;
  after: string;
}

interface FileCreated {
  path: string;
  content: string;
}

interface FileChanges {
  diffs: FileDiff[];
  created: FileCreated[];
}

async function detectAndRestoreChanges(
  dir: string,
  filesBefore: Map<string, string>,
): Promise<FileChanges> {
  const diffs: FileDiff[] = [];
  const created: FileCreated[] = [];

  // Detect modified files
  for (const [relPath, beforeContent] of filesBefore) {
    const fullPath = join(dir, relPath);
    const afterContent = await fs.readFile(fullPath, "utf8");
    if (afterContent !== beforeContent) {
      diffs.push({ path: relPath, before: beforeContent, after: afterContent });
      // Restore
      await fs.writeFile(fullPath, beforeContent);
    }
  }

  // Detect newly created files
  const entriesAfter = await fs.readdir(dir, { withFileTypes: true, recursive: true });
  for (const entry of entriesAfter) {
    if (!entry.isFile()) continue;
    const fullPath = join(entry.parentPath, entry.name);
    const relPath = relative(dir, fullPath).replace(/\\/g, "/");
    if (!filesBefore.has(relPath)) {
      const content = await fs.readFile(fullPath, "utf8");
      created.push({ path: relPath, content });
      await fs.rm(fullPath, { force: true });
    }
  }

  return {
    diffs: diffs.sort((a, b) => a.path.localeCompare(b.path)),
    created: created.sort((a, b) => a.path.localeCompare(b.path)),
  };
}

// --- Snapshot building ---

interface SnapshotData {
  args: string[];
  env: Record<string, string> | undefined;
  cwdRelative: string | null; // relative path from fixtures/ to cwd, null if fixtures/ is cwd
  tree: string[];
  stdout: string;
  stderr: string;
  exitCode: number;
  changes: FileChanges;
}

function buildSnapshot(data: SnapshotData): string {
  let snapshot = "";

  snapshot += `# Input\n\n`;
  const envPrefix = data.env
    ? Object.entries(data.env)
        .map(([k, v]) => `${k}=${v}`)
        .join(" ") + " "
    : "";
  snapshot += `## Command\n\`\`\`\n${envPrefix}oxfmt ${data.args.join(" ")}\n\`\`\`\n\n`;
  snapshot += `## File tree\n\`\`\`\n${buildTreeView(data.tree, data.cwdRelative)}\n\`\`\`\n\n`;

  snapshot += `# Result\n\n`;
  snapshot += `## Exit code\n${data.exitCode}\n\n`;
  snapshot += `## stdout\n\`\`\`\n${data.stdout}\n\`\`\`\n\n`;
  snapshot += `## stderr\n\`\`\`\n${data.stderr}\n\`\`\`\n\n`;

  const { diffs, created } = data.changes;

  snapshot += `## File changes\n`;
  if (diffs.length === 0 && created.length === 0) {
    snapshot += `No changes\n`;
  } else {
    for (const diff of diffs) {
      const patch = createPatch(diff.path, diff.before, diff.after);
      // Strip the header lines (Index, ===, --- / +++) since the file name is already in the heading
      const lines = patch.split("\n");
      const hunkStart = lines.findIndex((l) => l.startsWith("@@"));
      const patchBody = lines.slice(hunkStart).join("\n");
      snapshot += `\n### ${diff.path}\n\`\`\`\`\`diff\n${patchBody}\`\`\`\`\`\n`;
    }
    for (const file of created) {
      snapshot += `\n### ${file.path} (created)\n\`\`\`\n${file.content}\n\`\`\`\n`;
    }
  }

  return snapshot;
}

function buildTreeView(files: string[], cwdRelative: string | null): string {
  // Build a nested structure from flat file paths
  interface TreeNode {
    children: Map<string, TreeNode>;
  }
  const root: TreeNode = { children: new Map() };

  for (const file of files) {
    const parts = file.split("/");
    let node = root;
    for (const part of parts) {
      if (!node.children.has(part)) {
        node.children.set(part, { children: new Map() });
      }
      node = node.children.get(part)!;
    }
  }

  // Determine which path segments are the cwd
  const cwdParts = cwdRelative ? cwdRelative.split("/") : [];

  const lines: string[] = [];

  function render(node: TreeNode, depth: number, pathFromFixtures: string[]): void {
    const sorted = [...node.children.entries()].sort(([a], [b]) => a.localeCompare(b));
    for (const [name, child] of sorted) {
      const isDir = child.children.size > 0;
      const currentPath = [...pathFromFixtures, name];
      const isCwd =
        cwdParts.length > 0 &&
        currentPath.length === cwdParts.length &&
        currentPath.every((p, i) => p === cwdParts[i]);
      const indent = "  ".repeat(depth);
      const suffix = isDir ? "/" : "";
      const marker = isCwd ? " <CWD>" : "";
      lines.push(`${indent}- ${name}${suffix}${marker}`);
      if (isDir) {
        render(child, depth + 1, currentPath);
      }
    }
  }

  // Root is always "fixtures/"
  const isCwdRoot = cwdRelative === null;
  lines.push(`- fixtures/${isCwdRoot ? " <CWD>" : ""}`);
  render(root, 1, []);

  return lines.join("\n");
}

// --- Output normalization ---

function normalizeOutput(output: string, cwd: string): string {
  if (!output) return "";

  const cwdPath = cwd.replace(/\\/g, "/");
  const repoRoot = join(import.meta.dirname, "..", "..", "..", "..");
  const rootPath = repoRoot.replace(/\\/g, "/");

  return (
    output
      .replace(/\d+(?:\.\d+)?s|\d+ms/g, "<time>")
      .replace(/\\/g, "/")
      .replace(new RegExp(RegExp.escape(cwdPath), "g"), "<cwd>")
      .replace(new RegExp(RegExp.escape(rootPath), "g"), "<root>")
      // oxlint-disable-next-line no-control-regex
      .replace(/\x1b\[[0-9;]*m/g, "")
      .replace(/×/g, "x")
      .replace(/╭/g, ",")
      .replace(/─/g, "-")
      .replace(/│/g, "|")
      .replace(/·/g, ":")
      .replace(/┬/g, "|")
      .replace(/╰/g, "`")
      .replace(/[^\S\n]+$/gm, "")
  );
}
