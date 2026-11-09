function f({ a, ...rest }, b = a, let = 3) { return { let, b }; }
function named({ a, ...rest }, b = a, let = () => a) { return let.name; }
function destructured({ a, ...rest }, b = a, { let = 3 } = {}) { return let; }
function rest({ a, ...other }, b = a, ...let) { return let; }
function before(let, { a, ...rest }, b = a) { return let; }
function tdz({ a, ...rest }, b = a, let = let) { return let; }
function copied({ a, ...rest }, b = a, let = 3) { var let = 4; return let; }
function nested({ a, ...rest }, b = a, { let = function () {} } = {}) { return let.name; }
function array({ a, ...rest }, b = a, [let = () => a] = []) { return let.name; }
function klass({ a, ...rest }, b = a, { let = class { static inferred() { return this.name; } } } = {}) { return [let.name, let.inferred()]; }
function assignments({ a, ...rest }, b = a, let) {
  let = () => a;
  const names = [let.name];
  let = null;
  let ||= function () {};
  names.push(let.name);
  let &&= class {};
  names.push(let.name);
  let = null;
  let ??= () => a;
  names.push(let.name);
  [let = function () {}] = [];
  names.push(let.name);
  ({ x: let = class {} } = {});
  names.push(let.name);
  ({ let = () => a } = {});
  names.push(let.name);
  ({ let } = { let: 4 });
  return [names, let];
}
