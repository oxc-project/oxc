import { defineConfig } from 'vitest/config';

const { env } = process;
let exclude;
if (env.RUN_LAZY_TESTS !== 'true') {
  exclude = ['lazy-deserialization.test.ts'];
  if (env.RUN_RAW_TESTS !== 'true') exclude.push('parse-raw.test.ts');
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
