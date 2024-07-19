const foo = function (arg: any): arg is string {
  return typeof arg === 'string';
};
