class MyClass {
  /**
   * @param {string} name - the name
   * @returns {void}
   */
  method(name) {}

  /**
   * @type {number}
   */
  value = 0;

  /**
   * does something useful.
   * @param {Object} opts - options
   * @param {string} opts.name - the name
   * @param {number} opts.age - the age
   */
  complex(opts) {}
}

function outer() {
  /**
   * @return {boolean}
   */
  function inner() { return true; }

  if (true) {
    /**
     * @type {  string  |  number  }
     */
    const x = "hi";
  }
}
