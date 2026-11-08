class MyError extends Error {
  code;
  description;
  constructor(code, description) {
    if (description) {
      super(code + ': ' + description);
      this.code = code;
      this.description = description;
    } else {
      super(code);
      this.code = code;
      this.description = description;
    }
    this.name = 'MyError';
  }
}

class MyError2 extends Error {
  code;
  constructor(code) {
    switch (code) {
      case 'A':
        super('Error A');
        break;
      default:
        super(code);
    }
    this.code = code;
  }
}

class MyError3 extends Error {
  code;
  constructor(code) {
    super(code), init(this);
    this.code = code;
  }
}

class Unbraced extends Error {
  code;
  constructor(code, useCode) {
    if (useCode) {
      super(code), init(this);
      this.code = code;
    } else {
      super();
      this.code = code;
    }
  }
}

class NestedConditional extends Error {
  code;
  constructor(code, first, second) {
    if (first) {
      if (second) super(code);
      return {};
    } else {
      super(code);
    }
    this.code = code;
  }
}

class OneSuperBranch extends Error {
  code;
  constructor(code, skip) {
    if (skip) return {};
    else super(code);
    this.code = code;
  }
}

class MissingElse extends Error {
  code;
  constructor(code, initialize) {
    if (initialize) super(code);
    this.code = code;
  }
}
