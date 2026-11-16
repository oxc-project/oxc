import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

const CSS = ["a{color:red}", "a {\n  color: red;\n}\n"];
const GRAPHQL = ["type Query { hello:String }", "type Query {\n  hello: String\n}\n"];
const MARKDOWN = ["#  Title\n", "# Title\n"];
const YAML = ["key:   value\n", "key: value\n"];

describe("Language detection by file name", () => {
  it.each([
    ["app.wxss", ...CSS],
    ["styles.pcss", ...CSS],
    ["styles.postcss", ...CSS],
    ["schema.gql", ...GRAPHQL],
    ["schema.graphqls", ...GRAPHQL],
    ["a.md", ...MARKDOWN],
    ["a.markdown", ...MARKDOWN],
    ["README", ...MARKDOWN],
    ["contents.lr", ...MARKDOWN],
    ["config.yml", ...YAML],
    ["grammar.sublime-syntax", ...YAML],
    [".clang-format", ...YAML],
    ["CITATION.cff", ...YAML],
    // An rc file is JSON when the whole text parses as JSON, YAML otherwise
    [
      ".stylelintrc",
      `{\n  "singleQuote":true,   "semi": false\n}\n`,
      `{\n  "singleQuote": true,\n  "semi": false\n}\n`,
    ],
    [".prettierrc", "singleQuote:   true\nsemi: false\n", "singleQuote: true\nsemi: false\n"],
  ])("should format %s", async (filename, source, expected) => {
    const result = await format(filename, source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(expected);
  });
});
