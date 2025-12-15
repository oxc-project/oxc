const topLevelConstant = 1;
let topLevelLet = 2;
var topLevelVar = 3;

function topLevelFunction() {
  function innerFunction() {
    function nestedFunction() {
      "use strict";
    }
  }
  return Object;
}

module.exports = topLevelFunction();
