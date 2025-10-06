import { defineConfig } from 'vitest/config';

const { env, platform } = process;
const isEnabled = envValue => envValue === 'true' || envValue === '1';
const exclude = new Set<string>();
if (!isEnabled(env.RUN_LAZY_TESTS)) {
  exclude.add('lazy-deserialization.test.ts');
  if (!isEnabled(env.RUN_RAW_TESTS) && !isEnabled(env.RUN_RAW_RANGE_TESTS)) exclude.add('parse-raw.test.ts');
}
// TinyPool doesn't seem to work on Windows with Vitest
// Ref: https://github.com/vitest-dev/vitest/issues/8201
if (platform === 'win32') {
  exclude.add('parse-raw.test.ts');
}

export default defineConfig({
  test: {
    diff: {
      expand: false,
    },
    exclude: [...exclude],
  },
  plugins: [
    // Enable Codspeed plugin in CI only
    process.env.CI && (await import('@codspeed/vitest-plugin')).default(),
  ],
});
