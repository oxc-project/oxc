import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    diff: {
      expand: false,
    },
  },
  plugins: [
    // Enable Codspeed plugin in CI only
    process.env.CI && (await import('@codspeed/vitest-plugin')).default(),
  ],
});
