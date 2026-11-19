const object = {};
class C {
  static #m() {}
  [(C => #m in C)(object)]() {}
}
