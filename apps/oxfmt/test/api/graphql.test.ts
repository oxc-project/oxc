import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("GraphQL files (oxc_formatter_graphql)", () => {
  it("should format standalone GraphQL files in Rust", async () => {
    const source = `query { user(id:1){ name  email } }`;
    const result = await format("query.graphql", source);
    expect(result.code).toMatchInlineSnapshot(`
      "query {
        user(id: 1) {
          name
          email
        }
      }
      "
    `);
  });

  it("should format .gql and .graphqls extensions", async () => {
    const source = `type Query { hello:String }`;
    for (const filename of ["schema.gql", "schema.graphqls"]) {
      // oxlint-disable-next-line no-await-in-loop
      const result = await format(filename, source);
      expect(result.code).toBe("type Query {\n  hello: String\n}\n");
    }
  });

  it("should respect bracketSpacing and useTabs", async () => {
    const source = `{ user(filters: { active: true }) { name } }`;
    const result = await format("query.graphql", source, {
      bracketSpacing: false,
      useTabs: true,
    });
    expect(result.code).toMatchInlineSnapshot(`
      "{
      	user(filters: {active: true}) {
      		name
      	}
      }
      "
    `);
  });

  it("should fall back to Prettier for draft-spec syntax apollo-parser rejects", async () => {
    // Fragment arguments are graphql-js experimental syntax, not in the stable spec
    const source = `fragment F($x: Int) on T { f(arg: $x) }`;
    const result = await format("draft.graphql", source);
    expect(result.code).toMatchInlineSnapshot(`
      "fragment F($x: Int) on T {
        f(arg: $x)
      }
      "
    `);
  });

  it("should report Prettier's error when both parsers fail", async () => {
    const source = `query {{{`;
    const result = await format("broken.graphql", source);
    expect(result.code).toBe(source);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toMatch(/Syntax Error/);
  });
});
