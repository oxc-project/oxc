class C {
  #x;
  m(o) {
    for ((#x in o);;);
    for (let a = (#x in o);;);
    for (a = (#x in o) || o;;);
    for (a = o || (#x in o);;);
    for (a = (#x in o) ? 1 : 2;;);
    for (a = () => (#x in o);;);
    for (a = 1 * (#x in o == 1);;);
    for (a = (() => #x in o);;);
  }
}
