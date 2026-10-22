class F {
  #target = null
  next () {
    !(#target in this);
    -(#target in this);
    +(#target in this);
  }
}

// `#x in b` keeps the parens of a binary `in` expression
class C {
  #x;
  m(a, b, c) {
    x = (#x in b).c;
    x = (#x in b)[c];
    x = (#x in b)();
    x = new (#x in b)();
    x = (#x in b)`t`;
    x = (#x in b)?.c;
    x = [...(#x in b)];
    x = a + (#x in b);
    x = (#x in b) + a;
    x = a < (#x in b);
    x = (#x in b) < a;
    x = a in (#x in b);
    x = (#x in b) & a;
    x = a ** (#x in b);
    x = (#x in b) && a;
    x = (#x in b) ? a : c;
    x = class extends (#x in b) {};
    for (let y = (#x in b); ; );

    x = #x in (a < b);
    x = #x in (a in b);
    x = #x in (a + b);
    x = #x in (a && b);
    x = #x in (a ? b : c);
    x = #x in (() => a);
  }
  async n(b) {
    x = await (#x in b);
  }
}
