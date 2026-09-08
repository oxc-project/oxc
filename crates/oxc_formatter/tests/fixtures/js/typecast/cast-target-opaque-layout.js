// Prettier keeps the cast `ParenthesizedExpression` node, so layout rules that
// look through a chain (member chain head, call object, arrow body) stop at a cast target.

// Assignment: the member chain on a cast target is not a poorly breakable chain,
// the layout stays fluid and the member breaks, not the `=`.
const complex_selectors_list = /** @type {CSS.SelectorList} */ (selector.args).children;
const [get, set] = /** @type {SequenceExpression} */ (context.visit(attribute.expression)).expressions;
var events = /** @type {Record<string, Function[] | Function>} */ ($$props.$$events)?.[event.type];
const events2 = /** @type {Record<string, Function | Function[]>} */ (active_component_context.s.$$events)?.[/** @type {string} */ (type)];

// A plain chain still breaks after the `=`.
const complex_selectors_bbbbbbbb = selector.args.children_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;

// Last-argument arrow: a cast inside the call body keeps the hug, a cast-wrapped body does not.
const needs_version_increase = !s.reactions?.every((r) => /** @type {NonNullable<typeof v_reactions>} */ (v_reactions).has(r));
const needs_version_increase2 = !s.reactions?.every((r) => /** @type {NonNullable<typeof v_reactions>} */ (v_reactions.has(r)));

// Assignment: a cast-wrapped RHS is opaque to the never-break shapes too (template literal, literals, class).
// The line breaks after the `=` when the cast comment does not fit; a short one stays inline.
const fieldPath = /** @type {import("react-hook-form").Path<TFormValues>} */ (`${key}.${locale}`);
const short_cast = /** @type {T} */ (`short`);
const cast_bool = /** @type {import("react-hook-form").Path<TFormValues>} */ (true);
const cast_num = /** @type {import("react-hook-form").Path<TFormValuesXXXXXXX>} */ (123456);
const cast_tagged = /** @type {import("react-hook-form").Path<TFormValues>} */ (tag`${key}.${locale}`);
const cast_class = /** @type {import("react-hook-form").Path<TFormValues>} */ (class {});

// The other RHS shape rules stop at the cast too: `require`, assignment chains, arrows, the unary walk.
const someLongVariableNameForRequire = /** @type {typeof import("some-module")} */ (require("some-module"));
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa = bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb = /** @type {T} */ (cccccccccccccccc = dddddddddddddddd);
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa = /** @type {T} */ (bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb = cccccccccccccccc.ddddddddddddddd);
aaaaaaaaaaaaaaaaaaaa = bbbbbbbbbbbbbbbbbbbbb = /** @type {T} */ ((xxxxxxxxxxxxxxxx) => xxxxxxxxxxxxxxxx.yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy);
const someLongVariableNameForArrow3 = /** @type {Handler} */ ((a) => (b) => (c) => (d) => someLongFunctionCall(a, b, c, d, eeeeeeeeeee));
const someLongVariableNameHereForNot = !/** @type {Promise<string>} */ ("some long string literal here ok yes");
