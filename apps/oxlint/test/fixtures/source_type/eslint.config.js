import plugin from "./plugin.ts";

export default [
  {
    files: ["files/**/*.js", "files/**/*.cjs"],
    plugins: {
      "source-type-plugin": plugin,
    },
    rules: {
      "source-type-plugin/check": "error",
    },
  },
];
