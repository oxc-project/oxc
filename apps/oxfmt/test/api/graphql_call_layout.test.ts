import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("GraphQL call argument layout", () => {
  it("breaks inside the call when the embedded template does not fit beside the assignment", async () => {
    const source = `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(
  /* GraphQL */ \`
    query Q {
      x
    }
  \`
);
`;
    const options = { printWidth: 80, trailingComma: "es5" } as const;
    const result = await format("query.ts", source, options);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(source);
    expect((await format("query.ts", result.code, options)).code).toBe(result.code);
  });
  it.each([
    {
      name: "includes the configured trailing comma when arguments expand",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(
  /* GraphQL */ \`
    query Q {
      x
    }
  \`
);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(
  /* GraphQL */ \`
    query Q {
      x
    }
  \`,
);
`,
    },
    {
      name: "keeps the compact layout at a wider print width",
      options: {
        printWidth: 100,
        embeddedLanguageFormatting: "auto",
        trailingComma: "es5",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(
  /* GraphQL */ \`
    query Q {
      x
    }
  \`
);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(/* GraphQL */ \`
  query Q {
    x
  }
\`);
`,
    },
    {
      name: "keeps verbatim layout when embedding is disabled",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "off",
        trailingComma: "es5",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(
  /* GraphQL */ \`
    query Q {
      x
    }
  \`
);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  graphql(/* GraphQL */ \`
    query Q {
      x
    }
  \`);
`,
    },
    {
      name: "keeps the fallback layout for invalid GraphQL",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`query {{{\`);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  graphql(\`query {{{\`);
`,
    },
    {
      name: "formats empty templates without expanding the call",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`\`);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`\`);
`,
    },
    {
      name: "preserves the layout of comment-only templates",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`# comment
\`);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`
  # comment
\`);
`,
    },
    {
      name: "preserves trailing comments in a compact call",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`query Q{x y}\` /* tail */);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`
  query Q {
    x
    y
  }
\` /* tail */);
`,
    },
    {
      name: "also breaks calls whose original template is single-line",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "es5",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(/* GraphQL */ \`query Q{x y}\`);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(
  /* GraphQL */ \`
    query Q {
      x
      y
    }
  \`
);
`,
    },
    {
      name: "breaks the assignment when the callee itself does not fit",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(\`query Q{x y}\`);
`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  graphql(\`
    query Q {
      x
      y
    }
  \`);
`,
    },
    {
      name: "keeps verbatim layout when a later GraphQL quasi fails",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(/* GraphQL */ \`query Q{x}
\${fragment}
invalid\`);`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  graphql(/* GraphQL */ \`query Q{x}
\${fragment}
invalid\`);
`,
    },
    {
      name: "does not use a nested successful embed to expand a failed outer template",
      options: {
        printWidth: 80,
        embeddedLanguageFormatting: "auto",
        trailingComma: "all",
        arrowParens: "avoid",
      },
      source: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx = graphql(/* GraphQL */ \`
invalid \${graphql(/* GraphQL */ \`query Q{x}\`)} text
\`);`,
      expected: `const xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
  graphql(/* GraphQL */ \`
invalid \${graphql(/* GraphQL */ \`
    query Q {
      x
    }
  \`)} text
\`);
`,
    },
  ] as const)("$name", async ({ source, expected, options }) => {
    const result = await format("query.ts", source, options);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toBe(expected);
    expect((await format("query.ts", result.code, options)).code).toBe(result.code);
  });
});
