import plugin from "./plugin.ts";

export default [
  {
    files: ["files/**/*.js"],
    plugins: {
      "weakmap-cache-plugin": plugin,
    },
    rules: {
      "weakmap-cache-plugin/cache1": "error",
      "weakmap-cache-plugin/cache2": "error",
      "weakmap-cache-plugin/cache3": "error",
    },
  },
];
