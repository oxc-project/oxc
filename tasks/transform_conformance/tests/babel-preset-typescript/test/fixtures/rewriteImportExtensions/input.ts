import "./a.ts";
import "./a.mts";
import "./a.cts";
import "./react.tsx";
// .mtsx and .ctsx are not valid and should not be transformed.
import "./react.mtsx";
import "./react.ctsx";
import "a-package/file.ts";
// Bare import, it's either a node package or remapped by an import map
import "soundcloud.ts";
import "ipaddr.js";
// Dynamic imports should also be rewritten.
import("./a.ts");
import("./a.mts");
import("./a.cts");
import("./react.tsx");
import("a-package/file.ts");
// Bare dynamic import should not be rewritten.
import("soundcloud.ts");
// Non-string-literal dynamic import should not be rewritten.
import(dynamicPath);
