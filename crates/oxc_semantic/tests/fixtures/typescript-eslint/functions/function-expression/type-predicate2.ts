type T = string;
const foo = function (arg: any): arg is T {
  return typeof arg === 'string';
};
