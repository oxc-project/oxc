import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["./test/**/*.test.ts"],
    snapshotFormat: {
      escapeString: false,
      printBasicPrototype: false,
    },
    snapshotSerializers: [],
    testTimeout: 10000,
  },
});
