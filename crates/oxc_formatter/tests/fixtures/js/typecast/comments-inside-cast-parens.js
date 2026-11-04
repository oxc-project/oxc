// A comment inside the parens of a JSDoc type cast never drops the parens
// (without them tsc ignores the cast) and stays inside them, like Prettier.
// The traps are the sites that hide a child's trailing comments while printing it:
// statement terminators, callees, heritage (issues #26274, #26275).

// Statement terminators: same-line and own-line comments stay inside.
const declared = /** @type {string} */ (
  value // trailing
);
const ownLine = /** @type {string} */ (
  value
  // own-line
);
/** @type {string} */ (
  statement // trailing
);
export default /** @type {string} */ (
  value // trailing
);
chained = other = /** @type {string} */ (
  value // trailing
);
const arrow = () => /** @type {string} */ (
  value // trailing
);
class Property {
  field = /** @type {string} */ (
    value // trailing
  );
}
function returned() {
  return assigned = /** @type {string} */ (
    value // trailing
  );
}
// No source `;` (ASI): the cast `)` is not a dropped paren.
const asi = /** @type {string} */ (
  value // trailing
)

// Object and array targets lose the hug form, the comment breaks the parens.
const object = /** @type {{ value: string }} */ ({ value } // trailing
);
const array = /** @type {string[]} */ ([value]
  // own-line
);
const hugged = /** @type {{ value: string }} */ ({ value });

// Callee: the comment stays with the callee, not the arguments.
const called = /** @type {(a: number) => number} */ (
  fn
  // own-line
)(1);
const calledTrailing = /** @type {(a: number) => number} */ (
  fn // trailing
)(1);
// A comment between the cast `)` and the arguments stays outside.
const between = /** @type {(a: number) => number} */ (
  fn // trailing
) /* between */ (1);

// Other hiding sites (`FormatNodeWithoutTrailingComments`): class heritage.
class Heritage extends /** @type {typeof Base} */ (
  Base // trailing
) {}

// Member object.
const member = foo(/** @type {X} */ (
  fn
  // own-line
).prop);

// Same-line comments after the cast `)` still move behind the `;`.
const after = /** @type {string} */ (value) /* after */;
