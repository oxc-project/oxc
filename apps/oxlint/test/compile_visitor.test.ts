// oxlint-disable jest/no-conditional-expect

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  NODE_TYPES_COUNT,
  LEAF_NODE_TYPES_COUNT,
  NODE_TYPE_IDS_MAP,
} from "../src-js/generated/type_ids.ts";
import {
  addVisitorToCompiled,
  compiledVisitor,
  finalizeCompiledVisitor,
  initCompiledVisitor,
  VISITOR_EMPTY,
  VISITOR_NOT_EMPTY,
} from "../src-js/plugins/visitor.ts";

import type { Mock } from "vitest";
import type { Node } from "../src-js/plugins/types.ts";
import type { EnterExit, VisitFn } from "../src-js/plugins/visitor.ts";

const PROGRAM_TYPE_ID = NODE_TYPE_IDS_MAP.get("Program")!,
  EMPTY_STMT_TYPE_ID = NODE_TYPE_IDS_MAP.get("EmptyStatement")!,
  IDENTIFIER_TYPE_ID = NODE_TYPE_IDS_MAP.get("Identifier")!,
  JSX_IDENTIFIER_TYPE_ID = NODE_TYPE_IDS_MAP.get("JSXIdentifier")!,
  FUNC_DECL_TYPE_ID = NODE_TYPE_IDS_MAP.get("FunctionDeclaration")!,
  FUNC_EXPR_TYPE_ID = NODE_TYPE_IDS_MAP.get("FunctionExpression")!,
  ARROW_FUNC_TYPE_ID = NODE_TYPE_IDS_MAP.get("ArrowFunctionExpression")!;

// TODO: Add tests for CFG events

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

describe("compile visitor", () => {
  beforeEach(initCompiledVisitor);
  afterEach(finalizeCompiledVisitor);

  it("throws if visitor is not an object", () => {
    const expectedErr = new TypeError("Visitor returned from `create` method must be an object");
    expect(() => (addVisitorToCompiled as any)()).toThrow(expectedErr);
    expect(() => addVisitorToCompiled(null as any)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled(undefined as any)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled(true as any)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled("xyz" as any)).toThrow(expectedErr);
  });

  it("throws if visitor property is not a function", () => {
    const expectedErr = new TypeError("'Program' property of visitor object is not a function");
    expect(() => addVisitorToCompiled({ Program: null as any })).toThrow(expectedErr);
    expect(() => addVisitorToCompiled({ Program: undefined as any })).toThrow(expectedErr);
    expect(() => addVisitorToCompiled({ Program: true } as any)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled({ Program: {} } as any)).toThrow(expectedErr);
  });

  it("accepts unknown visitor key", () => {
    expect(() => addVisitorToCompiled({ Foo() {} })).not.toThrow();
    expect(() => addVisitorToCompiled({ "Foo:exit"() {} })).not.toThrow();
  });

  describe("registers enter visitor", () => {
    it("for leaf node", () => {
      const enter = () => {};
      addVisitorToCompiled({ EmptyStatement: enter });
      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);
      expect(compiledVisitor[EMPTY_STMT_TYPE_ID]).toBe(enter);
    });

    it("for non-leaf node", () => {
      const enter = () => {};
      addVisitorToCompiled({ Program: enter });
      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
      expect(enterExit).toEqual({ enter, exit: null });
      expect(enterExit.enter).toBe(enter);
    });
  });

  describe("registers exit visitor", () => {
    it("for leaf node", () => {
      const exit = () => {};
      addVisitorToCompiled({ "EmptyStatement:exit": exit });
      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);
      expect(compiledVisitor[EMPTY_STMT_TYPE_ID]).toBe(exit);
    });

    it("for non-leaf node", () => {
      const exit = () => {};
      addVisitorToCompiled({ "Program:exit": exit });
      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
      expect(enterExit).toEqual({ enter: null, exit });
      expect(enterExit.exit).toBe(exit);
    });
  });

  describe("registers enter and exit visitors", () => {
    describe("for leaf node", () => {
      it("defined in order", () => {
        const enter = vi.fn(() => {});
        const exit = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter, "EmptyStatement:exit": exit });
        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const node = { type: "EmptyStatement", ...SPAN };
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter).toHaveBeenCalledWith(node);
        expect(exit).toHaveBeenCalledWith(node);
        expect(enter).toHaveBeenCalledBefore(exit);
      });

      it("defined in reverse order", () => {
        const enter = vi.fn(() => {});
        const exit = vi.fn(() => {});
        addVisitorToCompiled({ "EmptyStatement:exit": exit, EmptyStatement: enter });
        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const node = { type: "EmptyStatement", ...SPAN };
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter).toHaveBeenCalledWith(node);
        expect(exit).toHaveBeenCalledWith(node);
        expect(enter).toHaveBeenCalledBefore(exit);
      });
    });

    describe("for non-leaf node", () => {
      it("defined in order", () => {
        const enter = () => {};
        const exit = () => {};
        addVisitorToCompiled({ Program: enter, "Program:exit": exit });
        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
        expect(enterExit).toEqual({ enter, exit });
        expect(enterExit.enter).toBe(enter);
        expect(enterExit.exit).toBe(exit);
      });

      it("defined in reverse order", () => {
        const enter = () => {};
        const exit = () => {};
        addVisitorToCompiled({ "Program:exit": exit, Program: enter });
        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
        expect(enterExit).toEqual({ enter, exit });
        expect(enterExit.enter).toBe(enter);
        expect(enterExit.exit).toBe(exit);
      });
    });
  });

  describe("combines multiple visitors", () => {
    describe("for leaf node", () => {
      it("defined in order", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter1, "EmptyStatement:exit": exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter2, "EmptyStatement:exit": exit2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const node = { type: "EmptyStatement", ...SPAN };
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });

      it("defined in reverse order", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ "EmptyStatement:exit": exit1, EmptyStatement: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ "EmptyStatement:exit": exit2, EmptyStatement: enter2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const node = { type: "EmptyStatement", ...SPAN };
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });

      it("defined in mixed order", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter1, "EmptyStatement:exit": exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ "EmptyStatement:exit": exit2, EmptyStatement: enter2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const node = { type: "EmptyStatement", ...SPAN };
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });

      it("defined in mixed order 2", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ "EmptyStatement:exit": exit1, EmptyStatement: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter2, "EmptyStatement:exit": exit2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const node = { type: "EmptyStatement", ...SPAN };
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });
    });

    describe("for non-leaf node", () => {
      it("defined in order", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter1, "Program:exit": exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter2, "Program:exit": exit2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = { type: "Program", ...SPAN };
        enterExit.enter!(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = { type: "Program", ...SPAN };
        enterExit.exit!(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });

      it("defined in reverse order", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ "Program:exit": exit1, Program: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ "Program:exit": exit2, Program: enter2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = { type: "Program", ...SPAN };
        enterExit.enter!(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = { type: "Program", ...SPAN };
        enterExit.exit!(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });

      it("defined in mixed order", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter1, "Program:exit": exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ "Program:exit": exit2, Program: enter2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = { type: "Program", ...SPAN };
        enterExit.enter!(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = { type: "Program", ...SPAN };
        enterExit.exit!(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });

      it("defined in mixed order 2", () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ "Program:exit": exit1, Program: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter2, "Program:exit": exit2 });

        expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = { type: "Program", ...SPAN };
        enterExit.enter!(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = { type: "Program", ...SPAN };
        enterExit.exit!(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });
    });

    it("many visitors", () => {
      const enter1 = vi.fn(() => {});
      const exit1 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter1, "EmptyStatement:exit": exit1 });

      const enter2 = vi.fn(() => {});
      const exit2 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter2, "EmptyStatement:exit": exit2 });

      const enter3 = vi.fn(() => {});
      const exit3 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter3, "EmptyStatement:exit": exit3 });

      const enter4 = vi.fn(() => {});
      const exit4 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter4, "EmptyStatement:exit": exit4 });

      const enter5 = vi.fn(() => {});
      const exit5 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter5, "EmptyStatement:exit": exit5 });

      const enter6 = vi.fn(() => {});
      const exit6 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter6, "EmptyStatement:exit": exit6 });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      const node = { type: "EmptyStatement", ...SPAN };
      (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);

      expect(enter1).toHaveBeenCalledWith(node);
      expect(exit1).toHaveBeenCalledWith(node);
      expect(enter1).toHaveBeenCalledBefore(exit1);

      expect(enter2).toHaveBeenCalledWith(node);
      expect(exit2).toHaveBeenCalledWith(node);
      expect(enter2).toHaveBeenCalledBefore(exit2);

      expect(enter3).toHaveBeenCalledWith(node);
      expect(exit3).toHaveBeenCalledWith(node);
      expect(enter3).toHaveBeenCalledBefore(exit3);

      expect(enter4).toHaveBeenCalledWith(node);
      expect(exit4).toHaveBeenCalledWith(node);
      expect(enter4).toHaveBeenCalledBefore(exit4);

      expect(enter5).toHaveBeenCalledWith(node);
      expect(exit5).toHaveBeenCalledWith(node);
      expect(enter5).toHaveBeenCalledBefore(exit5);

      expect(enter6).toHaveBeenCalledWith(node);
      expect(exit6).toHaveBeenCalledWith(node);
      expect(enter6).toHaveBeenCalledBefore(exit6);
    });
  });

  describe("selectors", () => {
    it("`*` adds visitor function for all node types", () => {
      const enter = vi.fn(() => {});
      const exit = vi.fn(() => {});
      addVisitorToCompiled({ "*": enter, "*:exit": exit });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      for (const [typeName, typeId] of NODE_TYPE_IDS_MAP) {
        if (typeId >= NODE_TYPES_COUNT) continue; // Skip CFG events

        if (typeId < LEAF_NODE_TYPES_COUNT) {
          const node = { type: typeName, ...SPAN };
          (compiledVisitor[typeId] as VisitFn)(node);
          expect(enter).toHaveBeenCalledWith(node);
          expect(exit).toHaveBeenCalledWith(node);
        } else {
          const enterExit = compiledVisitor[typeId] as EnterExit;
          expect(enterExit.enter).toBe(enter);
          expect(enterExit.exit).toBe(exit);
        }
      }
    });

    it("`:matches` adds visitor function for all specified node types", () => {
      const enter = vi.fn(() => {});
      const exit = vi.fn(() => {});
      // List `EmptyStatement` twice to ensure it's deduped
      addVisitorToCompiled({
        ":matches(Program, EmptyStatement, EmptyStatement)": enter,
        ":matches(Program, EmptyStatement, EmptyStatement):exit": exit,
      });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
      expect(enterExit.enter).toBe(enter);
      expect(enterExit.exit).toBe(exit);

      const node = { type: "EmptyStatement", ...SPAN };
      (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
      expect(enter).toHaveBeenCalledWith(node);
      expect(exit).toHaveBeenCalledWith(node);

      for (const typeId of NODE_TYPE_IDS_MAP.values()) {
        if ([PROGRAM_TYPE_ID, EMPTY_STMT_TYPE_ID].includes(typeId)) continue;
        expect(compiledVisitor[typeId]).toBeNull();
      }
    });

    it("`:matches` with attributes adds visitor function for all specified node types", () => {
      const enter = vi.fn(() => {});
      const exit = vi.fn(() => {});
      addVisitorToCompiled({
        ":matches(Program[type], EmptyStatement)": enter,
        ":matches(Program, EmptyStatement[type]):exit": exit,
      });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
      expect(enterExit.enter).not.toBe(enter);
      expect(enterExit.exit).not.toBe(exit);

      const enterNode = { type: "Program", ...SPAN };
      enterExit.enter!(enterNode);
      expect(enter).toHaveBeenCalledWith(enterNode);

      const exitNode = { type: "Program", ...SPAN };
      enterExit.exit!(exitNode);
      expect(exit).toHaveBeenCalledWith(exitNode);

      const node = { type: "EmptyStatement", ...SPAN };
      (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
      expect(enter).toHaveBeenCalledWith(node);
      expect(exit).toHaveBeenCalledWith(node);

      for (const typeId of NODE_TYPE_IDS_MAP.values()) {
        if ([PROGRAM_TYPE_ID, EMPTY_STMT_TYPE_ID].includes(typeId)) continue;
        expect(compiledVisitor[typeId]).toBeNull();
      }
    });

    it("attributes adds visitor function for all node types, but filtered", () => {
      const enter = vi.fn(() => {});
      const exit = vi.fn(() => {});
      addVisitorToCompiled({
        "[name=foo]": enter,
        "[name=foo]:exit": exit,
      });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      // Skip CFG events
      for (let typeId = 0; typeId < NODE_TYPES_COUNT; typeId++) {
        if (typeId < LEAF_NODE_TYPES_COUNT) {
          expect(compiledVisitor[typeId]).toBeTypeOf("function");
        } else {
          const enterExit = compiledVisitor[typeId] as EnterExit;
          expect(enterExit.enter).toBeTypeOf("function");
          expect(enterExit.exit).toBeTypeOf("function");
          expect(enterExit.enter).not.toBe(enter);
          expect(enterExit.exit).not.toBe(exit);
        }
      }

      const program = { type: "Program", ...SPAN };
      const programEnterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
      programEnterExit.enter!(program);
      programEnterExit.exit!(program);
      expect(enter).not.toHaveBeenCalled();
      expect(exit).not.toHaveBeenCalled();

      const identEnterExit = compiledVisitor[IDENTIFIER_TYPE_ID] as EnterExit;
      const jsxIdentVisit = compiledVisitor[JSX_IDENTIFIER_TYPE_ID] as VisitFn;

      let ident = { type: "Identifier", name: "bar", ...SPAN };
      identEnterExit.enter!(ident);
      identEnterExit.exit!(ident);
      expect(enter).not.toHaveBeenCalled();
      expect(exit).not.toHaveBeenCalled();

      ident = { type: "JSXIdentifier", name: "qux", ...SPAN };
      jsxIdentVisit(ident);
      expect(enter).not.toHaveBeenCalled();
      expect(exit).not.toHaveBeenCalled();

      ident = { type: "Identifier", name: "foo", ...SPAN };
      identEnterExit.enter!(ident);
      expect(enter).toHaveBeenCalledWith(ident);
      ident = { type: "Identifier", name: "foo", ...SPAN };
      identEnterExit.exit!(ident);
      expect(exit).toHaveBeenCalledWith(ident);

      ident = { type: "JSXIdentifier", name: "foo", ...SPAN };
      jsxIdentVisit(ident);
      expect(enter).toHaveBeenCalledWith(ident);
      expect(exit).toHaveBeenCalledWith(ident);
    });

    it("identifier with attribute adds visitor function for only specified node types, and filtered", () => {
      const enter = vi.fn(() => {});
      const exit = vi.fn(() => {});
      addVisitorToCompiled({
        "Identifier[name=foo]": enter,
        "Identifier[name=foo]:exit": exit,
      });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      const identEnterExit = compiledVisitor[IDENTIFIER_TYPE_ID] as EnterExit;
      expect(identEnterExit.enter).toBeTypeOf("function");
      expect(identEnterExit.exit).toBeTypeOf("function");
      expect(identEnterExit.enter).not.toBe(enter);
      expect(identEnterExit.exit).not.toBe(exit);

      for (const typeId of NODE_TYPE_IDS_MAP.values()) {
        if (typeId === IDENTIFIER_TYPE_ID) continue;
        expect(compiledVisitor[typeId]).toBeNull();
      }

      let ident = { type: "Identifier", name: "bar", ...SPAN };
      identEnterExit.enter!(ident);
      identEnterExit.exit!(ident);
      expect(enter).not.toHaveBeenCalled();
      expect(exit).not.toHaveBeenCalled();

      ident = { type: "Identifier", name: "foo", ...SPAN };
      identEnterExit.enter!(ident);
      expect(enter).toHaveBeenCalledWith(ident);
      ident = { type: "Identifier", name: "foo", ...SPAN };
      identEnterExit.exit!(ident);
      expect(exit).toHaveBeenCalledWith(ident);
    });

    it("class adds visitor function only for node types in those classes", () => {
      type ClassName = "statement" | "declaration" | "pattern" | "expression" | "function";

      const enterStatement = vi.fn(() => {});
      const exitStatement = vi.fn(() => {});
      const enterDeclaration = vi.fn(() => {});
      const exitDeclaration = vi.fn(() => {});
      const enterPattern = vi.fn(() => {});
      const exitPattern = vi.fn(() => {});
      const enterExpression = vi.fn(() => {});
      const exitExpression = vi.fn(() => {});
      const enterFunction = vi.fn(() => {});
      const exitFunction = vi.fn(() => {});

      // `:pattern` and `:expression` are "complex" selectors.
      // Visit fns are wrapped for them. The rest of classes are simple.
      const classes = new Map<
        ClassName,
        { enter: Mock<VisitFn>; exit: Mock<VisitFn>; isComplex: boolean }
      >([
        ["statement", { enter: enterStatement, exit: exitStatement, isComplex: false }],
        ["declaration", { enter: enterDeclaration, exit: exitDeclaration, isComplex: false }],
        ["pattern", { enter: enterPattern, exit: exitPattern, isComplex: true }],
        ["expression", { enter: enterExpression, exit: exitExpression, isComplex: true }],
        ["function", { enter: enterFunction, exit: exitFunction, isComplex: false }],
      ]);

      function resetSpies() {
        for (const { enter, exit } of classes.values()) {
          enter.mockClear();
          exit.mockClear();
        }
      }

      addVisitorToCompiled({
        ":statement": enterStatement,
        ":statement:exit": exitStatement,
        ":declaration": enterDeclaration,
        ":declaration:exit": exitDeclaration,
        ":pattern": enterPattern,
        ":pattern:exit": exitPattern,
        ":expression": enterExpression,
        ":expression:exit": exitExpression,
        ":function": enterFunction,
        ":function:exit": exitFunction,
      });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      for (const [typeName, typeId] of NODE_TYPE_IDS_MAP.entries()) {
        if (typeId >= NODE_TYPES_COUNT) continue; // Skip CFG events

        // Check that compiled visitor is set up correctly

        // Get classes that this node type matches
        const classNames: ClassName[] = [];

        if (typeName.endsWith("Statement")) {
          classNames.push("statement");
        } else if (typeName.endsWith("Declaration")) {
          // Declarations are also statements
          classNames.push("declaration", "statement");
        } else if (typeName.endsWith("Pattern")) {
          classNames.push("pattern");
        } else if (
          typeName.endsWith("Expression") ||
          typeName.endsWith("Literal") ||
          typeName === "Identifier" ||
          typeName === "MetaProperty"
        ) {
          // Expressions are also patterns
          classNames.push("expression", "pattern");
        }

        if (
          ["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"].includes(
            typeName,
          )
        ) {
          classNames.push("function");
        }

        const visit = compiledVisitor[typeId];

        // If no classes match this node type, then visitor should be `null`
        if (classNames.length === 0) {
          expect(visit).toBeNull();
          continue;
        }

        // Check the compiler visitor for this type is what it should be.
        // Check that visit fns have/have not been wrapped/merged, depending on the class.
        if (
          classNames.length === 1 && // Type is only matched by 1 class
          typeId >= LEAF_NODE_TYPES_COUNT && // Non leaf node
          !classes.get(classNames[0])!.isComplex // Class is not complex
        ) {
          const enterExit = visit as EnterExit;
          const { enter, exit } = classes.get(classNames[0])!;
          expect(enterExit.enter).toBe(enter);
          expect(enterExit.exit).toBe(exit);
        } else {
          expect(visit).not.toBeNull();

          // Check visit fn has been wrapped/merged
          if (typeId < LEAF_NODE_TYPES_COUNT) {
            // Leaf node
            expect(typeof visit).toBe("function");

            for (const { enter, exit } of classes.values()) {
              expect(visit).not.toBe(enter);
              expect(visit).not.toBe(exit);
            }
          } else {
            // Non-leaf node
            expect(typeof visit).toBe("object");

            const enterExit = visit as EnterExit;
            expect(typeof enterExit.enter).toBe("function");
            expect(typeof enterExit.exit).toBe("function");

            for (const { enter, exit } of classes.values()) {
              expect(enterExit.enter).not.toBe(enter);
              expect(enterExit.enter).not.toBe(exit);
              expect(enterExit.exit).not.toBe(enter);
              expect(enterExit.exit).not.toBe(exit);
            }
          }
        }

        // Check that correct functions are called when visiting a node of this type
        const node = {
          type: typeName,
          parent: { type: "Program", ...SPAN },
          ...SPAN,
        };

        if (typeId < LEAF_NODE_TYPES_COUNT) {
          // Leaf node
          (visit as VisitFn)(node);

          for (const [className, { enter, exit }] of classes.entries()) {
            if (classNames.includes(className)) {
              expect(enter).toHaveBeenCalledWith(node);
              expect(exit).toHaveBeenCalledWith(node);
            } else {
              expect(enter).not.toHaveBeenCalled();
              expect(exit).not.toHaveBeenCalled();
            }
          }

          resetSpies();
        } else {
          // Non-leaf node
          const enterExit = visit as EnterExit;

          // Test enter
          enterExit.enter!(node);

          for (const [className, { enter, exit }] of classes.entries()) {
            if (classNames.includes(className)) {
              expect(enter).toHaveBeenCalledWith(node);
            } else {
              expect(enter).not.toHaveBeenCalled();
            }
            expect(exit).not.toHaveBeenCalled();
          }

          resetSpies();

          // Test exit
          enterExit.exit!(node);

          for (const [className, { enter, exit }] of classes.entries()) {
            expect(enter).not.toHaveBeenCalled();
            if (classNames.includes(className)) {
              expect(exit).toHaveBeenCalledWith(node);
            } else {
              expect(exit).not.toHaveBeenCalled();
            }
          }

          resetSpies();
        }
      }
    });

    it("combined", () => {
      const enter1 = vi.fn(() => {});
      const exit1 = vi.fn(() => {});
      addVisitorToCompiled({
        "FunctionDeclaration[name=foo]": enter1,
        "FunctionDeclaration[name=foo]:exit": exit1,
      });
      const enter2 = vi.fn(() => {});
      const exit2 = vi.fn(() => {});
      addVisitorToCompiled({
        ":function": enter2,
        // Check case insensitivity
        ":FunCtioN:exit": exit2,
      });

      expect(finalizeCompiledVisitor()).toBe(VISITOR_NOT_EMPTY);

      const funcDeclEnterExit = compiledVisitor[FUNC_DECL_TYPE_ID] as EnterExit;
      expect(funcDeclEnterExit.enter).toBeTypeOf("function");
      expect(funcDeclEnterExit.exit).toBeTypeOf("function");
      expect(funcDeclEnterExit.enter).not.toBe(enter1);
      expect(funcDeclEnterExit.enter).not.toBe(enter2);
      expect(funcDeclEnterExit.exit).not.toBe(exit1);
      expect(funcDeclEnterExit.exit).not.toBe(exit2);

      const funcExprEnterExit = compiledVisitor[FUNC_EXPR_TYPE_ID] as EnterExit;
      expect(funcExprEnterExit.enter).toBe(enter2);
      expect(funcExprEnterExit.exit).toBe(exit2);

      const arrowFuncEnterExit = compiledVisitor[ARROW_FUNC_TYPE_ID] as EnterExit;
      expect(arrowFuncEnterExit.enter).toBe(enter2);
      expect(arrowFuncEnterExit.exit).toBe(exit2);

      for (const typeId of NODE_TYPE_IDS_MAP.values()) {
        if ([FUNC_DECL_TYPE_ID, FUNC_EXPR_TYPE_ID, ARROW_FUNC_TYPE_ID].includes(typeId)) continue;
        expect(compiledVisitor[typeId]).toBeNull();
      }

      let arrowExpr = { type: "ArrowFunctionExpression", ...SPAN };
      arrowFuncEnterExit.enter!(arrowExpr);
      expect(enter2).toHaveBeenCalledWith(arrowExpr);
      arrowExpr = { type: "ArrowFunctionExpression", ...SPAN };
      arrowFuncEnterExit.exit!(arrowExpr);
      expect(exit2).toHaveBeenCalledWith(arrowExpr);
      expect(enter1).not.toHaveBeenCalled();
      expect(exit1).not.toHaveBeenCalled();

      let funcExpr = { type: "FunctionExpression", name: "foo", ...SPAN };
      funcExprEnterExit.enter!(funcExpr);
      expect(enter2).toHaveBeenCalledWith(funcExpr);
      funcExpr = { type: "FunctionExpression", name: "foo", ...SPAN };
      funcExprEnterExit.exit!(funcExpr);
      expect(exit2).toHaveBeenCalledWith(funcExpr);
      expect(enter1).not.toHaveBeenCalled();
      expect(exit1).not.toHaveBeenCalled();

      let funcDecl = { type: "FunctionDeclaration", name: "bar", ...SPAN };
      funcDeclEnterExit.enter!(funcDecl);
      expect(enter2).toHaveBeenCalledWith(funcDecl);
      funcDecl = { type: "FunctionDeclaration", name: "bar", ...SPAN };
      funcDeclEnterExit.exit!(funcDecl);
      expect(exit2).toHaveBeenCalledWith(funcDecl);
      expect(enter1).not.toHaveBeenCalled();
      expect(exit1).not.toHaveBeenCalled();

      funcDecl = { type: "FunctionDeclaration", name: "foo", ...SPAN };
      funcDeclEnterExit.enter!(funcDecl);
      expect(enter1).toHaveBeenCalledWith(funcDecl);
      expect(enter2).toHaveBeenCalledWith(funcDecl);
      funcDecl = { type: "FunctionDeclaration", name: "foo", ...SPAN };
      funcDeclEnterExit.exit!(funcDecl);
      expect(exit1).toHaveBeenCalledWith(funcDecl);
      expect(exit2).toHaveBeenCalledWith(funcDecl);
    });
  });

  describe("`finalizeCompiledVisitor` returns false if all visitors empty", () => {
    it("no visitors", () => {
      expect(finalizeCompiledVisitor()).toBe(VISITOR_EMPTY);
    });

    it("1 visitor", () => {
      addVisitorToCompiled({});
      expect(finalizeCompiledVisitor()).toBe(VISITOR_EMPTY);
    });

    it("multiple visitors", () => {
      addVisitorToCompiled({});
      addVisitorToCompiled({});
      addVisitorToCompiled({});
      expect(finalizeCompiledVisitor()).toBe(VISITOR_EMPTY);
    });
  });
});
