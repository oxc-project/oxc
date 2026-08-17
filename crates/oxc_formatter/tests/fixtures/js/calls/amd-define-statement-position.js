// The AMD `define` layout (arguments hugged on one line, never broken) applies only in
// statement position, matching Prettier. A concise arrow body is not statement position.

// Statement position: AMD layout.
define(["aaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbb"], () => {});

define("name", ["aaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbb"], () => {});

// Concise arrow body: NOT statement position, so the arguments break normally.
const d = () => define(["aaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbb"], () => {});

// Block body: the call is a statement again, so AMD layout applies.
const e = () => {
  define(["aaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbb"], () => {});
};

// `require` is not gated on statement position, so a concise body keeps its layout.
const r = () => require("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
