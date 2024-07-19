type T = string;
const foo = (arg: any): arg is T => {
  return typeof arg === 'string';
};
