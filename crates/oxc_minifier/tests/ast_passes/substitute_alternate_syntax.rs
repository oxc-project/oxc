//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>

use crate::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::all_true();
    crate::test(source_text, expected, options);
}

fn test_same(source_text: &str) {
    test(source_text, source_text);
}

// Oxc

#[test]
fn cjs() {
    // Bail `cjs-module-lexer`.
    // Export is undefined when `enumerable` is "!0".
    // https://github.com/nodejs/cjs-module-lexer/issues/64
    test_same(
        "Object.defineProperty(exports, 'ConnectableObservable', {
          enumerable: true,
          get: function() {
            return ConnectableObservable_1.ConnectableObservable;
          }
        });",
    );
    // @babel/types/lib/index.js
    test_same(
        r#"Object.keys(_index6).forEach(function(key) {
          if (key === "default" || key === "__esModule") return;
          if (Object.prototype.hasOwnProperty.call(_exportNames, key)) return;
          if (key in exports && exports[key] === _index6[key]) return;
          Object.defineProperty(exports, key, {
            enumerable: true,
            get: function() {
              return _index6[key];
            }
          });
        });"#,
    );
}

// Google Closure Compiler

#[test]
fn fold_return_result() {
    test("function f(){return !1;}", "function f(){return !1}");
    test("function f(){return null;}", "function f(){return null}");
    test("function f(){return void 0;}", "function f(){return}");
    test("function f(){return void foo();}", "function f(){return void foo()}");
    test("function f(){return undefined;}", "function f(){return}");
    test("function f(){if(a()){return undefined;}}", "function f(){if(a())return}");
}

#[test]
fn undefined() {
    test("var x = undefined", "var x");
    test(
        "var undefined = 1;function f() {var undefined=2;var x;}",
        "var undefined=1;function f(){var undefined=2,x}",
    );
    test("function f(undefined) {}", "function f(undefined){}");
    test("try {} catch(undefined) {}", "try{}catch(undefined){}");
    test("for (undefined in {}) {}", "for(undefined in {}){}");
    test("undefined++", "undefined++");
    test("undefined += undefined", "undefined+=void 0");
}
