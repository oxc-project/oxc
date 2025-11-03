import fs from 'node:fs/promises';
import os from 'node:os';
import { join as pathJoin } from 'node:path';

import { execa } from 'execa';
import { expect } from 'vitest';

export const PACKAGE_ROOT_PATH = pathJoin(import.meta.dirname, '..');
export const FIXTURES_DIR_PATH = pathJoin(import.meta.dirname, 'fixtures');

const REPO_ROOT_PATH = pathJoin(PACKAGE_ROOT_PATH, '../../');

// Options to pass to `testFixtureWithCommand`.
interface TestFixtureOptions {
  // Command
  command: string;
  // Arguments to execute command with
  args: string[];
  // Fixture name
  fixtureName: string;
  // Name of the snapshot file
  snapshotName: string;
  // Function to get extra data to include in the snapshot
  getExtraSnapshotData?: (dirPath: string) => Promise<{ [key: string]: string }>;
}

/**
 * Run a test fixture.
 * @param options - Options for running the test
 */
export async function testFixtureWithCommand(options: TestFixtureOptions): Promise<void> {
  const fixtureDirPath = pathJoin(FIXTURES_DIR_PATH, options.fixtureName);
  const sourceFilesDir = pathJoin(fixtureDirPath, 'files');

  // Create a temporary directory to avoid modifying the original fixture files
  const tempDir = await fs.mkdtemp(pathJoin(os.tmpdir(), 'oxfmt-test-'));
  const tempFilesDir = pathJoin(tempDir, 'files');

  try {
    // TODO: use stdin instead of copy to tmp dir.

    // Copy fixture files to temp directory
    await fs.cp(sourceFilesDir, tempFilesDir, { recursive: true });

    let { stdout, stderr, exitCode } = await execa(options.command, options.args, {
      cwd: tempDir,
      reject: false,
    });

    const snapshotPath = pathJoin(fixtureDirPath, `${options.snapshotName}.snap.md`);

    stdout = normalizeStdout(stdout);
    stderr = normalizeStdout(stderr);
    let snapshot =
      `# Exit code\n${exitCode}\n\n` + `# stdout\n\`\`\`\n${stdout}\`\`\`\n\n` + `# stderr\n\`\`\`\n${stderr}\`\`\`\n`;

    if (options.getExtraSnapshotData) {
      const extraSnapshots = await options.getExtraSnapshotData(tempDir);
      for (const [name, data] of Object.entries(extraSnapshots)) {
        snapshot += `\n# ${name}\n\`\`\`\n${data}\n\`\`\`\n`;
      }
    }

    await expect(snapshot).toMatchFileSnapshot(snapshotPath);
  } finally {
    // Clean up temp directory
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

/**
 * Normalize output, so it's the same on every machine.
 *
 * - Remove timing + thread count info.
 * - Replace start of file paths with `<root>`.
 * - Normalize line breaks.
 *
 * @param stdout Output from process
 * @returns Normalized output
 */
function normalizeStdout(stdout: string): string {
  // Normalize line breaks, and trim line breaks from start and end
  stdout = stdout.replace(/\r\n?/g, '\n').replace(/^\n+/, '').replace(/\n+$/, '');
  if (stdout === '') return '';

  let lines = stdout.split('\n');

  // Remove timing and thread count info which can vary between runs
  // Match patterns like "Finished in 123ms on 5 files using 8 threads."
  // and per-file timings like "files/index.js 23ms (unchanged)" or "files/index.js (23ms)"
  lines = lines.map((line) => {
    // Normalize the summary line
    line = line.replace(
      /^Finished in \d+(?:\.\d+)?(?:s|ms|us|ns) on (\d+) file(s?) using \d+ threads\.$/,
      'Finished in Xms on $1 file$2 using X threads.',
    );
    // Normalize per-file timing: "files/index.js 23ms" -> "files/index.js Xms"
    line = line.replace(/^(.*?)\s+\d+(?:\.\d+)?(?:s|ms|us|ns)(\s|$)/, '$1 Xms$2');
    // Normalize check mode timing: "files/index.js (23ms)" -> "files/index.js (Xms)"
    line = line.replace(/\((\d+(?:\.\d+)?(?:s|ms|us|ns))\)/g, '(Xms)');
    return line;
  });

  // Replace absolute paths with <root>
  lines = lines.map((line) => {
    if (line.startsWith(REPO_ROOT_PATH)) {
      return `<root>/${line.slice(REPO_ROOT_PATH.length).replace(/\\/g, '/')}`;
    }
    return line;
  });

  if (lines.length === 0) return '';
  return lines.join('\n') + '\n';
}
