class C {}
(function (_C) {
  const x = _C.x = 1;
})(C || (C = {}));
use(C);
