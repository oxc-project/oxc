import { describe, expect, it } from "vitest";
import { parse, parseSync } from "./parser.ts";
import type { Comment } from "../src-js/index.d.ts";

type NodeComments = Record<"leading" | "trailing" | "dangling", Comment[] | null>;

// Walk only AST properties. Comment buckets are metadata, and parent pointers
// must not turn this ownership check into a cyclic traversal.
function nodes(
  value: unknown,
  seen = new Set<object>(),
): Array<{ type: string; comments?: NodeComments | null }> {
  if (value === null || typeof value !== "object" || seen.has(value)) return [];
  seen.add(value);
  if (Array.isArray(value)) return value.flatMap((v) => nodes(v, seen));
  const node = value as { type?: string; comments?: NodeComments | null };
  const result: Array<{ type: string; comments?: NodeComments | null }> =
    typeof node.type === "string" ? [node as { type: string; comments?: NodeComments | null }] : [];
  for (const [key, child] of Object.entries(value)) {
    if (key !== "comments" && key !== "parent") result.push(...nodes(child, seen));
  }
  return result;
}

const fixtures = [
  "import {} from 'm'\n/* source end */;",
  "import { x } from 'm'\n/* source end */;",
  "import * as ns from 'm'\n/* source end */;",
  "import x from 'm'\n/* source end */;",
  "",
  "console.log(1);\nconsole.log(2);",
  "// head\nconst values = [/* inside */];\nvalues; // tail\n",
  "/* one */ /* two */ call(/* argument */ value /* after */); // eof",
  "function f(/* params */) { /* body */ }",
  "function f(/* before */ x = /* value */ 1, .../* rest */ args) {}",
  "const f = (/* params */) => ({ /* object */ });",
  "try { work(); } catch (/* caught */ error) { /* body */ }",
  "import {/* specifiers */} from 'm' with {/* attributes */};",
  "import * as ns from 'm' /* source end */;",
  "export {/* exports */} from 'm' with {/* attributes */};",
  "const t = tag`head${/* before */ x /* after */}tail`;",
  "/* @__PURE__ */ call(); /* @__NO_SIDE_EFFECTS__ */ function f() {}",
  "const x = {/* @__KEY__ */ 'key': value};",
  "// v8 ignore file\nclass C { /* body */ }",
  "// 🦀\nconst x = [/* 🦀 */]; // 🦀",
  "#!/usr/bin/env node\n// head\ncall();",
];

const typescript = [
  "function f<T>(/* params */): T { return null as T; }",
  "interface I { f(/* params */): void }",
  "type F = (/* params */) => void;",
  "class C { constructor(public /* key */ x = /* init */ 1) {} }",
  "namespace A.B { /* inside */ export const x = 1; }",
  "@dec class C { @dec method(/* params */) {} }",
  "function f(.../* before rest */ args /* after rest */: string[]) {}",
  "type T = typeof A /* before dot */ . /* after dot */ B;",
  "class C extends A /* before dot */ . /* after dot */ B {}",
  "import x = A /* before dot */ . /* after dot */ B;",
  "type T = {new(/* params */): T};",
  "interface I {new(/* params */): I}",
];

describe("attached comments", () => {
  for (const [lang, sources] of [
    ["js", fixtures],
    ["ts", typescript],
  ] as const) {
    for (const source of sources) {
      it(`${lang}: ${source}`, () => {
        const options = { lang, attachComments: true };
        const json = parseSync(`test.${lang}`, source, options);
        const raw = parseSync(`test.${lang}`, source, {
          ...options,
          experimentalRawTransfer: true,
        });
        expect(json.errors).toEqual([]);
        expect(raw.errors).toEqual([]);
        expect(raw.program).toEqual(json.program);
        expect(raw.comments).toEqual(json.comments);
        for (const parsed of [json, raw]) {
          const { comments }: { comments: Comment[] } = parsed; // Also check access before program.
          const owners = new Map();
          for (const node of nodes(parsed.program)) {
            expect(Object.hasOwn(node, "comments")).toBe(true);
            if (node.comments == null) continue;
            expect(Object.keys(node.comments)).toEqual(["leading", "trailing", "dangling"]);
            for (const bucket of Object.values(node.comments) as Array<Comment[] | null>) {
              if (bucket === null) continue;
              expect(bucket.length).toBeGreaterThan(0);
              expect(bucket.map((c) => c.start)).toEqual(
                bucket.map((c) => c.start).sort((a, b) => a - b),
              );
              for (const comment of bucket) {
                expect(comments).toContain(comment);
                expect(owners.has(comment)).toBe(false);
                owners.set(comment, node);
              }
            }
          }
          const hashbang = source.startsWith("#!") ? 1 : 0;
          expect(owners.size).toBe(comments.length - hashbang);
        }
      });
    }
  }

  it("omits metadata when disabled", () => {
    for (const experimentalRawTransfer of [false, true]) {
      const parsed = parseSync("test.js", "/* head */ call();", { experimentalRawTransfer });
      for (const node of nodes(parsed.program)) expect(Object.hasOwn(node, "comments")).toBe(false);
      expect(Object.keys(parsed.comments[0])).toEqual(["type", "value", "start", "end"]);
    }
  });

  it("preserves detached eager objects when their buffer is reused", () => {
    const options = { experimentalRawTransfer: true, attachComments: true };
    const first = parseSync("test.js", "[/* first */];", options);
    const expected = structuredClone(first.program);
    parseSync("test.js", "function second() { /* second */ }", options);
    expect(first.program).toEqual(expected);
  });

  it.for([false, true])(
    "supports async, semantic diagnostics, ranges and parent pointers (eager=%s)",
    async (experimentalRawTransfer) => {
      const result = await parse("test.ts", "// 🦀\nfunction f(/* inside */) {}", {
        experimentalRawTransfer,
        attachComments: true,
        showSemanticErrors: true,
        range: true,
        experimentalParent: true,
      });
      expect(result.errors).toEqual([]);
      expect(nodes(result.program.body[0])[0].comments?.dangling?.[0]).toBe(result.comments[1]);
      expect(experimentalRawTransfer ? result.program.body[0].parent : undefined).toBe(
        experimentalRawTransfer ? result.program : undefined,
      );
      expect(result.program.range).toEqual([result.program.start, result.program.end]);
    },
  );
});
