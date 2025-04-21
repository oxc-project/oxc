export let Foo;

(function(_Foo) {
  const Bar = _Foo.Bar = 1;
})(Foo || (Foo = {}));