import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { execa } from 'execa';

const PACKAGE_ROOT_PATH = path.dirname(import.meta.dirname);
const ENTRY_POINT_PATH = path.join(PACKAGE_ROOT_PATH, 'src/index.js');

async function runOxlint(cwd: string, args: string[] = []) {
  return await execa('node', [ENTRY_POINT_PATH, ...args], {
    cwd: path.join(PACKAGE_ROOT_PATH, cwd),
    reject: false,
  });
}

function normalizeOutput(output: string): string {
  return output
    .replace(/Finished in \d+(\.\d+)?(s|ms|us|ns)/, 'Finished in Xms')
    .replace(/using \d+ threads./, 'using X threads.');
}

describe('cli options for bundling', () => {
  it('should lint a directory without errors', async () => {
    const { stdout, exitCode } = await runOxlint(
      'test/fixtures/built_in_no_errors',
    );

    expect(exitCode).toBe(0);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should lint a directory with errors', async () => {
    const { stdout, exitCode } = await runOxlint(
      'test/fixtures/built_in_errors',
    );

    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should load a custom plugin', async () => {
    const { stdout, exitCode } = await runOxlint(
      'test/fixtures/basic_custom_plugin',
    );

    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should report an error if a custom plugin cannot be loaded', async () => {
    const { stdout, exitCode } = await runOxlint(
      'test/fixtures/missing_custom_plugin',
    );

    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should report an error if a rule is not found within a custom plugin', async () => {
    const { stdout, exitCode } = await runOxlint(
      'test/fixtures/custom_plugin_missing_rule',
    );

    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });
});
