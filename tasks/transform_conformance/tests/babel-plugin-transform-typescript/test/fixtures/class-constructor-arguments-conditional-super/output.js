class MyError extends Error {
  constructor(code, description) {
    if (description) {
      super(code + ': ' + description);
    } else {
      super(code);
    }
    this.code = code;
    this.description = description;
    this.name = 'MyError';
  }
}

class MyError2 extends Error {
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
