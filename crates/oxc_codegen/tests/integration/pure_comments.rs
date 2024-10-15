use crate::snapshot;

#[test]
fn annotate_comment() {
    let cases = vec![
        r"
x([
  /* #__NO_SIDE_EFFECTS__ */ function() {},
  /* #__NO_SIDE_EFFECTS__ */ function y() {},
  /* #__NO_SIDE_EFFECTS__ */ function*() {},
  /* #__NO_SIDE_EFFECTS__ */ function* y() {},
  /* #__NO_SIDE_EFFECTS__ */ async function() {},
  /* #__NO_SIDE_EFFECTS__ */ async function y() {},
  /* #__NO_SIDE_EFFECTS__ */ async function*() {},
  /* #__NO_SIDE_EFFECTS__ */ async function* y() {},
])",
        r"
x([
  /* #__NO_SIDE_EFFECTS__ */ y => y,
  /* #__NO_SIDE_EFFECTS__ */ () => {},
  /* #__NO_SIDE_EFFECTS__ */ (y) => (y),
  /* #__NO_SIDE_EFFECTS__ */ async y => y,
  /* #__NO_SIDE_EFFECTS__ */ async () => {},
  /* #__NO_SIDE_EFFECTS__ */ async (y) => (y),
])",
        r"
x([
  /* #__NO_SIDE_EFFECTS__ */ y => y,
  /* #__NO_SIDE_EFFECTS__ */ () => {},
  /* #__NO_SIDE_EFFECTS__ */ (y) => (y),
  /* #__NO_SIDE_EFFECTS__ */ async y => y,
  /* #__NO_SIDE_EFFECTS__ */ async () => {},
  /* #__NO_SIDE_EFFECTS__ */ async (y) => (y),
])",
        r"
// #__NO_SIDE_EFFECTS__
function a() {}
// #__NO_SIDE_EFFECTS__
function* b() {}
// #__NO_SIDE_EFFECTS__
async function c() {}
// #__NO_SIDE_EFFECTS__
async function* d() {}
        ",
        r"
// #__NO_SIDE_EFFECTS__
function a() {}
// #__NO_SIDE_EFFECTS__
function* b() {}
// #__NO_SIDE_EFFECTS__
async function c() {}
// #__NO_SIDE_EFFECTS__
async function* d() {}
        ",
        r"
/* @__NO_SIDE_EFFECTS__ */ export function a() {}
/* @__NO_SIDE_EFFECTS__ */ export function* b() {}
/* @__NO_SIDE_EFFECTS__ */ export async function c() {}
/* @__NO_SIDE_EFFECTS__ */ export async function* d() {}",
        r"/* @__NO_SIDE_EFFECTS__ */ export function a() {}
/* @__NO_SIDE_EFFECTS__ */ export function* b() {}
/* @__NO_SIDE_EFFECTS__ */ export async function c() {}
/* @__NO_SIDE_EFFECTS__ */ export async function* d() {}
        ",
        // Only "c0" and "c2" should have "no side effects" (Rollup only respects "const" and only for the first one)
        r"
/* #__NO_SIDE_EFFECTS__ */ export var v0 = function() {}, v1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ export let l0 = function() {}, l1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ export const c0 = function() {}, c1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ export var v2 = () => {}, v3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ export let l2 = () => {}, l3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ export const c2 = () => {}, c3 = () => {}
        ",
        // Only "c0" and "c2" should have "no side effects" (Rollup only respects "const" and only for the first one)
        r"
/* #__NO_SIDE_EFFECTS__ */ var v0 = function() {}, v1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ let l0 = function() {}, l1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ const c0 = function() {}, c1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ var v2 = () => {}, v3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ let l2 = () => {}, l3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ const c2 = () => {}, c3 = () => {}
        ",
        r"
isFunction(options)
? // #8326: extend call and options.name access are considered side-effects
  // by Rollup, so we have to wrap it in a pure-annotated IIFE.
  /*#__PURE__*/ (() =>
    extend({ name: options.name }, extraOptions, { setup: options }))()
: options
                ",
        r"isFunction(options) ? /*#__PURE__*/ (() => extend({ name: options.name }, extraOptions, { setup: options }))() : options;
        ",
        r"
const obj = {
  props: /*#__PURE__*/ extend({}, TransitionPropsValidators, {
    tag: String,
    moveClass: String,
  }),
};
const p = /*#__PURE__*/ Promise.resolve();
        ",
        r"
const staticCacheMap = /*#__PURE__*/ new WeakMap()
        ",
        r#"
const builtInSymbols = new Set(
  /*#__PURE__*/
  Object.getOwnPropertyNames(Symbol)
    .filter(key => key !== "arguments" && key !== "caller")
)
        "#,
        "(/* @__PURE__ */ new Foo()).bar();\n",
        "(/* @__PURE__ */ Foo()).bar();\n",
        "(/* @__PURE__ */ new Foo())['bar']();\n",
        "(/* @__PURE__ */ Foo())['bar']();\n",
        // https://github.com/oxc-project/oxc/issues/4843
        r"
/* #__NO_SIDE_EFFECTS__ */
const defineSSRCustomElement = /* @__NO_SIDE_EFFECTS__ */ (
  options,
  extraOptions,
) => {
  return /* @__PURE__ */ defineCustomElement(options, extraOptions, hydrate);
};
        ",
        // Range leading comments
        r"
const defineSSRCustomElement = () => {
  return /* @__PURE__ */ /* @__NO_SIDE_EFFECTS__ */ /* #__NO_SIDE_EFFECTS__ */ defineCustomElement(options, extraOptions, hydrate);
};
        ",
        "
        const Component = // #__PURE__
        React.forwardRef((props, ref) => {});
        ",
        // Copy from <https://github.com/rolldown-rs/rolldown/blob/main/crates/rolldown/tests/esbuild/dce/remove_unused_pure_comment_calls/entry.js>
        "
        function bar() {}
        let bare = foo(bar);

        let at_yes = /* @__PURE__ */ foo(bar);
        let at_no = /* @__PURE__ */ foo(bar());
        let new_at_yes = /* @__PURE__ */ new foo(bar);
        let new_at_no = /* @__PURE__ */ new foo(bar());

        let nospace_at_yes = /*@__PURE__*/ foo(bar);
        let nospace_at_no = /*@__PURE__*/ foo(bar());
        let nospace_new_at_yes = /*@__PURE__*/ new foo(bar);
        let nospace_new_at_no = /*@__PURE__*/ new foo(bar());

        let num_yes = /* #__PURE__ */ foo(bar);
        let num_no = /* #__PURE__ */ foo(bar());
        let new_num_yes = /* #__PURE__ */ new foo(bar);
        let new_num_no = /* #__PURE__ */ new foo(bar());

        let nospace_num_yes = /*#__PURE__*/ foo(bar);
        let nospace_num_no = /*#__PURE__*/ foo(bar());
        let nospace_new_num_yes = /*#__PURE__*/ new foo(bar);
        let nospace_new_num_no = /*#__PURE__*/ new foo(bar());

        let dot_yes = /* @__PURE__ */ foo(sideEffect()).dot(bar);
        let dot_no = /* @__PURE__ */ foo(sideEffect()).dot(bar());
        let new_dot_yes = /* @__PURE__ */ new foo(sideEffect()).dot(bar);
        let new_dot_no = /* @__PURE__ */ new foo(sideEffect()).dot(bar());

        let nested_yes = [1, /* @__PURE__ */ foo(bar), 2];
        let nested_no = [1, /* @__PURE__ */ foo(bar()), 2];
        let new_nested_yes = [1, /* @__PURE__ */ new foo(bar), 2];
        let new_nested_no = [1, /* @__PURE__ */ new foo(bar()), 2];

        let single_at_yes = // @__PURE__
                foo(bar);
        let single_at_no = // @__PURE__
                foo(bar());
        let new_single_at_yes = // @__PURE__
                new foo(bar);
        let new_single_at_no = // @__PURE__
                new foo(bar());

        let single_num_yes = // #__PURE__
                foo(bar);
        let single_num_no = // #__PURE__
                foo(bar());
        let new_single_num_yes = // #__PURE__
                new foo(bar);
        let new_single_num_no = // #__PURE__
                new foo(bar());

        let bad_no = /* __PURE__ */ foo(bar);
        let new_bad_no = /* __PURE__ */ new foo(bar);

        let parens_no = (/* @__PURE__ */ foo)(bar);
        let new_parens_no = new (/* @__PURE__ */ foo)(bar);

        let exp_no = /* @__PURE__ */ foo() ** foo();
        let new_exp_no = /* @__PURE__ */ new foo() ** foo();
        ",
    ];

    snapshot("pure_comments", &cases);
}
