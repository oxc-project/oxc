function create() {
  return new class {
    static before = [#empty in this, #field in this, #method in this];
    static #empty;
    static #field = #field in this;
    static after = [#empty in this, #field in this, this.#field];
    static #method() {}

    hasFields(object) {
      return [#empty in object, #field in object, #method in object];
    }
  }();
}

const first = create();
const second = create();
expect(first.constructor.before).toEqual([false, false, true]);
expect(first.constructor.after).toEqual([true, true, false]);
expect(second.constructor.before).toEqual([false, false, true]);
expect(second.constructor.after).toEqual([true, true, false]);
expect(first.hasFields(first.constructor)).toEqual([true, true, true]);
expect(first.hasFields(second.constructor)).toEqual([false, false, false]);
expect(first.hasFields(first)).toEqual([false, false, false]);
expect(() => first.hasFields(undefined)).toThrow(TypeError);
