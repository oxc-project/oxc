import { describe, expect, it } from "vitest";
import { RuleTester } from "../src-js/package/rule_tester.ts";

import type { Rule } from "../src-js/plugins.ts";

// Each case covers a distinct association rule of ESLint 9's deprecated API.
const cases = [
  {
    name: "returns cached comments for adjacent declarations",
    code: "/** first */\nfunction first() {}\n/** second */ class Second {}",
    selector: "FunctionDeclaration, ClassDeclaration",
    expected: ["* first ", "* second "],
  },
  {
    name: "requires the nearest token to be JSDoc",
    code: "/** old */\n/* ordinary */\nfunction blocked() {}\n/* ordinary */\n/** nearest */\nfunction documented() {}\n// line\nfunction line() {}",
    selector: "FunctionDeclaration",
    expected: [null, "* nearest ", null],
  },
  {
    name: "rejects a blank line but accepts CRLF adjacency",
    code: "/** distant */\n\nfunction distant() {}\n/** close */\r\nfunction close() {}",
    selector: "FunctionDeclaration",
    expected: [null, "* close "],
  },
  {
    name: "uses export wrappers for declarations",
    code: "/** named */\nexport function named() {}\n/** default */\nexport default class {}\nexport /** inner */ function inner() {}",
    selector: "FunctionDeclaration, ClassDeclaration",
    expected: ["* named ", "* default ", null],
  },
  {
    name: "uses the grandparent for class expressions",
    code: "/** variable */\nconst C = class {};\n/** assignment */\nC.member = class {};",
    selector: "ClassExpression",
    expected: ["* variable ", "* assignment "],
  },
  {
    name: "preserves the class expression grandparent quirk",
    code: "const C = /** direct */ class {};\nconst obj = { /** property */ C: class {} };",
    selector: "ClassExpression",
    expected: [null, null],
  },
  {
    name: "finds function and arrow documentation through assignment ancestors",
    code: "/** function */\nconst f = function() {};\n/** arrow */\nf.member = () => {};",
    selector: "FunctionExpression, ArrowFunctionExpression",
    expected: ["* function ", "* arrow "],
  },
  {
    name: "stops at object properties and class methods",
    code: "const obj = { /** property */\nf: () => {} };\n/** class */\nclass C { bare() {}\n/** method */\ndocumented() {} }",
    selector: "FunctionExpression, ArrowFunctionExpression",
    expected: ["* property ", null, "* method "],
  },
  {
    name: "does not borrow enclosing function documentation",
    code: "/** declaration */\nfunction outer() { const inner = function() {}; }\n/** expression */\nconst wrapper = function() { return () => {}; };",
    selector: "FunctionExpression, ArrowFunctionExpression",
    expected: [null, "* expression ", null],
  },
  {
    name: "does not borrow call or constructor documentation",
    code: "/** call */\ncall(function() {});\n/** new */\nnew C(() => {});",
    selector: "FunctionExpression, ArrowFunctionExpression",
    expected: [null, null],
  },
  {
    name: "accepts documentation directly on callback arguments",
    code: "call(/** callback */ function() {});\nnew C(/** callback */ () => {});",
    selector: "FunctionExpression, ArrowFunctionExpression",
    expected: ["* callback ", "* callback "],
  },
  {
    name: "stops ancestor lookup at an ordinary comment",
    code: "/** outer */\nconst /* stop */ f = () => {};",
    selector: "ArrowFunctionExpression",
    expected: [null],
  },
  {
    name: "falls back to direct comments at the program boundary",
    code: "(/** direct */ () => {});\n(() => {});",
    selector: "ArrowFunctionExpression",
    expected: ["* direct ", null],
  },
  {
    name: "returns null for unsupported nodes",
    code: "/** variable */\nconst value = 1;",
    selector: "Program, VariableDeclaration, Identifier, Literal",
    expected: [null, null, null, null],
  },
] satisfies {
  name: string;
  code: string;
  selector: string;
  expected: (string | null)[];
}[];

RuleTester.describe = (_name, fn) => fn();
RuleTester.it = (_name, fn) => fn();

describe("getJSDocComment", () => {
  it.each(cases)("$name", ({ code, selector, expected }) => {
    const actual: (string | null)[] = [];
    const rule: Rule = {
      create(context) {
        const { sourceCode } = context;
        return {
          [selector](node) {
            const comment = sourceCode.getJSDocComment(node);
            actual.push(comment?.value ?? null);
            if (comment !== null) {
              expect(sourceCode.getAllComments()).toContain(comment);
              expect(sourceCode.getJSDocComment(node)).toBe(comment);
            }
          },
        };
      },
    };
    new RuleTester().run("jsdoc", rule, { valid: [code], invalid: [] });
    expect(actual).toEqual(expected);
  });
});
