a ? b ? c : () => async((d)) : e => f;
function* f() {
  a ? b ? c : yield () => async((d)) : e => f;
  a ? yield (() => async((b)) /* tail */) : c => d;
  a ? yield (() => async((b)) // tail
  ) : c => d;
  a ? yield (b
  /* own line */) : c;
  a ? b ? c : yield (() => async((d))
  /* own line */) : e => f;
  a ? yield (() => async((b)) // prettier-ignore
  ) : c => d;
  a ? yield (() => async((b)) /* prettier-ignore */
  ) : c => d;
  a ? b ? c : yield (() => async((d)) // oxfmt-ignore
  ) : e => f;
}
