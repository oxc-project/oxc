import { describe, expect, it } from "vitest";

import { transform, transformSync } from "../index";

const fixture = "const data = graphql`query FooQuery { id }`;\n";

describe("transformSync", () => {
  it("hoists an ES import by default", () => {
    const result = transformSync("foo.js", fixture);

    expect(result.errors).toEqual([]);
    expect(result.code).toMatchInlineSnapshot(`
      "import _FooQuery from "./__generated__/FooQuery.graphql.js";
      const data = _FooQuery;
      "
    `);
  });

  it("supports Relay options", () => {
    const result = transformSync("project/src/pages/Foo.tsx", fixture, {
      artifactDirectory: "project/src/__generated__",
      language: "typescript",
      eagerEsModules: false,
    });

    expect(result.errors).toEqual([]);
    expect(result.code).toContain('require("../__generated__/FooQuery.graphql.ts")');
  });

  it("reports transform and option errors", () => {
    const unnamed = transformSync("foo.js", "const data = graphql`{ id }`;");

    expect(unnamed.code).toBe("");
    expect(unnamed.errors[0].message).toContain("named GraphQL");

    // @ts-expect-error Testing runtime validation.
    const invalidOption = transformSync("foo.js", fixture, { language: "elm" });
    expect(invalidOption.code).toBe("");
    expect(invalidOption.errors[0].message).toContain("language");
  });
});

describe("transform", () => {
  it("transforms asynchronously", async () => {
    const result = await transform("foo.js", fixture, { eagerEsModules: false });

    expect(result.errors).toEqual([]);
    expect(result.code).toContain('require("./__generated__/FooQuery.graphql.js")');
  });
});
