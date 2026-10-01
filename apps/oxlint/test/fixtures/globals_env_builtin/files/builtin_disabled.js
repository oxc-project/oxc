// `builtin` env is explicitly disabled for this file, so ES builtins should not resolve
Number.isFinite(1);

// Enabled by `browser` env
window.alert("hello");
