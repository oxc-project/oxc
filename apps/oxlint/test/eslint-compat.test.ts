import { describe, it } from 'vitest';
import { testFixtureWithCommand } from './utils.js';

/**
 * Run ESLint on a test fixture.
 * @param fixtureName - Name of the fixture directory within `test/fixtures`
 */
async function testFixture(fixtureName: string): Promise<void> {
  await testFixtureWithCommand({
    command: 'pnpx',
    args: ['eslint'],
    fixtureName,
    snapshotName: 'eslint',
  });
}

describe('ESLint compatibility', () => {
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
