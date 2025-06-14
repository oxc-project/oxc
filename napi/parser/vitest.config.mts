import { defineConfig } from 'vitest/config';

const RUN_LAZY_TESTS = process.env.RUN_LAZY_TESTS === 'true';

export default defineConfig({
  test: {
    diff: {
      expand: false,
    },
    ...(
      !RUN_LAZY_TESTS &&
      { exclude: ['lazy-deserialization.test.ts'] }
    ),
  },
  plugins: [
    // Enable Codspeed plugin in CI only
    process.env.CI && (await import('@codspeed/vitest-plugin')).default(),
  ],
});
