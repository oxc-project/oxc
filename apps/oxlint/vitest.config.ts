import { configDefaults, defineConfig } from "vite-plus";

export default defineConfig({
  test: {
    exclude: [...configDefaults.exclude, "fixtures/**"],
  },
  define: {
    DEBUG: "true",
    CONFORMANCE: "false",
  },
});
