use crate::test_same;

#[test]
fn cjs() {
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
}
