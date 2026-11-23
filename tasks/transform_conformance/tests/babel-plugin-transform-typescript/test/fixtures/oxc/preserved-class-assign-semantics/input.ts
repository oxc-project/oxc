// Auto-accessors are kept, other fields use assignment semantics.
class StaticAccessor {
  static accessor a = b;
  static c: number = d;
}

class InstanceAccessor {
  accessor a = b;
  c: number = d;
  static e = f;
}

// Fields before a kept initializer stay, so initializers keep their order.
class FieldBeforeAccessor {
  c = d;
  accessor a = this.c;
  e = f;
}

// Class decorators are kept, fields use assignment semantics.
@dec
class DecoratedClass {
  a = b;
  static c = d;
  constructor(public p: number) {}
}

// Decorated fields are kept for the downstream decorator transform.
class DecoratedFields {
  a = b;
  @dec c = d;
  e = f;
  static g = h;
  @dec static i = j;
}

// Pending computed keys move past a kept field with a non-computed key.
class KeptLiteralKey {
  static [k()] = 1;
  @dec "x" = 2;
  @dec [m()] = 3;
}

// A `declare` field is erased, so pending computed keys skip it.
class KeptDeclareKey {
  static [n()] = 1;
  declare [o()]: number;
  accessor p = 1;
}
