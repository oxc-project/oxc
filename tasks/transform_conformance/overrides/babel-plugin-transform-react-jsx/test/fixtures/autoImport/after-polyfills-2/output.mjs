// The reason to override this test case:
// We generate the import statements before the first non-import statement,
// while the original test case expects them to be after all import statements.

import { jsx as _jsx } from "react/jsx-runtime";
ReactDOM.render(
	/* @__PURE__ */ _jsx("p", { children: "Hello, World!" }),
	document.getElementById("root"),
);
import "react-app-polyfill/ie11";
import "react-app-polyfill/stable";
import ReactDOM from "react-dom";
