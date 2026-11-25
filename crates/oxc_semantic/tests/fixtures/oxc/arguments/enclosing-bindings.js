function classMembers(arguments) {
  return class {
    constructor() {
      arguments;
    }
    get getter() {
      return arguments;
    }
    static method() {
      arguments;
    }
  };
}
function catchParameter() {
  try {
  } catch (arguments) {
    arguments;
    return function inner() {
      arguments;
    };
  }
}
function defaultFunction(value = function inner() { arguments; }) {
  let arguments;
  arguments;
}
function writes(arguments) {
  return function inner() {
    arguments = 1;
    arguments++;
  };
}
function sloppyVar() {
  var arguments;
  arguments;
  return () => arguments;
}
