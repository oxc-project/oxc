import testPlugin from './test_plugin/index.js';

export default [
  {
    files: ["files/*.js"],
    plugins: {
      testPlugin,
    },
    rules: {
      "testPlugin/create": "error",
      "testPlugin/create-once": "error",
    },
  },
];
