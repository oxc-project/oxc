const arguments = [];

function direct() {
  return function arguments() {
    arguments;
  };
}
function throughArrow() {
  return function arguments() {
    return () => arguments;
  };
}
function inDefault() {
  return function arguments(value = arguments) {};
}
function withParameter() {
  return function arguments(arguments) {
    arguments;
    return () => arguments;
  };
}
function withVar() {
  return function arguments() {
    var arguments;
    arguments;
    return () => arguments;
  };
}
