var _templateObject;
function f() {
  if (true) {
    return function() {
      return foo(_templateObject || (_templateObject = babelHelpers.taggedTemplateLiteral(["<\/script>"])));
    };
  }
}
