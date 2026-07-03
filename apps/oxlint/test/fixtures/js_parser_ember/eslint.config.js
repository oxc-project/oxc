import emberEslintParser from "ember-eslint-parser";
import ember from "eslint-plugin-ember";

export default [
  {
    files: ["files/**/*.{gjs,gts}"],
    plugins: { ember },
    languageOptions: {
      parser: emberEslintParser,
    },
    rules: {
      "ember/template-no-let-reference": "error",
    },
  },
];
