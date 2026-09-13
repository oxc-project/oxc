// Hand-built AST tests, for the shapes the conformance suites cannot reach.
//
// The suites print ASTs produced by a parser, so anything a parser never produces is invisible to
// them. The clearest case is a negative numeric literal: `-1` parses as a `UnaryExpression` around
// `Literal(1)`, so `Literal(-1)` never appears, and the printer's whole negative-number path is
// unreachable from any fixture. A stale identifier on that path once survived all 58,609 suite
// fixtures and only turned up in `node --run lint`.
//
// The ASTs here are written by hand to reach exactly those places, and each case states the output
// the printer is meant to produce for it. Anything which changes an expected string is either a bug
// or a deliberate change which needs the string updated with it.

import { parseSync } from "oxc-parser";
import { describe, expect, test } from "vitest";

import { printSync } from "../dist/index.js";

import type * as ESTree from "../../../npm/oxc-types/types.d.ts";

// Parse options for the cases which give source rather than an AST.
//
// `preserveParens: false` because this printer deliberately does not support the redundant
// `ParenthesizedExpression` node. `experimentalRawTransfer` is how the printer is meant to be used,
// so the ASTs are the shape it really sees.
//
// `experimentalRawTransfer` is not in the published `ParserOptions` type - it is experimental and
// untyped. Declaring the object here rather than inline at the call site keeps TypeScript's excess
// property check (which only applies to object literals) from rejecting it, without a cast.
const PARSE_OPTIONS = {
  preserveParens: false,
  experimentalRawTransfer: true,
};

// --- AST builders ---------------------------------------------------------------------------

/**
 * Build an AST node.
 *
 * The printer reads only the fields these builders set, so the nodes below carry none of the
 * `start` / `end` positions a parser would attach. This puts the cast that says so in one place,
 * rather than at every builder.
 *
 * @param fields - Fields the node needs
 * @returns The node
 */
const node = <T extends ESTree.Node>(fields: Partial<T>): T => fields as T;

const program = (...body: ESTree.Statement[]) =>
  node<ESTree.Program>({ type: "Program", sourceType: "module", body });
const stmt = (expression: ESTree.Expression) =>
  node<ESTree.ExpressionStatement>({ type: "ExpressionStatement", expression });
const num = (value: number) => node<ESTree.NumericLiteral>({ type: "Literal", value });
const str = (value: string) => node<ESTree.StringLiteral>({ type: "Literal", value });
const id = (name: string) => node<ESTree.IdentifierReference>({ type: "Identifier", name });
const regex = (pattern: string, flags: string) =>
  node<ESTree.RegExpLiteral>({ type: "Literal", value: null, regex: { pattern, flags } });
const big = (value: string) => node<ESTree.BigIntLiteral>({ type: "Literal", bigint: value });
const member = (object: ESTree.Expression, property: ESTree.IdentifierName) =>
  node<ESTree.StaticMemberExpression>({
    type: "MemberExpression",
    object,
    property,
    computed: false,
    optional: false,
  });
const call = (callee: ESTree.Expression, args: ESTree.Argument[] = []) =>
  node<ESTree.CallExpression>({
    type: "CallExpression",
    callee,
    arguments: args,
    optional: false,
  });
const unary = (operator: ESTree.UnaryOperator, argument: ESTree.Expression) =>
  node<ESTree.UnaryExpression>({ type: "UnaryExpression", operator, argument, prefix: true });
const update = (
  operator: ESTree.UpdateOperator,
  argument: ESTree.SimpleAssignmentTarget,
  prefix = true,
) => node<ESTree.UpdateExpression>({ type: "UpdateExpression", operator, argument, prefix });
const bin = (operator: ESTree.BinaryOperator, left: ESTree.Expression, right: ESTree.Expression) =>
  node<ESTree.BinaryExpression>({ type: "BinaryExpression", operator, left, right });
const seq = (...expressions: ESTree.Expression[]) =>
  node<ESTree.SequenceExpression>({ type: "SequenceExpression", expressions });
const arr = (...elements: ESTree.ArrayExpressionElement[]) =>
  node<ESTree.ArrayExpression>({ type: "ArrayExpression", elements });
const ret = (argument: ESTree.Expression) =>
  node<ESTree.ReturnStatement>({ type: "ReturnStatement", argument });

// One expression statement, which is how nearly every case below is shaped
const e = (expression: ESTree.Expression) => program(stmt(expression));

// --- Runner ---------------------------------------------------------------------------------

/** A case: its name, the AST to print, and the output the printer must produce for it. */
type Case = [name: string, ast: ESTree.Program | ESTree.Statement, output: string];

/**
 * Define one test per case, checking the printer's output is exactly the expected string.
 *
 * @param cases - The cases
 */
function checkCases(cases: Case[]): void {
  test.each(cases)("%s", (_name, ast, output) => {
    expect(printSync(ast).code).toBe(output);
  });
}

// --- Tests ----------------------------------------------------------------------------------

describe("single statements", () => {
  /** Parse one source statement for public-API tests which do not print a whole program. */
  function parseStatement(filename: string, sourceText: string): ESTree.Statement {
    const { program: parsed, errors } = parseSync(filename, sourceText, PARSE_OPTIONS);
    if (errors.length > 0) throw new Error(`fixture parse failed: ${errors[0].message}`);
    const [statement] = parsed.body;
    if (statement === undefined || parsed.body.length !== 1) {
      throw new Error(`Expected exactly one statement in ${filename}`);
    }
    return statement;
  }

  test("prints a JavaScript statement", () => {
    const statement = parseStatement("statement.js", "const value=1;");
    expect(printSync(statement).code).toBe("const value = 1;\n");
  });

  test("prints a TypeScript statement", () => {
    const statement = parseStatement("statement.ts", "type Box<T>={value:T};");
    expect(printSync(statement, { ts: true }).code).toBe("type Box<T> = {\n\tvalue: T;\n};\n");
  });

  test("returns a source map for a JavaScript statement", () => {
    const sourceText = "const value=1;";
    const statement = parseStatement("statement.js", sourceText);
    const { code, map } = printSync(statement, {
      sourcemap: true,
      sourceFilename: "statement.js",
      sourceText,
    });
    expect(code).toBe("const value = 1;\n");
    expect(map).toMatchObject({
      sources: ["statement.js"],
      sourcesContent: [sourceText],
    });
  });
});

describe("indent", () => {
  const ast = e(id("x"));

  test.each(["", "x", "\n", "\r\n", " x "])("rejects %j", (indent) => {
    expect(() => printSync(ast, { indent })).toThrow(
      new TypeError("`indent` must be a non-empty string containing only spaces and tabs"),
    );
  });

  test.each([4, null, {}])("rejects non-string value %j", (indent) => {
    expect(() => printSync(ast, { indent: indent as unknown as string })).toThrow(
      new TypeError("`indent` must be a non-empty string containing only spaces and tabs"),
    );
  });
});

describe("starting indent level", () => {
  const ast = e(id("x"));

  test("indents from a valid level", () => {
    expect(printSync(ast, { startingIndentLevel: 1 }).code).toBe("\tx;\n");
  });

  test("accepts the maximum level", () => {
    expect(printSync(ast, { startingIndentLevel: 1000 }).code).toBe(`${"\t".repeat(1000)}x;\n`);
  });

  test.each([
    ["infinity", Infinity],
    ["negative infinity", -Infinity],
    ["not a number", NaN],
    ["fraction", 0.5],
    ["negative", -1],
    ["above the maximum", 1001],
  ])("rejects %s", (_name, startingIndentLevel) => {
    expect(() => printSync(ast, { startingIndentLevel })).toThrow(
      "`startingIndentLevel` must be a non-negative safe integer no greater than 1000",
    );
  });
});

// A numeric literal followed by `.` needs a space when the number is plain digits, because
// `0.toExponential()` would lex the `.` as part of the number. This is the `needSpaceBeforeDot`
// path, and every case below is unreachable from a parsed fixture in at least one respect.
describe("member access on numbers", () => {
  checkCases([
    ["int-zero", e(call(member(num(0), id("toExponential")))), "0 .toExponential();\n"],
    ["int-one", e(member(num(1), id("x"))), "1 .x;\n"],
    ["int-large", e(member(num(255), id("x"))), "255 .x;\n"],
    ["int-999", e(member(num(999), id("x"))), "999 .x;\n"],
    ["int-1000", e(member(num(1000), id("x"))), "1e3.x;\n"],
    ["decimal", e(member(num(0.5), id("x"))), ".5.x;\n"],
    ["decimal-leading", e(member(num(1.5), id("x"))), "1.5.x;\n"],
    ["exponent-large", e(member(num(1e21), id("x"))), "1e21.x;\n"],
    ["exponent-small", e(member(num(1e-7), id("x"))), "1e-7.x;\n"],
    ["negative-int", e(member(num(-1), id("x"))), "(-1).x;\n"],
    ["negative-decimal", e(member(num(-0.5), id("x"))), "(-.5).x;\n"],
    ["negative-infinity", e(member(num(-Infinity), id("x"))), "(-Infinity).x;\n"],
    ["infinity", e(member(num(Infinity), id("x"))), "Infinity.x;\n"],
    ["nan", e(member(num(NaN), id("x"))), "NaN.x;\n"],
    ["chained-int", e(member(member(num(7), id("x")), id("y"))), "7 .x.y;\n"],
    ["call-arg-int", e(call(id("f"), [member(num(3), id("x"))])), "f(3 .x);\n"],
    // The shortened forms, which all carry a `.`, an `e` or an `x` of their own and so must NOT
    // gain the space, against the long integers which stay plain digits and must
    ["int-1500", e(member(num(1500), id("x"))), "1500 .x;\n"],
    ["int-max-safe", e(member(num(9007199254740991), id("x"))), "9007199254740991 .x;\n"],
    ["hex", e(member(num(281474976710655), id("x"))), "0xffffffffffff.x;\n"],
    ["exponent-fold", e(member(num(1.2e101), id("x"))), "12e100.x;\n"],
  ]);
});

describe("numbers", () => {
  // Bare numeric literals, including the negative and non-finite paths
  checkCases([
    ["int", e(num(42)), "42;\n"],
    ["zero", e(num(0)), "0;\n"],
    ["negative-zero", e(num(-0)), "-0;\n"],
    ["negative-int", e(num(-1)), "-1;\n"],
    ["negative-decimal", e(num(-0.5)), "-.5;\n"],
    ["negative-large", e(num(-1e21)), "-1e21;\n"],
    ["negative-small", e(num(-0.000001)), "-1e-6;\n"],
    ["infinity", e(num(Infinity)), "Infinity;\n"],
    ["negative-infinity", e(num(-Infinity)), "-Infinity;\n"],
    ["nan", e(num(NaN)), "NaN;\n"],
    ["exponent", e(num(1e21)), "1e21;\n"],
    ["return-negative", program(ret(num(-1))), "return -1;\n"],
    ["return-int", program(ret(num(1))), "return 1;\n"],
    ["binary-negative", e(bin("+", id("a"), num(-1))), "a + -1;\n"],
    [
      "array-negatives",
      e(arr(num(-1), num(-0), num(-Infinity))),
      "[\n\t-1,\n\t-0,\n\t-Infinity\n];\n",
    ],
  ]);

  // The shortest-form rules. A number prints as whichever of plain digits, hexadecimal and
  // exponent notation is shortest, and the suites reach almost none of it: real code writes small
  // integers. Every form and every "not worth it" boundary is here.
  checkCases([
    ["int-1500", e(num(1500)), "1500;\n"], // trailing zeros too few to pay for the `e`
    // 16 digits, and hexadecimal still does not pay
    ["int-max-safe", e(num(9007199254740991)), "9007199254740991;\n"],
    ["hex", e(num(281474976710655)), "0xffffffffffff;\n"],
    ["hex-negative", e(num(-281474976710655)), "-0xffffffffffff;\n"],
    // hexadecimal is tried first and wins, though `1e12` is shorter
    ["hex-over-exponent", e(num(1e12)), "0xe8d4a51000;\n"],
    // The two cases above reach the hexadecimal test from plain digits. These reach it from
    // exponent notation, where the length it is judged against is one less than the text `String`
    // gave, because the `+` of the exponent always goes. The second is the boundary: hexadecimal is
    // exactly as long as the exponent form, so it must lose - and would wrongly win, as
    // `0x21e2073de72ea800000`, if that one character were not taken off.
    ["hex-from-exponent", e(num(1.0000990573316814e21)), "0x36372999e429e40000;\n"],
    ["hex-from-exponent-boundary", e(num(1.0000473745167254e22)), "10000473745167254e6;\n"],
    ["exponent-fold", e(num(1.2e101)), "12e100;\n"], // the point folds into the exponent
    ["max-value", e(num(1.7976931348623157e308)), "17976931348623157e292;\n"],
    ["min-value", e(num(5e-324)), "5e-324;\n"],
    ["small-fold", e(num(1.5e-7)), "15e-8;\n"],
    ["small-no-fold", e(num(1.5e-9)), "1.5e-9;\n"], // folding would lengthen the exponent
    ["decimal-thousandth", e(num(0.001)), ".001;\n"], // leading zeros too few to pay for the `e-`
    ["decimal-ten-thousandth", e(num(0.0001)), "1e-4;\n"],
  ]);
});

// String printing has a deliberately broad fast path. In particular, a `<` by itself is harmless;
// only the case-insensitive `</script` sequence must be broken up for safe inline-script output.
describe("strings", () => {
  checkCases([
    ["harmless less-than", e(str("a < b; <div> is text")), '("a < b; <div> is text");\n'],
    ["script close tag", e(str("</script")), '("<\\/script");\n'],
    ["mixed-case script close tag", e(str("</ScRiPt")), '("<\\/ScRiPt");\n'],
    // This is a prefix check, matching the HTML parser behavior and the previous slow-path test.
    ["script close tag prefix", e(str("</scripture")), '("<\\/scripture");\n'],
    [
      "quote and controls",
      e(str('"\\\0' + "1\n\r\u0007\u000b\f\u001b\u00a0\u2028\u2029")),
      '("\\\"\\\\\\x001\\n\\r\\x07\\v\\f\\x1B\\xA0\\u2028\\u2029");\n',
    ],
    ["paired surrogate", e(str("\ud83d\ude00")), '("😀");\n'],
    ["lone high surrogate", e(str("\ud800")), '("\\ud800");\n'],
    ["lone low surrogate", e(str("\udc00")), '("\\udc00");\n'],
  ]);

  test("template literal quasis also escape script close tags", () => {
    const { program: parsed, errors } = parseSync(
      "fixture.js",
      "const value = `before </ScRiPt> after`;",
      PARSE_OPTIONS,
    );
    if (errors.length > 0) throw new Error(`fixture parse failed: ${errors[0].message}`);
    expect(printSync(parsed).code).toBe("const value = `before <\\/ScRiPt> after`;\n");
  });
});

// A negative bigint is parenthesized where the position binds tighter than a prefix operator, the
// same rule `printNumericLiteral` follows. `-1n` parses as a unary minus around a positive literal,
// so a `BigIntLiteral` whose text starts with `-` never comes from a parser and this whole branch
// is unreachable from any fixture.
describe("bigints", () => {
  checkCases([
    ["positive", e(big("1")), "1n;\n"],
    ["negative", e(big("-1")), "-1n;\n"],
    ["negative-hex", e(big("-0x10")), "-0x10n;\n"],
    ["positive-member", e(member(big("1"), id("x"))), "1n.x;\n"],
    ["negative-member", e(member(big("-1"), id("x"))), "(-1n).x;\n"],
    ["negative-typeof", e(unary("typeof", big("-1"))), "typeof -1n;\n"],
  ]);
});

// A regex with no flags leaves the closing `/` as the last thing written, which is what stops
// `/a//b/` lexing as a line comment and what forces a space before a following identifier.
describe("regexes", () => {
  checkCases([
    ["no-flags", e(regex("a", "")), "/a/;\n"],
    ["flags", e(regex("a", "g")), "/a/g;\n"],
    ["member-no-flags", e(call(member(regex("a", ""), id("test")), [id("x")])), "/a/.test(x);\n"],
    ["member-flags", e(call(member(regex("a", "gi"), id("test")), [id("x")])), "/a/gi.test(x);\n"],
    ["in-array", e(arr(regex("a", ""), regex("b", "g"))), "[/a/, /b/g];\n"],
    ["sequence", e(seq(regex("a", ""), regex("b", ""))), "/a/, /b/;\n"],
    ["typeof", e(unary("typeof", regex("a", ""))), "typeof /a/;\n"],
    ["divide", e(bin("/", regex("a", ""), id("x"))), "/a/ / x;\n"],
    ["instanceof", e(bin("instanceof", regex("a", ""), id("RegExp"))), "/a/ instanceof RegExp;\n"],
    ["script-pattern", e(regex("script", "")), "/script/;\n"],
    ["return-no-flags", program(ret(regex("a", ""))), "return /a/;\n"],
  ]);
});

// `+ +x` and `- -x` need a separating space; `+ -x` does not. This is the whole
// `printSpaceBeforeOperator` family.
describe("unary and update expressions", () => {
  checkCases([
    ["plus-plus", e(unary("+", unary("+", id("x")))), "+ +x;\n"],
    ["minus-minus", e(unary("-", unary("-", id("x")))), "- -x;\n"],
    ["plus-minus", e(unary("+", unary("-", id("x")))), "+-x;\n"],
    ["minus-plus", e(unary("-", unary("+", id("x")))), "-+x;\n"],
    ["plus-preinc", e(unary("+", update("++", id("x")))), "+ ++x;\n"],
    ["minus-predec", e(unary("-", update("--", id("x")))), "- --x;\n"],
    ["plus-predec", e(unary("+", update("--", id("x")))), "+--x;\n"],
    ["minus-preinc", e(unary("-", update("++", id("x")))), "-++x;\n"],
    ["not-not", e(unary("!", unary("!", id("x")))), "!!x;\n"],
    ["tilde-tilde", e(unary("~", unary("~", id("x")))), "~~x;\n"],
    ["not-predec", e(unary("!", update("--", id("x")))), "!--x;\n"],
    ["typeof", e(unary("typeof", id("x"))), "typeof x;\n"],
    ["void", e(unary("void", num(0))), "void 0;\n"],
    ["delete", e(unary("delete", member(id("x"), id("y")))), "delete x.y;\n"],
    ["delete-infinity", e(unary("delete", num(Infinity))), "delete (0, Infinity);\n"],
    ["minus-negative-literal", e(unary("-", num(-1))), "- -1;\n"],
    ["plus-negative-literal", e(unary("+", num(-1))), "+-1;\n"],
    ["preinc", e(update("++", id("x"))), "++x;\n"],
    ["postinc", e(update("++", id("x"), false)), "x++;\n"],
    ["predec", e(update("--", id("x"))), "--x;\n"],
    ["postdec", e(update("--", id("x"), false)), "x--;\n"],
    ["postinc-plus", e(bin("+", update("++", id("x"), false), id("y"))), "x++ + y;\n"],
    ["preinc-of-member-int", e(update("++", member(id("x"), id("y")))), "++x.y;\n"],
  ]);
});

// The directive prologue. A parenthesised string is NOT a directive, so the parentheses have to
// survive printing or it is read back as one - which can silently make a function strict. The
// parser never produces these shapes ambiguously, but it does produce ASTs where the only thing
// distinguishing the two is `.directive`, so these lock the round-trip in.
//
// These cases give source rather than an AST, because `.directive` - the field under test - is one
// the parser sets, and hand-building it correctly is exactly what is in question.
describe("directives", () => {
  test.each<[name: string, source: string, output: string]>([
    ["paren-string-alone", `("moon");`, `("moon");\n`],
    ["after-one", `"use strict";\n("moon");`, `"use strict";\n("moon");\n`],
    ["after-two", `"use strict";\n"use asm";\n("moon");`, `"use strict";\n"use asm";\n("moon");\n`],
    ["prologue-closed", `"use strict";\nfoo;\n("moon");`, `"use strict";\nfoo;\n"moon";\n`],
    ["mode-change", `"use asm";\n("use strict");`, `"use asm";\n("use strict");\n`],
    [
      "mode-change-fn",
      `function f(){ "use asm"; ("use strict"); return 1; }`,
      `function f() {\n\t"use asm";\n\t("use strict");\n\treturn 1;\n}\n`,
    ],
    ["plain", `"use strict";\nfoo;`, `"use strict";\nfoo;\n`],
    ["only", `"use strict";`, `"use strict";\n`],
    ["none", `foo;\nbar;`, `foo;\nbar;\n`],
    ["string-mid-body", `foo;\n"bar";`, `foo;\n"bar";\n`],
  ])("%s", (_name, source, output) => {
    const { program: parsed, errors } = parseSync("fixture.js", source, PARSE_OPTIONS);
    if (errors.length > 0) throw new Error(`fixture parse failed: ${errors[0].message}`);
    expect(printSync(parsed).code).toBe(output);
  });
});

// A keyword or identifier straight after something which ends in an identifier character needs
// a space - the `printSpaceBeforeIdentifier` path
describe("space before identifiers", () => {
  checkCases([
    ["return-ident", program(ret(id("x"))), "return x;\n"],
    ["typeof-typeof", e(unary("typeof", unary("typeof", id("x")))), "typeof typeof x;\n"],
    ["void-typeof", e(unary("void", unary("typeof", id("x")))), "void typeof x;\n"],
    ["string-concat", e(bin("+", str("a"), id("b"))), '"a" + b;\n'],
  ]);
});

// The three `export default` declaration forms - interface, class and function - each end with a
// newline, so whatever follows starts on its own line. `export default interface` printed without
// one until `oxc_codegen` was fixed, gluing the next statement onto the closing brace.
//
// No conformance fixture contains the construct, so these cases are the only thing holding it in
// place. They give source rather than an AST, because what is under test is the separation between
// two statements.
describe("export default declarations", () => {
  test.each<[name: string, source: string, output: string]>([
    [
      "interface",
      `export default interface X {}\nconst y = 1;`,
      `export default interface X {}\nconst y = 1;\n`,
    ],
    ["interface-alone", `export default interface X {}`, `export default interface X {}\n`],
    [
      "class",
      `export default class X {}\nconst y = 1;`,
      `export default class X {}\nconst y = 1;\n`,
    ],
    [
      "function",
      `export default function X() {}\nconst y = 1;`,
      `export default function X() {}\nconst y = 1;\n`,
    ],
  ])("%s", (_name, source, output) => {
    const { program: parsed, errors } = parseSync("fixture.ts", source, PARSE_OPTIONS);
    if (errors.length > 0) throw new Error(`fixture parse failed: ${errors[0].message}`);
    expect(printSync(parsed, { ts: true }).code).toBe(output);
  });
});

// A JSX string prints from its raw source text, and the only decision is which quote to wrap it in -
// the text itself is never escaped, because JSX strings have no escape sequences. The parser does
// not decode HTML entities at present, so `&quot;` is six characters and not a `"`, and `raw` and
// `value` therefore hold the same text.
//
// Each case is printed twice: once as parsed, which takes the `raw` path, and once with `raw`
// blanked, which takes the `value` path. Both must give the same output, and that is what breaks
// when entities start being decoded - `value` becomes the decoded text while `raw` does not, so the
// second assertion fails. Checking only parsed output would prove nothing, since the printer would
// go on reading the unchanged `raw`.
//
// The conformance suites cannot catch this either: both printers read the same parser output, so
// they would move together and stay byte-identical. Hence pinned expected strings here.
describe("JSX strings", () => {
  test.each<[name: string, source: string, output: string]>([
    ["plain", `<Foo bar="x" />;`, `<Foo bar="x" />;\n`],
    ["single-quote-in-value", `<Foo bar="'" />;`, `<Foo bar="'" />;\n`],
    ["double-quote-in-value", `<Foo bar='"' />;`, `<Foo bar='"' />;\n`],
    ["quote-normalized", `<Foo bar='&apos;' />;`, `<Foo bar="&apos;" />;\n`],
    ["entity-amp", `<Foo bar="&amp;" />;`, `<Foo bar="&amp;" />;\n`],
    // The three which change the moment entities are decoded - each holds an apostrophe alongside
    // a `&quot;` which becomes the double quote the chosen wrapper then cannot contain
    ["entity-quot", `<Foo bar="&quot;" />;`, `<Foo bar="&quot;" />;\n`],
    ["entity-quot-with-apostrophe", `<Foo bar="'&quot;" />;`, `<Foo bar="'&quot;" />;\n`],
    ["entity-quot-mixed", `<Foo bar="a'b&quot;c" />;`, `<Foo bar="a'b&quot;c" />;\n`],
    // `JSXText` reads `raw` the same way, so it carries the same hazard
    ["text-entity", `<Foo>a&amp;b</Foo>;`, `<Foo>a&amp;b</Foo>;\n`],
    ["text-entity-quot", `<Foo>&quot;</Foo>;`, `<Foo>&quot;</Foo>;\n`],
  ])("%s", (_name, source, output) => {
    const parse = () => {
      const { program: parsed, errors } = parseSync("fixture.jsx", source, PARSE_OPTIONS);
      if (errors.length > 0) throw new Error(`fixture parse failed: ${errors[0].message}`);
      return parsed;
    };

    // From `raw`, as a parser supplies it
    expect(printSync(parse(), { jsx: true }).code).toBe(output);

    // From `value`, which is where a change in the parser would show
    const fromValue = parse();
    dropJsxRaw(fromValue);
    expect(printSync(fromValue, { jsx: true }).code).toBe(output);
  });
});

/**
 * Blank `raw` on every JSX string node, so the printer prints from `value` instead.
 *
 * `printJSXAttributeValue` and the `JSXText` arm both prefer `raw` wherever there is one,
 * and a parser always supplies one, so parsed input never reaches the `value` path at all.
 * Only a hand-built AST does - and that path is the one which moves when the parser changes.
 *
 * @param node - Node, or any value which might contain one
 */
function dropJsxRaw(node: unknown): void {
  if (typeof node !== "object" || node === null) return;

  if (Array.isArray(node)) {
    for (const child of node) {
      dropJsxRaw(child);
    }
    return;
  }

  const fields = node as Record<string, unknown>;
  if (fields.type === "JSXText") {
    fields.raw = undefined;
  } else if (fields.type === "JSXAttribute") {
    const value = fields.value as Record<string, unknown> | null | undefined;
    if (value != null && value.type === "Literal") value.raw = undefined;
  }

  for (const key in fields) {
    dropJsxRaw(fields[key]);
  }
}
