//! ES2018 object spread transformation.
//!
//! PLACEHOLDER ONLY. NOT IMPLEMENTED YET. TODO.
//!
//! > This plugin is included in `preset-env`, in ES2018
//!
//! ## Example
//!
//! Input:
//! ```js
//! var { a, ...b } = x;
//! ```
//!
//! Output:
//! ```js
//! // TBD
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-object-rest-spread](https://babeljs.io/docs/babel-plugin-transform-object-rest-spread).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-object-rest-spread>
//! * Object rest/spread TC39 proposal: <https://github.com/tc39/proposal-object-rest-spread>

use super::ObjectRestSpreadOptions;

pub struct ObjectRest {
    _options: ObjectRestSpreadOptions,
}

impl ObjectRest {
    pub fn new(options: ObjectRestSpreadOptions) -> Self {
        Self { _options: options }
    }
}
