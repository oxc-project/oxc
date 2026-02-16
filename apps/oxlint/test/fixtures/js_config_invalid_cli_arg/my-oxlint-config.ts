import { defineConfig } from "#oxlint";

export default defineConfig({
  // @ts-expect-error - we are testing invalid CLI args, so we need to ignore the type error here
  x: {
    "no-debugger": "error",
    eqeqeq: "warn",
  },
});
