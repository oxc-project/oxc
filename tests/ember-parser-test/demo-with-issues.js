// Standard JavaScript file to demonstrate linting works

// Intentional issues for demo:
const unused_variable = 'this is never used';  // unused variable

function test() {
  debugger;  // debugger statement
  var oldStyle = 'var instead of const';  // prefer const/let
  console.log('Test:', oldStyle);  // console.log
}

test();
