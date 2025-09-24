import testPlugin from './test_plugin/index.js';

export default [
  {
    files: ["files/*.js"],
    plugins: {
      "define-plugin-plugin": testPlugin,
    },
    rules: {
      "define-plugin-plugin/create": "error",
      "define-plugin-plugin/create-once": "error",
      "define-plugin-plugin/create-once-before-false": "error",
      "define-plugin-plugin/create-once-before-only": "error",
      "define-plugin-plugin/create-once-after-only": "error",
      "define-plugin-plugin/create-once-no-hooks": "error",
    },
  },
];
