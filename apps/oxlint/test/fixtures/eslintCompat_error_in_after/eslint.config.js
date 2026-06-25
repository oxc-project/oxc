import plugin from "./plugin.ts";

export default [
  {
    files: ["files/*.js"],
    plugins: {
      "eslint-compat-plugin": plugin,
    },
    rules: {
      "eslint-compat-plugin/tracking": "error",
      "eslint-compat-plugin/throw-in-after": "error",
      "eslint-compat-plugin/tracking-late": "error",
    },
  },
];
