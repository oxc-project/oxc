export let Foo;

(function(_Foo) {
  const Bar = _Foo.Bar = 0;
})(Foo || (Foo = {}));

(function(_Foo2) {
  const Zoo = _Foo2.Zoo = 1;
})(Foo || (Foo = {}));