var _m_dec, _member_dec, _member_dec2, _m_dec3, _undefined_dec;
let _init, _Symbol$iterator, _key, _key2, _init2, _Symbol$iterator2, _babelHelpers$toPrope, _init4, _Symbol$iterator3, _key5, _key6, _init5, _babelHelpers$toPrope3, _key7, _key8, _init6, _babelHelpers$toPrope4;
_m_dec = [[void 0, dec]];
_Symbol$iterator = Symbol.iterator;
// A well-known symbol key can't collide with a string name, so methods keep their keys.
class A {
  constructor() {
    babelHelpers.runInitializers(_init, 5, this);
  }
  m() {}
  plain() {}
  *[_Symbol$iterator]() {}
}
_init = babelHelpers.decoratorStart(null);
babelHelpers.decorateElement(_init, 1, "m", _m_dec, A);
babelHelpers.decoratorMetadata(_init, A);
_member_dec = [[void 0, dec]];
_Symbol$iterator2 = Symbol.iterator;
_key = babelHelpers.decoratorKey();
_babelHelpers$toPrope = babelHelpers.toPropertyKey(Symbol.iterator);
_key2 = babelHelpers.decoratorKey();
// The same well-known symbol twice can collide.
class B {
  constructor() {
    babelHelpers.runInitializers(_init2, 5, this);
  }
  *[_key]() {}
  *[_key2]() {}
}
_init2 = babelHelpers.decoratorStart(null, B, [[
  1,
  _Symbol$iterator2,
  _key
], [
  1,
  _babelHelpers$toPrope,
  _key2
]]);
babelHelpers.decorateElement(_init2, 1, _Symbol$iterator2, _member_dec, B, _init2.p[0]);
babelHelpers.decorateElement(_init2, 1, _babelHelpers$toPrope, [], B, _init2.p[1]);
babelHelpers.decoratorMetadata(_init2, B);
// A shadowed `Symbol` can produce any key.
{
  var _m_dec2;
  let _key3, _key4, _init3, _babelHelpers$toPrope2;
  const Symbol = { iterator: "m" };
  _m_dec2 = [[void 0, dec]];
  _key3 = babelHelpers.decoratorKey();
  _babelHelpers$toPrope2 = babelHelpers.toPropertyKey(Symbol.iterator);
  _key4 = babelHelpers.decoratorKey();
  class C {
    constructor() {
      babelHelpers.runInitializers(_init3, 5, this);
    }
    [_key3]() {}
    [_key4]() {}
  }
  _init3 = babelHelpers.decoratorStart(null, C, [[
    1,
    "m",
    _key3
  ], [
    1,
    _babelHelpers$toPrope2,
    _key4
  ]]);
  babelHelpers.decorateElement(_init3, 1, "m", _m_dec2, C, _init3.p[0]);
  babelHelpers.decorateElement(_init3, 1, _babelHelpers$toPrope2, [], C, _init3.p[1]);
  babelHelpers.decoratorMetadata(_init3, C);
}
_member_dec2 = [[void 0, dec]];
_Symbol$iterator3 = Symbol.iterator;
// A decorated well-known symbol key keeps every key too.
class D {
  constructor() {
    babelHelpers.runInitializers(_init4, 5, this);
  }
  *[_Symbol$iterator3]() {}
  m() {}
}
_init4 = babelHelpers.decoratorStart(null);
babelHelpers.decorateElement(_init4, 1, _Symbol$iterator3, _member_dec2, D);
babelHelpers.decoratorMetadata(_init4, D);
_m_dec3 = [[void 0, dec]];
_key5 = babelHelpers.decoratorKey();
_babelHelpers$toPrope3 = babelHelpers.toPropertyKey(Symbol.foo);
_key6 = babelHelpers.decoratorKey();
// Other properties of `Symbol` can hold any key.
class E {
  constructor() {
    babelHelpers.runInitializers(_init5, 5, this);
  }
  [_key5]() {}
  [_key6]() {}
}
_init5 = babelHelpers.decoratorStart(null, E, [[
  1,
  "m",
  _key5
], [
  1,
  _babelHelpers$toPrope3,
  _key6
]]);
babelHelpers.decorateElement(_init5, 1, "m", _m_dec3, E, _init5.p[0]);
babelHelpers.decorateElement(_init5, 1, _babelHelpers$toPrope3, [], E, _init5.p[1]);
babelHelpers.decoratorMetadata(_init5, E);
_undefined_dec = [[void 0, dec]];
_key7 = babelHelpers.decoratorKey();
_babelHelpers$toPrope4 = babelHelpers.toPropertyKey(Symbol.asyncIterator);
_key8 = babelHelpers.decoratorKey();
// A later well-known symbol can be missing in an ES2015 engine, making the key "undefined".
class F {
  constructor() {
    babelHelpers.runInitializers(_init6, 5, this);
  }
  [_key7]() {}
  [_key8]() {}
}
_init6 = babelHelpers.decoratorStart(null, F, [[
  1,
  "undefined",
  _key7
], [
  1,
  _babelHelpers$toPrope4,
  _key8
]]);
babelHelpers.decorateElement(_init6, 1, "undefined", _undefined_dec, F, _init6.p[0]);
babelHelpers.decorateElement(_init6, 1, _babelHelpers$toPrope4, [], F, _init6.p[1]);
babelHelpers.decoratorMetadata(_init6, F);
