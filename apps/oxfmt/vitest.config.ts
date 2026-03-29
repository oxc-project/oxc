import { configDefaults, defineConfig } from "vite-plus";

export default defineConfig({
  test: {
    exclude: [...configDefaults.exclude],
    snapshotFormat: {
      escapeString: false,
      printBasicPrototype: false,
    },
    snapshotSerializers: [],
    testTimeout: 10000,
  },
});
