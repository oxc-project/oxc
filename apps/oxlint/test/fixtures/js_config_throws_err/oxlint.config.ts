import { defineConfig } from "#oxlint";

function throwError() {
  throw new Error("This is a test error");
}

throwError();

export default defineConfig({});
