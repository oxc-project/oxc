import { defineConfig } from 'vitest/config';

const { env } = process;
const isEnabled = envValue => envValue === 'true' || envValue === '1';
let exclude;
if (!isEnabled(env.RUN_LAZY_TESTS)) {
  exclude = ['lazy-deserialization.test.ts'];
  if (!isEnabled(env.RUN_RAW_TESTS) && !isEnabled(env.RUN_RAW_RANGE_TESTS)) exclude.push('parse-raw.test.ts');
}

export default defineConfig({
  test: {
    diff: {
      expand: false,
    },
    exclude,
  },
  plugins: [
    // Enable Codspeed plugin in CI only
    process.env.CI && (await import('@codspeed/vitest-plugin')).default(),
  ],
});
