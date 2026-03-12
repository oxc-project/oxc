import "./a";
import "./a";
import "./a";
import "./react";
// .mtsx and .ctsx are not valid and should not be transformed.
import "./react.mtsx";
import "./react.ctsx";
import "a-package/file";
// Bare import, it's either a node package or remapped by an import map
import "soundcloud.ts";
import "ipaddr.js";
// Dynamic imports should also be rewritten.
import("./a");
import("./a");
import("./a");
import("./react");
import("a-package/file");
// Bare dynamic import should not be rewritten.
import("soundcloud.ts");
// No-substitution template literal should also be rewritten.
import(`./a`);
// Non-string-literal dynamic import should not be rewritten.
import(dynamicPath);
