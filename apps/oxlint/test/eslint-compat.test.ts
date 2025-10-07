import { join as pathJoin } from 'node:path';
import { describe, it } from 'vitest';
import { testFixtureWithCommand } from './utils.js';

const ESLINT_PATH = pathJoin(import.meta.dirname, '../node_modules/.bin/eslint');

/**
 * Run ESLint on a test fixture.
 * @param fixtureName - Name of the fixture directory within `test/fixtures`
 */
async function testFixture(fixtureName: string): Promise<void> {
  await testFixtureWithCommand({
    command: ESLINT_PATH,
    args: [],
    fixtureName,
    snapshotName: 'eslint',
  });
}

// These tests take longer than 5 seconds on CI, so increase timeout to 10 seconds
describe('ESLint compatibility', { timeout: 10_000 }, () => {
  it('`definePlugin` should work', async () => {
    await testFixture('definePlugin');
  });

  it('`defineRule` should work', async () => {
    await testFixture('defineRule');
  });

  it('`definePlugin` and `defineRule` together should work', async () => {
    await testFixture('definePlugin_and_defineRule');
  });
});
