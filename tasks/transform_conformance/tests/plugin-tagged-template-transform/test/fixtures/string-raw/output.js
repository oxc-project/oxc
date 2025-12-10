var _templateObject;
var _templateObject2;
var _templateObject3;
expect(String.raw(_templateObject || (_templateObject = babelHelpers.taggedTemplateLiteral(["<\/script>"]))) === String.raw(_templateObject2 || (_templateObject2 = babelHelpers.taggedTemplateLiteral(["<\/script>"])))).toBe(true);
expect(String.raw(_templateObject3 || (_templateObject3 = babelHelpers.taggedTemplateLiteral(["<\/script>"]))) !== String.raw`<\/script>`).toBe(true);
