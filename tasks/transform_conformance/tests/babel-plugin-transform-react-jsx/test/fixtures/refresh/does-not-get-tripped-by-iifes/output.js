while (item) {
  var _s = $RefreshSig$();
  _s((item) => {
    _s();
    useFoo();
  }, "useFoo{}", true)(item);
}
