import testPlugin from './test_plugin/index.js';

export default [
  {
    files: ["files/*.js"],
    plugins: {
      "define-plugin-and-rule-plugin": testPlugin,
    },
    rules: {
      "define-plugin-and-rule-plugin/create": "error",
      "define-plugin-and-rule-plugin/create-once": "error",
      "define-plugin-and-rule-plugin/create-once-before-false": "error",
      "define-plugin-and-rule-plugin/create-once-before-only": "error",
      "define-plugin-and-rule-plugin/create-once-after-only": "error",
      "define-plugin-and-rule-plugin/create-once-no-hooks": "error",
    },
  },
];
