// Test that overrides work in oxlint.config.ts
export default {
  rules: {
    "no-debugger": "off",
  },
  overrides: [
    {
      files: ["*.ts"],
      rules: {
        "no-debugger": "error",
      },
    },
  ],
};
