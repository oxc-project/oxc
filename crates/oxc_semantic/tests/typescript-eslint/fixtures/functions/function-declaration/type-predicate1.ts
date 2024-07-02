function foo(arg: any): arg is string {
  return typeof arg === 'string';
}
