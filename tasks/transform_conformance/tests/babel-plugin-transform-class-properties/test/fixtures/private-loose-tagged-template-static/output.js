var _tag = /*#__PURE__*/ babelHelpers.classPrivateFieldLooseKey("tag");

class Foo {
  static getReceiver() {
    return babelHelpers.classPrivateFieldLooseBase(this, _tag)[_tag]`tagged template`;
  }
}

Object.defineProperty(Foo, _tag, {
  writable: true,
  value: function() {
    return this;
  }
});
