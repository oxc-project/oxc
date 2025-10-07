import { join as pathJoin } from 'node:path';

import { execa } from 'execa';
import { expect } from 'vitest';

export const PACKAGE_ROOT_PATH = pathJoin(import.meta.dirname, '..');
export const FIXTURES_DIR_PATH = pathJoin(import.meta.dirname, 'fixtures');

const REPO_ROOT_PATH = pathJoin(PACKAGE_ROOT_PATH, '../../');
const ROOT_URL = new URL('../../../', import.meta.url).href;
const FIXTURES_URL = new URL('./fixtures/', import.meta.url).href;

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

  let { stdout, stderr, exitCode } = await execa(options.command, options.args, {
    cwd: fixtureDirPath,
    reject: false,
  });

  const snapshotPath = pathJoin(fixtureDirPath, `${options.snapshotName}.snap.md`);

  stdout = normalizeStdout(stdout);
  stderr = normalizeStdout(stderr);
  let snapshot = `# Exit code\n${exitCode}\n\n` +
    `# stdout\n\`\`\`\n${stdout}\`\`\`\n\n` +
    `# stderr\n\`\`\`\n${stderr}\`\`\`\n`;

  if (options.getExtraSnapshotData) {
    const extraSnapshots = await options.getExtraSnapshotData(fixtureDirPath);
    for (const [name, data] of Object.entries(extraSnapshots)) {
      snapshot += `\n# ${name}\n\`\`\`\n${data}\n\`\`\`\n`;
    }
  }

  await expect(snapshot).toMatchFileSnapshot(snapshotPath);
}

/**
 * Normalize output, so it's the same on every machine.
 *
 * - Remove timing + thread count info.
 * - Replace start of file paths with `<root>`.
 * - Remove irrelevant lines from stack traces.
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
  lines[lines.length - 1] = lines[lines.length - 1].replace(
    /^Finished in \d+(?:\.\d+)?(?:s|ms|us|ns) on (\d+) file(s?) using \d+ threads.$/,
    'Finished in Xms on $1 file$2 using X threads.',
  );

  // Remove lines from stack traces which are outside `fixtures` directory.
  // Replace path to repo root in stack traces with `<root>`.
  lines = lines.flatMap((line) => {
    // e.g. ` | at file:///path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js:1:1`
    // e.g. ` | at whatever (file:///path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js:1:1)`
    let match = line.match(/^(\s*\|\s+at (?:.+?\()?)(.+)$/);
    if (match) {
      const [, preamble, at] = match;
      return at.startsWith(FIXTURES_URL) ? [`${preamble}<root>/${at.slice(ROOT_URL.length)}`] : [];
    } else {
      // e.g. ` | File path: /path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js`
      match = line.match(/^(\s*\|\s+File path: )(.+)$/);
      if (match) {
        const [, preamble, path] = match;
        if (path.startsWith(REPO_ROOT_PATH)) {
          return [`${preamble}<root>/${path.slice(REPO_ROOT_PATH.length).replace(/\\/g, '/')}`];
        }
      }
    }
    if (line.startsWith(REPO_ROOT_PATH)) line = `<root>/${line.slice(REPO_ROOT_PATH.length).replace(/\\/g, '/')}`;
    return [line];
  });

  if (lines.length === 0) return '';
  return lines.join('\n') + '\n';
}
