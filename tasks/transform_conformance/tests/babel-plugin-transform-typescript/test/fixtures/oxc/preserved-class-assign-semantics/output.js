let _k, _n;
// Auto-accessors are kept, other fields use assignment semantics.
class StaticAccessor {
  static accessor a = b;
  static {
    this.c = d;
  }
}
class InstanceAccessor {
  constructor() {
    this.c = d;
  }
  accessor a = b;
  c;
  static {
    this.e = f;
  }
}
// Fields before a kept initializer stay, so initializers keep their order.
class FieldBeforeAccessor {
  constructor() {
    this.e = f;
  }
  c = d;
  accessor a = this.c;
  e;
}
// Class decorators are kept, fields use assignment semantics.
@dec class DecoratedClass {
  a;
  static {
    this.c = d;
  }
  constructor(p) {
    this.p = p;
    this.a = b;
  }
}
// Decorated fields are kept for the downstream decorator transform.
class DecoratedFields {
  constructor() {
    this.e = f;
  }
  a = b;
  @dec c = d;
  e;
  static {
    this.g = h;
  }
  @dec static i = j;
}
// Pending computed keys move past a kept field with a non-computed key.
class KeptLiteralKey {
  static {
    this[_k] = 1;
  }
  @dec "x" = 2;
  @dec [(_k = k(), m())] = 3;
}
// A `declare` field is erased, so pending computed keys skip it.
class KeptDeclareKey {
  static {
    _n = n();
  }
  static {
    this[_n] = 1;
  }
  accessor p = 1;
}

