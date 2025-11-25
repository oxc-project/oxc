import { describe, expect, it } from "vitest";

import { parseSync, Visitor, visitorKeys, type VisitorObject } from "../src-js/index.js";

describe("visit", () => {
  // oxlint-disable-next-line vitest/expect-expect
  it("empty visitor", () => {
    const code = "const x = { y: 123 }";
    const { program } = parseSync("test.js", code);
    const visitor = new Visitor({});
    visitor.visit(program);
  });

  describe("invalid visitor", () => {
    it("undefined visitor object", () => {
      // @ts-ignore
      expect(() => new Visitor()).toThrow(new TypeError("Visitor must be an object"));
    });

    it("null visitor object", () => {
      expect(() => new Visitor(null)).toThrow(new TypeError("Visitor must be an object"));
    });

    it("boolean visitor object", () => {
      expect(() => new Visitor(true as unknown as VisitorObject)).toThrow(
        new TypeError("Visitor must be an object"),
      );
    });

    it("unknown type in entry", () => {
      expect(() => new Visitor({ Foo() {} } as VisitorObject)).toThrow(
        new Error("Unknown node type 'Foo' in visitor object"),
      );
    });

    it("unknown type in exit", () => {
      expect(() => new Visitor({ "Foo:exit"() {} } as VisitorObject)).toThrow(
        new Error("Unknown node type 'Foo' in visitor object"),
      );
    });

    it("invalid postfix", () => {
      expect(() => new Visitor({ "Identifier:foo"() {} } as VisitorObject)).toThrow(
        new Error("Unknown node type 'Identifier:foo' in visitor object"),
      );
    });
  });

  it("visit JS code", () => {
    const code = "const x = { y: 123 }";
    const { program } = parseSync("test.js", code);

    const visited = [];
    const visitor = new Visitor({
      Program(node) {
        visited.push(`enter: ${node.type}`);
      },
      "Program:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      VariableDeclaration(node) {
        visited.push(`enter: ${node.type}`);
      },
      "VariableDeclaration:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      VariableDeclarator(node) {
        visited.push(`enter: ${node.type}`);
      },
      "VariableDeclarator:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      Identifier(node) {
        visited.push(`enter: ${node.type} ${node.name}`);
      },
      "Identifier:exit"(node) {
        visited.push(`exit: ${node.type} ${node.name}`);
      },
      ObjectExpression(node) {
        visited.push(`enter: ${node.type}`);
      },
      "ObjectExpression:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      Property(node) {
        visited.push(`enter: ${node.type}`);
      },
      "Property:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      Literal(node) {
        visited.push(`enter: ${node.type} ${node.value}`);
      },
      "Literal:exit"(node) {
        visited.push(`exit: ${node.type} ${node.value}`);
      },
      // Should not be visited
      DebuggerStatement(node) {
        visited.push(`enter: ${node.type}`);
      },
      "DebuggerStatement:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      ClassExpression(node) {
        visited.push(`enter: ${node.type}`);
      },
      "ClassExpression:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
    });
    visitor.visit(program);

    expect(visited).toStrictEqual([
      "enter: Program",
      "enter: VariableDeclaration",
      "enter: VariableDeclarator",
      "enter: Identifier x",
      "exit: Identifier x",
      "enter: ObjectExpression",
      "enter: Property",
      "enter: Identifier y",
      "exit: Identifier y",
      "enter: Literal 123",
      "exit: Literal 123",
      "exit: Property",
      "exit: ObjectExpression",
      "exit: VariableDeclarator",
      "exit: VariableDeclaration",
      "exit: Program",
    ]);
  });

  it("visit TS code", () => {
    const code = "type T = string | Q;";
    const { program } = parseSync("test.ts", code);

    const visited = [];
    const visitor = new Visitor({
      Program(node) {
        visited.push(`enter: ${node.type}`);
      },
      "Program:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      TSTypeAliasDeclaration(node) {
        visited.push(`enter: ${node.type}`);
      },
      "TSTypeAliasDeclaration:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      Identifier(node) {
        visited.push(`enter: ${node.type} ${node.name}`);
      },
      "Identifier:exit"(node) {
        visited.push(`exit: ${node.type} ${node.name}`);
      },
      TSUnionType(node) {
        visited.push(`enter: ${node.type}`);
      },
      "TSUnionType:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      TSStringKeyword(node) {
        visited.push(`enter: ${node.type}`);
      },
      "TSStringKeyword:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      TSTypeReference(node) {
        visited.push(`enter: ${node.type}`);
      },
      "TSTypeReference:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      // Should not be visited
      DebuggerStatement(node) {
        visited.push(`enter: ${node.type}`);
      },
      "DebuggerStatement:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      ClassExpression(node) {
        visited.push(`enter: ${node.type}`);
      },
      "ClassExpression:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
    });
    visitor.visit(program);

    expect(visited).toStrictEqual([
      "enter: Program",
      "enter: TSTypeAliasDeclaration",
      "enter: Identifier T",
      "exit: Identifier T",
      "enter: TSUnionType",
      "enter: TSStringKeyword",
      "exit: TSStringKeyword",
      "enter: TSTypeReference",
      "enter: Identifier Q",
      "exit: Identifier Q",
      "exit: TSTypeReference",
      "exit: TSUnionType",
      "exit: TSTypeAliasDeclaration",
      "exit: Program",
    ]);
  });

  it("reuse visitor", () => {
    const visited = [];
    const visitor = new Visitor({
      Program(node) {
        visited.push(`enter: ${node.type}`);
      },
      "Program:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
      Identifier(node) {
        visited.push(`enter: ${node.type} ${node.name}`);
      },
      "Identifier:exit"(node) {
        visited.push(`exit: ${node.type} ${node.name}`);
      },
      // Should not be visited
      DebuggerStatement(node) {
        visited.push(`enter: ${node.type}`);
      },
      "DebuggerStatement:exit"(node) {
        visited.push(`exit: ${node.type}`);
      },
    });

    const code = "const x = { y: 123 }";
    const { program } = parseSync("test.js", code);
    visitor.visit(program);
    expect(visited).toStrictEqual([
      "enter: Program",
      "enter: Identifier x",
      "exit: Identifier x",
      "enter: Identifier y",
      "exit: Identifier y",
      "exit: Program",
    ]);

    visited.length = 0;

    const code2 = "let z";
    const program2 = parseSync("test.js", code2).program;
    visitor.visit(program2);
    expect(visited).toStrictEqual([
      "enter: Program",
      "enter: Identifier z",
      "exit: Identifier z",
      "exit: Program",
    ]);
  });
});

it("visitor keys", () => {
  expect(visitorKeys.Literal).toEqual([]);
  expect(visitorKeys.VariableDeclaration).toEqual(["declarations"]);
  expect(visitorKeys.ObjectPattern).toEqual(["decorators", "properties", "typeAnnotation"]);
  expect(visitorKeys.ParenthesizedExpression).toEqual(["expression"]);
  expect(visitorKeys.V8IntrinsicExpression).toEqual(["name", "arguments"]);
});
