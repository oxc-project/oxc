import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["./test/**/*.test.ts"],
    includeSource: ["./src-js/**/*.ts"],
    snapshotFormat: {
      escapeString: false,
      printBasicPrototype: false,
    },
    snapshotSerializers: [],
    testTimeout: 10000,
  },
});
