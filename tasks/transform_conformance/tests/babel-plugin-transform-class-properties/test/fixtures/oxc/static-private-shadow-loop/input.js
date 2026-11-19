const classes = [];
for (let i = 0; i < 2; i++) {
  class C {
    static #m() { return 42; }
    static has(C = this) { return #m in C; }
    static read(C = this) { return C.#m(); }
  }
  classes.push(C);
}
