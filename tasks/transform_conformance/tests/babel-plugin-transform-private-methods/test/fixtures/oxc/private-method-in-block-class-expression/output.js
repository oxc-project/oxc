function makeClasses() {
  const Classes = [];
  for (let captured = 0; captured < 2; captured++) {
    var _Class_brand;
    Classes.push((_Class_brand = /* @__PURE__ */ new WeakSet(), class {
      constructor() {
        babelHelpers.classPrivateMethodInitSpec(this, _Class_brand);
      }
      run() {
        return babelHelpers.assertClassBrand(_Class_brand, this, _read).call(this).next().value + 1;
      }
    }));
    function* _read() {
      yield captured;
    }
  }
  return Classes;
}
export function get() {
  const Classes = makeClasses();
  return [new Classes[0]().run(), new Classes[1]().run()];
}
