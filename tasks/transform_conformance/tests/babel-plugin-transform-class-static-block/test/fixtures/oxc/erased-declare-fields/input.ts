const calls = [];
class Before {
  static { calls.push("before"); }
  declare static omitted: unknown;
}
class After {
  declare static omitted: unknown;
  static { calls.push("after"); }
}
class Mixed {
  static { calls.push("first"); }
  declare static omitted: unknown;
  static present = (calls.push("field"), 1);
  declare static trailing: unknown;
  static { calls.push("last"); }
}
