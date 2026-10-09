export default {
  semi: process.env.NEVER_SET === "1",
  plugins: ["prettier-plugin-tailwindcss"],
  tailwindFunctions: ["clsx"],
  overrides: [
    { files: "*.svg", options: { parser: "html" } },
    {
      files: ["*.md"],
      excludeFiles: "CHANGELOG.md",
      options: { tabWidth: 4, tailwindFunctions: ["tw"], plugins: ["prettier-plugin-foo"] },
    },
  ],
};
