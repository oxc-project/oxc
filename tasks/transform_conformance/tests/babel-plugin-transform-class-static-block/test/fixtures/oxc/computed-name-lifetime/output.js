const classes = [];
for (const key of ["a", "b"]) {
  classes.push((() => {
    let _key;
    return class {
      [_key = babelHelpers.toPropertyKey(key)] = (() => {
        var _staticBlock;
        return { [_key]: class {
          static #_ = _staticBlock = () => (this.seen = this.name, this);
        } }[_key], _staticBlock();
      })();
    };
  })());
}
console.log(new classes[0]().a.seen, new classes[1]().b.seen);
function* suspended() {
  var _staticBlock2;
  let _key2;
  const classes = [];
  while (classes.length < 2 && classes.push((class _Class {
    static #_ = ((_value) => (_staticBlock2 = () => (Object.freeze(this), this), _value))((babelHelpers.setFunctionName(this, ""), _key2));
    [_key2 = babelHelpers.toPropertyKey(yield classes.length)] = (() => {
      var _staticBlock3;
      return { [_Class.#_]: class {
        static #_ = _staticBlock3 = () => (this.seen = this.name, this);
      } }[_Class.#_], _staticBlock3();
    })();
  }, _staticBlock2()))) {}
  return classes;
}
