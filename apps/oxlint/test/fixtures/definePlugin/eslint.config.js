import plugin from "./plugin.ts";

export default [
  {
    files: ["files/*.js"],
    plugins: {
      "define-plugin-plugin": plugin,
    },
    rules: {
      "define-plugin-plugin/create": "error",
      "define-plugin-plugin/create-once": "error",
      "define-plugin-plugin/create-once-before-false": "error",
      "define-plugin-plugin/create-once-before-only": "error",
      "define-plugin-plugin/create-once-after-only": "error",
      "define-plugin-plugin/create-once-hooks-only": "error",
      "define-plugin-plugin/create-once-no-hooks": "error",
    },
  },
];
