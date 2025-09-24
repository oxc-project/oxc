import { dirname, join as pathJoin } from 'node:path';

import { describe, expect, it } from 'vitest';

import { execa } from 'execa';

const PACKAGE_ROOT_PATH = dirname(import.meta.dirname);
const REPO_ROOT_PATH = pathJoin(PACKAGE_ROOT_PATH, '../..');

async function runEslint(cwd: string, args: string[] = []) {
  return await execa('pnpx', ['eslint', ...args], {
    cwd: pathJoin(PACKAGE_ROOT_PATH, cwd),
    reject: false,
  });
}

function normalizeOutput(output: string): string {
  const lines = output.split('\n');
  for (let i = lines.length - 1; i >= 0; i--) {
    const line = lines[i];
    if (line.startsWith(REPO_ROOT_PATH)) lines[i] = `<root>${line.slice(REPO_ROOT_PATH.length)}`;
  }
  return lines.join('\n');
}

describe('ESLint compatibility', () => {
  it('`defineRule` should work', async () => {
    const { stdout, exitCode } = await runEslint('test/fixtures/defineRule');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('`definePlugin` should work', async () => {
    const { stdout, exitCode } = await runEslint('test/fixtures/definePlugin');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('`definePlugin` and `defineRule` together should work', async () => {
    const { stdout, exitCode } = await runEslint('test/fixtures/definePlugin_and_defineRule');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });
});
