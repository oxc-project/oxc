import fs from 'node:fs/promises';
import { join as pathJoin } from 'node:path';
import { describe, it } from 'vitest';
import { PACKAGE_ROOT_PATH, testFixtureWithCommand } from './utils.js';

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, 'dist/cli.js');

// Options to pass to `testFixture`.
interface TestOptions {
  // Arguments to pass to the CLI.
  args?: string[];
  // Name of the snapshot file.
  // Defaults to `output`.
  // Supply a different name when there are multiple tests for a single fixture.
  snapshotName?: string;
  // Function to get extra data to include in the snapshot
  getExtraSnapshotData?: (dirPath: string) => Promise<{ [key: string]: string }>;
}

/**
 * Run a test fixture.
 * @param fixtureName - Name of the fixture directory within `test/fixtures`
 * @param options - Options to customize the test (optional)
 */
async function testFixture(fixtureName: string, options?: TestOptions): Promise<void> {
  const args = options?.args ?? [];

  await testFixtureWithCommand({
    // Use current NodeJS executable, rather than `node`, to avoid problems with a Node version manager
    // installed on system resulting in using wrong NodeJS version
    command: process.execPath,
    args: [CLI_PATH, ...args, 'files'],
    fixtureName,
    snapshotName: options?.snapshotName ?? 'output',
    getExtraSnapshotData: options?.getExtraSnapshotData,
  });
}

describe('oxfmt NAPI - Embedded Language Formatting', () => {
  // oxlint-disable-next-line vitest/expect-expect -- Assertion is inside testFixtureWithCommand helper
  it('should format embedded languages (CSS, GraphQL, HTML, Markdown)', async () => {
    await testFixture('embedded_languages', {
      getExtraSnapshotData: async (dirPath) => {
        const formattedCode = await fs.readFile(pathJoin(dirPath, 'files/index.js'), 'utf-8');
        return { 'Formatted Output': formattedCode };
      },
    });
  });
});
