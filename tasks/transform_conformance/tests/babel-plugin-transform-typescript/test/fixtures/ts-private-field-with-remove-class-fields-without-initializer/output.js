let _kTransferable, _kValue;
var _view = /* @__PURE__ */ new WeakMap();
_kTransferable = kTransferable;
_kValue = kValue;
export class ArrayBufferViewTransferable {
  constructor(view) {
    babelHelpers.classPrivateFieldInitSpec(this, _view, void 0);
    babelHelpers.classPrivateFieldSet2(_view, this, view);
  }

  get [_kTransferable]() {
    return babelHelpers.classPrivateFieldGet2(_view, this).buffer;
  }

  get [_kValue]() {
    return babelHelpers.classPrivateFieldGet2(_view, this);
  }
}
