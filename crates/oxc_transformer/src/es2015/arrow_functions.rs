//! ES2015 Arrow Functions
//!
//! This plugin transforms arrow functions (`() => {}`) to function expressions (`function () {}`).
//!
//! > This plugin is included in `preset-env`, in ES2015
//!
//! ## Missing features
//!
//! Implementation is incomplete at present. Still TODO:
//!
//! * `spec` option.
//! * Handle `arguments` in arrow functions.
//! * Handle `new.target` in arrow functions.
//! * Handle arrow function in function params (`function f(g = () => this) {}`).
//!   Babel gets this wrong: <https://babeljs.io/repl#?code_lz=GYVwdgxgLglg9mABMOcAUAPRBeRaCUOAfIlABYwDOhA3gL5A&presets=&externalPlugins=%40babel%2Fplugin-transform-arrow-functions%407.24.7>
//! * Error on arrow functions in class properties.
//!   <https://babeljs.io/repl#?code_lz=MYGwhgzhAEDC0G8BQ1oDMD2HoF5oAoBKXAPmgBcALASwgG4kBfJIA&presets=&externalPlugins=%40babel%2Fplugin-transform-arrow-functions%407.24.7>
//!   or we can support it:
//!     `class C { x = () => this; }`
//!     -> `class C { x = (function(_this) { return () => _this; })(this); }`
//! * Error on `super` in arrow functions.
//!   <https://babeljs.io/repl#?code_lz=MYGwhgzhAEBiD29oG8C-AoUkYCEwCdoBTADwBciA7AExgSWXWmgFsiyALeagCgEoUTZtHzsArvkrR-0ALwA-aBDEAHIvgB0AM0QBuIRgxA&presets=&externalPlugins=%40babel%2Fplugin-transform-arrow-functions%407.24.7>
//!
//! ## Example
//!
//! Input:
//! ```js
//! var a = () => {};
//! var a = b => b;
//!
//! const double = [1, 2, 3].map(num => num * 2);
//! console.log(double); // [2,4,6]
//!
//! var bob = {
//!   name: "Bob",
//!   friends: ["Sally", "Tom"],
//!   printFriends() {
//!     this.friends.forEach(f => console.log(this.name + " knows " + f));
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! Output:
//! ```js
//! var a = function() {};
//! var a = function(b) { return b; };
//!
//! const double = [1, 2, 3].map(function(num) {
//!   return num * 2;
//! });
//! console.log(double); // [2,4,6]
//!
//! var bob = {
//!   name: "Bob",
//!   friends: ["Sally", "Tom"],
//!   printFriends() {
//!     var _this = this;
//!     this.friends.forEach(function(f) {
//!       return console.log(_this.name + " knows " + f);
//!     });
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! ## Options
//!
//! ### `spec`
//!
//! `boolean`, defaults to `false`.
//!
//! This option enables the following:
//! * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is,
//!   instead of using a renamed this.
//! * Add a runtime check to ensure the functions are not instantiated.
//! * Add names to arrow functions.
//!
//! #### Example
//!
//! Using spec mode with the above example produces:
//!
//! ```js
//! var _this = this;
//!
//! var a = function a() {
//!   babelHelpers.newArrowCheck(this, _this);
//! }.bind(this);
//! var a = function a(b) {
//!   babelHelpers.newArrowCheck(this, _this);
//!   return b;
//! }.bind(this);
//!
//! const double = [1, 2, 3].map(
//!   function(num) {
//!     babelHelpers.newArrowCheck(this, _this);
//!     return num * 2;
//!   }.bind(this)
//! );
//! console.log(double); // [2,4,6]
//!
//! var bob = {
//!   name: "Bob",
//!   friends: ["Sally", "Tom"],
//!   printFriends() {
//!     var _this2 = this;
//!     this.friends.forEach(
//!       function(f) {
//!         babelHelpers.newArrowCheck(this, _this2);
//!         return console.log(this.name + " knows " + f);
//!       }.bind(this)
//!     );
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-arrow-functions](https://babel.dev/docs/babel-plugin-transform-arrow-functions).
//!
//! ## References:
//!
//! * Babel plugin implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-arrow-functions>
//! * Arrow function specification: <https://tc39.es/ecma262/#sec-arrow-function-definitions>

use serde::Deserialize;

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_data_structures::stack::NonEmptyStack;
use oxc_span::SPAN;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::context::TransformCtx;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct ArrowFunctionsOptions {
    /// This option enables the following:
    /// * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
    /// * Add a runtime check to ensure the functions are not instantiated.
    /// * Add names to arrow functions.
    #[serde(default)]
    pub spec: bool,
}

pub struct ArrowFunctions<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ArrowFunctions<'a, 'ctx> {
    pub fn new(options: ArrowFunctionsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for ArrowFunctions<'a, 'ctx> {
    fn enter_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.ctx.arrow_function_to_expression.enable_arrow_function();
    }
}
