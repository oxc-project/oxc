import { configDefaults, defineConfig } from "vitest/config";

const { env, platform } = process;
const isEnabled = (envValue) => envValue === "true" || envValue === "1";

const runLazyTests = isEnabled(env.RUN_LAZY_TESTS);
let runRawTests =
  runLazyTests || isEnabled(env.RUN_RAW_TESTS) || isEnabled(env.RUN_RAW_RANGE_TESTS);

// Raw tests use `tinypool`, which doesn't seem to work on Windows with Vitest
// Ref: https://github.com/vitest-dev/vitest/issues/8201
if (platform === "win32") runRawTests = false;

const exclude = [...configDefaults.exclude];
if (!runRawTests) exclude.push("parse-raw.test.ts");
if (!runLazyTests) exclude.push("lazy-deserialization.test.ts");

export default defineConfig({
  test: {
    diff: {
      expand: false,
    },
    exclude,
  },
  plugins: [
    // Enable Codspeed plugin in CI only
    process.env.CI && (await import("@codspeed/vitest-plugin")).default(),
  ],
});
