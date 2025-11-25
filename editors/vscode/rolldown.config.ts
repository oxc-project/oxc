import { defineConfig, type RolldownOptions } from "rolldown";
import { globSync } from "tinyglobby";

const input: RolldownOptions["input"] =
  process.env.TEST === "true" ? globSync("tests/**/*.ts") : ["client/extension.ts"];

const output: RolldownOptions["output"] = {
  sourcemap: true,
  format: "cjs",
  banner: `"use strict";\n`,
  minify: true,
};

if (process.env.TEST === "true") {
  output.dir = "out";
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
