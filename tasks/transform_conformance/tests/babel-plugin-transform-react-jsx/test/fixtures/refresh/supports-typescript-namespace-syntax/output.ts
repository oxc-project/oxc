namespace Foo {
  export namespace Bar {
    export const A = () => {};
    _c = A;
    function B() {}
    _c2 = B;
    ;
    export const B1 = B;
  }
  export const C = () => {};
  _c3 = C;
  export function D() {}
  _c4 = D;
  ;
  namespace NotExported {
    export const E = () => {};
  }
}

var _c, _c2, _c3, _c4;

$RefreshReg$(_c, "Foo$Bar$A");
$RefreshReg$(_c2, "Foo$Bar$B");
$RefreshReg$(_c3, "Foo$C");
$RefreshReg$(_c4, "Foo$D");
