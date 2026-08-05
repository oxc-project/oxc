class MyError extends Error {
  constructor(
    public code: string,
    public description?: string,
  ) {
    if (description) {
      super(code + ': ' + description);
    } else {
      super(code);
    }
    this.name = 'MyError';
  }
}

class MyError2 extends Error {
  constructor(
    public code: string,
  ) {
    switch (code) {
      case 'A':
        super('Error A');
        break;
      default:
        super(code);
    }
  }
}

class MyError3 extends Error {
  constructor(
    public code: string,
  ) {
    super(code), init(this);
  }
}

class Unbraced extends Error {
  constructor(public code: string, useCode: boolean) {
    if (useCode) super(code), init(this);
    else super();
  }
}

class NestedConditional extends Error {
  constructor(public code: string, first: boolean, second: boolean) {
    if (first) {
      if (second) super(code);
      return {};
    } else {
      super(code);
    }
  }
}

class OneSuperBranch extends Error {
  constructor(public code: string, skip: boolean) {
    if (skip) return {};
    else super(code);
  }
}

class MissingElse extends Error {
  constructor(public code: string, initialize: boolean) {
    if (initialize) super(code);
  }
}
