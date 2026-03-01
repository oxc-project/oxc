use oxc_transformer::{JsxOptions, TransformOptions};

use crate::test;

fn pure_options() -> TransformOptions {
    TransformOptions {
        jsx: JsxOptions { jsx_plugin: false, pure: true, ..JsxOptions::disable() },
        ..TransformOptions::default()
    }
}

fn test_pure(source: &str, expected: &str) {
    let result = test(source, &pure_options()).unwrap();
    assert_eq!(result.trim(), expected.trim(), "\n\nInput:\n{source}");
}

fn test_no_change(source: &str) {
    let result = test(source, &pure_options()).unwrap();
    assert_eq!(result.trim(), source.trim(), "\n\nInput:\n{source}");
}

#[test]
fn named_import_react() {
    test_pure(
        "import { forwardRef } from 'react'; forwardRef(() => {});",
        "import { forwardRef } from 'react';\n/* @__PURE__ */ forwardRef(() => {});",
    );
}

#[test]
fn named_import_multiple_methods() {
    test_pure(
        "import { memo, lazy, createContext } from 'react'; memo(Comp); lazy(() => import('./Foo')); createContext(null);",
        "import { memo, lazy, createContext } from 'react';\n/* @__PURE__ */ memo(Comp);\n/* @__PURE__ */ lazy(() => import('./Foo'));\n/* @__PURE__ */ createContext(null);",
    );
}

#[test]
fn named_import_react_dom() {
    test_pure(
        "import { createPortal } from 'react-dom'; createPortal(child, container);",
        "import { createPortal } from 'react-dom';\n/* @__PURE__ */ createPortal(child, container);",
    );
}

#[test]
fn default_import_react() {
    test_pure(
        "import React from 'react'; React.forwardRef(() => {});",
        "import React from 'react';\n/* @__PURE__ */ React.forwardRef(() => {});",
    );
}

#[test]
fn namespace_import_react() {
    test_pure(
        "import * as React from 'react'; React.memo(Comp);",
        "import * as React from 'react';\n/* @__PURE__ */ React.memo(Comp);",
    );
}

#[test]
fn default_import_react_dom() {
    test_pure(
        "import ReactDOM from 'react-dom'; ReactDOM.createPortal(child, container);",
        "import ReactDOM from 'react-dom';\n/* @__PURE__ */ ReactDOM.createPortal(child, container);",
    );
}

#[test]
fn no_annotation_for_unknown_method() {
    test_no_change("import { useState } from 'react';\nuseState(0);");
}

#[test]
fn no_annotation_for_unknown_module() {
    test_no_change("import { forwardRef } from 'preact';\nforwardRef(() => {});");
}

#[test]
fn no_annotation_for_non_react_member() {
    test_no_change("import Foo from 'foo';\nFoo.memo(Comp);");
}

#[test]
fn no_annotation_for_unknown_member_method() {
    test_no_change("import React from 'react';\nReact.useState(0);");
}

#[test]
fn all_react_pure_methods() {
    let methods = [
        "cloneElement",
        "createContext",
        "createElement",
        "createFactory",
        "createRef",
        "forwardRef",
        "isValidElement",
        "memo",
        "lazy",
    ];
    for method in methods {
        let source = format!("import {{ {method} }} from 'react'; {method}(arg);");
        let expected =
            format!("import {{ {method} }} from 'react';\n/* @__PURE__ */ {method}(arg);");
        test_pure(&source, &expected);
    }
}
