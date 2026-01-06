import os from "node:os";
import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    exclude: [...configDefaults.exclude],
    snapshotFormat: {
      escapeString: false,
      printBasicPrototype: false,
    },
    snapshotSerializers: [],
    testTimeout: 10_000,
    // Limit workers to avoid resource contention
    // - each test spawns CLI subprocesses with their own worker pools
    maxWorkers: Math.floor(os.cpus().length / 2),
  },
});
