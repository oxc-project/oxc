class C {
  static #field = #field in this;
  static #empty;

  static hasFields(object) {
    return [#field in object, #empty in object, #method in object, #accessor in object];
  }

  static #method() {}
  static get #accessor() {}
}
