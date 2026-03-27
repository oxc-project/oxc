import plugin from "./plugin.ts";

export default [
  {
    files: ["files/1.js"],
    plugins: {
      "test-plugin": plugin,
    },
    rules: {
      "test-plugin/tracking": "error",
      "test-plugin/throw-in-before": "error",
      "test-plugin/tracking-late": "error",
    },
  },
];
