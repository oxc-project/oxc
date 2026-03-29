import plugin from "./plugin.ts";

export default [
  {
    files: ["files/1.js", "files/2.js"],
    plugins: {
      "eslint-compat-plugin": plugin,
    },
    rules: {
      "eslint-compat-plugin/create": "error",
      "eslint-compat-plugin/create-once": "error",
      "eslint-compat-plugin/create-once-selector": "error",
      "eslint-compat-plugin/create-once-before-false": "error",
      "eslint-compat-plugin/create-once-before-only": "error",
      "eslint-compat-plugin/create-once-after-only": "error",
      "eslint-compat-plugin/create-once-hooks-only": "error",
      "eslint-compat-plugin/create-once-no-hooks": "error",
    },
  },
  {
    files: ["files/cfg.js"],
    plugins: {
      "eslint-compat-plugin": plugin,
    },
    rules: {
      "eslint-compat-plugin/create-once-cfg": "error",
      "eslint-compat-plugin/create-once-cfg2": "error",
    },
  },
];
