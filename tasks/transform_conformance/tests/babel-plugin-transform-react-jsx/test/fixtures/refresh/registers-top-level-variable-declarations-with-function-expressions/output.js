import { jsx as _jsx } from "react/jsx-runtime";
let Hello = function() {
	function handleClick() {}
	return _jsx("h1", {
		onClick: handleClick,
		children: "Hi"
	});
};
const Bar = function Baz() {
	return _jsx(Hello, {});
};
function sum() {}
let Baz = 10;
var Qux;
