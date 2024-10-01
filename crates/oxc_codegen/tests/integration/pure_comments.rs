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
    ];

    snapshot("pure_comments", &cases);
}
