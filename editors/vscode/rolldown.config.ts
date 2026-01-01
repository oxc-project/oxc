import { defineConfig, type RolldownOptions } from "rolldown";
import { globSync } from "tinyglobby";

const input: RolldownOptions["input"] =
  process.env.TEST === "true" ? globSync("tests/**/*.ts") : ["client/extension.ts"];

const output: RolldownOptions["output"] = {
  sourcemap: true,
  format: "cjs",
  banner: `"use strict";\n`,
  minify: true,
  cleanDir: true,
};

if (process.env.TEST === "true") {
  output.dir = "out_test";
  output.preserveModules = true;
  output.preserveModulesRoot = "tests";
} else {
  output.file = "out/main.js";
}

export default defineConfig({
  input,
  output,
  external: ["vscode"],
  platform: "node",
  transform: {
    target: "node16",
  },
});
