import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    exclude: [...configDefaults.exclude],
    snapshotFormat: {
      escapeString: false,
      printBasicPrototype: false,
    },
    snapshotSerializers: [],
  },
});
