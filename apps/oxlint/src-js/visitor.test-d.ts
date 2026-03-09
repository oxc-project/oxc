import type { ExpectExtends, ExpectTrue, ExpectFalse } from "@type-challenges/utils";
import type { VisitorObject } from "./generated/visitor.d.ts";
import type { Node, CallExpression } from "./generated/types.d.ts";

// Empty visitor object is allowed
const emptyVisitor = {};
type _Empty = ExpectTrue<ExpectExtends<VisitorObject, typeof emptyVisitor>>;

// Specific node visitors have a stricter type for the parameter
const callExpressionVisitor = {
  CallExpression: (_node: CallExpression) => {},
};
type _CallExpr = ExpectTrue<ExpectExtends<VisitorObject, typeof callExpressionVisitor>>;

const callExpressionExitVisitor = {
  "CallExpression:exit": (_node: CallExpression) => {},
};
type _CallExprExit = ExpectTrue<ExpectExtends<VisitorObject, typeof callExpressionExitVisitor>>;

const debuggerStatementWrongTypeVisitor = {
  DebuggerStatement: (_node: CallExpression) => {},
};
type _DebuggerStmtWrongType = ExpectFalse<
  ExpectExtends<VisitorObject, typeof debuggerStatementWrongTypeVisitor>
>;

// Node selectors have generic `Node` type for the parameter
const selectorsVisitor = {
  "FunctionExpression > Identifier": (_node: Node) => {},
  ":matches(FunctionExpression, FunctionDeclaration)": (_node: Node) => {},
};
type _Selectors = ExpectTrue<ExpectExtends<VisitorObject, typeof selectorsVisitor>>;

// Visitor functions can omit the node parameter
const callExpressionNoParamVisitor = {
  CallExpression: () => {},
};
type _CallExprNoParam = ExpectTrue<
  ExpectExtends<VisitorObject, typeof callExpressionNoParamVisitor>
>;

// Properties of visitor object can be `undefined`.
// Ideally we'd disallow this, but it's not possible without `exactOptionalPropertyTypes: true` in `tsconfig.json`.
// Obviously we can't force that on users.
const callExpressionUndefinedVisitor = {
  CallExpression: undefined,
};
type _CallExprUndefined = ExpectTrue<
  ExpectExtends<VisitorObject, typeof callExpressionUndefinedVisitor>
>;

// Other types are not allowed
const invalidNullVisitor = {
  CallExpression: null,
};
type _InvalidNull = ExpectFalse<ExpectExtends<VisitorObject, typeof invalidNullVisitor>>;

const invalidObjectVisitor = {
  CallExpression: {},
};
type _InvalidObject = ExpectFalse<ExpectExtends<VisitorObject, typeof invalidObjectVisitor>>;

const invalidStringVisitor = {
  CallExpression: "oh dear",
};
type _InvalidString = ExpectFalse<ExpectExtends<VisitorObject, typeof invalidStringVisitor>>;
