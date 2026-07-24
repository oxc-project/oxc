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
