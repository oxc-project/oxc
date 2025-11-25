import plugin from "./plugin.ts";

export default [
  {
    files: ["files/*.js"],
    plugins: {
      "define-rule-plugin": plugin,
    },
    rules: {
      "define-rule-plugin/create": "error",
      "define-rule-plugin/create-once": "error",
      "define-rule-plugin/create-once-before-false": "error",
      "define-rule-plugin/create-once-before-only": "error",
      "define-rule-plugin/create-once-after-only": "error",
      "define-rule-plugin/create-once-hooks-only": "error",
      "define-rule-plugin/create-once-no-hooks": "error",
    },
  },
];
