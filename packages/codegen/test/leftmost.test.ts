// Leftmost-position parenthesization.
//
// Three constructs record the output offset at which their child expression begins - an expression
// statement, a concise arrow body, and `export default`. A node printed at exactly that offset is
// the leftmost token of the construct, and several node types parenthesize themselves on finding
// that they are: an object literal or an object-destructuring assignment would otherwise open what
// reads as a block, and a function or class expression would read as a declaration.
//
// This suite pins that observable behaviour, not the mechanism which produces it. Every case is a
// source snippet printed by Rust `oxc_codegen` and by this package, and the two must agree - so
// nothing here states where the parens belong, only that both printers put them in the same places.
// It is meant to survive a rewrite of the position-marker machinery, and to be the gate on one.
//
// The matrix is every expression which can be the leftmost token of a construct, crossed with the
// three constructs which record an offset. A combination which cannot be written at all - `yield`
// in an arrow body, or anything needing an enclosing function after `export default` - is skipped
// with its reason, rather than quietly dropped.
//
// A snippet which does not parse is a fault in the snippet, not a reason to skip it, so every case
// requires `checkFixture` to have compared something.

import { describe, expect, it } from "vitest";

import { checkFixture } from "./utils/common.ts";

import type { Lang } from "./utils/common.ts";

// --- Cases ------------------------------------------------------------------------------------

/**
 * What an expression needs around it to be legal.
 *
 * Each context turns this into the enclosing code, or into the reason it cannot supply it.
 */
type Needs =
  | "nothing"
  /** `yield` */
  | "generator"
  /** `await` */
  | "async"
  /** `new.target` */
  | "function"
  /** A private name */
  | "class"
  /** `super` */
  | "derived-class";

/**
 * Where the bare source of an expression reads as something other than an expression.
 *
 * - `"block"` - a leading `{` opens a block at a statement start and a block body in an arrow, so
 *   both of those contexts parenthesize the source. `export default {}` is unambiguous, and takes it
 *   bare.
 * - `"comma"` - a top-level `,` ends both an arrow body and an `export default`, so both of those
 *   contexts parenthesize the source. A statement takes it bare.
 * - `"declaration"` - a leading `function` or `class` is a declaration at a statement start and after
 *   `export default`, so neither can host it. The `function-parenthesized` and `class-parenthesized`
 *   cases reach the marker in those two contexts instead.
 */
type Ambiguity = "block" | "comma" | "declaration";

interface Case {
  /** Test name, unique within the matrix */
  name: string;
  /** Source of the expression whose leftmost token is under test. Must be a single line. */
  expr: string;
  /** What the expression needs around it. `"nothing"` when absent. */
  needs?: Needs;
  /** Where `expr` does not read as an expression */
  ambiguity?: Ambiguity;
  /** Language to parse as. `"js"` when absent. */
  lang?: Lang;
  /** `true` where the snippet is only legal in a module */
  module?: boolean;
}

const CASES: Case[] = [
  // Identifiers, `this`, and every literal kind
  { name: "identifier", expr: "x" },
  { name: "this", expr: "this" },
  { name: "string", expr: `"str"` },
  { name: "number", expr: "1" },
  { name: "number-decimal", expr: "1.5" },
  { name: "number-negative", expr: "-1" },
  { name: "bigint", expr: "1n" },
  { name: "bigint-negative", expr: "-1n" },
  { name: "regex", expr: "/a/" },
  { name: "regex-flags", expr: "/a/g" },
  { name: "boolean-true", expr: "true" },
  { name: "boolean-false", expr: "false" },
  { name: "null", expr: "null" },

  // Member access, calls and `new`
  { name: "member-dot", expr: "x.y" },
  { name: "member-computed", expr: "x[0]" },
  { name: "member-deep", expr: "x.y.z.w" },
  { name: "member-private", expr: "this.#x", needs: "class" },
  { name: "call", expr: "f()" },
  { name: "call-arguments", expr: "f(1, 2)" },
  { name: "call-of-call", expr: "f()()" },
  { name: "call-of-member", expr: "x.y()" },
  { name: "call-parenthesized-callee", expr: "(f())()" },
  { name: "new", expr: "new C()" },
  { name: "new-no-arguments", expr: "new C" },
  { name: "new-member-callee", expr: "new x.C()" },
  { name: "new-call-callee", expr: "new (f())()" },
  { name: "template", expr: "`t`" },
  { name: "template-substitution", expr: "`a${b}c`" },
  { name: "tagged-template", expr: "tag`t`" },
  { name: "tagged-template-member-tag", expr: "x.tag`t`" },

  // Optional chaining, which arrives wrapped in a `ChainExpression`
  { name: "chain-member", expr: "x?.y" },
  { name: "chain-computed", expr: "x?.[0]" },
  { name: "chain-call", expr: "x?.()" },
  { name: "chain-deep", expr: "x?.y.z" },

  // Operators
  { name: "binary-add", expr: "x + y" },
  { name: "binary-exponent", expr: "x ** y" },
  { name: "binary-in", expr: "x in y" },
  { name: "binary-instanceof", expr: "x instanceof y" },
  { name: "logical-and", expr: "x && y" },
  { name: "logical-or", expr: "x || y" },
  { name: "logical-nullish", expr: "x ?? y" },
  { name: "private-in", expr: "#x in obj", needs: "class" },
  { name: "conditional", expr: "x ? y : z" },
  { name: "sequence", expr: "x, y", ambiguity: "comma" },
  { name: "unary-negate", expr: "-x" },
  { name: "unary-plus", expr: "+x" },
  { name: "unary-not", expr: "!x" },
  { name: "unary-bitwise-not", expr: "~x" },
  { name: "unary-typeof", expr: "typeof x" },
  { name: "unary-void", expr: "void 0" },
  { name: "unary-delete", expr: "delete x.y" },
  { name: "update-prefix-increment", expr: "++x" },
  { name: "update-prefix-decrement", expr: "--x" },
  { name: "update-postfix-increment", expr: "x++" },
  { name: "update-postfix-decrement", expr: "x--" },

  // Assignment, including the two destructuring targets
  { name: "assign", expr: "x = 1" },
  { name: "assign-compound", expr: "x += 1" },
  { name: "assign-logical", expr: "x ||= 1" },
  { name: "assign-member-target", expr: "x.y = 1" },
  { name: "assign-object-pattern", expr: "{ a } = b", ambiguity: "block" },
  { name: "assign-object-pattern-nested", expr: "{ a: { b } } = c", ambiguity: "block" },
  { name: "assign-object-pattern-default", expr: "{ a = 1 } = b", ambiguity: "block" },
  { name: "assign-object-pattern-rest", expr: "{ ...a } = b", ambiguity: "block" },
  { name: "assign-array-pattern", expr: "[a] = b" },
  { name: "assign-array-pattern-hole", expr: "[, a] = b" },
  { name: "assign-array-pattern-rest", expr: "[...a] = b" },

  // Object and array literals
  { name: "object-empty", expr: "{}", ambiguity: "block" },
  { name: "object-one-property", expr: "{ a: 1 }", ambiguity: "block" },
  { name: "object-two-properties", expr: "{ a: 1, b: 2 }", ambiguity: "block" },
  { name: "object-shorthand", expr: "{ a }", ambiguity: "block" },
  { name: "object-method", expr: "{ m() {} }", ambiguity: "block" },
  { name: "object-spread", expr: "{ ...x }", ambiguity: "block" },
  { name: "array-empty", expr: "[]" },
  { name: "array-elements", expr: "[1, 2]" },
  { name: "array-holding-object", expr: "[{}]" },

  // Functions, classes and arrows
  { name: "function", expr: "function () {}", ambiguity: "declaration" },
  { name: "function-named", expr: "function f() {}", ambiguity: "declaration" },
  { name: "function-async", expr: "async function () {}", ambiguity: "declaration" },
  { name: "function-generator", expr: "function* () {}", ambiguity: "declaration" },
  { name: "function-parenthesized", expr: "(function () {})" },
  { name: "function-parenthesized-named", expr: "(function f() {})" },
  { name: "class", expr: "class {}", ambiguity: "declaration" },
  { name: "class-named", expr: "class C {}", ambiguity: "declaration" },
  { name: "class-extends", expr: "class extends D {}", ambiguity: "declaration" },
  { name: "class-parenthesized", expr: "(class {})" },
  { name: "class-parenthesized-named", expr: "(class C {})" },
  { name: "arrow", expr: "() => 1" },
  { name: "arrow-async", expr: "async () => 1" },
  { name: "arrow-single-parameter", expr: "x => x" },
  { name: "arrow-block-body", expr: "() => {}" },
  { name: "arrow-object-body", expr: "() => ({ a: 1 })" },
  { name: "arrow-of-arrow-object-body", expr: "() => () => ({ a: 1 })" },

  // `await`, `yield`, `import()`, and the meta properties
  { name: "await", expr: "await x", needs: "async" },
  { name: "await-member", expr: "await x.y", needs: "async" },
  { name: "yield", expr: "yield x", needs: "generator" },
  { name: "yield-bare", expr: "yield", needs: "generator" },
  { name: "yield-delegate", expr: "yield* x", needs: "generator" },
  { name: "import-call", expr: `import("mod")` },
  { name: "import-call-member", expr: `import("mod").then` },
  { name: "import-meta", expr: "import.meta", module: true },
  { name: "import-meta-member", expr: "import.meta.url", module: true },
  { name: "new-target", expr: "new.target", needs: "function" },

  // `super`
  { name: "super-call", expr: "super()", needs: "derived-class" },
  { name: "super-property", expr: "super.x", needs: "derived-class" },
  { name: "super-computed", expr: "super[0]", needs: "derived-class" },

  // Sources whose parens survive into the AST in the preserve-parens mode
  { name: "parenthesized-identifier", expr: "(x)" },
  { name: "parenthesized-object", expr: "({})" },
  { name: "parenthesized-object-twice", expr: "(({}))" },
  { name: "parenthesized-sequence", expr: "(x, y)" },
  { name: "parenthesized-arrow", expr: "(() => 1)" },

  // Nested leftmost chains - the node at the recorded offset is arbitrarily deep
  { name: "nested-object-member", expr: "({}).x" },
  { name: "nested-object-member-deep", expr: "({}).a.b.c" },
  { name: "nested-object-computed", expr: "({})[0]" },
  { name: "nested-object-call", expr: "({}).x()" },
  { name: "nested-object-optional", expr: "({})?.x" },
  { name: "nested-object-assign", expr: "({ a: 1 }).b = 2" },
  { name: "nested-object-update", expr: "({}).x++" },
  { name: "nested-object-binary", expr: "({}) + 1" },
  { name: "nested-object-instanceof", expr: "({}) instanceof Object" },
  { name: "nested-object-logical", expr: "({}) && x" },
  { name: "nested-object-conditional", expr: "({}) ? a : b" },
  { name: "nested-object-sequence", expr: "({}), 1", ambiguity: "comma" },
  { name: "nested-object-tagged-template", expr: "({}).tag`t`" },
  { name: "nested-object-new-callee", expr: "new (({}).C)()" },
  { name: "nested-object-pattern-assign-member", expr: "({ a } = b).c" },
  { name: "nested-function-call", expr: "(function () {})()" },
  { name: "nested-function-member", expr: "(function () {}).call()" },
  { name: "nested-function-assign", expr: "(function () {}).x = 1" },
  { name: "nested-function-binary", expr: "(function () {}) + 1" },
  { name: "nested-function-tagged-template", expr: "(function () {})`t`" },
  { name: "nested-function-async-call", expr: "(async function () {})()" },
  { name: "nested-function-generator-call", expr: "(function* () {})()" },
  { name: "nested-class-member", expr: "(class {}).x" },
  { name: "nested-class-call", expr: "(class {})()" },
  { name: "nested-class-binary", expr: "(class {}) + 1" },
  { name: "nested-arrow-call", expr: "(() => 1)()" },

  // Shapes which must NOT be parenthesized - the node sits past the recorded offset
  { name: "object-as-argument", expr: "f({})" },
  { name: "object-as-assignment-value", expr: "x = {}" },
  { name: "object-as-property-value", expr: "({ a: {} })" },
  { name: "object-as-right-operand", expr: "1 + {}" },
  { name: "object-after-unary-not", expr: "!{}" },
  { name: "object-after-typeof", expr: "typeof {}" },
  { name: "function-as-argument", expr: "f(function () {})" },
  { name: "function-as-assignment-value", expr: "x = function () {}" },
  { name: "class-as-assignment-value", expr: "x = class {}" },

  // JSX
  { name: "jsx-element", expr: "<div />", lang: "jsx" },
  { name: "jsx-element-children", expr: "<div>text</div>", lang: "jsx" },
  { name: "jsx-element-member-name", expr: "<a.b />", lang: "jsx" },
  { name: "jsx-fragment", expr: "<></>", lang: "jsx" },
  { name: "nested-jsx-member", expr: "(<div />).props", lang: "jsx" },

  // TypeScript
  { name: "ts-as", expr: "x as any", lang: "ts" },
  { name: "ts-satisfies", expr: "x satisfies object", lang: "ts" },
  { name: "ts-non-null", expr: "x!", lang: "ts" },
  { name: "ts-type-assertion", expr: "<string> x", lang: "ts" },
  { name: "ts-instantiation", expr: "f<string>", lang: "ts" },
  { name: "ts-as-object", expr: "{} as any", lang: "ts", ambiguity: "block" },
  { name: "ts-satisfies-object", expr: "{} satisfies object", lang: "ts", ambiguity: "block" },
  { name: "ts-type-assertion-object", expr: "<any> {}", lang: "ts" },
  { name: "nested-ts-as-object", expr: "({} as any).x", lang: "ts" },
  { name: "nested-ts-non-null-object", expr: "({})!", lang: "ts" },
  { name: "nested-ts-non-null-object-member", expr: "({})!.x", lang: "ts" },
  { name: "nested-ts-as-function", expr: "(function () {}) as any", lang: "ts" },
  // `TSParenthesizedType`, the type-level counterpart, which the preserve-parens mode keeps
  { name: "ts-as-parenthesized-type", expr: "x as (string)", lang: "ts" },
  { name: "ts-type-assertion-parenthesized-type", expr: "<(string)> x", lang: "ts" },
  { name: "nested-ts-as-parenthesized-type-object", expr: "({} as (any)).x", lang: "ts" },
];

// --- Contexts ---------------------------------------------------------------------------------

/** A built snippet, or the reason a context cannot host the expression. */
type Built = { source: string } | { skip: string };

interface Context {
  /** Name of the construct which records the offset */
  name: string;
  /** `true` where the construct is only legal in a module */
  module: boolean;
  /** Build the snippet for a case, or give the reason this construct cannot host it */
  build: (testCase: Case) => Built;
}

/**
 * Put a statement inside whatever it needs around it.
 *
 * Every snippet opens with a throwaway `null;` immediately before the construct under test, so that
 * the construct never starts at output offset 0 - where a marker still holding its initial value
 * would match by accident - and so that a leading string literal is an expression statement rather
 * than a directive.
 *
 * @param needs - What the statement needs around it
 * @param statement - The statement, which must be a single line
 * @returns Snippet source
 */
function enclose(needs: Needs, statement: string): string {
  switch (needs) {
    case "async":
      return `async function f() {\n\tnull;\n\t${statement}\n}\n`;
    case "generator":
      return `function* g() {\n\tnull;\n\t${statement}\n}\n`;
    case "function":
      return `function f() {\n\tnull;\n\t${statement}\n}\n`;
    case "class":
      return `class C {\n\t#x;\n\tm(obj) {\n\t\tnull;\n\t\t${statement}\n\t}\n}\n`;
    case "derived-class":
      return `class C extends D {\n\tconstructor() {\n\t\tnull;\n\t\t${statement}\n\t}\n}\n`;
    default:
      return `null;\n${statement}\n`;
  }
}

/** The reason `export default` cannot host an expression needing an enclosing construct. */
const EXPORT_DEFAULT_NEEDS_SKIPS: Partial<Record<Needs, string>> = {
  generator:
    "`yield` needs an enclosing generator, and `export default` is only legal at the top level of a module",
  function:
    "`new.target` needs an enclosing function, and `export default` is only legal at the top level of a module",
  class:
    "a private name needs an enclosing class body, and `export default` is only legal at the top level of a module",
  "derived-class":
    "`super` needs an enclosing class method, and `export default` is only legal at the top level of a module",
};

const CONTEXTS: Context[] = [
  {
    name: "expression statement",
    module: false,
    build({ expr, needs = "nothing", ambiguity }) {
      if (ambiguity === "declaration") {
        return {
          skip:
            "a statement starting `function` or `class` is a declaration, not an expression - " +
            "the `function-parenthesized` and `class-parenthesized` cases cover this construct instead",
        };
      }
      return { source: enclose(needs, `${ambiguity === "block" ? `(${expr})` : expr};`) };
    },
  },
  {
    name: "arrow body",
    module: false,
    build({ expr, needs = "nothing", ambiguity }) {
      if (needs === "generator") {
        return {
          skip: "an arrow body is parsed without the enclosing generator's `yield`, so `yield` there is a syntax error",
        };
      }
      const body = ambiguity === "block" || ambiguity === "comma" ? `(${expr})` : expr;
      // An `await` body needs the arrow itself to be async, rather than an enclosing async function
      if (needs === "async") return { source: enclose("nothing", `async () => ${body};`) };
      return { source: enclose(needs, `() => ${body};`) };
    },
  },
  {
    name: "export default",
    module: true,
    build({ expr, needs = "nothing", ambiguity }) {
      if (ambiguity === "declaration") {
        return {
          skip:
            "`export default function () {}` is a `FunctionDeclaration` and `export default class {}` a " +
            "`ClassDeclaration`, neither of which records an offset - the `function-parenthesized` and " +
            "`class-parenthesized` cases cover this construct instead",
        };
      }
      // `await` needs nothing: a module allows it at the top level
      const needsSkip = EXPORT_DEFAULT_NEEDS_SKIPS[needs];
      if (needsSkip !== undefined) return { skip: needsSkip };

      return { source: `null;\nexport default ${ambiguity === "comma" ? `(${expr})` : expr};\n` };
    },
  },
];

// --- Tests ------------------------------------------------------------------------------------

describe.concurrent("leftmost position", () => {
  describe.for(CONTEXTS)("$name", (context) => {
    it.for(CASES)("$name", (testCase, ctx) => {
      const built = context.build(testCase);

      // Reported as skipped, carrying the reason, rather than left out of the run
      if ("skip" in built) return ctx.skip(built.skip);

      const lang = testCase.lang ?? "js";
      const sourceType = context.module || testCase.module ? "module" : "script";
      const checked = checkFixture(`leftmost.${lang}`, built.source, lang, sourceType);
      expect(checked, `snippet does not parse:\n${built.source}`).toBe(true);
    });
  });
});
