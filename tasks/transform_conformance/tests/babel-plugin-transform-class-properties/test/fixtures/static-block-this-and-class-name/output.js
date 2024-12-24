var _C, _C2, _C3, _C4, _C5, _Nested;

class C {}
_C = C;
_C.a();

class C2 {}
_C2 = C2;
_C2.b();

class C3 {}
_C3 = C3;
(() => {
  _C3.c();
  _C3.d();
})();

let C4 = (_C4 = class C {}, _C4.e(), _C4);

let C5 = (_C5 = class C {}, (() => {
  _C5.f();
  C5.g();
})(), _C5.h(), _C5);

class Nested {}
_Nested = Nested;
(() => {
  _Nested.i = () => _Nested.j();
  function inner() {
    return [this, _Nested];
  }
  otherIdent;
})();
