a ? (b?) => ({ c: (d) }) : e => f;
a ? () => (b, (c)) : d => e;
a ? () => async((b)) : c => d;
a ? result = () => (b, (c)) : d => e;
a ? b ? c : () => async((d)) : e => f;
a ? () => async(/* argument */ (b)) : /* alternate */ c => d;
a ? () => async((b)) : (c => d)();
a ? () => async((b)) : (c => d, e);
a ? () => async((b)) : (c => d) as T;
a ? () => async((b)) : (c => d) satisfies T;
a ? () => async((b)) : (c => d).name;
a ? () => async((b)) : (c => d)!;
a ? () => async((b)) : (c => d) || e;
a ? () => b : c;
a ? () => {} : c => d;
function* f() {
  a ? yield () => async((b)) : c => d;
  a ? yield* () => async((b)) : c => d;
  a ? yield yield () => async((b)) : c => d;
  a ? yield (b ? c : () => async((d))) : e => f;
  a ? yield ((b) = () => async((c))) : d => e;
  a ? yield ((b) = 1) : c => d;
  a ? b ? c : yield () => async((d)) : e => f;
}
