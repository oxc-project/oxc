import testPlugin from './test_plugin/index.js';

export default [
  {
    files: ["files/*.js"],
    plugins: {
      "define-rule-plugin": testPlugin,
    },
    rules: {
      "define-rule-plugin/create": "error",
      "define-rule-plugin/create-once": "error",
    },
  },
];
