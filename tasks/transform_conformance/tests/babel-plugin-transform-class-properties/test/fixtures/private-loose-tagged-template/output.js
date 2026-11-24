var _tag = /*#__PURE__*/ babelHelpers.classPrivateFieldLooseKey("tag");

class Foo {
  constructor() {
    Object.defineProperty(this, _tag, {
      writable: true,
      value: { "#tag": function() {
        return this;
      } }["#tag"]
    });
  }

  getReceiver() {
    return babelHelpers.classPrivateFieldLooseBase(this, _tag)[_tag]`tagged template`;
  }
}
