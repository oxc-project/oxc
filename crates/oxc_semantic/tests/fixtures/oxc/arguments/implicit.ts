const arguments = [];
type arguments = number;

arguments;
const arrow = () => arguments;
function ordinary(value = arguments) {
  arguments;
  { arguments; }
  const inherited = () => arguments;
  type T = arguments;
  type Q = typeof arguments;
}
function parameter(arguments = []) {
  arguments;
  const inherited = () => arguments;
  function nested(value = arguments) { arguments; }
}
function local() {
  const arguments = [];
  arguments;
  const inherited = () => arguments;
  function nested() { arguments; }
}
function defaults(value = () => arguments) {
  const arguments = [];
  arguments;
}
const object = {
  method(value = arguments) { arguments; },
};
