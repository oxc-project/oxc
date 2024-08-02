type T = string;
function foo(arg: any): arg is T {
  return typeof arg === 'string';
}
