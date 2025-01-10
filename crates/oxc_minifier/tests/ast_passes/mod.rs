mod dead_code_elimination;

use oxc_minifier::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::default();
    crate::test(source_text, expected, options);
}

fn test_same(source_text: &str) {
    test(source_text, source_text);
}

// Oxc Integration Tests

#[test]
fn integration() {
    test(
        "function writeInteger(int) {
          if (int >= 0)
            if (int <= 0xffffffff)
              return this.u32(int);
            else if (int > -0x80000000)
              return this.n32(int);
        }",
        "function writeInteger(int) {
          if (int >= 0) {
            if (int <= 4294967295) return this.u32(int);
            if (int > -2147483648) return this.n32(int);
          }
        }",
    );
}

#[test] // https://github.com/oxc-project/oxc/issues/4341
fn tagged_template() {
    test_same("(1, o.f)()");
    test_same("(1, o.f)``");
    test_same("(!0 && o.f)()");
    test_same("(!0 && o.f)``");
    test("(!0 ? o.f : !1)()", "(0 ? !1: o.f)()");
    test("(!0 ? o.f : !1)``", "(0 ? !1: o.f)``");

    test("foo(true && o.f)", "foo(o.f)");
    test("foo(true ? o.f : false)", "foo(o.f)");
}

#[test]
fn cjs() {
    // Bail `cjs-module-lexer`.
    test_same("0 && (module.exports = { version });");

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
    test_same(
        "Object.defineProperty(exports, 'ConnectableObservable', {
          enumerable: true,
          get: function() {
            return ConnectableObservable_1.ConnectableObservable;
          }
        });",
    );
    // @babel/types/lib/index.js
    test(
        r#"
    Object.defineProperty(exports, "__esModule", {
      value: true
    });
    Object.defineProperty(exports, "TargetNames", {
      enumerable: true,
      get: function () {
        return _options.TargetNames;
      }
    });"#,
        r#"
    Object.defineProperty(exports, "__esModule", { value: true }), Object.defineProperty(exports, "TargetNames", {
        enumerable: true,
        get: function() {
            return _options.TargetNames;
        }
    });"#,
    );

    test(
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
        "
        Object.keys(_index6).forEach(function(key) {
        if (key === 'default' || key === '__esModule') return;
        if (Object.prototype.hasOwnProperty.call(_exportNames, key)) return;
        if (key in exports && exports[key] === _index6[key]) return;
        Object.defineProperty(exports, key, {
                enumerable: true,
                get: function() {
                        return _index6[key];
                }
        });
});",
    );
}
