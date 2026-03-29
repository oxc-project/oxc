import { defineConfig } from "#oxlint";

export default defineConfig({
  // @ts-expect-error - we are testing invalid config
  extends: ["./base.ts"],
});
