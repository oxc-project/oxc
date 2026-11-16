a ? b ? c : () => async((d)) : e => f;
function* f() {
  a ? b ? c : yield () => async((d)) : e => f;
  a ? yield (() => async((b)) /* tail */) : c => d;
  a ? yield (() => async((b)) // tail
  ) : c => d;
}
