const arguments = {};

function ordinary() {
  arguments;
  () => arguments;

  function nested() {
    arguments;
  }
}

const arrow = () => arguments;

const named = function arguments() {
  arguments;
};

function explicitlyShadowed(arguments) {
  arguments;
}
