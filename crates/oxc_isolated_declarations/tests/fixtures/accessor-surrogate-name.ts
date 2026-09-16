// Accessor pairs match by the complete JavaScript property name, including lone surrogates.
export class Lead {
  get "\uD800"() {
    return 1;
  }
  set "\uD800"(v) {}
}

export class Trail {
  get "\uDC00"() {
    return "s";
  }
  set ["\uDC00"](v) {}
}

export class Text {
  get "a\uD800b"() {
    return "s";
  }
  set [`a\uD800b`](v) {}
}

// Different lone surrogates are different accessors.
export class Different {
  get "\uD800"() {
    return 1;
  }
  set "\uDC00"(v) {}
}

export interface Signatures {
  get "\uD800"(): number;
  set "\uD800"(v);
  get "\uDC00"();
  set "\uDC00"(v: string);
}
