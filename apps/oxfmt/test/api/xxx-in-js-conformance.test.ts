import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import prettier from "prettier";
import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// NOTE: Fixtures can be downloaded by `pnpm download-prettier-fixtures`
const FIXTURES_DIR = join(import.meta.dirname, "../../prettier-fixtures");

describe("Prettier conformance for graphql-in-js", () => {
  const graphqlFixtures = collectFixtures(".js", "js/multiparser-graphql", [
    "format.test.js",
    "comment-tag.js", // /* GraphQL */
  ]);
  // TODO: Fix in next PR
  //   graphqlFixtures.push(
  //     {
  //       name: "inline/gql-trailing-space.js",
  //       content: `export const schema = gql\`
  //   type Mutation {
  //     create__TYPE_NAME__(input: Create__TYPE_NAME__Input!): __TYPE_NAME__!
  //       @skipAuth
  //     update__TYPE_NAME__(
  //       id: Int!
  //       input: Update__TYPE_NAME__Input!
  //     ): __TYPE_NAME__! @skipAuth
  //     delete__TYPE_NAME__(id: Int!): __TYPE_NAME__! @skipAuth
  //   }
  // \`
  // `,
  //     },
  //   );
  graphqlFixtures.push({
    name: "inline/gql-dummy.js",
    content: `export const schema = gql\`
  type Relation {
    id: Int!
    name: String!
  }
  \`
  `,
  });

  describe.concurrent.each(graphqlFixtures)("$name", ({ name, content }) => {
    it.each([{ printWidth: 80 }])("%j", async (options) => {
      const [oxfmtRes, prettierRes] = await compareWithPrettier(name, content, "babel", options);
      expect(oxfmtRes).toBe(prettierRes);
    });
  });
});

// ---

type TestCase = { name: string; content: string };

function collectFixtures(ext: string, subdir: string, excludes: string[] = []): TestCase[] {
  const dir = join(FIXTURES_DIR, subdir);
  // NOTE: In CI, the fixtures might not be present, just skip and only run edge cases.
  if (!existsSync(dir)) return [];

  const results: TestCase[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true, recursive: true })) {
    if (!entry.isFile() || !entry.name.endsWith(ext)) continue;

    const fullPath = join(entry.parentPath, entry.name);
    const relPath = relative(dir, fullPath);
    if (excludes.some((s) => relPath.includes(s))) continue;

    results.push({ name: relPath, content: readFileSync(fullPath, "utf8") });
  }

  return results.sort((a, b) => a.name.localeCompare(b.name));
}

async function compareWithPrettier(
  fileName: string,
  content: string,
  parser: string,
  options = {},
) {
  let prettierResult;
  try {
    prettierResult = await prettier.format(content, {
      parser,
      filepath: fileName,
      ...options,
    });
  } catch {
    prettierResult = "ERROR";
  }

  let oxfmtResult;
  const res = await format(fileName, content, options);
  if (res.errors.length !== 0) {
    oxfmtResult = "ERROR";
  } else {
    oxfmtResult = res.code;
  }

  return [oxfmtResult, prettierResult];
}
