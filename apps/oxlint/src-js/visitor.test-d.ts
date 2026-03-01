import type { VisitorObject } from "./generated/visitor";
import type { Node, CallExpression } from "./generated/types";
import type { ExpectExtends, ExpectTrue } from "@type-challenges/utils";

const emptyVisitorObject = {};
const callExpressionVisitorObject = {
  CallExpression: (_node: CallExpression) => {},
};
const callExpressionExitVisitorObject = {
  "CallExpression:exit": (_node: CallExpression) => {},
};
const genericVisitorObject = {
  "FunctionExpression > Identifier": (_node: Node) => {},
  ":matches(FunctionExpression, FunctionDeclaration)": (_node: Node) => {},
};

export type cases1 = [
  // Empty visitor object is allowed
  ExpectTrue<ExpectExtends<VisitorObject, typeof emptyVisitorObject>>,
  // Specific node visitors has a stricter type for the parameter
  ExpectTrue<ExpectExtends<VisitorObject, typeof callExpressionVisitorObject>>,
  ExpectTrue<ExpectExtends<VisitorObject, typeof callExpressionExitVisitorObject>>,
  // Node selectors has generic Node type for the parameter
  ExpectTrue<ExpectExtends<VisitorObject, typeof genericVisitorObject>>,
];
