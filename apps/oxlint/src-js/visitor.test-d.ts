import { test, expectTypeOf } from "vitest";
import type { VisitorObject } from "./generated/visitor";
import type { Node, CallExpression } from "./generated/types";

test("VisitorObject", () => {
  // Empty visitor object is allowed
  expectTypeOf({}).toExtend<VisitorObject>();

  // Specific node visitors has a stricter type for the parameter
  expectTypeOf({
    CallExpression: (_node: CallExpression) => {},
  }).toExtend<VisitorObject>();
  expectTypeOf({
    "CallExpression:exit": (_node: CallExpression) => {},
  }).toExtend<VisitorObject>();

  // Node selectors has generic Node type for the parameter
  expectTypeOf({
    "FunctionExpression > Identifier": (_node: Node) => {},
    ":matches(FunctionExpression, FunctionDeclaration)": (_node: Node) => {},
  }).toExtend<VisitorObject>();
});
