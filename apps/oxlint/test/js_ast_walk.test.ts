import { describe, expect, it, vi } from "vitest";
import { compileJsVisitors, walkParserAst } from "../src-js/plugins/js_ast_walk.ts";

import type { JsParserNode } from "../src-js/plugins/parsers.ts";
import type { Visitor } from "../src-js/plugins/types.ts";

/**
 * Create a small AST containing a non-standard `CustomTemplate` node type:
 *
 * ```
 * Program
 * └── VariableDeclaration
 *     └── VariableDeclarator
 *         ├── Identifier (id)
 *         └── CustomTemplate (init)
 *             └── Identifier (expression)
 * ```
 */
function createAst(): { ast: JsParserNode; nodes: Record<string, JsParserNode> } {
  const idNode: JsParserNode = { type: "Identifier", name: "x", range: [6, 7] };
  const exprNode: JsParserNode = { type: "Identifier", name: "y", range: [13, 14] };
  const templateNode: JsParserNode = {
    type: "CustomTemplate",
    quasis: [],
    expressions: [exprNode],
    range: [10, 16],
  };
  const declaratorNode: JsParserNode = {
    type: "VariableDeclarator",
    id: idNode,
    init: templateNode,
    range: [6, 16],
  };
  const declarationNode: JsParserNode = {
    type: "VariableDeclaration",
    kind: "const",
    declarations: [declaratorNode],
    range: [0, 17],
  };
  const ast: JsParserNode = {
    type: "Program",
    body: [declarationNode],
    sourceType: "module",
    range: [0, 17],
  };

  return {
    ast,
    nodes: { ast, declarationNode, declaratorNode, idNode, templateNode, exprNode },
  };
}

// Visitor keys covering the custom node type.
// Standard types are resolved via these keys too (the real caller passes merged keys).
const VISITOR_KEYS: Record<string, readonly string[]> = {
  Program: ["body"],
  VariableDeclaration: ["declarations"],
  VariableDeclarator: ["id", "init"],
  Identifier: [],
  CustomTemplate: ["quasis", "expressions"],
};

function walk(ast: JsParserNode, visitors: Visitor[], visitorKeys = VISITOR_KEYS): void {
  walkParserAst(ast, visitorKeys, compileJsVisitors(visitors));
}

describe("compileJsVisitors", () => {
  it("returns empty compiled visitor for empty visitors", () => {
    const compiled = compileJsVisitors([{}]);
    expect(compiled.hasVisitors).toBe(false);
  });

  it("throws if visitor is not an object", () => {
    expect(() => compileJsVisitors([null as unknown as Visitor])).toThrow(
      "Visitor returned from `create` method must be an object",
    );
  });

  it("throws if visitor property is not a function", () => {
    expect(() => compileJsVisitors([{ Program: 123 } as unknown as Visitor])).toThrow(
      "'Program' property of visitor object is not a function",
    );
  });

  it("throws for code path analysis events", () => {
    expect(() => compileJsVisitors([{ onCodePathStart() {} } as unknown as Visitor])).toThrow(
      "Rules using code path analysis ('onCodePathStart') are not supported " +
        "for files parsed by a custom parser",
    );
  });
});

describe("walkParserAst", () => {
  it("visits nodes with unknown types via bare type keys", () => {
    const { ast, nodes } = createAst();
    const visitFn = vi.fn();
    walk(ast, [{ CustomTemplate: visitFn }]);

    expect(visitFn).toHaveBeenCalledTimes(1);
    expect(visitFn).toHaveBeenCalledWith(nodes.templateNode);
  });

  it("calls enter and exit handlers in correct order", () => {
    const { ast } = createAst();
    const calls: string[] = [];
    walk(ast, [
      {
        Program: () => calls.push("Program"),
        "Program:exit": () => calls.push("Program:exit"),
        CustomTemplate: () => calls.push("CustomTemplate"),
        "CustomTemplate:exit": () => calls.push("CustomTemplate:exit"),
        Identifier: (node) => calls.push(`Identifier:${(node as { name?: string }).name}`),
      },
    ]);

    expect(calls).toEqual([
      "Program",
      "Identifier:x",
      "CustomTemplate",
      "Identifier:y",
      "CustomTemplate:exit",
      "Program:exit",
    ]);
  });

  it("sets `parent` on all visited nodes", () => {
    const { ast, nodes } = createAst();
    walk(ast, [{ Program() {} }]);

    expect(ast.parent).toBe(null);
    expect(nodes.declarationNode.parent).toBe(ast);
    expect(nodes.declaratorNode.parent).toBe(nodes.declarationNode);
    expect(nodes.idNode.parent).toBe(nodes.declaratorNode);
    expect(nodes.templateNode.parent).toBe(nodes.declaratorNode);
    expect(nodes.exprNode.parent).toBe(nodes.templateNode);
  });

  it("matches esquery selectors, including for unknown node types", () => {
    const { ast, nodes } = createAst();
    const declaratorTemplate = vi.fn();
    const programDescendantId = vi.fn();
    const nonMatching = vi.fn();
    walk(ast, [
      {
        "VariableDeclarator > CustomTemplate": declaratorTemplate,
        "CustomTemplate Identifier": programDescendantId,
        "Program > CustomTemplate": nonMatching,
      },
    ]);

    expect(declaratorTemplate).toHaveBeenCalledTimes(1);
    expect(declaratorTemplate).toHaveBeenCalledWith(nodes.templateNode);
    expect(programDescendantId).toHaveBeenCalledTimes(1);
    expect(programDescendantId).toHaveBeenCalledWith(nodes.exprNode);
    expect(nonMatching).not.toHaveBeenCalled();
  });

  it("matches attribute selectors", () => {
    const { ast, nodes } = createAst();
    const visitFn = vi.fn();
    walk(ast, [{ 'Identifier[name="y"]': visitFn }]);

    expect(visitFn).toHaveBeenCalledTimes(1);
    expect(visitFn).toHaveBeenCalledWith(nodes.exprNode);
  });

  it("supports the universal selector `*`, called before more specific handlers", () => {
    const { ast } = createAst();
    const calls: string[] = [];
    walk(ast, [
      {
        CustomTemplate: () => calls.push("type"),
        "*": (node) => calls.push(`*:${node.type}`),
        "*:exit": (node) => calls.push(`*:exit:${node.type}`),
      },
    ]);

    expect(calls).toEqual([
      "*:Program",
      "*:VariableDeclaration",
      "*:VariableDeclarator",
      "*:Identifier",
      "*:exit:Identifier",
      "*:CustomTemplate",
      "type",
      "*:Identifier",
      "*:exit:Identifier",
      "*:exit:CustomTemplate",
      "*:exit:VariableDeclarator",
      "*:exit:VariableDeclaration",
      "*:exit:Program",
    ]);
  });

  it("falls back to iterating object keys for node types without visitor keys", () => {
    const unknownChild: JsParserNode = { type: "UnknownChild", range: [2, 3] };
    const unknownNode: JsParserNode = {
      type: "TotallyUnknown",
      something: unknownChild,
      others: [{ type: "UnknownChild", range: [4, 5] }],
      notANode: { foo: "bar" },
      range: [0, 6],
      loc: { start: { line: 1, column: 0 }, end: { line: 1, column: 6 } },
    };
    const ast: JsParserNode = { type: "Program", body: [unknownNode], range: [0, 6] };

    const visited: string[] = [];
    walk(ast, [{ "*": (node) => visited.push(node.type) }], { Program: ["body"] });

    expect(visited).toEqual(["Program", "TotallyUnknown", "UnknownChild", "UnknownChild"]);
    expect(unknownChild.parent).toBe(unknownNode);
  });

  it("runs handlers from multiple visitors on the same node", () => {
    const { ast } = createAst();
    const first = vi.fn();
    const second = vi.fn();
    walk(ast, [{ CustomTemplate: first }, { CustomTemplate: second }]);

    expect(first).toHaveBeenCalledTimes(1);
    expect(second).toHaveBeenCalledTimes(1);
  });
});
