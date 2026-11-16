let escaped;
const error = new Error("initializer");

expect(() => {
  class C {
    static #initialized = false;
    static #throwing = (() => {
      escaped = this;
      throw error;
    })();
    static #uninitialized;

    static hasFields(object) {
      return [
        #initialized in object,
        #throwing in object,
        #uninitialized in object,
        #method in object,
        #accessor in object,
      ];
    }

    static #method() {}
    static get #accessor() {}
  }
}).toThrow(error);

expect(escaped.hasFields(escaped)).toEqual([true, false, false, true, true]);
expect(escaped.hasFields({})).toEqual([false, false, false, false, false]);
expect(() => escaped.hasFields(null)).toThrow(TypeError);
